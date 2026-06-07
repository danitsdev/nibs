use crate::findings::{FindingCategory, RiskLevel, SafetyClass};
use crate::rules::Rule;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

const CLAUDE_CODE: &str = include_str!("../../cleaners/dev-ai/claude-code.yaml");
const GEMINI_CLI: &str = include_str!("../../cleaners/dev-ai/gemini-cli.yaml");
const OPENCODE: &str = include_str!("../../cleaners/dev-ai/opencode.yaml");
const ANTIGRAVITY: &str = include_str!("../../cleaners/dev-ai/antigravity.yaml");
const FIREFOX: &str = include_str!("../../cleaners/browsers/firefox.yaml");
const CHROMIUM: &str = include_str!("../../cleaners/browsers/chromium.yaml");
const DISCORD: &str = include_str!("../../cleaners/desktop-apps/discord.yaml");
const SPOTIFY: &str = include_str!("../../cleaners/desktop-apps/spotify.yaml");
const NPM: &str = include_str!("../../cleaners/dev-tools/npm.yaml");
const CARGO: &str = include_str!("../../cleaners/dev-tools/cargo.yaml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanerRecipe {
    pub id: String,
    pub name: String,
    pub category: FindingCategory,
    #[serde(default)]
    pub platforms: Vec<String>,
    #[serde(default)]
    pub running: Vec<String>,
    pub items: Vec<CleanerItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanerItem {
    pub id: String,
    pub name: String,
    pub category: Option<FindingCategory>,
    pub risk: RiskLevel,
    pub safety_class: SafetyClass,
    pub default_action: String,
    pub paths: Vec<String>,
    pub reason: String,
    pub kept: Option<Vec<String>>,
    pub restore: Option<Vec<String>>,
    #[serde(default)]
    pub block_if_running: bool,
}

impl CleanerRecipe {
    fn into_rules(self, running_processes: &HashSet<String>) -> Vec<Rule> {
        let currently_running = matching_running_processes(&self.running, running_processes);

        self.items
            .into_iter()
            .map(|item| Rule {
                id: format!("{}.{}", self.id, item.id),
                name: item.name,
                category: item.category.unwrap_or(self.category),
                risk: item.risk,
                patterns: item.paths,
                reason: item.reason,
                restore: item.restore,
                default_action: Some(item.default_action),
                cleaner_id: Some(self.id.clone()),
                cleaner_name: Some(self.name.clone()),
                safety_class: Some(item.safety_class),
                kept: item.kept,
                block_if_running: item.block_if_running,
                process_names: self.running.clone(),
                running_processes: if item.block_if_running {
                    currently_running.clone()
                } else {
                    Vec::new()
                },
            })
            .collect()
    }
}

pub fn load_all_cleaner_rules() -> Vec<Rule> {
    let mut rules = load_embedded_cleaner_rules();

    if let Ok(extra_dir) = std::env::var("NIBBLE_CLEANERS_DIR") {
        let extra_dir = Path::new(&extra_dir);
        match load_cleaner_rules_from_dir(extra_dir) {
            Ok(mut local_rules) => {
                tracing::info!(
                    "Loaded {} cleaner rules dynamically from {:?}",
                    local_rules.len(),
                    extra_dir
                );
                rules.append(&mut local_rules);
            }
            Err(error) => {
                tracing::warn!("Failed to load extra cleaner catalog: {:?}", error);
            }
        }
    }

    rules
}

pub fn load_embedded_cleaner_rules() -> Vec<Rule> {
    let running_processes = collect_running_process_names();
    let mut rules = Vec::new();
    let sources = [
        CLAUDE_CODE,
        GEMINI_CLI,
        OPENCODE,
        ANTIGRAVITY,
        FIREFOX,
        CHROMIUM,
        DISCORD,
        SPOTIFY,
        NPM,
        CARGO,
    ];

    for (index, source) in sources.iter().enumerate() {
        match serde_yaml::from_str::<CleanerRecipe>(source) {
            Ok(recipe) => rules.extend(recipe.into_rules(&running_processes)),
            Err(error) => {
                tracing::error!(
                    "Failed to parse embedded cleaner recipe #{}: {:?}",
                    index,
                    error
                );
            }
        }
    }

    rules
}

pub fn load_cleaner_rules_from_dir(dir: &Path) -> Result<Vec<Rule>> {
    let running_processes = collect_running_process_names();
    let mut rules = Vec::new();
    load_cleaner_rules_from_dir_inner(dir, &running_processes, &mut rules)?;
    Ok(rules)
}

fn load_cleaner_rules_from_dir_inner(
    dir: &Path,
    running_processes: &HashSet<String>,
    rules: &mut Vec<Rule>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir).context("Failed to read cleaners directory")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            load_cleaner_rules_from_dir_inner(&path, running_processes, rules)?;
        } else if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read cleaner recipe: {:?}", path))?;
            let recipe = serde_yaml::from_str::<CleanerRecipe>(&content)
                .with_context(|| format!("Failed to parse cleaner recipe: {:?}", path))?;
            rules.extend(recipe.into_rules(running_processes));
        }
    }
    Ok(())
}

fn collect_running_process_names() -> HashSet<String> {
    let mut names = HashSet::new();
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return names;
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        if !file_name
            .to_string_lossy()
            .chars()
            .all(|c| c.is_ascii_digit())
        {
            continue;
        }

        let proc_dir = entry.path();
        if let Ok(comm) = std::fs::read_to_string(proc_dir.join("comm")) {
            let comm = comm.trim();
            if !comm.is_empty() {
                names.insert(comm.to_lowercase());
            }
        }

        if let Ok(cmdline) = std::fs::read(proc_dir.join("cmdline")) {
            for part in cmdline.split(|byte| *byte == 0) {
                if part.is_empty() {
                    continue;
                }
                let raw = String::from_utf8_lossy(part);
                if let Some(name) = Path::new(raw.as_ref()).file_name() {
                    names.insert(name.to_string_lossy().to_lowercase());
                }
            }
        }
    }

    names
}

pub fn find_running_processes(expected: &[String]) -> Vec<String> {
    let running_processes = collect_running_process_names();
    matching_running_processes(expected, &running_processes)
}

fn matching_running_processes(
    expected: &[String],
    running_processes: &HashSet<String>,
) -> Vec<String> {
    expected
        .iter()
        .filter(|name| running_processes.contains(&name.to_lowercase()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_cleaners_load_as_rules() {
        let rules = load_embedded_cleaner_rules();
        assert!(rules.iter().any(|rule| rule.id == "claude-code.debug-logs"));
        assert!(rules.iter().any(|rule| rule.id == "firefox.web-cache"));
        assert!(
            rules
                .iter()
                .any(|rule| rule.safety_class == Some(SafetyClass::SecretOrAuth))
        );
    }
}
