# Nibble

<p align="center">
  <strong>A safe terminal cleaner for Linux developers.</strong>
</p>

<p align="center">
  <a href="https://github.com/danits/nibble/stargazers"><img src="https://img.shields.io/github/stars/danits/nibble?style=flat-square" alt="GitHub stars"></a>
  <a href="https://github.com/danits/nibble/commits/main"><img src="https://img.shields.io/github/commit-activity/m/danits/nibble?style=flat-square" alt="Commit activity"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
  <a href="https://github.com/danits/nibble/actions"><img src="https://img.shields.io/github/actions/workflow/status/danits/nibble/ci.yml?branch=main&style=flat-square" alt="CI status"></a>
</p>

Nibble is a Rust-powered terminal app that finds rebuildable development junk, generated artifacts, language caches, framework caches, temporary files, and other recoverable disk waste.

> Nibble never guesses. It explains before it cleans.

Nibble is not a shady system optimizer. It is a conservative developer tool with a polished TUI, clear explanations, JSON reports, and safety-first cleanup behavior.

## Features

- **Developer junk scanning**: finds `node_modules`, Rust `target`, Python caches, framework caches, build outputs, coverage directories, and more.
- **Safe Linux scopes**: scans a chosen directory by default, and uses protected system-safe behavior when scanning `/`.
- **Transparent rules**: human-readable YAML rules explain what each finding means and how to rebuild it.
- **Interactive TUI**: grouped findings, risk labels, details, explanations, selection, and confirmation flows.
- **JSON reports**: script-friendly output for automation and review.
- **Trash-first cleanup**: cleanup routes through recoverable trash behavior instead of permanent deletion.
- **Disk analyzer, app remnants, and status tools**: early integrated tools for local Linux maintenance workflows.

## Quick Start

```bash
cargo build --release
./target/release/nibs
```

During development:

```bash
cargo run
cargo run -- scan . --json
cargo test
cargo fmt --check
```

## Usage

```bash
nibs                           # Interactive scan of the current directory
nibs ~/Projects                # Interactive scan of a specific directory
nibs /                         # Protected system-safe scan

nibs scan . --json             # JSON report
nibs scan . --dry-run          # Preview cleanup behavior
nibs analyze ~/Downloads       # Disk usage explorer
nibs status                    # Live system status dashboard
nibs uninstall discord         # Inspect app remnants before trashing
```

## Safety Model

Nibble is designed around conservative cleanup boundaries:

- no permanent deletion by default
- no broad unknown system cleanup
- no automatic Docker volume deletion
- no symlink following by default
- protected Linux paths are skipped or treated as info-only
- risky and unknown findings require review
- every cleanup action should be explainable before it happens

Read [docs/safety.md](docs/safety.md) for the detailed model.

## Rules

Rules live in [`rules/`](rules/) and cleaner catalog entries live in [`cleaners/`](cleaners/). They are intended to be readable, reviewable, and contributor-friendly.

Read [docs/rules.md](docs/rules.md) before adding a new cleanup rule.

## Contributing

Nibble is open to contributions, especially:

- new safe cleanup rules
- Linux distro compatibility fixes
- TUI polish
- safety tests
- documentation improvements

Start with [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md). Security-sensitive cleanup changes need extra review.

The small terminal mascot is **Nibs**, a sleepy little mouse who moves the broom from a distance.

## Roadmap

See [docs/roadmap.md](docs/roadmap.md).

## License

Nibble is released under the [MIT License](LICENSE).
