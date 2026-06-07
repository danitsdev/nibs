use crate::findings::Finding;
use crate::scanner::ScanWarning;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct ScanReport {
    pub scan_path: PathBuf,
    pub timestamp: DateTime<Utc>,
    pub total_findings: usize,
    pub total_warnings: usize,
    pub total_reclaimable_bytes: u64,
    pub findings: Vec<Finding>,
    pub warnings: Vec<ScanWarning>,
}

/// Generates a JSON string representing the scan report.
pub fn generate_json_report(
    scan_path: PathBuf,
    findings: &[Finding],
    warnings: &[ScanWarning],
) -> Result<String> {
    let total_reclaimable_bytes = findings.iter().map(|f| f.size_bytes).sum();
    let report = ScanReport {
        scan_path,
        timestamp: Utc::now(),
        total_findings: findings.len(),
        total_warnings: warnings.len(),
        total_reclaimable_bytes,
        findings: findings.to_vec(),
        warnings: warnings.to_vec(),
    };

    let json = serde_json::to_string_pretty(&report)?;
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::findings::{FindingCategory, RiskLevel};

    #[test]
    fn test_generate_json_report() {
        let path = PathBuf::from("/home/user/workspace");
        let findings = vec![Finding {
            path: PathBuf::from("/home/user/workspace/node_modules"),
            size_bytes: 1024,
            category: FindingCategory::RebuildableDependency,
            risk: RiskLevel::Safe,
            rule_id: "node_modules".to_string(),
            rule_name: "Node modules".to_string(),
            reason: "Node modules dependency folder".to_string(),
            restore: Some(vec!["npm install".to_string()]),
            default_action: Some("review".to_string()),
            cleaner_id: None,
            cleaner_name: None,
            safety_class: None,
            kept: None,
            block_if_running: false,
            process_names: Vec::new(),
            running_processes: Vec::new(),
            last_modified: None,
        }];
        let warnings = vec![ScanWarning {
            path: Some(PathBuf::from("/home/user/workspace/locked")),
            message: "Permission denied".to_string(),
        }];

        let result = generate_json_report(path, &findings, &warnings);
        assert!(result.is_ok());
        let json_str = result.unwrap();

        // Assert json contains keys
        assert!(json_str.contains("total_findings"));
        assert!(json_str.contains("node_modules"));
        assert!(json_str.contains("Permission denied"));
    }
}
