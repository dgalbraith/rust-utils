# Examples

Real-world examples and use cases for rust-utils commands.

## File System Remapping

### Container Migration Between Hosts

When moving containers between systems with different user namespace configurations:

```bash
# Source system: UIDs 100000-165535
# Target system: UIDs 50000000-50065535

# 1. Stop the container
lxc-stop -n mycontainer

# 2. Preview the remapping
rust-utils remap /var/lib/lxc/mycontainer/rootfs \
  --from-base 100000 --to-base 50000000 --dry-run --verbose

# 3. Perform the remapping
rust-utils remap /var/lib/lxc/mycontainer/rootfs \
  --from-base 100000 --to-base 50000000

# 4. Update container configuration and start
lxc-start -n mycontainer
```

### Converting Privileged to Unprivileged Containers

Converting a privileged container (where root=0) to use user namespaces:

```bash
# Map system IDs (0-65535) to unprivileged range (100000-165535)
rust-utils remap /var/lib/lxc/privileged-container/rootfs \
  --from-base 0 --to-base 100000 --range-size 65536 \
  --exclude "proc/*" --exclude "sys/*" --exclude "dev/*"
```

### Application-Specific ID Ranges

Some applications create users with specific UID ranges:

```bash
# Database application uses UIDs 999-1010
# Remap to container range 100999-101010
rust-utils remap /var/lib/containers/database/rootfs \
  --from-base 999 --to-base 100999 --range-size 12
```

### Selective Remapping with Complex Exclusions

Remap while preserving logs, caches, and temporary files:

```bash
rust-utils remap /var/lib/containers/webserver/rootfs \
  --from-base 100000 --to-base 50000000 \
  --exclude "var/log/*" \
  --exclude "var/cache/*" \
  --exclude "tmp/*" \
  --exclude "run/*" \
  --exclude "*.log" \
  --exclude "*.sock" \
  --verbose
```

### UID-Only Remapping

When you need to change user IDs but preserve system groups:

```bash
# Change UIDs for user namespace but keep system groups
rust-utils remap /var/lib/containers/app/rootfs \
  --from-base 100000 --to-base 50000000 --uid-only
```

### Large Filesystem with Progress Monitoring

For very large container filesystems:

```bash
# Monitor progress and save output
rust-utils remap /var/lib/containers/large-app/rootfs \
  --from-base 100000 --to-base 50000000 --verbose \
  2>&1 | tee remap-$(date +%Y%m%d-%H%M%S).log
```

## Batch Operations

### Processing Multiple Containers

```bash
#!/bin/bash
# remap-containers.sh

OLD_BASE=100000
NEW_BASE=50000000
CONTAINERS=("web-frontend" "api-backend" "database" "cache")

for container in "${CONTAINERS[@]}"; do
    echo "=== Processing container: $container ==="
    
    # Check if container exists
    if [[ ! -d "/var/lib/lxc/$container/rootfs" ]]; then
        echo "Warning: Container $container not found, skipping"
        continue
    fi
    
    # Stop container if running
    if lxc-info -n "$container" -s | grep -q RUNNING; then
        echo "Stopping container $container..."
        lxc-stop -n "$container"
    fi
    
    # Dry run first
    echo "Dry run for $container..."
    if rust-utils remap "/var/lib/lxc/$container/rootfs" \
       --from-base $OLD_BASE --to-base $NEW_BASE --dry-run; then
        
        # Perform actual remap
        echo "Remapping $container..."
        rust-utils remap "/var/lib/lxc/$container/rootfs" \
          --from-base $OLD_BASE --to-base $NEW_BASE \
          --exclude "var/log/*" --exclude "tmp/*" \
          --exclude "run/*"
        
        echo "✓ Completed: $container"
    else
        echo "✗ Dry run failed for $container, skipping"
    fi
    
    echo ""
done

echo "Batch remapping completed!"
```

### Kubernetes/Docker Container Remapping

```bash
#!/bin/bash
# remap-k8s-volumes.sh

VOLUME_PATH="/var/lib/kubelet/pods"
FROM_BASE=100000
TO_BASE=50000000

# Find all container volumes
find "$VOLUME_PATH" -name "rootfs" -type d | while read -r rootfs; do
    echo "Processing: $rootfs"
    
    # Extract pod/container info from path
    POD_ID=$(echo "$rootfs" | cut -d'/' -f6)
    echo "Pod ID: $POD_ID"
    
    # Remap with standard exclusions
    rust-utils remap "$rootfs" \
      --from-base $FROM_BASE --to-base $TO_BASE \
      --exclude "proc/*" --exclude "sys/*" --exclude "dev/*" \
      --exclude "tmp/*" --exclude "var/cache/*"
done
```

## Advanced Workflows

### Pre-Migration Validation

Before performing remapping, validate your environment:

```bash
#!/bin/bash
# validate-remap.sh

ROOTFS="/var/lib/lxc/mycontainer/rootfs"
FROM_BASE=100000
TO_BASE=50000000

echo "=== Pre-migration validation ==="

# 1. Check current ID usage
echo "Checking current UID/GID distribution..."
find "$ROOTFS" -exec stat -c "%u %g" {} \; 2>/dev/null | \
    awk -v from=$FROM_BASE -v size=65536 '
    $1 >= from && $1 < from+size { uid_count++ }
    $2 >= from && $2 < from+size { gid_count++ }
    END { 
        print "Files with UIDs in range:", uid_count+0
        print "Files with GIDs in range:", gid_count+0
    }'

# 2. Check available space
echo "Checking filesystem space..."
df -h "$ROOTFS"

# 3. Dry run with detailed output
echo "Performing dry run..."
rust-utils remap "$ROOTFS" \
  --from-base $FROM_BASE --to-base $TO_BASE --dry-run --verbose \
  > "migration-plan-$(date +%Y%m%d).txt"

echo "Validation complete. Review migration-plan-$(date +%Y%m%d).txt"
```

### Rollback Preparation

While rust-utils doesn't have built-in rollback, you can prepare for it:

```bash
#!/bin/bash
# prepare-rollback.sh

ROOTFS="$1"
FROM_BASE="$2"
TO_BASE="$3"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)

# Create rollback info
cat > "rollback-info-$TIMESTAMP.txt" << EOF
Original mapping: $FROM_BASE -> $TO_BASE
Reverse mapping: $TO_BASE -> $FROM_BASE
Date: $(date)
Rootfs: $ROOTFS
EOF

# Create snapshot if using LVM/ZFS
if lvdisplay | grep -q "$(dirname "$ROOTFS")"; then
    echo "Creating LVM snapshot..."
    # Add LVM snapshot commands here
fi

# Perform the remap
echo "Performing forward remap..."
rust-utils remap "$ROOTFS" \
  --from-base $FROM_BASE --to-base $TO_BASE \
  --exclude "var/log/*" --exclude "tmp/*"

echo "Rollback command (if needed):"
echo "rust-utils remap '$ROOTFS' --from-base $TO_BASE --to-base $FROM_BASE"
```

## Troubleshooting Examples

### Permission Issues

```bash
# Run as root for system containers
sudo rust-utils remap /var/lib/lxc/container/rootfs \
  --from-base 100000 --to-base 50000000

# Or with specific capabilities
sudo -E rust-utils remap /var/lib/containers/app/rootfs \
  --from-base 100000 --to-base 50000000
```

### Large Directory Optimization

```bash
# Use ionice to reduce I/O impact
ionice -c 3 rust-utils remap /var/lib/containers/huge-app/rootfs \
  --from-base 100000 --to-base 50000000

# Process in smaller chunks by excluding large directories first
rust-utils remap /var/lib/containers/app/rootfs \
  --from-base 100000 --to-base 50000000 \
  --exclude "var/lib/mysql/*" \
  --exclude "var/log/*" \
  --exclude "var/cache/*"

# Then process excluded directories separately if needed
```

### Verification After Remapping

```bash
#!/bin/bash
# verify-remap.sh

ROOTFS="$1"
OLD_BASE="$2"
NEW_BASE="$3"
RANGE_SIZE="${4:-65536}"

echo "=== Verification Report ==="

# Check that old IDs are gone
OLD_COUNT=$(find "$ROOTFS" -uid "+$OLD_BASE" \
    -uid "-$((OLD_BASE + RANGE_SIZE))" 2>/dev/null | wc -l)
echo "Files with old UIDs remaining: $OLD_COUNT"

# Check that new IDs are present
NEW_COUNT=$(find "$ROOTFS" -uid "+$NEW_BASE" \
    -uid "-$((NEW_BASE + RANGE_SIZE))" 2>/dev/null | wc -l)
echo "Files with new UIDs: $NEW_COUNT"

# Verify container can start (if LXC)
if [[ "$ROOTFS" =~ /var/lib/lxc/([^/]+)/rootfs ]]; then
    CONTAINER="${BASH_REMATCH[1]}"
    echo "Testing container startup: $CONTAINER"
    lxc-start -n "$CONTAINER" && sleep 5 && lxc-stop -n "$CONTAINER"
    if [[ $? -eq 0 ]]; then
        echo "✓ Container starts successfully"
    else
        echo "✗ Container failed to start"
    fi
fi
```