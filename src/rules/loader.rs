use crate::rules::Rule;
use anyhow::{Context, Result};
use std::path::Path;

const DEFAULT_NODE: &str = include_str!("../../rules/node.yaml");
const DEFAULT_PYTHON: &str = include_str!("../../rules/python.yaml");
const DEFAULT_RUST: &str = include_str!("../../rules/rust.yaml");
const DEFAULT_GO: &str = include_str!("../../rules/go.yaml");
const DEFAULT_LINUX: &str = include_str!("../../rules/linux-cache.yaml");
const DEFAULT_JAVA: &str = include_str!("../../rules/java.yaml");
const DEFAULT_DEVELOPER_EXTRAS: &str = include_str!("../../rules/developer-extras.yaml");
const DEFAULT_DEEP_CLEAN: &str = include_str!("../../rules/deep-clean.yaml");

/// Loads embedded YAML rules.
pub fn load_embedded_rules(include_deep: bool) -> Vec<Rule> {
    let mut rules = Vec::new();
    let sources = [
        DEFAULT_NODE,
        DEFAULT_PYTHON,
        DEFAULT_RUST,
        DEFAULT_GO,
        DEFAULT_LINUX,
        DEFAULT_JAVA,
        DEFAULT_DEVELOPER_EXTRAS,
    ];

    for (i, src) in sources.iter().enumerate() {
        match serde_yaml::from_str::<Vec<Rule>>(src) {
            Ok(mut parsed) => rules.append(&mut parsed),
            Err(e) => {
                // We trace the error but don't panic so the app remains resilient
                tracing::error!("Failed to parse embedded rule source #{}: {:?}", i, e);
            }
        }
    }
    if include_deep {
        match serde_yaml::from_str::<Vec<Rule>>(DEFAULT_DEEP_CLEAN) {
            Ok(mut parsed) => rules.append(&mut parsed),
            Err(e) => tracing::error!("Failed to parse embedded deep-clean rules: {:?}", e),
        }
    }
    rules
}

/// Loads YAML rules from a specific directory.
pub fn load_rules_from_dir(dir: &Path, include_deep: bool) -> Result<Vec<Rule>> {
    let mut rules = Vec::new();
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir).context("Failed to read rules directory")? {
            let entry = entry?;
            let path = entry.path();
            if !include_deep
                && path
                    .file_name()
                    .is_some_and(|name| name == "deep-clean.yaml" || name == "deep-clean.yml")
            {
                continue;
            }
            if path
                .extension()
                .is_some_and(|ext| ext == "yaml" || ext == "yml")
            {
                let content = std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read rule file: {:?}", path))?;
                let mut parsed = serde_yaml::from_str::<Vec<Rule>>(&content)
                    .with_context(|| format!("Failed to parse rule file: {:?}", path))?;
                rules.append(&mut parsed);
            }
        }
    }
    Ok(rules)
}

/// Tries to load rules from the local `rules/` directory, falling back to embedded rules if not found or empty.
pub fn load_all_rules(include_deep: bool) -> Vec<Rule> {
    let local_rules_dir = Path::new("rules");
    let cleaner_rules = crate::cleaners::load_all_cleaner_rules();

    if local_rules_dir.is_dir() {
        match load_rules_from_dir(local_rules_dir, include_deep) {
            Ok(rules) if !rules.is_empty() => {
                let mut all_rules = cleaner_rules;
                all_rules.extend(rules);
                tracing::info!(
                    "Loaded {} rules dynamically from cleaners and rules/",
                    all_rules.len()
                );
                return all_rules;
            }
            Ok(_) => {
                tracing::warn!("Local rules/ directory is empty. Falling back to embedded rules.");
            }
            Err(e) => {
                tracing::warn!(
                    "Error loading local rules: {:?}. Falling back to embedded rules.",
                    e
                );
            }
        }
    }
    let mut all_rules = cleaner_rules;
    all_rules.extend(load_embedded_rules(include_deep));
    tracing::info!("Loaded {} default embedded rules", all_rules.len());
    all_rules
}
