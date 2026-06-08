pub mod loader;
pub mod matcher;

use crate::findings::{FindingCategory, RiskLevel, SafetyClass};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub category: FindingCategory,
    pub risk: RiskLevel,
    pub patterns: Vec<String>,
    pub reason: String,
    pub restore: Option<Vec<String>>,
    pub default_action: Option<String>,
    pub cleaner_id: Option<String>,
    pub cleaner_name: Option<String>,
    pub safety_class: Option<SafetyClass>,
}

pub use loader::load_all_rules;
pub use matcher::matches_pattern;
