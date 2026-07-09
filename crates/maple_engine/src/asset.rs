use std::{
    any::{Any, TypeId},
    collections::HashMap,
    error::Error,
    fmt::Display,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::Duration,
};

use parking_lot::{ArcRwLockReadGuard, ArcRwLockWriteGuard, Mutex, RawRwLock, RwLock};

#[derive(Debug, Clone)]
pub enum LoadErr {
    Import(String),
    IntoAsset(String),
    Missing,
    Timeout,
}

impl Display for LoadErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadErr::Import(e) => {
                write!(f, "failed to import asset: {}", e)
            }
            LoadErr::IntoAsset(e) => {
                write!(f, "failed to convert asset: {}", e)
            }
            LoadErr::Timeout => {
                write!(f, "scene loading timed out")
            }
            LoadErr::Missing => {
                write!(f, "asset is missing")
            }
        }
    }
}

impl Error for LoadErr {}

/// A asset loader is a factory that is used to create Assets
///
/// it can contains resources such as a render device that is needed during loading but not usage
pub trait AssetLoader: Any + Send + Sync + 'static {
    type Asset: Asset<Loader = Self>;
}

/// This loader can load an Asset from a file
pub trait FileLoader: AssetLoader {
    fn load_path(&self, path: &Path, library: &AssetLibrary) -> Result<Self::Asset, LoadErr>;
}

/// An Asset is type of resource which is loaded at runtime and can be placed around a scene or
/// within a node
pub trait Asset: Send + Sync + 'static {
    type Loader: AssetLoader<Asset = Self>;
}

pub struct AssetRef<T: Asset> {
    guard: ArcRwLockReadGuard<RawRwLock, T>,
}

impl<T: Asset> Deref for AssetRef<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

pub struct AssetMut<T: Asset> {
    guard: ArcRwLockWriteGuard<RawRwLock, T>,
}

impl<T: Asset> Deref for AssetMut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<T: Asset> DerefMut for AssetMut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

pub trait IntoAsset<T: Asset>: Send + Sync + 'static {
    fn into_asset(self, loader: &T::Loader, library: &AssetLibrary) -> Result<T, LoadErr>;
}

impl<T: Asset> IntoAsset<T> for T {
    fn into_asset(
        self,
        _loader: &<T as Asset>::Loader,
        _library: &AssetLibrary,
    ) -> Result<T, LoadErr> {
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub enum AssetId {
    Path(PathBuf),
    Id(u64),
}
impl AssetId {
    pub fn new_id() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        AssetId::Id(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug)]
pub struct AssetHandle<T: Asset> {
    pub id: AssetId,
    _ty: PhantomData<T>,
}

impl<T: Asset> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        AssetHandle {
            id: self.id.clone(),
            _ty: PhantomData,
        }
    }
}

struct AssetSlot<T: Asset> {
    state: AssetState<T>,
    pending: Vec<Box<dyn FnOnce(&mut T) + Send>>,
}

impl<T: Asset> AssetSlot<T> {
    fn loading() -> Self {
        Self {
            state: AssetState::Loading,
            pending: Vec::new(),
        }
    }

    fn loaded(asset: T) -> Self {
        Self {
            state: AssetState::Loaded(Arc::new(RwLock::new(asset))),
            pending: Vec::new(),
        }
    }
}

pub struct AssetLibrary {
    states: Arc<Mutex<HashMap<AssetId, Arc<dyn Any + Send + Sync>>>>,
    loaders: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl Clone for AssetLibrary {
    fn clone(&self) -> Self {
        Self {
            states: Arc::clone(&self.states),
            loaders: Arc::clone(&self.loaders),
        }
    }
}

#[derive(Debug)]
pub enum AssetState<T: Asset> {
    Loading,
    Loaded(Arc<RwLock<T>>),
    Error(LoadErr),
}

impl<T: Asset> AssetState<T> {
    pub fn asset(&self) -> Option<Arc<RwLock<T>>> {
        self.clone().into()
    }

    pub fn is_loaded(&self) -> bool {
        match self {
            AssetState::Loaded(_) => true,
            _ => false,
        }
    }
    pub fn is_loading(&self) -> bool {
        match self {
            AssetState::Loading => true,
            _ => false,
        }
    }
}

impl<T: Asset> From<AssetState<T>> for Option<Arc<RwLock<T>>> {
    fn from(value: AssetState<T>) -> Self {
        match value {
            AssetState::Loading => None,
            AssetState::Loaded(asset) => Some(asset),
            AssetState::Error(_) => None,
        }
    }
}

impl<T: Asset> Clone for AssetState<T> {
    fn clone(&self) -> Self {
        match self {
            AssetState::Loading => AssetState::Loading,
            AssetState::Loaded(asset) => AssetState::Loaded(Arc::clone(asset)),
            AssetState::Error(err) => AssetState::Error(err.clone()),
        }
    }
}

impl Default for AssetLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetLibrary {
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
            loaders: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    pub fn map<S, T, F>(&self, source: AssetHandle<S>, f: F) -> AssetHandle<T>
    where
        S: Asset,
        T: Asset,
        F: Fn(&S) -> Option<AssetHandle<T>> + Send + Sync + 'static,
    {
        let id = AssetId::new_id();
        let state = Arc::new(Mutex::new(AssetSlot::<T>::loading()));
        {
            let mut states = self.states.lock();
            states.insert(id.clone(), state.clone());
        }
        let library = self.clone();
        let id_clone = id.clone();
        thread::spawn(move || {
            let inner_handle = loop {
                match library.status(&source) {
                    AssetState::Loaded(source) => match f(&source.read()) {
                        Some(handle) => break handle,
                        None => {
                            state.lock().state = AssetState::Error(LoadErr::Missing);
                            return;
                        }
                    },
                    AssetState::Error(err) => {
                        state.lock().state = AssetState::Error(err);
                        return;
                    }
                    AssetState::Loading => thread::sleep(Duration::from_millis(4)),
                }
            };

            let inner_states = library.states.lock();
            let Some(inner_state_any) = inner_states.get(&inner_handle.id).cloned() else {
                return;
            };
            drop(inner_states);

            let Ok(inner_slot) = inner_state_any.clone().downcast::<Mutex<AssetSlot<T>>>() else {
                return;
            };

            // Merge any pending mutations queued on the outer handle into the
            // real (inner) slot, so they aren't lost when we swap the alias.
            {
                let mut outer_lock = state.lock();
                let outer_pending = std::mem::take(&mut outer_lock.pending);
                drop(outer_lock);

                let mut inner_lock = inner_slot.lock();
                match &mut inner_lock.state {
                    AssetState::Loaded(lock) => {
                        let mut data = lock.write();
                        for f in outer_pending {
                            f(&mut data);
                        }
                    }
                    AssetState::Loading => {
                        inner_lock.pending.extend(outer_pending);
                    }
                    AssetState::Error(_) => {
                        // nothing to apply mutations to
                    }
                }
            }

            let mut states = library.states.lock();
            states.insert(id_clone, inner_state_any);
        });
        AssetHandle {
            id,
            _ty: PhantomData,
        }
    }

    pub fn modify<T: Asset>(
        &self,
        handle: &AssetHandle<T>,
        f: impl FnOnce(&mut T) + Send + 'static,
    ) -> bool {
        let states = self.states.lock();
        let Some(slot_any) = states.get(&handle.id) else {
            return false;
        };
        let Some(slot) = slot_any.downcast_ref::<Mutex<AssetSlot<T>>>() else {
            return false;
        };
        let mut slot_lock = slot.lock();

        match &mut slot_lock.state {
            AssetState::Loaded(lock) => {
                f(&mut lock.write());
                true
            }
            AssetState::Loading => {
                slot_lock.pending.push(Box::new(f));
                true
            }
            AssetState::Error(_) => false,
        }
    }

    fn finish_slot<T: Asset>(slot: &Mutex<AssetSlot<T>>, result: Result<T, LoadErr>) {
        let mut slot_lock = slot.lock();
        slot_lock.state = match result {
            Ok(asset) => AssetState::Loaded(Arc::new(RwLock::new(asset))),
            Err(err) => AssetState::Error(err),
        };

        // split borrow so we can drain `pending` while mutating through `state`
        let AssetSlot { state, pending } = &mut *slot_lock;
        if let AssetState::Loaded(lock) = state {
            let mut data = lock.write();
            for f in pending.drain(..) {
                f(&mut data);
            }
        }
        // if it errored, pending mutations are just dropped — nothing to apply them to
    }

    pub fn is_loaded<T: Asset>(&self, handle: &AssetHandle<T>) -> bool {
        let states = self.states.lock();
        let Some(state_any) = states.get(&handle.id) else {
            return false;
        };
        let Some(state) = state_any.downcast_ref::<Mutex<AssetSlot<T>>>() else {
            return false;
        };

        state.lock().state.is_loaded()
    }

    pub fn is_loading<T: Asset>(&self, handle: &AssetHandle<T>) -> bool {
        let states = self.states.lock();
        let Some(state_any) = states.get(&handle.id) else {
            return false;
        };
        let Some(state) = state_any.downcast_ref::<Mutex<AssetSlot<T>>>() else {
            return false;
        };

        state.lock().state.is_loading()
    }

    pub fn register_loader<L: AssetLoader>(&self, loader: L) {
        let type_id = TypeId::of::<L::Asset>();
        let mut loaders = self.loaders.write();
        loaders.insert(type_id, Arc::new(loader));
    }

    fn get_loader<T: Asset>(&self) -> Option<Arc<T::Loader>> {
        let loaders = self.loaders.read();
        loaders
            .get(&TypeId::of::<T>())
            .and_then(|l| l.clone().downcast::<T::Loader>().ok())
    }

    /// register a already loaded asset
    pub fn register<T: Asset>(&self, asset: T) -> AssetHandle<T> {
        let id = AssetId::new_id();
        let state = Arc::new(Mutex::new(AssetSlot::loaded(asset)));
        let mut state_lock = self.states.lock();
        state_lock.insert(id.clone(), state);

        AssetHandle {
            id,
            _ty: PhantomData,
        }
    }

    fn spawn_loader<T: Asset>(
        &self,
        path: PathBuf,
        loader: Arc<T::Loader>,
        slot: Arc<Mutex<AssetSlot<T>>>,
        library: AssetLibrary,
    ) where
        T::Loader: FileLoader,
    {
        thread::spawn(move || {
            let result = loader.load_path(&path, &library);
            Self::finish_slot(&slot, result);
        });
    }

    pub fn load<T: Asset>(&self, path: impl AsRef<Path>) -> AssetHandle<T>
    where
        T::Loader: FileLoader,
    {
        let path = path.as_ref().to_path_buf();
        let id = AssetId::Path(path.clone());

        let mut states = self.states.lock();
        if states.contains_key(&id) {
            return AssetHandle {
                id,
                _ty: PhantomData,
            };
        }

        let loader = self
            .get_loader::<T>()
            .expect("Loader not registered for this asset type");

        let state = Arc::new(Mutex::new(AssetSlot::<T>::loading()));
        states.insert(id.clone(), state.clone());
        drop(states);

        self.spawn_loader::<T>(path.clone(), loader, state, self.clone());

        AssetHandle {
            id,
            _ty: PhantomData,
        }
    }

    pub fn status<T: Asset>(&self, handle: &AssetHandle<T>) -> AssetState<T> {
        let states = self.states.lock();
        if let Some(state_any) = states.get(&handle.id)
            && let Some(state) = state_any.downcast_ref::<Mutex<AssetSlot<T>>>()
        {
            return state.lock().state.clone();
        }
        AssetState::Error(LoadErr::Missing)
    }

    pub fn get<T: Asset>(&self, handle: &AssetHandle<T>) -> Option<AssetRef<T>> {
        let states = self.states.lock();
        let slot_any = states.get(&handle.id)?;
        let slot = slot_any.downcast_ref::<Mutex<AssetSlot<T>>>()?;
        let slot_lock = slot.lock();

        match &slot_lock.state {
            AssetState::Loaded(lock) => Some(AssetRef {
                guard: lock.read_arc(),
            }),
            _ => None,
        }
    }

    pub fn get_mut<T: Asset>(&self, handle: &AssetHandle<T>) -> Option<AssetMut<T>> {
        let states = self.states.lock();
        let slot_any = states.get(&handle.id)?;
        let slot = slot_any.downcast_ref::<Mutex<AssetSlot<T>>>()?;
        let slot_lock = slot.lock();

        match &slot_lock.state {
            AssetState::Loaded(lock) => Some(AssetMut {
                guard: lock.write_arc(),
            }),
            _ => None,
        }
    }

    fn spawn_converter<T: Asset>(
        &self,
        source: impl IntoAsset<T>,
        loader: Arc<T::Loader>,
        slot: Arc<Mutex<AssetSlot<T>>>,
        library: AssetLibrary,
    ) {
        thread::spawn(move || {
            let result = source.into_asset(&loader, &library);
            Self::finish_slot(&slot, result);
        });
    }

    pub fn add<T: Asset>(&self, source: impl IntoAsset<T>) -> AssetHandle<T> {
        let id = AssetId::new_id();

        let loader = self
            .get_loader::<T>()
            .expect("Loader not registered for this asset");

        let state = Arc::new(Mutex::new(AssetSlot::loading()));
        {
            let mut states_lock = self.states.lock();
            states_lock.insert(id.clone(), state.clone());
        }

        self.spawn_converter(source, loader, state, self.clone());

        AssetHandle {
            id,
            _ty: PhantomData,
        }
    }
}
