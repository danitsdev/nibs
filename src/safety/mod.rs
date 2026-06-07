pub mod protected_paths;
pub mod scope;

pub use protected_paths::{is_protected_path, is_restricted_path};
pub use scope::ScanScope;
