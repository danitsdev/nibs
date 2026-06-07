# Nibble Safety Model

Nibble is designed from the ground up as a **safe, developer-first disk space cleaner**. The primary product promise is:
> Nibble never guesses. It explains before it cleans.

---

## 1. Safe-by-Default Behavior
- **No Permanent Deletion By Default**: Nibble must not permanently delete files as a default workflow.
- **Trash-First Cleanup**: Cleanup routes recoverable targets through the system Trash where supported.
- **Dry-Run Support**: Use dry-run/report modes to preview what Nibble would clean before changing anything.
- **Verification and Confirmed Action**: Cleanup is never automatic for risky paths or duplicate files. Even for safe findings, destructive actions require explicit user intent.

---

## 2. Scan Scopes & Boundaries

Depending on the input arguments, Nibble operates in three distinct scopes:

1. **Project Scan** (Default: `nibs` or `nibs .`): Scans only the current directory.
2. **Directory Scan** (`nibs <path>`): Scans only the specified directory tree.
3. **System Safe Scan** (`nibs /`): Activated only when scanning the root path.

### Safety Rules for `nibs /`
When scanning the root directory, Nibble protects system stability by enforcing several strict rules:
- **No Cross-Filesystem Boundaries**: Nibble will not cross filesystem mount boundaries (e.g. into external drives, network mounts, or `/sys`/`/proc`) unless explicitly configured to do so.
- **No Symlink Following**: Nibble does not follow symbolic links by default, preventing loops and unexpected traversal into system directories.

---

## 3. Protected System Paths

Nibble will **always skip** the following virtual, kernel, or critical system paths:

| Path | Reason |
| :--- | :--- |
| `/proc` | Virtual filesystem containing kernel and process information. |
| `/sys` | Virtual filesystem containing hardware and driver attributes. |
| `/dev` | Device nodes representing hardware interfaces. |
| `/run` | Runtime volatile data for running services. |
| `/boot` | Kernel, initial ramdisk, and bootloader files. |
| `/lost+found` | Recovered file fragments after filesystem corruption. |

### Restricted System Paths
Additionally, Nibble treats the following paths as **restricted/read-only or skips them** unless explicitly overridden:
- `/etc`, `/bin`, `/sbin`, `/lib`, `/lib64`, `/usr/bin`, `/usr/sbin`, `/usr/lib`, `/var/lib`
- Docker daemon state: `/var/lib/docker/volumes`
- Databases: `/var/lib/postgresql`, `/var/lib/mysql`
- Virtualization: `/var/lib/libvirt`
- Package managers: `/snap`, `/flatpak`

---

## 4. Risk Levels

Nibble groups findings into four risk levels:

1. **`safe`**: Rebuildable directories or temporary build/cache files (e.g. `__pycache__`, `.pytest_cache`). Cleanable without compromising project integrity.
2. **`review`**: Items that are usually safe but might require user inspection. This includes project virtual environments (`.venv`), or cargo/npm package caches that are large or expensive to fetch again.
3. **`risky`**: Paths containing system configurations, database files, or user data. These are **never** auto-cleaned.
4. **`info`**: Items reported strictly for user information and awareness, containing no cleanup recommendations.

---

## 5. Duplicate Files Policy
- **Hash-on-Demand**: Nibble only hashes files if duplicate detection is requested. It does not perform expensive hashes during normal directory scanning.
- **Exact Matches Only**: Duplicate detection uses cryptographically secure **BLAKE3** hashing for exact size and byte comparisons.
- **Review Risk**: Duplicates are always marked as `review` and will never be selected automatically for deletion.

---

## 6. Docker Volumes Policy
- **Docker Info Only**: Nibble detects the presence of Docker and reports usage stats using standard commands (like `docker system df`) as `info` only.
- **No Automatic Volume Deletion**: Nibble must never automatically prune Docker volumes. Safe suggestions may be listed for manual review.

---

## 7. Inactivity Age Filter (`--min-age`)
- **Active Project Safety**: To prevent cleaning up files or dependency folders (like `node_modules` or `target`) of projects currently under active development, Nibble supports filtering by inactivity age.
- **Dynamic age computation**: When `--min-age <DAYS>` is specified, Nibble checks the latest modification date among all files recursively located inside a matched finding folder. If any file inside the directory was modified within the specified days, the entire folder is skipped.
- **Resilient Fallback**: If file metadata is unreadable and an age filter is requested, Nibble conservatively skips the finding.

---

## 8. Brute System Clean Mode (`--brute`)
- **Restricted Zone Access**: By default, Nibble completely skips restricted system paths (like `/etc`, `/bin`, `/var/lib`, `/snap`) during root scans to protect system stability.
- **Safe Aggression**: When `--brute` is enabled (intended for running as `root`), Nibble is allowed to scan these restricted directories to identify and clean up large system caches (such as package manager repository caches or rotated historical logs).
- **Core Virtual Protection**: Even in `--brute` mode, Nibble **always skips** kernel-virtual directories (`/proc`, `/sys`, `/dev`, `/run`, `/boot`, `/lost+found`) as walking them is dangerous and can hang or crash the system.
