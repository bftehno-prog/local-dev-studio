use std::{
    collections::HashMap,
    path::PathBuf,
    process::Child,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::JoinHandle,
};

#[derive(Default)]
pub(crate) struct ManagedProcesses {
    pub(crate) children: HashMap<String, Child>,
}

pub(crate) struct ManagedProxy {
    pub(crate) port: u16,
    pub(crate) target_port: u16,
    pub(crate) stop: Arc<AtomicBool>,
    pub(crate) handle: Option<JoinHandle<()>>,
}

impl ManagedProxy {
    pub(crate) fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[derive(Default)]
pub(crate) struct ManagedProxies {
    pub(crate) proxies: HashMap<String, ManagedProxy>,
}

pub(crate) struct AppState {
    pub(crate) db_path: PathBuf,
    pub(crate) processes: Arc<Mutex<ManagedProcesses>>,
    pub(crate) proxies: Arc<Mutex<ManagedProxies>>,
}
