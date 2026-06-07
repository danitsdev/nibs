# Security Policy

Nibble is a local cleanup tool. Bugs in path validation, cleanup boundaries, symlink handling, Trash routing, or privilege behavior can cause real data loss, so security-sensitive reports should be handled carefully.

## Reporting A Vulnerability

Please report suspected security issues privately.

Use GitHub private vulnerability reporting once it is enabled for the repository. If private reporting is not enabled yet, contact the maintainer before sharing exploit details publicly.

Include:

- Nibble version or commit
- Linux distribution and version
- Exact command or workflow
- Reproduction steps
- Whether the issue involves deletion boundaries, symlinks, permissions, protected paths, Docker, app remnants, or release integrity

## Supported Versions

Security fixes are targeted at:

- the current `main` branch
- the latest published release once releases exist

## Security-Relevant Areas

Examples of security-relevant issues:

- cleanup outside intended scope
- protected path bypass
- symlink or path traversal issue
- unexpected permanent deletion
- unsafe Docker volume cleanup
- unsafe handling of secrets, credentials, databases, or app state
- release, installer, or checksum integrity issue

Usually not security issues:

- missed cleanup opportunities
- false positives that are blocked by review/confirmation
- cosmetic UI bugs
- requests for more aggressive cleanup

When unsure, report privately first.
