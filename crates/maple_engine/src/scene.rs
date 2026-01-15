use std::{
    any::TypeId,
    collections::{HashMap, VecDeque},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use parking_lot::{ArcRwLockReadGuard, ArcRwLockWriteGuard, RawRwLock, RwLock};

use crate::{
    GameContext, Node,
    prelude::{EventCtx, EventLabel, EventReceiver, Ready, node_transform::WorldTransform},
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

/// Typed Handle to a node in the scene
///
/// Allows access to a node in the scene without locks. This doesnt store the Node and is
/// cheap to copy. The actual node can be accessed via '.read()' and '.write()'.
///
/// lifetime is tied to the scene
pub struct NodeHandle<'a, T: Node> {
    id: NodeId,
    scene: &'a Scene,
    _ty: PhantomData<T>,
}

/// RAII guard for immutible access to a node.
pub struct NodeReadGuard<T: Node> {
    guard: ArcRwLockReadGuard<RawRwLock, Box<dyn Node>>,
    _ty: PhantomData<T>,
}

/// RAII guard for mutible access to a node.
pub struct NodeWriteGuard<T: Node> {
    guard: ArcRwLockWriteGuard<RawRwLock, Box<dyn Node>>,
    _ty: PhantomData<T>,
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

impl<'a, T: Node> NodeHandle<'a, T> {
    /// returns the id of this node
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// returns the name of the node
    pub fn name(&self) -> Option<String> {
        self.scene.node_name(self.id)
    }

    /// returns the children of this node
    pub fn children(&self) -> Vec<NodeId> {
        self.scene.children(self.id)
    }

    /// add a node as a child of this node
    pub fn spawn_child<C: Node>(&self, name: impl Into<String>, node: C) -> NodeHandle<'a, C> {
        self.scene.spawn_as_child(name, node, self.id)
    }

    /// merge a different node as a child of this node
    pub fn merge_scene(&self, other: Scene) -> Vec<NodeId> {
        self.scene.merge_as_child(other, self.id)
    }

    pub fn on<E: EventLabel>(
        &self,
        handler: impl FnMut(EventCtx<E, T>) + Send + Sync + 'static,
    ) -> &Self {
        self.scene.on(self.id(), handler);
        self
    }

    /// provides immutible access to this node.
    ///
    /// Multiple reader can access the same node at the same time but blocks if a writer holds the
    /// lock.
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

    /// provides mutible access to this node.
    ///
    /// Only one writer can access a node at a time.
    /// Blocks if any readers or writers hold a lock.
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

/// A hierarchical scene graph for storing and organizing nodes.
///
/// the scene manages the Scene Tree which stores Nodes in a Tree structure meaning Nodes can have
/// children. Nodes are stored internally using a RWLock to allow mutibility because of this borrow
/// checking is runtime managed and calling .write on the same node twice at once will panic.
///
/// # Example
/// ```ignore
/// let scene = Scene::new();
/// let camera = scene.add("main_camera", Camera3D::default());
/// let player = Scene.add("player", Player::default());
/// player.add_child("Tool", Tool::new());
/// ```
///
///
pub struct Scene {
    nodes: RwLock<HashMap<NodeId, NodeStorage>>,

    heirarchy: RwLock<HashMap<NodeId, SceneNode>>,

    events: RwLock<HashMap<NodeId, EventReceiver>>,

    /// ready event queue since nodes added after engine ready wouldnt run ready otherwise and we
    /// dont have context on add
    ready_queue: RwLock<VecDeque<NodeId>>,
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
            events: RwLock::new(HashMap::new()),
            ready_queue: RwLock::new(VecDeque::new()),
        }
    }

    /// Adds a node to the root of the scene with no parents.
    pub fn spawn<T: Node>(&'a self, name: impl Into<String>, node: T) -> NodeHandle<'a, T> {
        self.spawn_with_parent(name, node, None)
    }

    /// Adds a node to the scene with a parent
    pub fn spawn_as_child<T: Node>(
        &'a self,
        name: impl Into<String>,
        node: T,
        parent: NodeId,
    ) -> NodeHandle<'a, T> {
        self.spawn_with_parent(name, node, Some(parent))
    }

    pub fn on<E: EventLabel, N: Node>(
        &self,
        node: NodeId,
        handler: impl FnMut(EventCtx<E, N>) + Send + Sync + 'static,
    ) {
        self.events
            .write()
            .entry(node)
            .or_default()
            .on::<E, N, _>(handler);
    }

    fn spawn_with_parent<T: Node>(
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

        {
            let mut ready_queue = self.ready_queue.write();
            ready_queue.push_back(id);
        }

        NodeHandle {
            id,
            scene: self,
            _ty: PhantomData,
        }
    }

    /// merge a different scene into this one preserving the hierarchy.
    pub fn merge(&self, other: Scene) -> Vec<NodeId> {
        self.merge_as_child_of(other, None)
    }

    /// merge a different scene as a child of a specified node
    pub fn merge_as_child(&self, other: Scene, parent: NodeId) -> Vec<NodeId> {
        self.merge_as_child_of(other, Some(parent))
    }

    fn merge_as_child_of(&self, other: Scene, parent: Option<NodeId>) -> Vec<NodeId> {
        let mut other_hierarchy = other.heirarchy.write();
        let mut other_nodes = other.nodes.write();
        let mut other_events = other.events.write();

        let root_ids: Vec<NodeId> = other_hierarchy
            .iter()
            .filter(|(_, node)| node.parent.is_none())
            .map(|(id, _)| *id)
            .collect();

        {
            let mut self_heirarchy = self.heirarchy.write();
            let mut self_nodes = self.nodes.write();
            let mut self_events = self.events.write();

            for (id, mut scene_node) in other_hierarchy.drain() {
                if scene_node.parent.is_none() {
                    scene_node.parent = parent;
                }
                self_heirarchy.insert(id, scene_node);
            }

            for (id, node_data) in other_nodes.drain() {
                self_nodes.insert(id, node_data);
            }

            for (id, events) in other_events.drain() {
                self_events.insert(id, events);
            }

            self.ready_queue
                .write()
                .append(&mut other.ready_queue.write());

            if let Some(parent_id) = parent
                && let Some(parent_node) = self_heirarchy.get_mut(&parent_id)
            {
                parent_node.children.extend(&root_ids);
            }
        }

        root_ids
    }

    /// get handle to a node via an id
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

    /// get a node by name
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

    /// get the parent of the node
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.heirarchy.read().get(&id).and_then(|n| n.parent)
    }

    /// get the children of the node
    pub fn children(&self, id: NodeId) -> Vec<NodeId> {
        self.heirarchy
            .read()
            .get(&id)
            .map(|n| n.children.clone())
            .unwrap_or_default()
    }

    /// get the name of a node
    pub fn node_name(&self, id: NodeId) -> Option<String> {
        self.heirarchy.read().get(&id).map(|n| n.name.clone())
    }

    /// collects all nodes of a specific type
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

    /// get all the root node ids
    pub fn root_ids(&self) -> Vec<NodeId> {
        let hierarchy = self.heirarchy.read();
        hierarchy
            .iter()
            .filter(|(_, node)| node.parent.is_none())
            .map(|(id, _)| *id)
            .collect()
    }

    /// emit an event to the scene (this will also update world space transforms)
    pub fn emit<E: EventLabel>(&self, event: &E, ctx: &GameContext) {
        for root_id in self.root_ids() {
            self.emit_recursive(root_id, event, ctx);
        }
    }

    fn emit_recursive<E: EventLabel>(&self, id: NodeId, event: &E, ctx: &GameContext) {
        // if an event receiver exist trigger the event to it
        if let Some(events) = self.events.read().get(&id) {
            events.trigger(event, self, id, ctx);
        }

        let children = self.children(id);
        for child_id in children {
            self.emit_recursive(child_id, event, ctx);
        }
    }

    /// goes through every node and updates the world position recursively
    ///
    /// this is done once per frame after update
    pub fn sync_world_transform(&self) {
        for id in self.root_ids() {
            self.sync_world_transform_recursive(id, WorldTransform::default());
        }
    }

    fn sync_world_transform_recursive(&self, id: NodeId, parent_world: WorldTransform) {
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

        drop(node);

        let children = self.children(id);
        for child in children {
            self.sync_world_transform_recursive(child, current_world);
        }
    }

    pub(crate) fn pop_ready_queue(&self, ctx: &GameContext) {
        while let Some(id) = self.ready_queue.write().pop_front() {
            self.emit_to(id, &Ready, ctx);
        }
    }

    /// emit an event to a single node
    pub fn emit_to<E: EventLabel>(&self, id: NodeId, event: &E, ctx: &GameContext) {
        // if an event receiver exist trigger the event to it
        if let Some(events) = self.events.read().get(&id) {
            events.trigger(event, self, id, ctx);
        }
    }

    /// run a callback on each node of a specific type
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

    /// run a callback for each node of a specific type
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

    /// run a callback on each node of a specific type and get the NodeId
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

pub trait SceneBuilder {
    fn build(&mut self) -> Scene;
}

impl<T: SceneBuilder> From<T> for Scene {
    fn from(mut builder: T) -> Self {
        builder.build()
    }
}
