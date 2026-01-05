use std::{
    any::TypeId,
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use parking_lot::{ArcRwLockReadGuard, ArcRwLockWriteGuard, RawRwLock, RwLock};

use crate::{
    GameContext, Node,
    prelude::{EventLabel, node_transform::WorldTransform},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct NodeId(u64);

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        NodeId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct SceneNode {
    _id: NodeId,
    name: String,
    children: Vec<NodeId>,
    parent: Option<NodeId>,
    type_id: TypeId,
}

type NodeStorage = Arc<RwLock<Box<dyn Node>>>;

pub struct Scene {
    nodes: RwLock<HashMap<NodeId, NodeStorage>>,

    heirarchy: RwLock<HashMap<NodeId, SceneNode>>,
}

pub struct NodeHandle<'a, T: Node> {
    id: NodeId,
    scene: &'a Scene,
    _ty: PhantomData<T>,
}

pub struct NodeReadGuard<T: Node> {
    guard: ArcRwLockReadGuard<RawRwLock, Box<dyn Node>>,
    _ty: PhantomData<T>,
}

pub struct NodeWriteGuard<T: Node> {
    guard: ArcRwLockWriteGuard<RawRwLock, Box<dyn Node>>,
    _ty: PhantomData<T>,
}

impl<'a, T: Node> NodeHandle<'a, T> {
    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn name(&self) -> Option<String> {
        self.scene.node_name(self.id)
    }

    pub fn children(&self) -> Vec<NodeId> {
        self.scene.children(self.id)
    }

    pub fn add_child<C: Node>(&self, name: impl Into<String>, node: C) -> NodeHandle<'a, C> {
        self.scene.add_child(name, node, self.id)
    }

    pub fn merge_scene(&self, other: Scene) -> Vec<NodeId> {
        self.scene.merge_as_child(other, self.id)
    }

    pub fn read(&self) -> NodeReadGuard<T> {
        let node_lock = {
            let nodes = self.scene.nodes.read();
            Arc::clone(nodes.get(&self.id).expect("Node not found"))
        };

        // Use read_arc instead of read - it takes ownership semantics of the Arc
        let guard = RwLock::read_arc(&node_lock);
        NodeReadGuard {
            guard,
            _ty: PhantomData,
        }
    }

    pub fn write(&self) -> NodeWriteGuard<T> {
        let node_lock = {
            let nodes = self.scene.nodes.read();
            Arc::clone(nodes.get(&self.id).expect("Node not found"))
        };

        let guard = RwLock::write_arc(&node_lock);

        NodeWriteGuard {
            guard,
            _ty: PhantomData,
        }
    }
}

impl<T: Node> Deref for NodeReadGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.as_any().downcast_ref::<T>().unwrap()
    }
}

impl<T: Node> Deref for NodeWriteGuard<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.as_any().downcast_ref::<T>().unwrap()
    }
}

impl<T: Node> DerefMut for NodeWriteGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.as_any_mut().downcast_mut::<T>().unwrap()
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Scene {
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            heirarchy: RwLock::new(HashMap::new()),
        }
    }

    pub fn add<T: Node>(&'a self, name: impl Into<String>, node: T) -> NodeHandle<'a, T> {
        self.add_with_parent(name, node, None)
    }

    pub fn add_child<T: Node>(
        &'a self,
        name: impl Into<String>,
        node: T,
        parent: NodeId,
    ) -> NodeHandle<'a, T> {
        self.add_with_parent(name, node, Some(parent))
    }

    fn add_with_parent<T: Node>(
        &'a self,
        name: impl Into<String>,
        node: T,
        parent: Option<NodeId>,
    ) -> NodeHandle<'a, T> {
        let id = NodeId::new();

        let scene_node = SceneNode {
            _id: id,
            name: name.into(),
            children: Vec::new(),
            parent,
            type_id: TypeId::of::<T>(),
        };

        {
            let mut hierarchy = self.heirarchy.write();
            if let Some(parent_id) = parent
                && let Some(parent_node) = hierarchy.get_mut(&parent_id)
            {
                parent_node.children.push(id);
            }
            hierarchy.insert(id, scene_node);
        }

        {
            let mut nodes = self.nodes.write();
            nodes.insert(id, Arc::new(RwLock::new(Box::new(node))));
        }

        NodeHandle {
            id,
            scene: self,
            _ty: PhantomData,
        }
    }

    pub fn merge(&self, other: Scene) -> Vec<NodeId> {
        self.merge_as_child_of(other, None)
    }

    pub fn merge_as_child(&self, other: Scene, parent: NodeId) -> Vec<NodeId> {
        self.merge_as_child_of(other, Some(parent))
    }

    fn merge_as_child_of(&self, other: Scene, parent: Option<NodeId>) -> Vec<NodeId> {
        let mut other_hierarchy = other.heirarchy.write();
        let mut other_nodes = other.nodes.write();

        let root_ids: Vec<NodeId> = other_hierarchy
            .iter()
            .filter(|(_, node)| node.parent.is_none())
            .map(|(id, _)| *id)
            .collect();

        {
            let mut self_heirarchy = self.heirarchy.write();
            let mut self_nodes = self.nodes.write();

            for (id, mut scene_node) in other_hierarchy.drain() {
                if scene_node.parent.is_none() {
                    scene_node.parent = parent;
                }
                self_heirarchy.insert(id, scene_node);
            }

            for (id, node_data) in other_nodes.drain() {
                self_nodes.insert(id, node_data);
            }

            if let Some(parent_id) = parent
                && let Some(parent_node) = self_heirarchy.get_mut(&parent_id)
            {
                parent_node.children.extend(&root_ids);
            }
        }

        root_ids
    }

    pub fn get<T: Node>(&'a self, id: NodeId) -> Option<NodeHandle<'a, T>> {
        let hierarchy = self.heirarchy.read();
        let scene_node = hierarchy.get(&id)?;

        if scene_node.type_id != TypeId::of::<T>() {
            return None;
        }

        Some(NodeHandle {
            id,
            scene: self,
            _ty: PhantomData,
        })
    }

    pub fn get_by_name<T: Node>(&'a self, name: &str) -> Option<NodeHandle<'a, T>> {
        let hierarchy = self.heirarchy.read();
        let type_id = TypeId::of::<T>();

        for (id, scene_node) in hierarchy.iter() {
            if scene_node.name == name && scene_node.type_id == type_id {
                return Some(NodeHandle {
                    id: *id,
                    scene: self,
                    _ty: PhantomData,
                });
            }
        }
        None
    }

    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.heirarchy.read().get(&id).and_then(|n| n.parent)
    }

    pub fn children(&self, id: NodeId) -> Vec<NodeId> {
        self.heirarchy
            .read()
            .get(&id)
            .map(|n| n.children.clone())
            .unwrap_or_default()
    }

    pub fn node_name(&self, id: NodeId) -> Option<String> {
        self.heirarchy.read().get(&id).map(|n| n.name.clone())
    }

    pub fn collect<T: Node>(&'a self) -> Vec<NodeHandle<'a, T>> {
        let heirarchy = self.heirarchy.read();
        let type_id = TypeId::of::<T>();

        heirarchy
            .iter()
            .filter(|(_, node)| node.type_id == type_id)
            .map(|(id, _)| NodeHandle {
                id: *id,
                scene: self,
                _ty: PhantomData,
            })
            .collect()
    }

    /// emit an event to the scene (this will also update world space transforms)
    pub fn emit<E: EventLabel>(&self, event: &E, ctx: &GameContext) {
        let root_ids: Vec<NodeId> = {
            let hierarchy = self.heirarchy.read();
            hierarchy
                .iter()
                .filter(|(_, node)| node.parent.is_none())
                .map(|(id, _)| *id)
                .collect()
        };

        for root_id in root_ids {
            self.emit_recursive(root_id, event, ctx, WorldTransform::default());
        }
    }

    fn emit_recursive<E: EventLabel>(
        &self,
        id: NodeId,
        event: &E,
        ctx: &GameContext,
        parent_world: WorldTransform,
    ) {
        let (current_world, mut events) = {
            let node_lock = {
                let nodes = self.nodes.read();
                nodes.get(&id).map(Arc::clone)
            };

            let Some(node_lock) = node_lock else {
                return;
            };

            let mut node = node_lock.write();

            node.get_transform().get_world_space(parent_world);
            let current_world = *node.get_transform().world_space();

            let events = std::mem::take(node.get_events());

            (current_world, events)
        };

        events.trigger(event, self, id, ctx);

        {
            let nodes = self.nodes.read();
            if let Some(node_lock) = nodes.get(&id) {
                let mut node = node_lock.write();
                *node.get_events() = events;
            }
        }

        let children = self.children(id);
        for child_id in children {
            self.emit_recursive(child_id, event, ctx, current_world);
        }
    }

    /// emit an event to a single node
    pub fn emit_to<E: EventLabel>(&self, id: NodeId, event: &E, ctx: &GameContext) {
        let node_lock = {
            let nodes = self.nodes.read();
            nodes.get(&id).map(Arc::clone)
        };

        let Some(node_lock) = node_lock else {
            return;
        };

        let mut node = node_lock.write();
        let mut events = std::mem::take(node.get_events());
        drop(node); // release node so that user can write during event

        events.trigger(event, self, id, ctx);

        let mut node = node_lock.write();
        *node.get_events() = events;
    }

    pub fn for_each<T: Node>(&self, f: &mut impl FnMut(&mut T)) {
        let type_id = TypeId::of::<T>();

        let node_locks: Vec<NodeStorage> = {
            let hierarchy = self.heirarchy.read();
            let nodes = self.nodes.read();

            hierarchy
                .iter()
                .filter(|(_, node)| node.type_id == type_id)
                .filter_map(|(id, _)| nodes.get(id).map(Arc::clone))
                .collect()
        };

        for node_lock in node_locks {
            let mut node = node_lock.write();
            if let Some(concrete) = node.as_any_mut().downcast_mut::<T>() {
                f(concrete);
            }
        }
    }

    pub fn for_each_ref<T: Node>(&self, f: &mut impl FnMut(&T)) {
        let type_id = TypeId::of::<T>();

        let node_locks: Vec<NodeStorage> = {
            let hierarchy = self.heirarchy.read();
            let nodes = self.nodes.read();

            hierarchy
                .iter()
                .filter(|(_, node)| node.type_id == type_id)
                .filter_map(|(id, _)| nodes.get(id).map(Arc::clone))
                .collect()
        };

        for node_lock in node_locks {
            let node = node_lock.read();
            if let Some(concrete) = node.as_any().downcast_ref::<T>() {
                f(concrete);
            }
        }
    }

    pub fn for_each_with_id<T: Node>(&self, f: &mut impl FnMut(NodeId, &mut T)) {
        let type_id = TypeId::of::<T>();

        let node_data: Vec<(NodeId, NodeStorage)> = {
            let hierarchy = self.heirarchy.read();
            let nodes = self.nodes.read();

            hierarchy
                .iter()
                .filter(|(_, node)| node.type_id == type_id)
                .filter_map(|(id, _)| nodes.get(id).map(|n| (*id, Arc::clone(n))))
                .collect()
        };

        for (id, node_lock) in node_data {
            let mut node = node_lock.write();
            if let Some(concrete) = node.as_any_mut().downcast_mut::<T>() {
                f(id, concrete);
            }
        }
    }
}
