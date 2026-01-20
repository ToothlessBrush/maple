use std::{
    any::{Any, TypeId},
    collections::HashMap,
    error::Error,
    fmt::Display,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
    thread,
};

use parking_lot::{Mutex, RwLock};

#[derive(Debug, Clone)]
pub enum LoadErr {
    Import(String),
    Missing,
    Timeout,
}

impl Display for LoadErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadErr::Import(e) => {
                write!(f, "{}", e)
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

pub trait AssetLoader: Any + Send + Sync + 'static {
    type Asset: Asset<Loader = Self>;

    fn load(&self, path: &Path, library: &AssetLibrary) -> Result<Arc<Self::Asset>, LoadErr>;
}

pub trait Asset: Send + Sync + 'static {
    type Loader: AssetLoader<Asset = Self>;
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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

#[derive(Clone, Debug)]
pub struct AssetHandle<T: Asset> {
    id: AssetId,
    _ty: PhantomData<T>,
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
    Loaded(Arc<T>),
    Error(LoadErr),
}

impl<T: Asset> AssetState<T> {
    pub fn asset(&self) -> Option<Arc<T>> {
        self.clone().into()
    }
}

impl<T: Asset> From<AssetState<T>> for Option<Arc<T>> {
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

impl AssetLibrary {
    pub fn new() -> Self {
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
            loaders: Arc::new(RwLock::new(HashMap::new())),
        }
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
        let state = Arc::new(Mutex::new(AssetState::Loaded(Arc::new(asset))));
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
        state: Arc<Mutex<AssetState<T>>>,
        library: AssetLibrary,
    ) {
        thread::spawn(move || {
            let result = loader.load(&path, &library);
            let mut state_lock = state.lock();
            *state_lock = match result {
                Ok(asset) => AssetState::Loaded(asset),
                Err(err) => AssetState::Error(err),
            };
        });
    }

    pub fn load<T: Asset>(&self, path: impl AsRef<Path>) -> AssetHandle<T> {
        let path = path.as_ref().to_path_buf();
        let id = AssetId::Path(path.clone());

        let states = self.states.lock();
        if states.contains_key(&id) {
            drop(states);
            return AssetHandle {
                id,
                _ty: PhantomData,
            };
        }
        drop(states);

        let loader = self
            .get_loader::<T>()
            .expect("Loader not registered for this asset type");

        let state = Arc::new(Mutex::new(AssetState::<T>::Loading));
        {
            let mut states_lock = self.states.lock();
            states_lock.insert(id.clone(), state.clone());
        }

        self.spawn_loader::<T>(path.clone(), loader, state, self.clone());

        AssetHandle {
            id,
            _ty: PhantomData,
        }
    }

    pub fn get<T: Asset>(&self, handle: &AssetHandle<T>) -> AssetState<T> {
        let states = self.states.lock();
        if let Some(state_any) = states.get(&handle.id) {
            if let Some(state) = state_any.downcast_ref::<Mutex<AssetState<T>>>() {
                return state.lock().clone();
            }
        }
        AssetState::Error(LoadErr::Missing)
    }
}
