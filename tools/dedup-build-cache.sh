#!/usr/bin/env bash
# Retired: hardlinking mutable Cargo outputs poisoned mtimes and truncated live
# linker inputs across worktrees. Idleness at link creation does not make the
# links safe during the next build. There is deliberately no force override.
echo 'RETIRED: live build outputs must not be hardlinked across worktrees.' >&2
echo 'Use vibeos-cli prune for deletion-only retention; see docs/engineering/disk-optimization.md.' >&2
exit 2
