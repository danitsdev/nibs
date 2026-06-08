use crate::findings::{FindingCategory, RiskLevel, SafetyClass};
use crate::rules::Rule;
use crate::rules::matches_pattern;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const CLAUDE_CODE: &str = include_str!("../../cleaners/dev-ai/claude-code.yaml");
const AI_DESKTOP: &str = include_str!("../../cleaners/dev-ai/ai-desktop.yaml");
const LOCAL_AI_MODELS: &str = include_str!("../../cleaners/dev-ai/local-ai-models.yaml");
const GEMINI_CLI: &str = include_str!("../../cleaners/dev-ai/gemini-cli.yaml");
const OPENCODE: &str = include_str!("../../cleaners/dev-ai/opencode.yaml");
const ANTIGRAVITY: &str = include_str!("../../cleaners/dev-ai/antigravity.yaml");
const FIREFOX: &str = include_str!("../../cleaners/browsers/firefox.yaml");
const CHROMIUM: &str = include_str!("../../cleaners/browsers/chromium.yaml");
const LIBREWOLF: &str = include_str!("../../cleaners/browsers/librewolf.yaml");
const MICROSOFT_EDGE: &str = include_str!("../../cleaners/browsers/microsoft-edge.yaml");
const OPERA: &str = include_str!("../../cleaners/browsers/opera.yaml");
const VIVALDI: &str = include_str!("../../cleaners/browsers/vivaldi.yaml");
const VSCODE_FAMILY: &str = include_str!("../../cleaners/code-editors/vscode-family.yaml");
const ZED: &str = include_str!("../../cleaners/code-editors/zed.yaml");
const JETBRAINS: &str = include_str!("../../cleaners/code-editors/jetbrains.yaml");
const SUBLIME_TEXT: &str = include_str!("../../cleaners/code-editors/sublime-text.yaml");
const SLACK: &str = include_str!("../../cleaners/communication/slack.yaml");
const SIGNAL: &str = include_str!("../../cleaners/communication/signal.yaml");
const ELEMENT: &str = include_str!("../../cleaners/communication/element.yaml");
const ZOOM: &str = include_str!("../../cleaners/communication/zoom.yaml");
const TELEGRAM: &str = include_str!("../../cleaners/communication/telegram.yaml");
const MICROSOFT_TEAMS: &str = include_str!("../../cleaners/communication/microsoft-teams.yaml");
const DISCORD: &str = include_str!("../../cleaners/desktop-apps/discord.yaml");
const SPOTIFY: &str = include_str!("../../cleaners/desktop-apps/spotify.yaml");
const NPM: &str = include_str!("../../cleaners/dev-tools/npm.yaml");
const CARGO: &str = include_str!("../../cleaners/dev-tools/cargo.yaml");
const ANDROID_STUDIO: &str = include_str!("../../cleaners/dev-tools/android-studio.yaml");
const JAVASCRIPT_PACKAGE_MANAGERS: &str =
    include_str!("../../cleaners/dev-tools/javascript-package-managers.yaml");
const CLOUD_CLI: &str = include_str!("../../cleaners/dev-tools/cloud-cli.yaml");
const DOCKER_TOOLING: &str = include_str!("../../cleaners/dev-tools/docker-tooling.yaml");
const VIRTUALBOX: &str = include_str!("../../cleaners/dev-tools/virtualbox.yaml");
const VLC: &str = include_str!("../../cleaners/media/vlc.yaml");
const TRANSMISSION: &str = include_str!("../../cleaners/media/transmission.yaml");
const QBITTORRENT: &str = include_str!("../../cleaners/media/qbittorrent.yaml");
const THUNDERBIRD: &str = include_str!("../../cleaners/productivity/thunderbird.yaml");
const FIGMA: &str = include_str!("../../cleaners/productivity/figma.yaml");
const STEAM: &str = include_str!("../../cleaners/gaming/steam.yaml");
const OBS_STUDIO: &str = include_str!("../../cleaners/creative/obs-studio.yaml");
const BLENDER: &str = include_str!("../../cleaners/creative/blender.yaml");
const KRITA: &str = include_str!("../../cleaners/creative/krita.yaml");
const GIMP: &str = include_str!("../../cleaners/creative/gimp.yaml");
const INKSCAPE: &str = include_str!("../../cleaners/creative/inkscape.yaml");
const KDENLIVE: &str = include_str!("../../cleaners/creative/kdenlive.yaml");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanerRecipe {
    pub id: String,
    pub name: String,
    pub category: FindingCategory,
    #[serde(default)]
    pub platforms: Vec<String>,
    pub detect: CleanerDetect,
    pub items: Vec<CleanerItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CleanerDetect {
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub desktop_files: Vec<String>,
    #[serde(default)]
    pub paths: Vec<String>,
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
    pub restore: Option<Vec<String>>,
}

impl CleanerRecipe {
    fn into_rules(self) -> Vec<Rule> {
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
            })
            .collect()
    }
}

impl CleanerDetect {
    fn is_detected(&self) -> bool {
        self.commands.iter().any(|command| command_exists(command))
            || self
                .desktop_files
                .iter()
                .any(|pattern| desktop_file_exists(pattern))
            || self.paths.iter().any(|path| expand_home(path).exists())
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
    let mut rules = Vec::new();

    for (index, source) in embedded_cleaner_sources().iter().enumerate() {
        match load_cleaner_rules_from_source(source, true) {
            Ok(mut recipe_rules) => rules.append(&mut recipe_rules),
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

fn embedded_cleaner_sources() -> &'static [&'static str] {
    &[
        CLAUDE_CODE,
        AI_DESKTOP,
        LOCAL_AI_MODELS,
        GEMINI_CLI,
        OPENCODE,
        ANTIGRAVITY,
        FIREFOX,
        CHROMIUM,
        LIBREWOLF,
        MICROSOFT_EDGE,
        OPERA,
        VIVALDI,
        VSCODE_FAMILY,
        ZED,
        JETBRAINS,
        SUBLIME_TEXT,
        SLACK,
        SIGNAL,
        ELEMENT,
        ZOOM,
        TELEGRAM,
        MICROSOFT_TEAMS,
        DISCORD,
        SPOTIFY,
        NPM,
        CARGO,
        ANDROID_STUDIO,
        JAVASCRIPT_PACKAGE_MANAGERS,
        CLOUD_CLI,
        DOCKER_TOOLING,
        VIRTUALBOX,
        VLC,
        TRANSMISSION,
        QBITTORRENT,
        THUNDERBIRD,
        FIGMA,
        STEAM,
        OBS_STUDIO,
        BLENDER,
        KRITA,
        GIMP,
        INKSCAPE,
        KDENLIVE,
    ]
}

pub fn load_cleaner_rules_from_dir(dir: &Path) -> Result<Vec<Rule>> {
    let mut rules = Vec::new();
    load_cleaner_rules_from_dir_inner(dir, &mut rules)?;
    Ok(rules)
}

fn load_cleaner_rules_from_dir_inner(dir: &Path, rules: &mut Vec<Rule>) -> Result<()> {
    for entry in std::fs::read_dir(dir).context("Failed to read cleaners directory")? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            load_cleaner_rules_from_dir_inner(&path, rules)?;
        } else if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read cleaner recipe: {:?}", path))?;
            rules.extend(
                load_cleaner_rules_from_source(&content, true)
                    .with_context(|| format!("Failed to parse cleaner recipe: {:?}", path))?,
            );
        }
    }
    Ok(())
}

fn load_cleaner_rules_from_source(source: &str, require_detection: bool) -> Result<Vec<Rule>> {
    let recipe = serde_yaml::from_str::<CleanerRecipe>(source)?;
    if require_detection && !recipe.detect.is_detected() {
        return Ok(Vec::new());
    }
    Ok(recipe.into_rules())
}

fn command_exists(command: &str) -> bool {
    let command_path = Path::new(command);
    if command_path.components().count() > 1 {
        return is_executable_file(command_path);
    }

    std::env::var_os("PATH")
        .map(|paths| {
            std::env::split_paths(&paths).any(|dir| {
                let candidate = dir.join(command);
                is_executable_file(&candidate)
            })
        })
        .unwrap_or(false)
}

fn is_executable_file(path: &Path) -> bool {
    path.metadata()
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn desktop_file_exists(pattern: &str) -> bool {
    desktop_search_dirs()
        .into_iter()
        .flat_map(|dir| std::fs::read_dir(dir).into_iter().flatten().flatten())
        .any(|entry| matches_pattern(&entry.path(), pattern))
}

fn desktop_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share/applications"));
    }
    if let Some(data_dirs) = std::env::var_os("XDG_DATA_DIRS") {
        dirs.extend(std::env::split_paths(&data_dirs).map(|dir| dir.join("applications")));
    } else {
        dirs.push(PathBuf::from("/usr/local/share/applications"));
        dirs.push(PathBuf::from("/usr/share/applications"));
    }
    dirs
}

fn expand_home(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/")
        && let Some(home) = std::env::var_os("HOME")
    {
        return PathBuf::from(home).join(stripped);
    }
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_cleaners_load_as_rules() {
        let rules = load_cleaner_rules_from_source(CLAUDE_CODE, false).unwrap();
        assert!(rules.iter().any(|rule| rule.id == "claude-code.debug-logs"));
        assert!(
            rules
                .iter()
                .any(|rule| rule.safety_class == Some(SafetyClass::SecretOrAuth))
        );
    }

    #[test]
    fn all_embedded_cleaner_sources_parse() {
        for source in embedded_cleaner_sources() {
            let recipe = serde_yaml::from_str::<CleanerRecipe>(source).unwrap();
            assert!(!recipe.id.is_empty());
            assert!(!recipe.items.is_empty());
        }
    }

    #[test]
    fn detected_recipe_becomes_rules() {
        let marker = std::env::temp_dir().join("nibble_recipe_detected_marker");
        let _ = std::fs::remove_dir_all(&marker);
        std::fs::create_dir_all(&marker).unwrap();

        let source = format!(
            r#"
id: sample
name: Sample
category: language_cache
platforms: [linux]
detect:
  paths: ["{}"]
items:
  - id: cache
    name: Sample cache
    risk: safe
    safety_class: safe
    default_action: clean
    paths: ["**/.sample-cache"]
    reason: "Temporary sample cache."
"#,
            marker.display()
        );

        let rules = load_cleaner_rules_from_source(&source, true).unwrap();
        let _ = std::fs::remove_dir_all(&marker);

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "sample.cache");
    }

    #[test]
    fn undetected_recipe_is_skipped() {
        let missing = std::env::temp_dir().join("nibble_recipe_missing_marker");
        let _ = std::fs::remove_dir_all(&missing);
        let source = format!(
            r#"
id: sample
name: Sample
category: language_cache
platforms: [linux]
detect:
  paths: ["{}"]
items:
  - id: cache
    name: Sample cache
    risk: safe
    safety_class: safe
    default_action: clean
    paths: ["**/.sample-cache"]
    reason: "Temporary sample cache."
"#,
            missing.display()
        );

        let rules = load_cleaner_rules_from_source(&source, true).unwrap();
        assert!(rules.is_empty());
    }
}
