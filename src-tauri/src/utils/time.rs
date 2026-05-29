use chrono::Utc;

pub(crate) fn now() -> String {
    Utc::now().to_rfc3339()
}
