# Nibble Agent Guide

This file is the shared source of truth for humans and AI agents working on Nibble. Put machine-specific notes in `AGENTS.local.md`; it is gitignored.

## Project

Nibble is a safe, Rust-powered terminal cleaner for Linux developers. It finds rebuildable development junk, generated artifacts, caches, temporary files, app remnants, and recoverable disk waste, then explains what it found before anything is cleaned.

Core promise:

> Nibble never guesses. It explains before it cleans.

## Repository Map

- `src/main.rs` - process entrypoint and tracing setup.
- `src/cli.rs` - clap command definitions and CLI action resolution.
- `src/app.rs` - top-level command orchestration.
- `src/tui/` - ratatui UI model, rendering, event handling, and app screens.
- `src/scanner/` - filesystem walking, size accounting, duplicate detection, and scan progress.
- `src/rules/` - YAML rule loading and pattern matching.
- `src/findings/` - finding categories, risk levels, and cleanup metadata.
- `src/cleaner/` - cleanup execution and Trash routing.
- `src/safety/` - scan scope resolution and protected Linux path policy.
- `src/report/` - JSON report generation.
- `src/analyze/` - interactive disk analyzer.
- `src/status/` - live system status data collection.
- `src/uninstall/` - installed app discovery and remnant cleanup.
- `src/doctor/` - local environment checks.
- `rules/` - built-in cleanup rules.
- `cleaners/` - app/tool cleaner catalog entries.
- `docs/` - safety, rules, roadmap, and mascot behavior docs.

## Commands

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run
cargo run -- scan . --json
cargo run -- scan . --dry-run
cargo run -- analyze .
cargo run -- status
```

Use `cargo fmt` only for files you intentionally touched unless the task is a repo-wide formatting pass.

## Critical Safety Rules

- Never add permanent deletion as a default behavior.
- Prefer moving recoverable cleanup targets to Trash.
- Do not follow symlinks by default.
- Do not cross filesystem boundaries during `/` scans unless an explicit future flag enables it.
- Keep `/proc`, `/sys`, `/dev`, `/run`, `/boot`, and `/lost+found` protected.
- Treat system config, databases, Docker volumes, package-manager state, secrets, and unknown large files as review/risky/info-only.
- Docker volumes must never be cleaned automatically.
- Duplicate findings must never be part of automatic clean-all.
- Permission errors, broken symlinks, deleted files during scan, and invalid UTF-8 paths must not panic normal workflows.

## Working Rules

- Prefer small, explicit Rust modules over broad abstractions.
- Keep rule files human-readable and auditable.
- When adding a cleanup rule, include reason, risk, category, default action, and restore hints when applicable.
- Use structured path APIs instead of ad hoc string manipulation when possible.
- Keep TUI colors theme-aware. Prefer ANSI/named colors over fixed RGB unless there is a strong reason.
- Mascot changes must keep the fixed 3-row, 12-column contract described in `docs/mascot_animations.md`.
- Do not vendor comparison projects or downloaded binaries into this repo.
- Do not add AI attribution trailers to commits.

## Test Expectations

Run targeted tests while developing, then run `cargo test` before handing off.

Riskier areas need focused tests:

- `src/safety/` - path scope and protected path tests.
- `src/scanner/` - size, duplicate, symlink, permission, and filtering behavior.
- `src/rules/` - rule matching and YAML loading.
- `src/report/` - JSON output compatibility.
- `src/cleaner/` and `src/uninstall/` - Trash routing and dry-run behavior.
- `src/tui/` - mascot consistency and interaction regressions.

## Documentation

Update docs with behavior changes:

- Safety boundary changes: `docs/safety.md`.
- Rule format or catalog changes: `docs/rules.md`.
- UI mascot changes: `docs/mascot_animations.md`.
- Milestone or product direction changes: `docs/roadmap.md`.
- Contributor process changes: `CONTRIBUTING.md`.

## GitHub And Collaboration

- Keep issues public for normal bugs and feature requests.
- Use private security reporting for deletion-boundary, path traversal, privilege, or release-integrity issues.
- Pull requests touching cleanup sinks, protected path logic, Trash behavior, symlink handling, or app remnant discovery need careful review and tests.
- Do not push, tag, or publish releases unless the maintainer explicitly asks.
