use std::collections::HashMap;

use crate::render_graph::node::RenderNode;

pub struct RenderGraph {
    pub nodes: HashMap<&'static str, Box<dyn RenderNode>>,
}
