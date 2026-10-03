#!/bin/bash
# Reap orphan QEMUs: those whose owner (the shell that ran qemu/start.sh, or the pytest
# process behind tests/qemu/vm.py) is dead, and any VM past its lifetime deadline. A QEMU
# whose owner is alive and within its cap -- possibly another agent's -- is NEVER touched.
#
# A wrapper: the logic is tests/qemu/vm.py:reap_orphan_qemus, also run in-process at the
# start of every smoke session. It was ~200 lines of bash reading JSON through `jq`, and
# with jq not installed every field read as empty -- so every lock looked stale and each
# new smoke purged every live VM's lock. It also killed windows-host VMs by their WSL
# interop shim pid, which can leave the real qemu.exe running with no lock at all.
#
# Usage: qemu/reap.sh [--dry-run] [--verbose]
#   [reap] action lines go to stderr; a one-line summary to stdout.
exec "$(cd "$(dirname "$0")/.." && pwd)/vibeos-cli" reap "$@"
