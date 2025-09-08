use std::collections::{HashMap, VecDeque};

use anyhow::{Result, anyhow};
use image::GenericImage;

use crate::{
    core::{DescriptorSet, RenderContext, Renderer},
    render_graph::node::{RenderNode, RenderNodeWrapper},
    types::world::World,
};

/// a render graph is a way to organize different passes into a graph structure it lets you define
/// inputs and outputs
#[derive(Default)]
pub struct RenderGraph {
    nodes: HashMap<&'static str, RenderNodeWrapper>,
    edges: HashMap<&'static str, &'static str>,
    pub context: RenderGraphContext,
}

/// the context contains shared resources within the render graph
///
/// these resources are not error checked so be sure to add edges to properly order the nodes
#[derive(Default)]
pub struct RenderGraphContext {
    resources: HashMap<&'static str, DescriptorSet>,
}

pub struct GraphBuilder<'a> {
    renderer: &'a mut Renderer,
}

impl<'a> GraphBuilder<'a> {
    pub(crate) fn create(renderer: &'a mut Renderer) -> Self {
        Self { renderer }
    }

    pub fn add_node<T>(&mut self, name: &'static str, node: T)
    where
        T: RenderNode + 'static,
    {
        let wrapper = self.renderer.setup_render_node(Some(name), node);

        self.renderer.render_graph.add_node(name, wrapper);
    }

    pub fn add_edge(&mut self, output: &'static str, input: &'static str) {
        self.renderer.render_graph.add_edge(output, input);
    }
}

impl RenderGraphContext {
    pub fn add_shared_resource(&mut self, name: &'static str, set: DescriptorSet) {
        self.resources.insert(name, set);
    }

    pub fn get_shared_resource(&mut self, name: &'static str) -> Option<&DescriptorSet> {
        self.resources.get(name)
    }
}

impl RenderGraph {
    pub(crate) fn add_node(&mut self, name: &'static str, wrapper: RenderNodeWrapper) {
        self.nodes.insert(name, wrapper);
    }

    /// edges of the graph for render order example output -> input output will be rendered before
    /// input
    pub(crate) fn add_edge(&mut self, output: &'static str, input: &'static str) {
        self.edges.insert(output, input);
    }

    pub(crate) fn render(&mut self, rcx: &RenderContext) -> Result<()> {
        let order = self.order_nodes()?;

        for key in order {
            let node = self
                .nodes
                .get_mut(key)
                .ok_or(anyhow!("failed to get node: {key}"))?;

            // temporary we have no world yet
            let world = World::default();

            // draw the nodes renderer for calling renderer.draw(...) node context for pipeline
            // graph context for shared resources and world for scene data
            node.pass
                .draw(rcx, &mut node.context, &mut self.context, world)?;
        }

        Ok(())
    }

    /// calls resize for all the nodes
    pub(crate) fn resize(&mut self, dimensions: [u32; 2]) -> Result<()> {
        for node in self.nodes.values_mut() {
            node.pass.resize(dimensions)?;
        }
        Ok(())
    }

    /// returns the nodes with their render order or an Error if the graph contains cycles
    fn order_nodes(&self) -> Result<Vec<&'static str>> {
        // indegree for all declared nodes
        let mut indegree: HashMap<&'static str, usize> =
            self.nodes.keys().copied().map(|k| (k, 0usize)).collect();

        // validate edges & build indegrees
        for (u, v) in &self.edges {
            if !self.nodes.contains_key(u) {
                return Err(anyhow!("edge references unknown node: {u}"));
            }
            if !self.nodes.contains_key(v) {
                return Err(anyhow!("edge references unknown node: {v}"));
            }
            *indegree.get_mut(v).expect("v exists by contains_key") += 1;
        }

        let mut adj: HashMap<&'static str, Vec<&'static str>> = HashMap::new();
        for (u, v) in &self.edges {
            adj.entry(*u).or_default().push(*v);
        }

        // queue of nodes with indegree 0
        let mut q: VecDeque<&'static str> = indegree
            .iter()
            .filter_map(|(&k, &d)| if d == 0 { Some(k) } else { None })
            .collect();

        let mut order = Vec::with_capacity(self.nodes.len());

        while let Some(u) = q.pop_front() {
            order.push(u);
            if let Some(vs) = adj.remove(u) {
                for v in vs {
                    let d = indegree.get_mut(v).expect("v in indegree map");
                    *d -= 1;
                    if *d == 0 {
                        q.push_back(v);
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            return Err(anyhow!("render graph contains a cycle"));
        }

        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_nodes_empty_graph_returns_empty() {
        let g = RenderGraph::default();
        let order = g.order_nodes().expect("empty graph should topo-sort");
        assert!(
            order.is_empty(),
            "expected no nodes in order for empty graph"
        );
    }

    #[test]
    fn order_nodes_with_unknown_nodes_errors() {
        let mut g = RenderGraph::default();

        // Add an edge between nodes that don't exist in `g.nodes`.
        g.add_edge("A", "B");

        let err = g
            .order_nodes()
            .expect_err("should error on unknown node in edge");
        let msg = err.to_string();
        assert!(
            msg.contains("unknown node"),
            "error should mention unknown node, got: {msg}"
        );
    }
}
