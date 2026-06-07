# Nibble Product Roadmap

Nibble is built to be:
> **ncdu + BleachBit + Czkawka-lite + Mole vibes, but Linux-first, Rust, trash-first, and explainable.**

To avoid feature bloat, Nibble concentrates strictly on safety, transparent explanations, and developer-first disk reclamation.

---

## [v0.1] Polished Core (Current MVP)
Focuses on delivering a fully functional, safe developer cleaner, interactive tree analyzer, app remnants hunter, and basic telemetry:

- **Scan Engine**: Custom compiler-embedded and local YAML rules (`rules/`), recursive inactivity age filter (`--min-age`), and parallel filesystem walking.
- **TUI/Text Scan**: Checkbox checklist navigation in Ratatui TUI, plain text summaries, and structured `--json` output.
- **Interactive Analyzer (`nibs analyze`)**: Navigable directory tree sorted by size (ncdu-style) with percentage-of-parent visual bars.
- **Remnants Uninstaller (`nibs uninstall <app>`)**: Scans user config, cache, local share, snaps, and flatpak folders. Dedupes nested paths.
- **Telemetry Dashboard (`nibs status`)**: Visualizes memory pressure, CPU load, mount points, and reclaimable Docker sizes.
- **Safety**: Safe system mounts blacklist (`/proc`, `/sys`, etc.) and FreeDesktop-compliant trash integration.

---

## [v0.2] Trust & Audit (Next Release)
Focuses on auditing, transparent risk assessment, and system health checks:

- [ ] **Enhanced Explain Screen**: Expand the TUI details panel with clear color-coded indicators of what is deleted and what is safe.
- [ ] **Dynamic Risk Scoring**: Calculate a numerical/badge risk score per finding based on directory safety and age.
- [ ] **Cleanup History Log**: Write a persistent log to `~/.config/nibble/history.json` tracking exactly when files were moved to trash, their paths, sizes, and categories.
- [ ] **Restore Guidance**: Provide a CLI helper command or TUI instructions showing how to restore folders back from the FreeDesktop trash.
- [ ] **`nibs doctor` command**: Environment validator verifying systemd journal size constraints, Docker daemon connectivity, flatpak/snap paths, and disk write permissions.

---

## [v0.3] Linux Assistant (Read-only Recommendations)
Focuses on actionable optimization guidelines for the Linux system layer without executing destructive actions automatically:

- [ ] **Package Cleanup Assistant**: Detects package manager caches (APT, DNF, Pacman) and displays the exact commands to optimize them (e.g. `paccache -rk2`, `dnf clean packages`, `apt autoremove`).
- [ ] **Systemd Journal Vacuuming Guidelines**: Recommends size/time vacuum commands (like `journalctl --vacuum-time=7d`) if log caches exceed thresholds.
- [ ] **Docker Advanced Analyzer**: Recommends volume and builder cache pruning actions.
- [ ] **System Services telemetry**: Highlights failed systemd user/system services or heavy startup applications.
