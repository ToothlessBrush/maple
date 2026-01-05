use std::{
    any::TypeId,
    collections::HashMap,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use parking_lot::{ArcRwLockReadGuard, ArcRwLockWriteGuard, RawRwLock, RwLock};

use crate::{Node, scene};

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct NodeId(u64);

impl NodeId {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        NodeId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct SceneNode {
    id: NodeId,
    name: String,
    children: Vec<NodeId>,
    parent: Option<NodeId>,
    type_id: TypeId,
}

pub struct Scene {
    nodes: RwLock<HashMap<NodeId, Arc<RwLock<Box<dyn Node>>>>>,

    heirarchy: RwLock<HashMap<NodeId, SceneNode>>,

    root_id: NodeId,
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

impl<'a> Scene {
    pub fn new() -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            heirarchy: RwLock::new(HashMap::new()),
            root_id: NodeId::new(),
        }
    }

    pub fn add<T: Node>(&'a self, name: impl Into<String>, node: T) -> NodeHandle<'a, T> {
        self.add_with_parent(name, node, None)
    }

    pub fn add_child<T: Node>(
        &self,
        name: impl Into<String>,
        node: T,
        parent: NodeId,
    ) -> NodeHandle<T> {
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
            id,
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

    pub fn get<T: Node>(&self, id: NodeId) -> Option<NodeHandle<T>> {
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

    pub fn get_by_name<T: Node>(&self, name: &str) -> Option<NodeHandle<T>> {
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
}
