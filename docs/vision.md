# Nibs Vision — Smart Linux Cleaner

> Nibs is a safe Linux cleaner that finds reclaimable space across apps, caches, packages, projects, trash and leftovers — without touching your secrets, configs, sessions or personal files.

**Tagline:** *Nibs — sniff junk, skip secrets.*

**Safe by default. Powerful when reviewed.**

---

## Core Philosophy

Nibs is not a developer toolbox. It is a **smart, fast Linux cleaner** that anyone runs when disk is full, with extra power for devs.

```txt
install → run nibs → first option → smart scan → shows how much to free
→ select safe/recommended → clean → done, 10/20/50 GB freed
```

---

## User Flow

```txt
Clean general       = fast fix
Analyze             = manually find what's heavy
Apps                = large/unused apps + leftovers
Optimize            = speed up slow system
Settings            = Trash vs direct delete, recipes
```

---

## Home Screen

Should show immediately:

```
1. free disk space
2. last scan time
3. active safety mode
4. what to do now
```

---

## Menu Structure

```
● Smart Clean         scan apps, cache, packages, projects
○ Deep Clean          review Docker, games, downloads, models
○ Analyze Disk        walk folders manually
○ Apps & Leftovers    large apps, unused apps, orphaned files
○ Optimize System     safe performance & startup fixes
○ Trash               review or empty moved files
○ Settings            safety, recipes, delete mode
```

---

## Smart Clean — Core Feature

Scans everything safe/relevant:

- App caches
- Browser caches
- Package manager caches
- Trash
- Old logs
- Thumbnails
- Temp files
- Build folders
- Project caches
- Docker/Podman leftovers
- Flatpak/Snap cache
- Download installers
- .rpm/.deb/.AppImage files
- Orphaned app leftovers
- Large unused app data

Result display:

```
Smart Clean complete

Potential cleanup     24.8 GiB
Recommended           14.2 GiB
Needs review           8.9 GiB
Protected              1.7 GiB
```

With always-visible counts:

```
Recommended cleanup    14.2 GiB
Maximum cleanup        24.8 GiB
Protected data          1.7 GiB
```

---

## Safety Classification

### Safe — selected by default
- Rebuildable caches
- Old logs
- Temp files
- Thumbnails
- Package download cache
- Known build folders
- Crash reports
- Trash (if user chooses)

### Review — not selected by default
- Docker unused images
- Old node_modules
- Old target dirs
- Large Downloads
- .rpm/.deb/.AppImage
- Game caches
- Flatpak/Snap app data
- Old kernels
- Orphaned app data

### Risky / Protected — never auto-selected
- Tokens, configs, passwords
- Sessions, cookies, history
- AI memories
- Projects, personal docs
- Photos, videos
- Save games

---

## Deep Clean

Finds big things worth reviewing:

- Docker images/volumes/build cache
- Large games + shader caches
- Apps not opened in 30/90 days
- Large downloads, videos, ISOs
- Old .rpm/.deb/.AppImage
- Old node_modules/target/venv
- Giant .cache directories
- Local AI models (Ollama, LM Studio)
- Steam shader cache
- Flatpak/Snap leftovers
- Orphaned config/cache

Not automatic — gives insights, not clean-all.

---

## Analyze

Manual disk map with intelligence:

```
Size       Risk        Type          Path
120 GiB    user        videos        ~/Videos
84 GiB     review      games         ~/.steam
42 GiB     review      docker        ~/.local/share/containers
12 GiB     safe-ish    builds        ~/Projects/*/target
8 GiB      user        downloads     ~/Downloads
```

Classifies as: safe, review, user data, protected, unknown.

---

## Apps & Leftovers

Shows:

- Which apps are large
- Which are unused
- Which left orphaned files
- Which have giant caches

Before uninstalling, show: cache, config, binary, desktop file, leftovers, last used, size.

Priority order: clean cache → remove leftovers → uninstall app.

---

## Optimize System

Safe actions:

- Vacuum journal logs
- Clean package manager cache
- Review startup apps
- Review heavy services
- Remove old kernels (careful)

Avoids placebo/dangerous: no RAM cache purging, no random kernel tweaks, no auto-killing processes.

---

## Trash — Trust Center

```
Current trash size     8.4 GiB
Moved by Nibs          5.1 GiB
Other files            3.3 GiB

[Enter] Review files
[E] Empty Trash
[R] Restore selected
[D] Delete directly
```

Post-clean screen:

```
Clean complete

Moved to Trash        14.2 GiB
Files moved           18,420
Can restore from      ~/.local/share/Trash
Protected             tokens, configs, sessions, memories
```

Default mode: **Move to Trash**.

---

## Settings

- Safety Mode
- Cleanup Method (Trash vs Direct)
- Protect Secrets
- Protect Sessions
- Protect AI Memories
- Recipe Updates
- Show Advanced Items

---

## CLI Commands

```bash
nibs              TUI
nibs clean        Smart Clean interactive
nibs clean --safe Clean recommended with preview
nibs clean --safe --yes    Script mode, no TUI
nibs deep         Advanced/Deep Clean
nibs analyze      Disk analyzer
nibs apps         Apps & leftovers
nibs optimize     System optimizer
nibs trash        Trash manager
nibs status       System monitor
nibs recipes      Supported recipes
nibs doctor       Environment checks
```

---

## Recipes

Community-contributed cleaner definitions. Each recipe has:

```yaml
id: capcut
name: CapCut
category: media
platforms: [linux]
detect:
  commands: [capcut]
  desktop_files: ["*capcut*.desktop"]
  paths: ["~/.config/CapCut", "~/.cache/CapCut"]
rules:
  - id: capcut-cache
    label: CapCut render/cache files
    paths: ["~/.cache/CapCut"]
    safety: safe
    selected_by_default: true
    impact: "CapCut may rebuild thumbnails/previews."
```

---

## Roadmap

### Phase 1 — Fix current product
1. Rename main flow to Smart Clean
2. Create aggregated scan
3. Result with Recommended / Maximum / Protected
4. Clean moves to Trash
5. Beautiful post-clean screen
6. Simple Trash manager

### Phase 2 — Deep Clean intelligence
1. Detect large files in Downloads
2. Detect old installers .rpm/.deb/.AppImage
3. Detect Docker/Podman
4. Detect old target/node_modules/venv
5. Detect removed apps with leftovers
6. Detect local AI models

### Phase 3 — Recipes
1. YAML/TOML format
2. Built-in recipes
3. Recipes screen
4. Recipe validator
5. Community repo

### Phase 4 — Polish
1. README with GIF
2. Homebrew/Linuxbrew
3. AUR
4. .deb/.rpm
5. Simple website
6. Tauri app

---

## Copy

**Tagline:** `Nibs — sniff junk, skip secrets.`

**Home subtitle:** Safe Linux cleanup for apps, caches, packages, projects and leftovers.

**Smart Clean:** Find safe junk across your system. Nibs protects configs, tokens, sessions and personal files by default.

**Deep Clean:** Find big things worth reviewing. Useful for Docker, games, downloads, models and old project builds.

**Trash:** Everything moved by Nibs can be reviewed before permanent deletion.

**Post-clean:** Clean complete. Nibs moved X GiB to Trash and skipped protected data.

**Product pitch:** Nibs is a safe Linux cleaner that finds reclaimable space across apps, caches, packages, projects, trash and leftovers — without touching your secrets, configs, sessions or personal files.
