# Nibble

<p align="center">
  <strong>A safe, Linux-first terminal cleaner for developers.</strong>
</p>

<p align="center">
  <a href="https://github.com/danitsdev/nibble/stargazers"><img src="https://img.shields.io/github/stars/danitsdev/nibble?style=flat-square" alt="GitHub stars"></a>
  <a href="https://github.com/danitsdev/nibble/commits/main"><img src="https://img.shields.io/github/commit-activity/m/danitsdev/nibble?style=flat-square" alt="Commit activity"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
  <a href="https://github.com/danitsdev/nibble/actions"><img src="https://img.shields.io/github/actions/workflow/status/danitsdev/nibble/ci.yml?branch=main&style=flat-square" alt="CI status"></a>
</p>

Nibble is a Rust-powered terminal app that finds rebuildable junk, generated artifacts, app caches, old project dependencies, local tool caches, recoverable disk waste, and risky leftovers before anything is cleaned.

> Nibble never guesses. It explains before it cleans.

The project is strongly inspired by [tw93/Mole](https://github.com/tw93/Mole): fast terminal workflows, practical cleanup commands, delightful presentation, and safety-first maintenance tooling. Nibble takes that spirit in a Linux-focused direction with Trash-first cleanup, explicit risk labels, JSON reports, readable YAML rules, and conservative handling for Docker, SDKs, secrets, databases, and user data.

Nibble is currently an early Linux preview. It is open to contributions across cleaner recipes, Linux distro support, app detection, documentation, TUI design, screenshots, release packaging, tests, and safety review.

## What It Does

- **Smart Clean**: quick daily scan for safe, boring, rebuildable cleanup targets.
- **Deep Clean**: review-first scan for old or heavy data such as Docker state, Android SDK versions, local AI models, game runtimes, installers, large downloads, and duplicate files.
- **Analyze Disk**: ncdu-style folder explorer for manual inspection.
- **Apps & Leftovers**: inspect app remnants before trashing them.
- **Status & Doctor**: local storage, Docker, package-cache, journal, and environment checks.
- **Cleaner Recipes**: app/tool-specific YAML recipes that activate only when the app is detected.
- **JSON Reports**: script-friendly output for automation and review.
- **Trash-first Cleanup**: recoverable cleanup by default instead of permanent deletion.

## Installation

Install the precompiled binary directly (Linux x86_64):

```bash
curl -fsSL https://raw.githubusercontent.com/danitsdev/nibble/main/install.sh | bash
```

Or build from source:

```bash
cargo build --release
./target/release/nibs
```

During development:

```bash
cargo run
cargo run -- scan . --json
cargo run -- deep --json
make verify
```

## Command Surface

```bash
nibs                         # Open the interactive home screen
nibs clean                   # Smart Clean in the TUI
nibs clean --no-tui          # Smart Clean as terminal output
nibs clean --dry-run         # Preview recommended safe cleanup
nibs scan . --json           # JSON report for a specific path

nibs deep                    # Deep review scan of $HOME
nibs deep ~/Projects         # Deep review scan of a specific path
nibs deep --json             # JSON report for deep-review findings
nibs deep --no-duplicates    # Faster deep scan without duplicate hashing

nibs analyze ~/Downloads     # Manual disk explorer
nibs status                  # Live system status dashboard
nibs doctor                  # Environment and cache diagnostics
nibs trash                   # Review, restore, or empty Trash
nibs uninstall discord       # Inspect app remnants before cleanup
```

## Smart vs Deep

| Profile | Purpose | Default cleanup behavior |
| :--- | :--- | :--- |
| Smart Clean | Fast daily cleanup for safe caches and generated files. | Auto-selects only `risk=safe`, `default_action=clean`, `safety_class=safe`. |
| Deep Clean | Finds older, heavier, or more expensive data worth reviewing. | Selects nothing automatically. Review first. |
| Analyze Disk | Manual exploration for unknown large folders. | User chooses paths manually. |

Deep Clean defaults to a 7-day inactivity filter and includes heavier review-only rules. It can surface Docker and Podman storage, Android SDK components, AVDs, local AI model stores, old installers, large downloads, and exact duplicates, but protected findings remain blocked by `default_action=never`.

## Safety Model

Nibble is designed around conservative cleanup boundaries:

- no permanent deletion by default
- no broad unknown system cleanup
- no automatic Docker or Podman volume deletion
- no symlink following by default
- no crossing filesystem boundaries during `/` scans by default
- protected Linux paths are skipped
- risky, duplicate, Docker, system, and unknown findings require review
- every cleanup target must explain what it contains and how it can be restored when possible

See the Safety Model section above and [AGENTS.md](AGENTS.md) for detailed clean-up boundaries.

## Cleaner Catalog

Generic rules live in [rules/](rules/). App/tool-specific cleaners live in [cleaners/](cleaners/) and activate through lightweight detection: commands, desktop files, and known paths.

Current catalog areas include browsers, code editors, communication apps, creative tools, desktop apps, AI tools, developer tools, gaming, media, and productivity apps.

Before adding a rule or cleaner, read:

- [cleaners/README.md](cleaners/README.md)
- [AGENTS.md](AGENTS.md) for core safety invariants

## Project Status

Nibble is usable for local testing and contributor feedback, but it is still pre-1.0. The core safety posture is intentionally strict; polish work is welcome before broader user-facing releases.

Good next contributions:

- cleaner recipes for popular Linux apps
- safer app-detection patterns
- Flatpak/Snap/AppImage coverage
- TUI layout and interaction polish
- screenshots, GIFs, and README presentation
- release packaging and install docs
- distro-specific cache behavior
- tests for protected paths, Trash behavior, and rule loading

## Contributing

Contributions are welcome. Design improvements, presentation polish, cleaner recipes, app detection, docs, tests, and safety reviews are all useful.

Start with [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md). Pull requests touching cleanup sinks, protected paths, Trash routing, symlink handling, Docker/Podman behavior, app remnant discovery, or cleaner rules need careful review and tests.

## Verification

```bash
make verify
```

This runs:

```bash
cargo fmt --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## License

Nibble is released under the [MIT License](LICENSE).
