<div align="center">
  <h1>Nibble</h1>
  <p><em>🐭 Free up space on Linux without deleting the wrong things.</em></p>
</div>

<p align="center">
  <a href="https://github.com/danitsdev/nibble/stargazers"><img src="https://img.shields.io/github/stars/danitsdev/nibble?style=flat-square" alt="GitHub stars"></a>
  <a href="https://github.com/danitsdev/nibble/releases"><img src="https://img.shields.io/github/v/tag/danitsdev/nibble?label=version&style=flat-square" alt="Version"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square" alt="License"></a>
  <a href="https://github.com/danitsdev/nibble/actions"><img src="https://img.shields.io/github/actions/workflow/status/danitsdev/nibble/ci.yml?branch=main&style=flat-square" alt="CI status"></a>
</p>

<p align="center">
  <strong>Clean caches, logs, installers, project builds, app leftovers, Trash, and other reclaimable junk from a friendly terminal UI.</strong>
</p>

<p align="center">
  <code>nibs</code> sniffs junk, explains what it found, and skips secrets by default.
</p>

---

## Why Nibble?

Your Linux machine can lose tens or hundreds of gigabytes to boring junk:

- old package downloads
- app caches
- browser caches
- logs and crash reports
- Trash
- `.rpm`, `.deb`, `.AppImage`, `.tar.gz` installers
- `target/`, `node_modules/`, `dist/`, `build/`, `.venv`
- Docker/Podman leftovers
- old SDKs, emulators, game caches, and local AI model stores
- files left behind by apps that are no longer installed

Nibble helps you find that space quickly.

It does **not** blindly delete your home folder. It separates things into:

- **safe** — boring rebuildable junk selected by default
- **review** — large things worth checking first
- **risky** — configs, sessions, credentials, memories, and personal data kept protected

> Nibble is built for the moment when your disk is full and you want 10–50 GB back without fear.

---

## Features

- **Smart Clean**  
  Fast general cleanup for app caches, package caches, logs, Trash, installers, and generated files.

- **Deep Clean**  
  Finds bigger review-only targets: Docker data, old downloads, local AI models, game caches, SDKs, duplicate files, and old project artifacts.

- **Trash-first cleanup**  
  Moves files to Trash by default, so cleanup is recoverable before permanent deletion.

- **App leftovers**  
  Detects apps, caches, config folders, desktop entries, and remnants from apps that may no longer be installed.

- **Disk analyzer**  
  ncdu-style explorer for manually finding what is using your storage.

- **Developer cleanup**  
  Finds rebuildable project junk such as `target/`, `node_modules/`, `dist/`, `build/`, `.venv`, package caches, and tool caches.

- **Recipe system**  
  YAML-based cleaner rules for apps and tools. Community recipes can teach Nibble where safe cache lives.

- **Scriptable output**  
  JSON reports and dry runs for automation, debugging, and review.

---

## Quick Start

### Install

**Via Cargo:**

```bash
cargo install nibs
```

**Via precompiled binary script (Linux x86_64/ARM64):**

```bash
curl -fsSL https://raw.githubusercontent.com/danitsdev/nibble/main/install.sh | bash
```

### Run

```bash
nibs
```

That opens the interactive terminal UI.

### Common commands

```bash
nibs                         # Open the interactive UI
nibs clean                   # Smart Clean
nibs clean --dry-run         # Preview safe cleanup
nibs clean --no-tui          # Print Smart Clean results in the terminal

nibs deep                    # Find large review-only cleanup targets
nibs analyze ~/Downloads     # Explore disk usage manually
nibs apps                    # Inspect apps and leftovers
nibs trash                   # Review, restore, or empty Trash
nibs status                  # Live system dashboard
nibs doctor                  # Check environment, distro, tools, and cleanup support

nibs scan . --json           # JSON scan report for a path
nibs deep --json             # JSON Deep Clean report
```

---

## The Main Flow

```bash
nibs clean
```

Nibble scans common cleanup locations and shows something like:

```txt
Smart Clean complete

Recommended cleanup     14.2 GiB
Maximum possible         31.8 GiB
Needs review             17.6 GiB
Protected data           skipped

[x] safe      App caches                         3.2 GiB
[x] safe      Package manager cache              2.1 GiB
[x] safe      Logs and crash reports             820 MiB
[x] safe      Project build folders              5.8 GiB
[ ] review    Docker unused data                 6.4 GiB
[ ] review    Old installers                     1.2 GiB
[ ] review    Large downloads                    8.1 GiB
[ ] risky     Browser sessions                   protected
```

Press clean, confirm, and Nibble moves selected files to Trash.

```txt
Clean complete

Moved to Trash          14.2 GiB
Files moved             18,420
Risky items touched     0
Can restore from        ~/.local/share/Trash
```

---

## Smart Clean vs Deep Clean

| Command        | Use it when                                                    | Behavior                                                                  |
| :------------- | :------------------------------------------------------------- | :------------------------------------------------------------------------ |
| `nibs clean`   | You want space back quickly.                                   | Selects safe, boring, rebuildable junk by default.                        |
| `nibs deep`    | Your disk is still full and you want to investigate big stuff. | Finds large review-only targets. Nothing risky is selected automatically. |
| `nibs analyze` | You want to manually walk through folders.                     | Shows what is using space and lets you inspect paths yourself.            |
| `nibs apps`    | You want to clean app caches, leftovers, or remove apps.       | Shows app data separately from protected configs/sessions.                |

---

## Safety Model

Nibble is conservative by default.

It avoids permanent deletion unless you explicitly enable it.

By default, Nibble protects:

* passwords, tokens, credentials, and API keys
* browser sessions, cookies, and login state
* app preferences and personal configuration
* AI agent memories and project instructions
* databases and unknown stateful directories
* personal files such as documents, photos, videos, and save files
* symlinks and protected system paths

When Nibble is unsure, it marks the item as **review**, **risky**, or **protected** instead of cleaning it automatically.

```txt
safe      selected by default
review    shown, but not selected
risky     blocked unless explicitly reviewed
protected never cleaned by normal actions
```

---

## Trash-first by Default

Nibble moves files to the system Trash by default.

That means the normal cleanup path is recoverable:

```bash
nibs trash
```

Use it to review, restore, or permanently empty files moved by Nibble.

Direct deletion can be enabled later, but Trash mode is the recommended default.

---

## Examples

### Clean safe system junk

```bash
nibs clean
```

Finds safe cleanup targets such as caches, logs, package downloads, Trash, installers, and generated files.

### Preview before cleaning

```bash
nibs clean --dry-run
```

Shows what would be cleaned without moving anything.

### Find what is still eating space

```bash
nibs deep
```

Looks for larger review-only targets such as Docker storage, old downloads, local models, SDKs, games, and project artifacts.

### Explore a folder manually

```bash
nibs analyze ~/Downloads
```

Opens an ncdu-style disk explorer.

### Inspect app leftovers

```bash
nibs apps
```

Shows installed apps, app caches, configs, desktop files, and possible remnants.

### Scan a project

```bash
nibs scan . --json
```

Useful for checking build artifacts, generated folders, and cache directories in a project.

---

## Cleaner Recipes

Nibble learns cleanup rules through recipes.

Generic rules live in:

```txt
rules/
```

App and tool-specific cleaners live in:

```txt
cleaners/
```

Recipes can detect apps through commands, desktop files, known paths, package names, and config directories.

A recipe can say:

```txt
This path is safe cache.
This path is review-only.
This path contains settings.
This path may contain secrets.
Never clean this by default.
```

That lets Nibble support more apps over time without hardcoding every cleaner into the core.

Examples of recipe areas:

* browsers
* package managers
* code editors
* AI tools
* Docker and Podman
* games
* media apps
* communication apps
* Flatpak, Snap, AppImage
* creative tools
* desktop apps

Before adding recipes, read:

* [cleaners/README.md](cleaners/README.md)
* [AGENTS.md](AGENTS.md)

---

## Project Status

Nibble is currently an early Linux alpha.

The core cleanup model is intentionally strict. Some cleaners may find less than expected, but they should not delete broadly or guess dangerously.

Good contributions right now:

* cleaner recipes for popular Linux apps
* distro-specific package cache support
* Flatpak, Snap, AppImage, RPM, and DEB coverage
* better app leftover detection
* TUI layout and copy polish
* screenshots and GIFs
* release packaging
* tests for Trash behavior, protected paths, symlinks, and rule loading

---

## Build from Source

```bash
cargo build --release
./target/release/nibs
```

During development:

```bash
cargo run
cargo run -- clean
cargo run -- scan . --json
make verify
```

---

## Verification

```bash
make verify
```

Runs:

```bash
cargo fmt --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

---

## Inspired By

Nibble is inspired by tools like Mole, CleanMyMac, AppCleaner, DaisyDisk, ncdu, and modern terminal assistants.

The goal is different:

> a Linux-first cleaner that is fast enough for normal users, careful enough for developers, and readable enough that you understand what will happen before anything moves.

---

## License

MIT License.
