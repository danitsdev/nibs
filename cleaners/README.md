# Cleaner Recipes

Cleaner recipes describe app-specific cleanup targets. They are separate from generic rules in `rules/` because a recipe belongs to a known app or tool and should only activate when that app is detected.

## Schema

```yaml
id: firefox
name: Firefox
category: browser_cache
platforms: [linux]
detect:
  commands: ["firefox", "firefox-esr"]
  desktop_files: ["*firefox*.desktop"]
  paths:
    - "~/.mozilla/firefox"
    - "~/.cache/mozilla/firefox"
items:
  - id: web-cache
    name: Firefox web cache
    risk: safe
    safety_class: safe
    default_action: clean
    paths:
      - "**/.cache/mozilla/firefox/*/cache2"
    reason: "Explain what this target contains and what is not removed."
    restore:
      - "Reopen Firefox to rebuild cache files."
```

`detect` is intentionally lightweight. Nibble checks configured commands, desktop files, and paths. It does not scan running processes.

Prefer focused `items` over broad directory rules. Sensitive config, credentials, sessions, and user data should be separate `risk: risky` or `safety_class: secret_or_auth` items with `default_action: never`.

## Catalog Organization

The embedded catalog is grouped by app family:

- `browsers/` - browser-family caches and protected profile data.
- `code-editors/` - IDE and editor caches, logs, and protected settings.
- `communication/` - chat, meeting, and messaging app caches with auth/session protection.
- `creative/` - graphics, video, streaming, and creative-tool caches.
- `desktop-apps/` - general Electron/native desktop apps.
- `dev-ai/` - AI agents, AI desktop apps, and local model stores.
- `dev-tools/` - developer tools that are better modeled with app/tool detection.
- `gaming/` - game launchers and runtime caches.
- `media/` - media players and torrent clients.
- `productivity/` - mail, design, and productivity apps.

Use `rules/` for generic language, framework, filesystem, and Linux ecosystem patterns. Use `cleaners/` when a target belongs to a known app or tool and should only activate after detection. Do not keep a second "deep" copy of an existing generic rule just to change its size expectation; the scanner should evaluate one authoritative pattern with a clear explanation.

## Research Notes

When adding recipes, prefer primary sources such as upstream app docs, installed desktop files, and established open source cleaner catalogs. It is fine to use projects like Mole, BleachBit, and CleanerML to discover candidate apps and path families, but write Nibble recipes in this schema with Linux-first safety review instead of copying foreign cleaner definitions wholesale.

High-risk targets should stay protected:

- credentials, cookies, token stores, keyrings, and auth state
- user-created files, downloads, recordings, VM disks, and local model files
- Docker or container volumes
- broad config directories when a focused cache path exists
