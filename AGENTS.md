# Nibble Agent Guide

This file is the shared source of truth for humans and AI agents working on Nibble. Put machine-specific notes in `AGENTS.local.md`; it is gitignored.

## Project Standard

Nibble is a safe, Rust-powered terminal cleaner for Linux developers. It finds rebuildable development junk, generated artifacts, app caches, temporary files, app remnants, duplicate candidates, and recoverable disk waste, then explains what it found before anything is cleaned.

Core promise:

> Nibble never guesses. It explains before it cleans.

The public bar is a focused, contributor-friendly project in the spirit of tools like Mole: clear command surface, fast workflows, strong safety defaults, readable docs, and a cleaner catalog that is easy to audit.

## Product Shape

- `Smart Clean` is the everyday path. It should be fast and may preselect only clearly safe findings.
- `Deep Clean` is review-first. It surfaces older, heavier, or more sensitive targets and must not auto-select findings.
- `Analyze Disk` is for manual inspection of unknown large folders.
- `Apps & Leftovers` is for app-specific remnant review before cleanup.
- `Status`, `Doctor`, `Trash`, and `Optimize` support maintenance without broad destructive behavior.

## Repository Map

- `src/main.rs` - process entrypoint and tracing setup.
- `src/cli.rs` - clap command definitions and CLI action resolution.
- `src/app.rs` - top-level command orchestration.
- `src/tui/` - ratatui UI model, rendering, event handling, and app screens.
- `src/scanner/` - filesystem walking, size accounting, duplicate detection, and scan progress.
- `src/rules/` - YAML rule loading and pattern matching.
- `src/findings/` - finding categories, risk levels, safety classes, and cleanup metadata.
- `src/cleaner/` - cleanup execution and Trash routing.
- `src/safety/` - scan scope resolution, protected Linux path policy, and path allow/deny helpers.
- `src/report/` - JSON report generation.
- `src/analyze/` - interactive disk analyzer.
- `src/status/` - live system status data collection.
- `src/uninstall/` - installed app discovery and remnant cleanup.
- `src/doctor/` - local environment checks.
- rules/ - generic built-in cleanup rules.
- cleaners/ - app/tool cleaner catalog entries.

## Commands

```bash
make verify

cargo fmt --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test

cargo run
cargo run -- clean --dry-run
cargo run -- scan . --json
cargo run -- deep --json
cargo run -- analyze .
cargo run -- status
cargo run -- doctor
```

Use `cargo fmt` only for files you intentionally touched unless the task is a repo-wide formatting pass.

## Safety Invariants

- Never add permanent deletion as a default behavior.
- Prefer moving recoverable cleanup targets to Trash.
- Do not follow symlinks by default.
- Do not cross filesystem boundaries during `/` scans unless an explicit future flag enables it.
- Keep `/proc`, `/sys`, `/dev`, `/run`, `/boot`, and `/lost+found` protected.
- Treat system config, databases, Docker/Podman volumes, package-manager state, secrets, sessions, and unknown large files as review/risky/info-only.
- Docker and Podman volumes must never be cleaned automatically.
- Duplicate findings must never be part of automatic clean-all.
- `default_action: never` is a hard block, not a suggestion.
- `Deep Clean` must select nothing automatically.
- System optimization actions must not run sudo, purge RAM caches, delete logs directly, or rewrite databases without an explicit maintainer-approved flow.
- Private keys, tokens, credentials, and secret-looking files must never be committed or used as cleanup test fixtures.
- Permission errors, broken symlinks, deleted files during scan, and invalid UTF-8 paths must not panic normal workflows.

## Rule And Recipe Standards

Generic cleanup patterns belong in `rules/`. App/tool-specific patterns belong in `cleaners/` and should activate only after lightweight detection by command, desktop file, or known path.

Every cleanup rule or cleaner item needs:

- stable `id`
- user-facing `name`
- `category`
- `risk`
- `safety_class`
- `default_action`
- focused `paths` or `patterns`
- clear `reason`
- `restore` hints when recovery or rebuild behavior matters

Use `safe` only for data that is clearly rebuildable or disposable. Use `review`, `risky`, or `info` when user judgment is needed. Keep sensitive children as separate protected findings instead of matching a broad parent directory.

Research from Mole, BleachBit, CleanerML, app docs, desktop files, and package layouts is useful, but do not vendor comparison projects, downloaded binaries, or copied cleaner definitions into this repo. Translate ideas into Nibble's Linux-first schema and safety model.

## Working Rules

- Prefer small, explicit Rust modules over broad abstractions.
- Keep rule files human-readable and auditable.
- Use structured path APIs instead of ad hoc string manipulation when possible.
- Keep Smart and Deep profile behavior distinct in tests and docs.
- Keep TUI colors theme-aware. Prefer ANSI/named colors over fixed RGB unless there is a strong reason.
- Avoid UI text overlap and keep compact screens scannable.
- Do not add AI attribution trailers to commits.

## Test Expectations

Run targeted tests while developing, then run `make verify` before handing off when practical.

Riskier areas need focused tests:

- `src/safety/` - path scope, protected paths, allow/deny helpers.
- `src/scanner/` - size, duplicate, symlink, permission, and filtering behavior.
- `src/rules/` - profile-aware rule loading and YAML matching.
- `src/report/` - JSON output compatibility.
- `src/cleaner/` and `src/uninstall/` - Trash routing, dry-run behavior, and remnant safety.
- `src/tui/` - selection defaults, navigation, and interaction regressions.
- `cleaners/` and `rules/` - recipe schema, risk labels, default actions, and detection behavior.

## Documentation

Update docs with behavior changes:

- Rule format or catalog changes: cleaners/README.md.
- Contributor process and public changes: CONTRIBUTING.md and README.md.

Keep public docs concise, current, and contributor-friendly. Remove internal brainstorm docs once their useful decisions have moved into README, roadmap, rules, or tests.

## GitHub And Collaboration

- Keep issues public for normal bugs, cleaner requests, design polish, distro support, and feature requests.
- Use private security reporting for deletion-boundary, path traversal, privilege, secret exposure, or release-integrity issues.
- Pull requests touching cleanup sinks, protected path logic, Trash behavior, symlink handling, scanner traversal, Docker/Podman behavior, app remnant discovery, or cleaner recipes need careful review and tests.
- Do not push, tag, publish releases, or change repository visibility unless the maintainer explicitly asks.
