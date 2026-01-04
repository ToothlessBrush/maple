use std::{
    any::{Any, TypeId, type_name},
    collections::{HashMap, VecDeque},
    error::Error,
};

use anyhow::{Result, anyhow};
use maple_engine::Scene;
use parking_lot::RwLock;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    core::{RenderContext, Renderer},
    render_graph::node::{RenderNode, RenderNodeWrapper},
};

pub trait NodeLabel: Any {}

/// a render graph is a way to organize different passes into a graph structure it lets you define
/// inputs and outputs
#[derive(Default)]
pub struct RenderGraph {
    nodes: HashMap<TypeId, RwLock<RenderNodeWrapper>>,
    edges: HashMap<TypeId, TypeId>,
    pub context: RwLock<RenderGraphContext>,
}

pub trait GraphResource: Any + Send + Sync {}

/// the context contains shared resources within the render graph
///
/// these resources are not error checked so be sure to add edges to properly order the nodes
#[derive(Default)]
pub struct RenderGraphContext {
    resources: HashMap<&'static str, Box<dyn Any + Send + Sync>>,
}

pub struct GraphBuilder<'a> {
    renderer: &'a mut Renderer,
}

impl<'a> GraphBuilder<'a> {
    pub(crate) fn create(renderer: &'a mut Renderer) -> Self {
        Self { renderer }
    }

    pub fn add_node<E, T>(&mut self, label: E, node: T)
    where
        E: NodeLabel,
        T: RenderNode + 'static,
    {
        let _name = type_name::<E>();

        let wrapper = self.renderer.setup_render_node(node);

        self.renderer.render_graph.add_node(label, wrapper);
    }

    pub fn add_edge<Output: NodeLabel, Input: NodeLabel>(&mut self, output: Output, input: Input) {
        self.renderer.render_graph.add_edge(output, input);
    }
}

impl RenderGraphContext {
    pub fn add_shared_resource<T: GraphResource>(&mut self, name: &'static str, res: T) {
        self.resources.insert(name, Box::new(res));
    }

    pub fn get_shared_resource<T: GraphResource>(&self, name: &'static str) -> Option<&T> {
        self.resources.get(name)?.downcast_ref()
    }
}

impl RenderGraph {
    pub(crate) fn add_node<E: NodeLabel>(&mut self, _label: E, wrapper: RenderNodeWrapper) {
        let id = TypeId::of::<E>();
        self.nodes.insert(id, RwLock::new(wrapper));
    }

    /// edges of the graph for render order example output -> input output will be rendered before
    /// input
    pub(crate) fn add_edge<Output: NodeLabel, Input: NodeLabel>(
        &mut self,
        _output: Output,
        _input: Input,
    ) {
        let output_id = TypeId::of::<Output>();
        let input_id = TypeId::of::<Input>();

        self.edges.insert(output_id, input_id);
    }

    pub(crate) fn render(&mut self, rcx: &RenderContext, scene: &Scene) -> Result<()> {
        let layers = self.order_nodes_layered()?;

        for layer in layers {
            layer.par_iter().try_for_each(|&node_id| -> Result<()> {
                let node = self
                    .nodes
                    .get(&node_id)
                    .ok_or(anyhow!("failed to get node: {node_id:?}"))?;

                let mut node_guard = node.write();
                let mut ctx_guard = self.context.write();

                node_guard.pass.draw(rcx, &mut *ctx_guard, scene);
                Ok(())
            })?;
        }

        Ok(())
    }

    /// calls resize for all the nodes
    pub(crate) fn resize(&mut self, render_ctx: &RenderContext, dimensions: [u32; 2]) {
        for node_lock in self.nodes.values_mut() {
            let mut node = node_lock.write();
            node.resize(render_ctx, dimensions);
        }
    }

    /// returns the nodes with their render order or an Error if the graph contains cycles
    fn order_nodes_layered(&self) -> Result<Vec<Vec<TypeId>>> {
        let mut indegree: HashMap<TypeId, usize> =
            self.nodes.keys().copied().map(|k| (k, 0usize)).collect();

        // Validate edges & build indegrees
        for (u, v) in &self.edges {
            if !self.nodes.contains_key(u) {
                return Err(anyhow!("edge references unknown node: {u:?}"));
            }
            if !self.nodes.contains_key(v) {
                return Err(anyhow!("edge references unknown node: {v:?}"));
            }
            *indegree.get_mut(v).expect("v exists by contains_key") += 1;
        }

        let mut adj: HashMap<TypeId, Vec<TypeId>> = HashMap::new();
        for (u, v) in &self.edges {
            adj.entry(*u).or_default().push(*v);
        }

        let mut layers: Vec<Vec<TypeId>> = Vec::new();
        let mut processed = 0;

        loop {
            // All nodes with indegree 0 form the current layer
            let current_layer: Vec<TypeId> = indegree
                .iter()
                .filter_map(|(&k, &d)| if d == 0 { Some(k) } else { None })
                .collect();

            if current_layer.is_empty() {
                break;
            }

            processed += current_layer.len();

            // Remove processed nodes from indegree map and update neighbors
            for &u in &current_layer {
                indegree.remove(&u);
                if let Some(vs) = adj.get(&u) {
                    for &v in vs {
                        if let Some(d) = indegree.get_mut(&v) {
                            *d -= 1;
                        }
                    }
                }
            }

            layers.push(current_layer);
        }

        if processed != self.nodes.len() {
            return Err(anyhow!("render graph contains a cycle"));
        }

        Ok(layers)
    }
}

#[cfg(test)]
mod tests {
    use crate::render_graph::node::Marker;

    use super::*;

    #[test]
    fn order_nodes_empty_graph_returns_empty() {
        let g = RenderGraph::default();
        let layers = g
            .order_nodes_layered()
            .expect("empty graph should topo-sort");
        assert!(layers.is_empty(), "expected no layers for empty graph");
    }

    #[test]
    fn order_nodes_with_unknown_nodes_errors() {
        let mut g = RenderGraph::default();
        // Add an edge between nodes that don't exist in `g.nodes`.
        // Use dummy TypeIds for nodes that don't exist
        use std::any::TypeId;
        struct DummyNode1;
        struct DummyNode2;
        g.edges
            .insert(TypeId::of::<DummyNode1>(), TypeId::of::<DummyNode2>());
        let err = g
            .order_nodes_layered()
            .expect_err("should error on unknown node in edge");
        let msg = err.to_string();
        assert!(
            msg.contains("unknown node"),
            "error should mention unknown node, got: {msg}"
        );
    }

    #[test]
    fn order_nodes_linear_chain() {
        let mut g = RenderGraph::default();

        struct Node1;
        struct Node2;
        struct Node3;

        impl NodeLabel for Node1 {}
        impl NodeLabel for Node2 {}
        impl NodeLabel for Node3 {}

        // Create dummy nodes
        let wrapper1 = RenderNodeWrapper::create(Box::new(Marker));
        let wrapper2 = RenderNodeWrapper::create(Box::new(Marker));
        let wrapper3 = RenderNodeWrapper::create(Box::new(Marker));

        g.add_node(Node1, wrapper1);
        g.add_node(Node2, wrapper2);
        g.add_node(Node3, wrapper3);

        // Create chain: Node1 -> Node2 -> Node3
        g.add_edge(Node1, Node2);
        g.add_edge(Node2, Node3);

        let layers = g.order_nodes_layered().expect("should topo-sort");

        assert_eq!(layers.len(), 3, "should have 3 layers");
        assert_eq!(layers[0].len(), 1, "first layer should have 1 node");
        assert_eq!(layers[1].len(), 1, "second layer should have 1 node");
        assert_eq!(layers[2].len(), 1, "third layer should have 1 node");
    }

    #[test]
    fn order_nodes_parallel_nodes() {
        let mut g = RenderGraph::default();

        struct Node1;
        struct Node2;
        struct Node3;
        struct Node4;

        impl NodeLabel for Node1 {}
        impl NodeLabel for Node2 {}
        impl NodeLabel for Node3 {}
        impl NodeLabel for Node4 {}

        let wrapper1 = RenderNodeWrapper::create(Box::new(Marker));
        let wrapper2 = RenderNodeWrapper::create(Box::new(Marker));
        let wrapper3 = RenderNodeWrapper::create(Box::new(Marker));
        let wrapper4 = RenderNodeWrapper::create(Box::new(Marker));

        g.add_node(Node1, wrapper1);
        g.add_node(Node2, wrapper2);
        g.add_node(Node3, wrapper3);
        g.add_node(Node4, wrapper4);

        // Create diamond: Node1 -> Node2, Node3 -> Node4
        //                 Node1 -> Node3
        g.add_edge(Node1, Node2);
        g.add_edge(Node1, Node3);
        g.add_edge(Node2, Node4);
        g.add_edge(Node3, Node4);

        let layers = g.order_nodes_layered().expect("should topo-sort");

        assert_eq!(layers.len(), 3, "should have 3 layers");
        assert_eq!(layers[0].len(), 1, "first layer should have 1 node (Node1)");
        assert_eq!(
            layers[1].len(),
            2,
            "second layer should have 2 nodes (Node2, Node3 in parallel)"
        );
        assert_eq!(layers[2].len(), 1, "third layer should have 1 node (Node4)");
    }

    #[test]
    fn order_nodes_detects_cycle() {
        let mut g = RenderGraph::default();

        struct Node1;
        struct Node2;
        struct Node3;

        impl NodeLabel for Node1 {}
        impl NodeLabel for Node2 {}
        impl NodeLabel for Node3 {}

        let wrapper1 = RenderNodeWrapper::create(Box::new(Marker));
        let wrapper2 = RenderNodeWrapper::create(Box::new(Marker));
        let wrapper3 = RenderNodeWrapper::create(Box::new(Marker));

        g.add_node(Node1, wrapper1);
        g.add_node(Node2, wrapper2);
        g.add_node(Node3, wrapper3);

        // Create cycle: Node1 -> Node2 -> Node3 -> Node1
        g.add_edge(Node1, Node2);
        g.add_edge(Node2, Node3);
        g.add_edge(Node3, Node1);

        let err = g.order_nodes_layered().expect_err("should detect cycle");
        let msg = err.to_string();
        assert!(
            msg.contains("cycle"),
            "error should mention cycle, got: {msg}"
        );
    }
}
