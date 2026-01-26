use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::platform::SendSync;
use anyhow::{Result, anyhow};
use maple_engine::GameContext;
use parking_lot::RwLock;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    core::{RenderContext, Renderer},
    render_graph::node::RenderNode,
};

pub trait NodeLabel: Any {}

/// a render graph is a way to organize different passes into a graph structure it lets you define
/// inputs and outputs
#[derive(Default)]
pub struct RenderGraph {
    nodes: HashMap<TypeId, RwLock<Box<dyn RenderNode>>>,
    edges: HashMap<TypeId, Vec<TypeId>>,
    pub context: RwLock<RenderGraphContext>,
}

pub trait GraphResource: Any + SendSync {}

/// the context contains shared resources within the render graph
///
/// these resources are not error checked so be sure to add edges to properly order the nodes
#[derive(Default)]
pub struct RenderGraphContext {
    #[cfg(not(target_arch = "wasm32"))]
    resources: HashMap<&'static str, Box<dyn Any + Send + Sync>>,
    #[cfg(target_arch = "wasm32")]
    resources: HashMap<&'static str, Box<dyn Any>>,
}

pub struct GraphBuilder<'a> {
    renderer: &'a mut Renderer,
}

impl<'a> GraphBuilder<'a> {
    pub(crate) fn create(renderer: &'a mut Renderer) -> Self {
        Self { renderer }
    }

    pub fn add_node<T>(&mut self, node: T)
    where
        T: RenderNode + 'static,
    {
        self.renderer.render_graph.add_node(node);
    }

    pub fn add_node_with<F, T>(&mut self, factory: F)
    where
        F: FnOnce(&RenderContext, &mut RenderGraphContext) -> T,
        T: RenderNode + 'static,
    {
        let node = factory(
            &self.renderer.context,
            &mut self.renderer.render_graph.context.write(),
        );
        self.renderer.render_graph.add_node(node);
    }

    pub fn add_edge<Output: RenderNode + 'static, Input: RenderNode + 'static>(&mut self) {
        self.renderer.render_graph.add_edge::<Output, Input>();
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
    pub(crate) fn add_node<T: RenderNode + 'static>(&mut self, node: T) {
        let id = TypeId::of::<T>();
        self.nodes.insert(id, RwLock::new(Box::new(node)));
    }

    /// edges of the graph for render order example output -> input output will be rendered before
    /// input
    pub(crate) fn add_edge<Output: RenderNode + 'static, Input: RenderNode + 'static>(&mut self) {
        let output_id = TypeId::of::<Output>();
        let input_id = TypeId::of::<Input>();

        self.edges.entry(output_id).or_default().push(input_id)
    }

    pub(crate) fn render(&mut self, rcx: &RenderContext, game_ctx: &GameContext) -> Result<()> {
        let layers = self.order_nodes_layered()?;

        for layer in layers {
            #[cfg(not(target_arch = "wasm32"))]
            layer.par_iter().try_for_each(|&node_id| -> Result<()> {
                let node = self
                    .nodes
                    .get(&node_id)
                    .ok_or(anyhow!("failed to get node: {node_id:?}"))?;

                let mut node_guard = node.write();
                let mut ctx_guard = self.context.write();

                node_guard.draw(rcx, &mut ctx_guard, game_ctx);
                Ok(())
            })?;

            #[cfg(target_arch = "wasm32")]
            for &node_id in layer.iter() {
                let node = self
                    .nodes
                    .get(&node_id)
                    .ok_or(anyhow!("failed to get node: {node_id:?}"))?;

                let mut node_guard = node.write();
                let mut ctx_guard = self.context.write();

                node_guard.draw(rcx, &mut ctx_guard, scene);
            }
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
        for (u, vs) in &self.edges {
            if !self.nodes.contains_key(u) {
                return Err(anyhow!("edge references unknown node: {u:?}"));
            }
            for v in vs {
                if !self.nodes.contains_key(v) {
                    return Err(anyhow!("edge references unknown node: {v:?}"));
                }
                *indegree.get_mut(v).expect("v exists by contains_key") += 1;
            }
        }

        let adj = self.edges.clone();

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
