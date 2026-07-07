#!/usr/bin/env bash
#
# canon-run.sh — run a build command with this worktree bind-mounted at the
# CANONICAL BUILD PATH (/tmp/vibeos), inside an unprivileged user+mount namespace.
#
# WHY THIS EXISTS (build-cache-and-determinism-plan.md → "canonical build path")
# ------------------------------------------------------------------------------
# Every worktree building at its own absolute path was the root of three problems:
#   1. sccache could not share Rust compiles across worktrees (the rustc argv embeds
#      the worktree path), and the det-remap `--remap-path-prefix` workaround made
#      every Rust compile NON-CACHEABLE in sccache (measured 2026-07-03: sccache 0.8.2
#      reports `Non-cacheable reasons: --remap-path-prefix`).
#   2. Byte-identity across worktrees needed per-path remap flags in every toolchain
#      (rustc, clang, cc-rs) — fragile, incomplete, and cache-hostile (see 1).
#   3. CMake bakes absolute paths into build dirs, forcing the cp -a + sed-repoint
#      ceremony for every worktree build dir (AGENTS.md "Worktree Discipline" #3).
#
# Building under ONE canonical path dissolves all three: every worktree compiles at
# the identical absolute path, so compiler argv, embedded paths (__FILE__, panic
# messages, debug info), getcwd(), and CMake caches are worktree-independent.
# Measured 2026-07-03: 100% sccache Rust hit compiling the same crate from a second
# worktree, zero remap flags.
#
# The bind mount lives in a private mount namespace (`unshare --map-root-user
# --mount`), so nothing leaks to the host and no root is needed. WSL2 allows
# unprivileged userns (validated on this kernel). getcwd() inside the namespace
# reports the canonical path (mounts are real, unlike symlinks), so nothing
# re-derives the physical worktree path.
#
# /tmp/vibeos is only a MOUNTPOINT (an empty dir on the real fs) — no artifacts are
# stored under /tmp; everything writes through the bind into the real worktree.
#
# SCCACHE SERVER AFFINITY (the one sharp edge)
# --------------------------------------------
# sccache is client/server; the server resolves paths in the mount namespace it was
# SPAWNED in. A server left over from a build in another worktree would resolve
# /tmp/vibeos to THAT worktree and compile the wrong sources. So we stop any live
# server on entry; the first rustc in this namespace auto-spawns a fresh one that
# sees our mount. (Builds are host-exclusive via hostlock, so this is race-free.
# A stale in-namespace server is harmless to non-canonical compiles — its namespace
# still sees all real paths.)
#
# USAGE
#   tools/canon-run.sh <cmd> [args...]     # cwd is mapped into the canonical tree
#   tools/canon-run.sh ninja ntoskrnl      # e.g. from inside build-<arch>/
#   VIBEOS_CANON=/some/path ...            # override the canonical mountpoint
#
set -euo pipefail

CANON="${VIBEOS_CANON:-/tmp/vibeos}"

if [ $# -lt 1 ]; then
    echo "usage: tools/canon-run.sh <cmd> [args...]  (run from inside the worktree)" >&2
    exit 2
fi

ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || {
    echo "canon-run: not inside a git worktree" >&2
    exit 2
}
REL="$(realpath --relative-to="$ROOT" "$PWD")"
case "$REL" in
    ..*) echo "canon-run: cwd $PWD is outside the worktree $ROOT" >&2; exit 2 ;;
esac

mkdir -p "$CANON"
# Refuse if real content sits at the mountpoint on the host (would be shadowed by
# the mount — likely something squatting it). EMPTY dirs are fine: a stray tool run
# against a canonical build dir outside the mount (e.g. a cmake regenerate) creates
# hollow directory skeletons here; mounting over them is harmless.
if find "$CANON" -mindepth 1 \( -type f -o -type l \) -print -quit 2>/dev/null | grep -q .; then
    echo "canon-run: $CANON contains real files on the host — refusing (nothing" >&2
    echo "should store artifacts at the mountpoint; inspect and clear it manually)" >&2
    exit 2
fi

# sccache is client/server and the server resolves /tmp/vibeos paths in the
# mount namespace it was STARTED in. The old "stop any server so ours respawns
# in our namespace" approach is a fleet hazard: raw canon-run invocations don't
# hold the hostlock, so two worktrees building concurrently would stop each
# other's servers mid-build — cache-miss compiles then execute inside the OTHER
# worktree's namespace and silently read ITS sources (2026-07-07 phantom-build
# incident: three gate builds compiled another lane's in-progress lib.rs).
# Fix: pin a PER-WORKTREE server port (stable hash of the worktree root). Each
# worktree gets its own daemon, first spawned inside its own namespace, so
# /tmp/vibeos always resolves to the right tree; the on-disk cache stays shared
# and content-addressed (safe by design).
if command -v sccache >/dev/null 2>&1 && [ -z "${SCCACHE_SERVER_PORT:-}" ]; then
    _hash=$(printf '%s' "$ROOT" | cksum | cut -d' ' -f1)
    export SCCACHE_SERVER_PORT=$((4300 + _hash % 100))
fi

exec unshare --map-root-user --mount /bin/sh -c '
    root="$1"; canon="$2"; rel="$3"; shift 3
    mount --bind "$root" "$canon" || exit 1
    cd "$canon/$rel" || exit 1
    exec "$@"
' canon-sh "$ROOT" "$CANON" "$REL" "$@"
