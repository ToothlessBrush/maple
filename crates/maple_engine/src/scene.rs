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
    asset::{Asset, AssetHandle, AssetLibrary, AssetStatus},
    nodes::{Instanceable, node::IntoNode},
    platform::SendSync,
    prelude::{Event, EventCtx, EventReceiver, Ready, node_transform::WorldTransform},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct NodeId(u64);

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeId {
    pub const NULL: NodeId = NodeId(0);

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        NodeId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct SceneNode {
    _id: NodeId,
    name: Option<String>,
    children: Vec<NodeId>,
    parent: Option<NodeId>,
    type_id: TypeId,
}

type NodeStorage = Arc<RwLock<Box<dyn Node>>>;

#[derive(Debug)]
pub struct NodeHandle<T: Node> {
    id: NodeId,
    _ty: PhantomData<T>,
}

impl<T: Node> Clone for NodeHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _ty: PhantomData,
        }
    }
}

impl<T: Node> Copy for NodeHandle<T> {}

impl<T: Node> From<NodeHandle<T>> for NodeId {
    fn from(value: NodeHandle<T>) -> Self {
        value.id
    }
}

impl<T: Node> NodeHandle<T> {
    pub const NULL: NodeHandle<T> = NodeHandle {
        id: NodeId::NULL,
        _ty: PhantomData,
    };

    pub fn is_null(&self) -> bool {
        self.id.is_null()
    }

    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            _ty: PhantomData,
        }
    }
}

/// A view into a node within a scene
///
/// Allows access to the scene from the perspective of this node for adding children with
/// [`Self::spawn_child`], attaching events with [`Self::on`], or accessing the node with
/// [`Self::get_ref`]/[`Self::get_mut`]
///
/// lifetime is tied to the scene
pub struct NodeView<'a, T: Node> {
    id: NodeId,
    scene: &'a Scene,
    _ty: PhantomData<T>,
}

impl<'a, T: Node> Clone for NodeView<'a, T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            scene: self.scene,
            _ty: PhantomData,
        }
    }
}

pub trait OptionNodeViewExt<'a, T: Node> {
    fn get_mut(self) -> Option<NodeMut<T>>;
    fn get_ref(self) -> Option<NodeRef<T>>;
}

impl<'a, T> OptionNodeViewExt<'a, T> for Option<NodeView<'a, T>>
where
    T: Node,
{
    fn get_ref(self) -> Option<NodeRef<T>> {
        self.map(|node| node.get_ref())
    }

    fn get_mut(self) -> Option<NodeMut<T>> {
        self.map(|node| node.get_mut())
    }
}

impl<'a, T: Node> Copy for NodeView<'a, T> {}

/// RAII guard for immutible access to a node.
pub struct NodeRef<T: Node> {
    guard: ArcRwLockReadGuard<RawRwLock, Box<dyn Node>>,
    _ty: PhantomData<T>,
}

/// RAII guard for mutible access to a node.
pub struct NodeMut<T: Node> {
    guard: ArcRwLockWriteGuard<RawRwLock, Box<dyn Node>>,
    _ty: PhantomData<T>,
}

impl<T: Node> Deref for NodeRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.as_any().downcast_ref::<T>().unwrap()
    }
}

impl<T: Node> Deref for NodeMut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.guard.as_any().downcast_ref::<T>().unwrap()
    }
}

impl<T: Node> DerefMut for NodeMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.as_any_mut().downcast_mut::<T>().unwrap()
    }
}

impl<'a, T: Node> NodeView<'a, T> {
    pub fn handle(&self) -> NodeHandle<T> {
        NodeHandle {
            id: self.id,
            _ty: PhantomData,
        }
    }

    /// returns the id of this node
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// returns the name of the node
    pub fn name(&self) -> Option<String> {
        self.scene.node_name(self.id)
    }

    /// wraps a function that takes the handle as a arguement and returns self
    ///
    /// this is useful when you want to use [`Self::spawn_child`] but want to keep the handle
    /// afterwords without creating a variable
    pub fn with<F>(&self, f: F) -> &Self
    where
        F: Fn(&NodeView<T>),
    {
        f(self);
        self
    }

    /// returns the children of this node
    pub fn children_ids(&self) -> Vec<NodeId> {
        self.scene.children_ids(self.id)
    }

    /// returns the nodes parent id if there is one
    pub fn parent_id(&self) -> Option<NodeId> {
        self.scene.parent_id(self.id)
    }

    /// returns the children of this node with the given type
    pub fn children<C>(&self) -> Vec<NodeHandle<C>>
    where
        C: Node,
    {
        self.scene.children(self.id)
    }

    /// returns the parent of this node if it exists and the type matches
    pub fn parent<C>(&self) -> Option<NodeHandle<C>>
    where
        C: Node,
    {
        self.scene.parent(self.id)
    }

    /// add a node as a child of this node
    pub fn spawn_child<C, M>(&'a self, node: C) -> NodeView<'a, C::Node>
    where
        C: IntoNode<M>,
    {
        self.scene.spawn_as_child(node.into_node(), self.id)
    }

    /// like [`Self::spawn_child`] but attaches a name to the child for fetching later with
    /// [`Scene::get_by_name`]
    ///
    /// node names are not unique and a node with the same type and name will have the scene just return
    /// the first match within the scene
    pub fn spawn_child_with_name<C, M>(
        &self,
        name: impl Into<String>,
        node: C,
    ) -> NodeView<'a, C::Node>
    where
        C: IntoNode<M>,
    {
        self.scene
            .spawn_as_child_with_name(name, node.into_node(), self.id)
    }

    /// merge a different node as a child of this node
    pub fn child_scene(&self, other: Scene) -> Vec<NodeId> {
        self.scene.merge_as_child(other, self.id)
    }

    /// merge a scene asset into the scene
    pub fn child_asset<A: SceneAsset>(&self, handle: AssetHandle<A>) {
        self.scene.merge_asset_as_child(handle, self.id);
    }

    /// add an event to the node
    ///
    /// this function takes a callback which takes [`EventCtx`] as an arguement which contains event
    /// info and a refrence to the node scene and resources.
    ///
    /// # Example
    /// ```
    ///  # use glam::Vec3;
    ///  # use maple_engine::prelude::*;
    ///  let scene = Scene::default();
    ///
    ///  scene.spawn(Empty::default()).on::<Update>(|ctx| {
    ///      // mut refrence to the empty node
    ///      let mut node = ctx.node_mut();
    ///      // get a game resource
    ///      let input = ctx.get_resource::<Input>();
    ///
    ///      if input.keys.contains(&KeyCode::KeyW) {
    ///          let forward = node.transform.get_forward_vector();
    ///          // use event fields through the context
    ///          node.transform.position += Vec3::new(forward.x * ctx.dt, 0.0, forward.z * ctx.dt);
    ///      }
    ///  });
    /// ```
    pub fn on<E: Event>(
        &self,
        handler: impl FnMut(EventCtx<E, T>) + Send + Sync + 'static,
    ) -> &Self {
        self.scene.on(self.handle(), handler);
        self
    }

    /// provides immutible access to this node.
    ///
    /// Multiple reader can access the same node at the same time but blocks if a writer holds the
    /// lock.
    pub fn get_ref(&self) -> NodeRef<T> {
        let node_lock = {
            let nodes = self.scene.nodes.read();
            Arc::clone(nodes.get(&self.id).expect("Node not found"))
        };

        // Use read_arc instead of read - it takes ownership semantics of the Arc
        let guard = RwLock::read_arc(&node_lock);
        NodeRef {
            guard,
            _ty: PhantomData,
        }
    }

    /// provides mutible access to this node.
    ///
    /// Only one writer can access a node at a time.
    /// Blocks if any readers or writers hold a lock.
    pub fn get_mut(&self) -> NodeMut<T> {
        let node_lock = {
            let nodes = self.scene.nodes.read();
            Arc::clone(nodes.get(&self.id).expect("Node not found"))
        };

        let guard = RwLock::write_arc(&node_lock);

        NodeMut {
            guard,
            _ty: PhantomData,
        }
    }
}

type PendingAssetEntry = (Box<dyn PendingSceneAsset>, Option<NodeId>);

/// A hierarchical scene graph for storing and organizing nodes.
///
/// the scene manages the Scene Tree which stores Nodes in a Tree structure meaning Nodes can have
/// children. Nodes are stored internally using a RWLock to allow mutibility because of this borrow
/// checking is runtime managed and calling .write on the same node twice at once will panic.
///
/// # Example
/// ```
/// # use maple_engine::prelude::*;
/// // spawn a camera with a child container
/// let scene = Scene::default();
/// let camera = scene.spawn(Empty::default())
///     .spawn_child(Container::new(10.0));
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

    pending_assets: RwLock<Vec<PendingAssetEntry>>,
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
            pending_assets: RwLock::new(Vec::new()),
        }
    }

    /// Adds a node to the root of the scene with no parents.
    pub fn spawn<T, M>(&'a self, node: T) -> NodeView<'a, T::Node>
    where
        T: IntoNode<M>,
    {
        self.spawn_with_parent::<T::Node, String>(None, node.into_node(), None)
    }

    pub fn spawn_with_name<T, M>(
        &'a self,
        name: impl Into<String>,
        node: T,
    ) -> NodeView<'a, T::Node>
    where
        T: IntoNode<M>,
    {
        self.spawn_with_parent(Some(name), node.into_node(), None)
    }

    /// Adds a node to the scene with a parent
    pub fn spawn_as_child<T: Node, N: Into<NodeId>>(
        &'a self,
        node: T,
        parent: N,
    ) -> NodeView<'a, T> {
        self.spawn_with_parent::<T, String>(None, node, Some(parent.into()))
    }

    pub fn spawn_as_child_with_name<T: Node, N: Into<NodeId>>(
        &'a self,
        name: impl Into<String>,
        node: T,
        parent: N,
    ) -> NodeView<'a, T> {
        self.spawn_with_parent(Some(name), node, Some(parent.into()))
    }

    /// add an event to a node
    pub fn on<E: Event, N: Node>(
        &self,
        node: NodeHandle<N>,
        handler: impl FnMut(EventCtx<E, N>) + SendSync + 'static,
    ) {
        self.events
            .write()
            .entry(node.id)
            .or_default()
            .on::<E, N, _>(handler);
    }

    fn spawn_with_parent<T: Node, N: Into<String>>(
        &'a self,
        name: Option<N>,
        node: T,
        parent: Option<NodeId>,
    ) -> NodeView<'a, T> {
        let id = NodeId::new();

        let scene_node = SceneNode {
            _id: id,
            name: name.map(|s| s.into()),
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

        NodeView {
            id,
            scene: self,
            _ty: PhantomData,
        }
    }

    /// merge a different scene into this one preserving the hierarchy.
    pub fn merge(&self, other: impl Into<Scene>) -> Vec<NodeId> {
        self.merge_as_child_of(other.into(), None)
    }

    /// merge a different scene as a child of a specified node
    pub fn merge_as_child(&self, other: impl Into<Scene>, parent: NodeId) -> Vec<NodeId> {
        self.merge_as_child_of(other.into(), Some(parent))
    }

    /// merge a scene without blocking the load
    pub fn merge_asset<T: Asset + SceneAsset>(&self, handle: AssetHandle<T>) {
        let pending = TypedPendingAsset { handle };
        self.pending_assets.write().push((Box::new(pending), None));
    }

    /// merge a scene assent as a child of a node
    pub fn merge_asset_as_child<T: Asset + SceneAsset, N: Into<NodeId>>(
        &self,
        handle: AssetHandle<T>,
        parent: N,
    ) {
        let pending = TypedPendingAsset { handle };
        self.pending_assets
            .write()
            .push((Box::new(pending), Some(parent.into())));
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

            self.pending_assets
                .write()
                .append(&mut other.pending_assets.write());

            if let Some(parent_id) = parent
                && let Some(parent_node) = self_heirarchy.get_mut(&parent_id)
            {
                parent_node.children.extend(&root_ids);
            }
        }

        root_ids
    }

    pub fn get_view<T: Node>(&self, handle: NodeHandle<T>) -> Option<NodeView<'_, T>> {
        if handle.is_null() {
            return None;
        }
        let hierarchy = self.heirarchy.read();
        let node = hierarchy.get(&handle.id)?;
        (node.type_id == TypeId::of::<T>()).then(|| NodeView {
            id: handle.id,
            scene: self,
            _ty: PhantomData,
        })
    }

    pub fn get_view_from_id<T: Node>(&self, id: NodeId) -> Option<NodeView<'_, T>> {
        if id.is_null() {
            return None;
        }
        let hierarchy = self.heirarchy.read();
        let node = hierarchy.get(&id)?;
        (node.type_id == TypeId::of::<T>()).then(|| NodeView {
            id: id,
            scene: self,
            _ty: PhantomData,
        })
    }

    pub fn get_ref<T: Node>(&'a self, handle: NodeHandle<T>) -> Option<NodeRef<T>> {
        let node_lock = {
            let nodes = self.nodes.read();
            Arc::clone(nodes.get(&handle.id)?)
        };

        let guard = RwLock::read_arc(&node_lock);

        Some(NodeRef {
            guard,
            _ty: PhantomData,
        })
    }

    pub fn get_mut<T: Node>(&'a self, handle: NodeHandle<T>) -> Option<NodeMut<T>> {
        let node_lock = {
            let nodes = self.nodes.read();
            Arc::clone(nodes.get(&handle.id)?)
        };

        let guard = RwLock::write_arc(&node_lock);

        Some(NodeMut {
            guard,
            _ty: PhantomData,
        })
    }

    /// get a node by name
    pub fn get_by_name<T: Node>(&'a self, name: &str) -> Option<NodeHandle<T>> {
        let hierarchy = self.heirarchy.read();
        let type_id = TypeId::of::<T>();

        for (id, scene_node) in hierarchy.iter() {
            let Some(node_name) = &scene_node.name else {
                continue;
            };
            if node_name == name && scene_node.type_id == type_id {
                return Some(NodeHandle {
                    id: *id,
                    _ty: PhantomData,
                });
            }
        }
        None
    }

    /// get the parent of the node
    pub fn parent_id(&self, id: NodeId) -> Option<NodeId> {
        self.heirarchy.read().get(&id).and_then(|n| n.parent)
    }

    pub fn parent<T, N>(&self, id: N) -> Option<NodeHandle<T>>
    where
        T: Node,
        N: Into<NodeId>,
    {
        let parent_id = self.parent_id(id.into())?;
        let heirarchy = self.heirarchy.read();
        let node = heirarchy.get(&parent_id)?;

        (node.type_id == TypeId::of::<T>()).then(|| NodeHandle {
            id: parent_id,
            _ty: PhantomData,
        })
    }

    /// get the children of the node
    pub fn children_ids(&self, id: NodeId) -> Vec<NodeId> {
        self.heirarchy
            .read()
            .get(&id)
            .map(|n| n.children.clone())
            .unwrap_or_default()
    }

    pub fn children<T, N>(&self, id: N) -> Vec<NodeHandle<T>>
    where
        T: Node,
        N: Into<NodeId>,
    {
        let target_type = TypeId::of::<T>();
        let heirarchy = self.heirarchy.read();

        let Some(node) = heirarchy.get(&id.into()) else {
            return Vec::new();
        };

        node.children
            .iter()
            .filter(|child_id| {
                heirarchy
                    .get(child_id)
                    .map(|n| n.type_id == target_type)
                    .unwrap_or(false)
            })
            .map(|&id| NodeHandle {
                id,
                _ty: PhantomData,
            })
            .collect()
    }

    /// get the name of a node
    pub fn node_name(&self, id: NodeId) -> Option<String> {
        self.heirarchy
            .read()
            .get(&id)
            .map(|n| n.name.clone())
            .flatten()
    }

    /// collects all nodes of a specific type
    pub fn collect<T: Node>(&'a self) -> Vec<NodeView<'a, T>> {
        let heirarchy = self.heirarchy.read();
        let type_id = TypeId::of::<T>();

        heirarchy
            .iter()
            .filter(|(_, node)| node.type_id == type_id)
            .map(|(id, _)| NodeView {
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
    pub fn emit<E: Event>(&self, event: &E, ctx: &GameContext) {
        for root_id in self.root_ids() {
            self.emit_recursive(root_id, event, ctx);
        }
    }

    fn emit_recursive<E: Event>(&self, id: NodeId, event: &E, ctx: &GameContext) {
        // if an event receiver exist trigger the event to it
        if let Some(events) = self.events.read().get(&id) {
            events.trigger(event, self, id, ctx);
        }

        let children = self.children_ids(id);
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

        let children = self.children_ids(id);
        for child in children {
            self.sync_world_transform_recursive(child, current_world);
        }
    }

    pub(crate) fn pop_ready_queue(&self, ctx: &GameContext) {
        loop {
            let id = self.ready_queue.write().pop_front();
            let Some(id) = id else { break };
            self.emit_to(id, &Ready, ctx);
        }
    }

    /// emit an event to a single node
    pub fn emit_to<E: Event>(&self, id: NodeId, event: &E, ctx: &GameContext) {
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

    /// polls pending assets and adds them if ready
    pub fn poll_async(&mut self, assets: &AssetLibrary) {
        // Take the whole pending list out from behind the lock so we don't
        // hold any guard while `poll_and_load` runs — it can re-enter and
        // write to `self.pending_assets` (e.g. via merge_as_child_of).
        let mut pending = {
            let mut guard = self.pending_assets.write();
            if guard.is_empty() {
                return;
            }
            std::mem::take(&mut *guard)
        }; // lock dropped here

        log::trace!("Polling {} scene assets", pending.len());

        let mut loaded_indices = Vec::new();
        for (i, (asset, parent)) in pending.iter().enumerate() {
            if asset.poll_and_load(assets, self, *parent) {
                loaded_indices.push(i);
            }
        }

        for &i in loaded_indices.iter().rev() {
            log::info!("merging loaded scene into scene");
            pending.swap_remove(i);
        }

        // Anything still not-ready goes back. Note poll_and_load may have
        // pushed *new* pending assets onto self.pending_assets in the
        // meantime (e.g. a merge that queues more loads) — extend rather
        // than overwrite so those aren't lost.
        if !pending.is_empty() {
            self.pending_assets.write().extend(pending);
        }
    }
}

trait PendingSceneAsset: Send + Sync {
    /// Poll this asset and load it into the scene if ready
    /// Returns true if done (loaded or errored), false if still loading
    fn poll_and_load(&self, assets: &AssetLibrary, scene: &Scene, parent: Option<NodeId>) -> bool;
}

// Concrete implementation that wraps a typed handle
struct TypedPendingAsset<T: Asset + SceneAsset> {
    handle: AssetHandle<T>,
}

impl<T: Asset + SceneAsset> PendingSceneAsset for TypedPendingAsset<T> {
    fn poll_and_load(&self, assets: &AssetLibrary, scene: &Scene, parent: Option<NodeId>) -> bool {
        match assets.get_status(&self.handle) {
            AssetStatus::Loaded(asset) => {
                asset.load(scene, parent);
                true
            }
            AssetStatus::Error(e) => {
                log::error!("Failed to load scene asset: {:?}", e);
                true
            }
            AssetStatus::Removed => true,
            AssetStatus::Loading => false, // Still loading - keep in pending
            _ => false,
        }
    }
}
pub trait SceneAsset: Asset {
    fn load(&self, scene: &Scene, parent: Option<NodeId>);
}

pub trait IntoScene<Marker> {
    fn into_scene(self, assets: &AssetLibrary) -> Scene;
}

pub struct SceneMarker;
// M is the return type of the function since its generic
pub struct NoArgsMarker<M>(PhantomData<M>);
pub struct AssetsMarker<M>(PhantomData<M>);
pub struct BuilderMarker;

impl IntoScene<SceneMarker> for Scene {
    fn into_scene(self, _assets: &AssetLibrary) -> Scene {
        self
    }
}

impl<F, S, M> IntoScene<NoArgsMarker<M>> for F
where
    F: Fn() -> S,
    S: IntoScene<M>,
{
    fn into_scene(self, assets: &AssetLibrary) -> Scene {
        self().into_scene(assets)
    }
}

impl<F, S, M> IntoScene<AssetsMarker<M>> for F
where
    F: Fn(&AssetLibrary) -> S,
    S: IntoScene<M>,
{
    fn into_scene(self, assets: &AssetLibrary) -> Scene {
        self(assets).into_scene(assets)
    }
}

pub trait SceneBuilder {
    fn build(self, assets: &AssetLibrary) -> Scene;
}

impl<T: SceneBuilder> IntoScene<BuilderMarker> for T {
    fn into_scene(self, assets: &AssetLibrary) -> Scene {
        self.build(assets)
    }
}

type InstanceableNodeStorage = Arc<RwLock<Box<dyn Instanceable>>>;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct InstanceId(NodeId);

pub struct InstanceSceneNode {
    _id: InstanceId,
    name: String,
    children: Vec<InstanceId>,
    parent: Option<InstanceId>,
    type_id: TypeId,
}

pub struct InstancableScene {
    nodes: RwLock<HashMap<InstanceId, InstanceableNodeStorage>>,

    heirarchy: RwLock<HashMap<InstanceId, InstanceSceneNode>>,
}

impl Default for InstancableScene {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> InstancableScene {
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            heirarchy: RwLock::new(HashMap::new()),
        }
    }

    pub fn instance(&self) -> Scene {
        let nodes = self.nodes.read();
        let hierarchy = self.heirarchy.read();

        let id_map: HashMap<InstanceId, NodeId> =
            hierarchy.keys().map(|&iid| (iid, NodeId::new())).collect();

        let mut new_nodes = HashMap::with_capacity(nodes.len());
        for (iid, node_storage) in nodes.iter() {
            let new_id = id_map[iid];
            let cloned: Box<dyn Node> = node_storage.read().instance();
            new_nodes.insert(new_id, Arc::new(RwLock::new(cloned)));
        }

        let mut new_hierarchy = HashMap::with_capacity(hierarchy.len());
        for (iid, scene_node) in hierarchy.iter() {
            let new_id = id_map[iid];
            new_hierarchy.insert(
                new_id,
                SceneNode {
                    _id: new_id,
                    name: Some(scene_node.name.clone()),
                    children: scene_node.children.iter().map(|c| id_map[c]).collect(),
                    parent: scene_node.parent.map(|p| id_map[&p]),
                    type_id: scene_node.type_id,
                },
            );
        }

        let new_ready_queue: VecDeque<NodeId> = id_map.values().copied().collect();

        Scene {
            nodes: RwLock::new(new_nodes),
            heirarchy: RwLock::new(new_hierarchy),
            events: RwLock::new(HashMap::new()),
            ready_queue: RwLock::new(new_ready_queue),
            pending_assets: RwLock::new(Vec::new()),
        }
    }

    /// Adds a node to the root of the scene with no parents.
    pub fn spawn<T: Instanceable>(&'a self, name: impl Into<String>, node: T) -> InstanceId {
        self.spawn_with_parent(name, node, None)
    }

    /// Adds a node to the scene with a parent
    pub fn spawn_as_child<T: Instanceable>(
        &'a self,
        name: impl Into<String>,
        node: T,
        parent: InstanceId,
    ) -> InstanceId {
        self.spawn_with_parent(name, node, Some(parent))
    }

    fn spawn_with_parent<T: Instanceable>(
        &'a self,
        name: impl Into<String>,
        node: T,
        parent: Option<InstanceId>,
    ) -> InstanceId {
        let id = InstanceId(NodeId::new());

        let scene_node = InstanceSceneNode {
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

        id
    }

    /// get the parent of the node
    pub fn parent(&self, id: InstanceId) -> Option<InstanceId> {
        self.heirarchy.read().get(&id).and_then(|n| n.parent)
    }

    /// get the children of the node
    pub fn children(&self, id: InstanceId) -> Vec<InstanceId> {
        self.heirarchy
            .read()
            .get(&id)
            .map(|n| n.children.clone())
            .unwrap_or_default()
    }

    /// get the name of a node
    pub fn node_name(&self, id: InstanceId) -> Option<String> {
        self.heirarchy.read().get(&id).map(|n| n.name.clone())
    }

    /// get all the root node ids
    pub fn root_ids(&self) -> Vec<InstanceId> {
        let hierarchy = self.heirarchy.read();
        hierarchy
            .iter()
            .filter(|(_, node)| node.parent.is_none())
            .map(|(id, _)| *id)
            .collect()
    }
}
