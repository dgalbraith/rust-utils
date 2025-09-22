# Command Reference

Complete reference for all `rust-utils` commands and options.

## remap

Safely remap user and group IDs across filesystem hierarchies. Perfect for container migrations, privilege changes, and system administration tasks.

### Syntax

```bash
rust-utils remap [OPTIONS] <BASE_DIRECTORY>
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `BASE_DIRECTORY` | Path to directory tree to remap | ✅ Yes |

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--from-base` | int | | Source UID/GID base range (required) |
| `--to-base` | int | | Target UID/GID base range (required) |
| `--range-size` | int | 65536 | Size of ID range to remap |
| `--dry-run` | flag | false | Preview changes without executing |
| `--verbose` | flag | false | Show detailed file-by-file output |
| `--exclude` | string | | Exclude pattern (repeatable) |
| `--uid-only` | flag | false | Only remap UIDs, preserve GIDs |
| `--gid-only` | flag | false | Only remap GIDs, preserve UIDs |
| `--help` | flag | | Show command help |

### Basic Usage

```bash
# Dry run to preview changes
rust-utils remap /path/to/directory \
  --from-base 100000 --to-base 50000000 --dry-run

# Perform actual remapping
rust-utils remap /path/to/directory \
  --from-base 100000 --to-base 50000000
```

### Advanced Usage

```bash
# Remap with exclusions and verbose output
rust-utils remap /path/to/rootfs \
  --from-base 100000 --to-base 50000000 \
  --exclude "var/log/*" --exclude "tmp/*" \
  --exclude "*.log" \
  --verbose

# UID-only remapping (preserve group IDs)
rust-utils remap /path/to/rootfs \
  --from-base 100000 --to-base 200000 --uid-only

# Custom range size for specific applications
rust-utils remap /path/to/app \
  --from-base 1000 --to-base 101000 --range-size 1001

# Multiple exclusion patterns
rust-utils remap /path/to/rootfs \
  --from-base 100000 --to-base 50000000 \
  --exclude "tmp/*" \
  --exclude "var/cache/*" \
  --exclude "*.sock" \
  --exclude "proc/*"
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Invalid arguments or permission error |
| 2 | Directory not found |
| 3 | Remapping operation failed |

### Pattern Matching

Exclusion patterns support basic glob-style wildcards:

- `*` - Matches any sequence of characters
- `dir/*` - Matches all files in directory
- `*.ext` - Matches all files with extension
- `exact/path` - Exact path match

### Performance Tips

- Use `--dry-run` first to validate changes and estimate scope
- Enable `--verbose` for progress monitoring on large filesystems
- Consider `--uid-only` or `--gid-only` if you only need to change one type
- Use exclusion patterns to skip temporary files and logs

### Safety Features

- **Hard link detection**: Prevents filesystem corruption by tracking inodes
- **Atomic operations**: Changes are applied file-by-file consistently
- **Input validation**: Prevents invalid range specifications
- **Error recovery**: Continues processing after individual file failures

### Common Workflows

#### Container Migration
```bash
# Step 1: Dry run to check scope
rust-utils remap /var/lib/lxc/container/rootfs \
  --from-base 100000 --to-base 50000000 --dry-run

# Step 2: Perform remapping
rust-utils remap /var/lib/lxc/container/rootfs \
  --from-base 100000 --to-base 50000000 \
  --exclude "var/log/*" --exclude "tmp/*"
```

#### Privilege Conversion
```bash
# Convert privileged (root=0) to unprivileged (root=100000)
rust-utils remap /var/lib/containers/privileged/rootfs \
  --from-base 0 --to-base 100000 --range-size 65536
```

#### Batch Processing
```bash
#!/bin/bash
for container in web db cache; do
    echo "Processing $container..."
    rust-utils remap "/containers/$container/rootfs" \
      --from-base 100000 --to-base 50000000 \
      --exclude "var/log/*"
done
```