use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Safe,
    Review,
    Risky,
    Info,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Safe => write!(f, "safe"),
            RiskLevel::Review => write!(f, "review"),
            RiskLevel::Risky => write!(f, "risky"),
            RiskLevel::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum FindingCategory {
    RebuildableDependency,
    BuildArtifact,
    FrameworkCache,
    LanguageCache,
    TemporaryFile,
    Trash,
    ExactDuplicate,
    LargeUnknown,
    DockerInfo,
    SystemInfoOnly,
    DevAiAgent,
    BrowserCache,
    DesktopAppCache,
}

impl std::fmt::Display for FindingCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindingCategory::RebuildableDependency => write!(f, "rebuildable_dependency"),
            FindingCategory::BuildArtifact => write!(f, "build_artifact"),
            FindingCategory::FrameworkCache => write!(f, "framework_cache"),
            FindingCategory::LanguageCache => write!(f, "language_cache"),
            FindingCategory::TemporaryFile => write!(f, "temporary_file"),
            FindingCategory::Trash => write!(f, "trash"),
            FindingCategory::ExactDuplicate => write!(f, "exact_duplicate"),
            FindingCategory::LargeUnknown => write!(f, "large_unknown"),
            FindingCategory::DockerInfo => write!(f, "docker_info"),
            FindingCategory::SystemInfoOnly => write!(f, "system_info_only"),
            FindingCategory::DevAiAgent => write!(f, "dev_ai_agent"),
            FindingCategory::BrowserCache => write!(f, "browser_cache"),
            FindingCategory::DesktopAppCache => write!(f, "desktop_app_cache"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SafetyClass {
    Safe,
    UsuallySafe,
    Rebuildable,
    UserData,
    SecretOrAuth,
}

impl std::fmt::Display for SafetyClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SafetyClass::Safe => write!(f, "safe"),
            SafetyClass::UsuallySafe => write!(f, "usually_safe"),
            SafetyClass::Rebuildable => write!(f, "rebuildable"),
            SafetyClass::UserData => write!(f, "user_data"),
            SafetyClass::SecretOrAuth => write!(f, "secret_or_auth"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub category: FindingCategory,
    pub risk: RiskLevel,
    pub rule_id: String,
    pub rule_name: String,
    pub reason: String,
    pub restore: Option<Vec<String>>,
    pub default_action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleaner_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleaner_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_class: Option<SafetyClass>,
    pub last_modified: Option<DateTime<Utc>>,
}

impl Finding {
    pub fn is_recommended_clean(&self) -> bool {
        self.risk == RiskLevel::Safe
            && self
                .default_action
                .as_deref()
                .is_some_and(|action| action.eq_ignore_ascii_case("clean"))
            && self
                .safety_class
                .is_none_or(|class| class == SafetyClass::Safe)
            && self.category != FindingCategory::ExactDuplicate
            && self.category != FindingCategory::DockerInfo
            && self.category != FindingCategory::SystemInfoOnly
    }

    pub fn is_safe_clean_candidate(&self) -> bool {
        self.risk == RiskLevel::Safe
            && self
                .safety_class
                .is_none_or(|class| matches!(class, SafetyClass::Safe | SafetyClass::UsuallySafe))
            && self.category != FindingCategory::ExactDuplicate
            && self.category != FindingCategory::DockerInfo
            && self.category != FindingCategory::SystemInfoOnly
    }
}
