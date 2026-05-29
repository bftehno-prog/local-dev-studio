use std::{
    collections::HashMap,
    path::PathBuf,
    process::Child,
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub(crate) struct ManagedProcesses {
    pub(crate) children: HashMap<String, Child>,
}

pub(crate) struct AppState {
    pub(crate) db_path: PathBuf,
    pub(crate) processes: Arc<Mutex<ManagedProcesses>>,
}
