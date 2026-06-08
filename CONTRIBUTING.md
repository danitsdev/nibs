# Contributing to Nibs

Thanks for helping make Nibs safer and more useful.

Nibs touches local files, so safety is more important than aggressive cleanup. When in doubt, report, explain, or require review instead of deleting more.

## Setup

```bash
rustup update
cargo build
cargo test
```

Useful checks:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Or run:

```bash
make verify
```

## Development Guidelines

- Keep cleanup behavior conservative.
- Add or update tests for safety-sensitive changes.
- Prefer Trash/recoverable cleanup over permanent deletion.
- Keep rule files readable and specific.
- Avoid broad globs for system paths, app data, secrets, databases, or Docker volumes.
- Handle permission errors calmly.
- Prefer ANSI/named terminal colors over fixed RGB values.
- Update docs when behavior changes.

## Adding A Rule

Rules should include:

- `id`
- `name`
- `category`
- `risk`
- `patterns`
- `reason`
- `restore` when applicable
- `default_action`

Use `safe` only when the data is clearly rebuildable or disposable. Use `review`, `risky`, or `info` when user judgment is needed.

## Pull Requests

1. Fork the repo and create a branch from `main`.
2. Make a focused change.
3. Run `make verify`.
4. Update docs and tests if behavior changed.
5. Open a PR with a clear summary and verification notes.

PRs touching cleanup sinks, protected paths, symlink behavior, Trash routing, app remnant discovery, or Docker handling need extra scrutiny.

## Security

Do not open a public issue for deletion-boundary bypasses, path traversal, privilege issues, or release-integrity problems. Use the process in [SECURITY.md](SECURITY.md).
