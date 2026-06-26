#!/usr/bin/env bash
#
# dedup-build-cache.sh — reclaim disk by hardlinking byte-identical build
# artifacts across all worktree caches.
#
# WHY THIS EXISTS
# ---------------
# Each worktree carries a ~10-26 GB `sdk/lib/rust/target` cargo tree plus
# ~7 GB per-arch `build-<arch>` dirs. The cargo tree is dominated by build-std
# (core/alloc/compiler_builtins) which cargo recompiles into a SEPARATE
# per-crate target dir for every cargo-EXE module (~98 of them) — see the
# "PER-CRATE dir (not shared)" rationale in
# `sdk/lib/rust/reactos-link/src/lib.rs`: a single shared CARGO_TARGET_DIR is a
# CORRECTNESS hazard (cross-crate feature-unification emits wrong-variant
# symbols that --allow-multiple-definition resolves first-match). So the
# redundancy is the deliberate price of per-crate isolation, and the safe way
# to reclaim it is content-identical hardlinking, NOT a shared target dir.
#
# The same build-std + (post-determinism) the same worktree crates are also
# byte-identical ACROSS worktrees, so dedup spans the whole fleet.
#
# Measured (2026-06-21, 11 worktrees, 281 GB of caches):
#   intra-worktree cargo target:  9.7G -> 3.3G   (6.4 GiB)
#   FULL FLEET cargo targets:     113 GiB reclaimable (60,899 files)
#
# SAFETY
# ------
# Cargo always emits artifacts via tempfile + atomic rename, never mutating a
# file in place. So when a worktree later rebuilds, the rebuilt unit gets a NEW
# inode (the hardlink breaks cleanly) — siblings sharing the old inode are
# untouched. Worst case is a spurious rebuild (mtime disturbance), never wrong
# output. Validated end-to-end: dedup -> warm i386 build -> smoke ✓ boot 83s.
# Re-bloat after a warm build is partial (3.3G->4.9G here); re-run periodically.
#
# ext4 has no reflink/CoW, so hardlink is the only mechanism here. util-linux
# `hardlink` is required (`--ignore-time` is essential: identical artifacts are
# built at different times, so the default mtime check would hide ~all of it).
#
# USAGE
#   tools/dedup-build-cache.sh                 # dry-run: report reclaimable GiB
#   tools/dedup-build-cache.sh --apply         # actually hardlink
#   tools/dedup-build-cache.sh --apply --force # skip the active-build guard
#   tools/dedup-build-cache.sh --targets-only  # cargo target trees only (skip build-*)
#
set -euo pipefail

APPLY=0
FORCE=0
TARGETS_ONLY=0
RECENT_SECS=180   # treat a tree written in the last N seconds as a live build

for a in "$@"; do
    case "$a" in
        --apply)        APPLY=1 ;;
        --force)        FORCE=1 ;;
        --targets-only) TARGETS_ONLY=1 ;;
        -h|--help)      sed -n '2,40p' "$0"; exit 0 ;;
        *) echo "unknown arg: $a" >&2; exit 2 ;;
    esac
done

command -v hardlink >/dev/null || { echo "ERROR: util-linux 'hardlink' not installed (apt-get install util-linux)" >&2; exit 1; }

repo_root="$(git rev-parse --show-toplevel)"
# All worktrees share one filesystem (verified: same /dev mount), so a single
# hardlink pass across every tree dedups intra- AND cross-worktree at once.
mapfile -t worktrees < <(git -C "$repo_root" worktree list --porcelain | awk '/^worktree /{print $2}')

now=$(date +%s)
trees=()
skipped=()
for wt in "${worktrees[@]}"; do
    cands=("$wt/sdk/lib/rust/target")
    [ "$TARGETS_ONLY" -eq 1 ] || cands+=("$wt"/build-i386 "$wt"/build-amd64 "$wt"/build-arm64)
    for t in "${cands[@]}"; do
        [ -d "$t" ] || continue
        if [ "$FORCE" -eq 0 ]; then
            # Short-circuit on the FIRST file newer than the cutoff (cheap —
            # -quit stops the walk immediately) rather than scanning the whole
            # multi-GB tree to find the true newest mtime.
            recent=$(find "$t" -type f -newermt "@$((now - RECENT_SECS))" -print -quit 2>/dev/null)
            if [ -n "$recent" ]; then
                skipped+=("$t (recent write — likely a live build; use --force)")
                continue
            fi
        fi
        trees+=("$t")
    done
done

[ "${#skipped[@]}" -gt 0 ] && printf 'SKIP %s\n' "${skipped[@]}" >&2
[ "${#trees[@]}" -gt 0 ] || { echo "no cache trees to dedup"; exit 0; }

echo "Dedup over ${#trees[@]} cache tree(s):"
printf '  %s\n' "${trees[@]}"
echo

if [ "$APPLY" -eq 1 ]; then
    hardlink --ignore-time --verbose "${trees[@]}" 2>&1 | tail -6
else
    echo "(dry-run — pass --apply to execute)"
    hardlink --dry-run --ignore-time "${trees[@]}" 2>&1 | tail -6
fi
