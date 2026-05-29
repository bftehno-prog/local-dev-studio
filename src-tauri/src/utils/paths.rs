use std::path::PathBuf;

pub(crate) fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("LocalDevStudio")
}
