//! organizes [`RenderNodes`](crate::render_graph::node::RenderNode) as a graph of dependencies
//!
//! some render passes may need previous pass data such as a color pass needed shadow depth textures
//! the graph organizes these so that nodes that are dependent of other will run after them

#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
use std::{
    any::{self, Any, TypeId},
    collections::HashMap,
};

use crate::{core::Frame, platform::SendSync, types::Dimensions};
use maple_engine::GameContext;
use parking_lot::RwLock;

use crate::{
    core::{RenderContext, Renderer},
    render_graph::node::RenderNode,
};

/// the render stage the [`RenderNode`] will be ran at
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    /// ran first
    PrePass,
    /// ran during shadow depth pass
    Shadow,
    /// ran during opaque color pass
    Opaque,
    /// ran during transparent color pass
    Transparent,
    /// ran after color pass
    PostProcess,
    /// render ui components after postprocessing
    Ui,
    /// present textures to surface
    Present,
}

/// a render graph is a way to organize different passes into a graph structure it lets you define
/// inputs and outputs
#[derive(Default)]
pub struct RenderGraph {
    nodes: HashMap<TypeId, (String, RwLock<Box<dyn RenderNode>>)>,
    edges: HashMap<TypeId, Vec<TypeId>>,
    pub(crate) context: RwLock<RenderGraphContext>,
}

/// a resource that exists within the graph and can be accesses through the [`RenderGraphContext`]
pub trait GraphResource: Any + SendSync {}

/// the context contains shared resources within the render graph
///
/// these resources are not error checked so be sure to add edges to properly order the nodes
#[derive(Default)]
pub struct RenderGraphContext {
    #[cfg(not(target_arch = "wasm32"))]
    resources: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    #[cfg(target_arch = "wasm32")]
    resources: HashMap<TypeId, Box<dyn Any>>,
}

/// a builder for adding nodes and edges
pub struct GraphBuilder<'a> {
    renderer: &'a mut Renderer,
}

impl<'a> GraphBuilder<'a> {
    pub(crate) fn create(renderer: &'a mut Renderer) -> Self {
        Self { renderer }
    }

    /// add a render node that already exists
    pub fn add_node<T>(&mut self, node: T)
    where
        T: RenderNode + 'static,
    {
        self.renderer.render_graph.add_node(node);
    }

    /// constructs and adds a render node
    pub fn setup_and_add_node<T>(&mut self)
    where
        T: RenderNode + 'static,
    {
        let node = T::setup(
            &self.renderer.context,
            &mut self.renderer.render_graph.context.write(),
        );
        self.renderer.render_graph.add_node(node);
    }

    /// link 2 render nodes in the graph
    pub fn add_edge<Output: RenderNode + 'static, Input: RenderNode + 'static>(&mut self) {
        self.renderer.render_graph.add_edge::<Output, Input>();
    }
}

impl RenderGraphContext {
    /// add a resource that can be accessed by other nodes
    pub fn add_shared_resource<T: Any + SendSync>(&mut self, res: T) {
        self.resources.insert(TypeId::of::<T>(), Box::new(res));
    }

    /// get a resource that exists within the graph for sharing data between nodes
    pub fn get_shared_resource<T: Any>(&self) -> Option<&T> {
        let resource = self.resources.get(&TypeId::of::<T>())?.downcast_ref();
        if resource.is_none() {
            log::warn!(
                "Tried to get render graph resource: {} but it does not exist",
                any::type_name::<T>()
            )
        }
        resource
    }

    pub fn get_shared_resource_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.resources.get_mut(&TypeId::of::<T>())?.downcast_mut()
    }
}

impl RenderGraph {
    pub(crate) fn add_node<T: RenderNode + 'static>(&mut self, node: T) {
        let id = TypeId::of::<T>();
        self.nodes
            .insert(id, (T::label().into(), RwLock::new(Box::new(node))));
    }

    /// edges of the graph for render order example output -> input output will be rendered before
    /// input
    pub(crate) fn add_edge<Output: RenderNode + 'static, Input: RenderNode + 'static>(&mut self) {
        let output_id = TypeId::of::<Output>();
        let input_id = TypeId::of::<Input>();

        self.edges.entry(output_id).or_default().push(input_id)
    }

    pub(crate) fn render(
        &mut self,
        rcx: &RenderContext,
        game_ctx: &GameContext,
        frame: &mut Frame,
    ) {
        let layers = self.order_nodes_layered();

        #[cfg(not(target_arch = "wasm32"))]
        let mut timings: HashMap<String, Duration> = HashMap::new();

        for layer in layers {
            layer.iter().for_each(|&node_id| {
                let (name, node) = self.nodes.get(&node_id).unwrap();

                let mut node_guard = node.write();
                let mut ctx_guard = self.context.write();

                #[cfg(not(target_arch = "wasm32"))]
                let start = Instant::now();
                node_guard.draw(rcx, frame, &mut ctx_guard, game_ctx);
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let elapsed = start.elapsed();
                    let entry = timings.entry(name.clone()).or_insert(elapsed);
                    *entry = elapsed;
                }
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        game_ctx
            .get_resource_mut::<maple_engine::resources::Frame>()
            .timings
            .extend(timings);
    }

    /// calls resize for all the nodes
    pub(crate) fn resize(&mut self, render_ctx: &RenderContext, dimensions: Dimensions) {
        for (_, node_lock) in self.nodes.values_mut() {
            let mut node = node_lock.write();
            node.resize(render_ctx, dimensions);
        }
    }

    /// returns the nodes with their render order or an Error if the graph contains cycles
    fn order_nodes_layered(&self) -> Vec<Vec<TypeId>> {
        // Validate edges first
        for (u, vs) in &self.edges {
            if !self.nodes.contains_key(u) {
                panic!("edge added to node which doesnt exist in rendergraph")
            }
            for v in vs {
                if !self.nodes.contains_key(v) {
                    panic!("edge added to node which doesnt exist in rendergraph")
                }
            }
        }

        let mut adj = self.edges.clone();
        let staged: Vec<(TypeId, Stage)> = self
            .nodes
            .iter()
            .map(|(id, (_, node))| (*id, node.read().stage()))
            .collect();

        // add edges for stages
        for &(a, stage_a) in &staged {
            for &(b, stage_b) in &staged {
                if a == b {
                    continue;
                }
                if stage_a < stage_b {
                    let entry = adj.entry(a).or_default();
                    if !entry.contains(&b) {
                        entry.push(b);
                    }
                }
            }
        }

        // Build indegree AFTER adj (incl. stage edges) is finalized
        let mut indegree: HashMap<TypeId, usize> =
            self.nodes.keys().copied().map(|k| (k, 0usize)).collect();
        for (_, vs) in &adj {
            for v in vs {
                *indegree.get_mut(v).expect("v exists by contains_key") += 1;
            }
        }

        let mut layers: Vec<Vec<TypeId>> = Vec::new();
        let mut processed = 0;
        loop {
            let current_layer: Vec<TypeId> = indegree
                .iter()
                .filter_map(|(&k, &d)| if d == 0 { Some(k) } else { None })
                .collect();
            if current_layer.is_empty() {
                break;
            }
            processed += current_layer.len();
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
            panic!("cycle detected within rendergraph")
        }
        layers
    }
}
