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
# SAFETY — READ BEFORE RUNNING (2026-07-04: both original safety claims fell)
# ------
# This tool is ONLY safe on trees that stay IDLE until their next cold build.
# It is NO LONGER invoked automatically (build.py's detached daily call is
# disabled). Two independent failure modes were caught live on 2026-07-04:
#
#   1. (build-* trees — now excluded below) in-place O_TRUNC writers.
#   2. (cargo target trees — the remaining scope) MTIME POISONING: cargo's
#      invalidation is mtime-based, and dedup turns fingerprints,
#      invoked.timestamp, build-script binaries and output captures into
#      inodes SHARED across worktrees. A sibling worktree's build refreshes
#      those mtimes; your cargo then sees "fingerprint newer than source",
#      concludes fresh, and silently re-runs a STALE build script / reuses
#      stale artifacts (observed: an edited build.rs whose old compiled binary
#      kept executing — the edit never took effect, no error anywhere).
#      "Worst case is a spurious rebuild" was exactly backwards: the worst
#      case is a silently MISSING rebuild — false-green.
#
# The original claim (cargo emits via tempfile+rename, so rebuilds break links
# cleanly) is true for the .a/.rlib artifacts but irrelevant: the DECISION to
# rebuild depends on the shared-mtime state above. If you must reclaim disk on
# live trees, use `vibeos-cli prune` (idle worktrees) instead. After a dedup,
# a tree's next build should be COLD (`vibeos-cli build --clean`) to purge
# poisoned fingerprints.
# (Historical: validated dedup -> warm i386 build -> smoke boot 83s — that
# validation predates understanding failure mode 2 and should not be trusted.)
#
# cmake `build-<arch>` trees are EXCLUDED (2026-07-04, hard lesson): the
# atomic-rename story is FALSE there. spec2def/`cmake -E copy`/configure
# outputs and ccache-MISS compiles (the real clang writes straight to the
# output path) all open O_TRUNC IN PLACE — through a cross-worktree hardlink
# that truncates the shared inode under a sibling's running `ld.lld` mmap →
# SIGBUS (two amd64 builds died exactly this way: d3dx9 links, `clang: Bus
# error`, while another worktree rebuilt the same deduped objs). The 180s
# live-build guard below can't help: the hazard is the links LEFT BEHIND, not
# dedup racing a build. If your build-* still carries old cross-tree links
# (`find build-amd64 -type f -links +1`), break them:
#   find build-<arch> -type f -links +1 -print0 | \
#     xargs -0 -P8 -I{} sh -c 'cp -p "{}" "{}.t" && mv "{}.t" "{}"'
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
RECENT_SECS=180   # treat a tree written in the last N seconds as a live build

for a in "$@"; do
    case "$a" in
        --apply)        APPLY=1 ;;
        --force)        FORCE=1 ;;
        --targets-only) ;; # no-op: targets-only is now the ONLY mode (see SAFETY)
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
    # Cargo target trees ONLY — never cmake build-* (non-atomic writers; see
    # SAFETY above). --targets-only kept as an accepted no-op for back-compat.
    cands=("$wt/sdk/lib/rust/target")
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
