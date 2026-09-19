"""Canonical build path — every worktree builds at the SAME absolute path.

`tools/canon-run.sh` bind-mounts the current worktree at `/tmp/vibeos` inside an
unprivileged user+mount namespace and runs the build command there. Because the
absolute path is identical for every worktree, compiler argv, embedded paths
(`__FILE__`, panic messages, debug info), `getcwd()`, and CMake caches are all
worktree-independent. That is what makes cross-worktree cache sharing WORK:

  * sccache Rust hits across worktrees (measured 100% on 2026-07-03) — and no
    `--remap-path-prefix` needed, which sccache 0.8.2 refuses to cache at all
    (`Non-cacheable reasons: --remap-path-prefix`; the det-remap flag had silently
    disabled Rust caching entirely).
  * ccache C/C++ hits without BASEDIR path-rewriting subtleties.
  * cross-worktree byte-identity for free (deterministic-builds-plan.md rung 1's
    goal, without per-toolchain remap flags).
  * a canonical build dir is path-portable: provisioning a new worktree is a
    plain fresh `cmake` configure (fast on cache hits) — no cp -a + sed-repoint.

This module is the python-side owner: detection (is a build dir configured
canonically?), the wrapper argv, the userns availability probe, and the one-time
migration (fresh canonical configure) that `vibeos-cli build` performs
automatically on a legacy real-path build dir.
"""

from __future__ import annotations

import os
import subprocess
from functools import lru_cache
from pathlib import Path

CANON = "/tmp/vibeos"


def canon_path() -> str:
    return os.environ.get("VIBEOS_CANON", CANON)


@lru_cache(maxsize=1)
def available() -> bool:
    """Can this host do unprivileged user+mount namespaces? (WSL2: yes.)"""
    try:
        cp = subprocess.run(
            ["unshare", "--map-root-user", "--mount", "true"],
            capture_output=True,
            timeout=10,
        )
        return cp.returncode == 0
    except (OSError, subprocess.SubprocessError):
        return False


def wrap(repo: Path, cmd: list[str]) -> list[str]:
    """Wrap a build command to run under the canonical mount. The caller keeps
    its real `cwd`; canon-run.sh maps it to the same location in the canonical
    tree (cwd must be inside the worktree)."""
    return ["bash", str(repo / "tools" / "canon-run.sh"), *cmd]


def cmake_home(build_dir: Path) -> str | None:
    """CMAKE_HOME_DIRECTORY from CMakeCache.txt — where this build dir believes
    its source tree lives. None if not configured."""
    cache = build_dir / "CMakeCache.txt"
    try:
        for line in cache.read_text(errors="ignore").splitlines():
            if line.startswith("CMAKE_HOME_DIRECTORY:"):
                return line.split("=", 1)[1].strip()
    except OSError:
        return None
    return None


def is_canonical(build_dir: Path) -> bool:
    return cmake_home(build_dir) == canon_path()


# Cache vars worth carrying across the one-time canonize reconfigure (everything
# else re-derives from defaults; ARCH is passed explicitly by the caller).
_CARRY = ("CMAKE_BUILD_TYPE", "REACTOS_TEST_KERNEL", "SARCH", "OARCH", "TUNE")


def carried_options(build_dir: Path) -> list[str]:
    """-D options scraped from an existing legacy cache so the canonical
    reconfigure reproduces the same build flavor."""
    cache = build_dir / "CMakeCache.txt"
    opts: list[str] = []
    try:
        text = cache.read_text(errors="ignore")
    except OSError:
        return opts
    for line in text.splitlines():
        for var in _CARRY:
            if line.startswith(f"{var}:"):
                val = line.split("=", 1)[1].strip()
                if val:
                    opts.append(f"-D{var}={val}")
    return opts


def ensure_toolchain(repo: Path) -> bool:
    """`.build-tools/llvm-mingw` is a gitignored, machine-local symlink — fresh
    worktrees lack it, and the historical RELATIVE symlink (../../llvm-mingw-*)
    dangles under the canonical mount (it would resolve to /tmp/llvm-mingw-*).
    Ensure an ABSOLUTE symlink, resolving the toolchain from an existing target
    or a sibling llvm-mingw-* under the worktree parent. Returns False if no
    toolchain could be found."""
    link = repo / ".build-tools" / "llvm-mingw"
    target = None
    if link.exists():  # follows symlinks; True only if resolvable
        if not link.is_symlink():
            # A REAL directory here, not a link. Restoring this tree from a 7-Zip
            # extraction of the WSL image produced exactly that in all 8 worktrees:
            # the archiver materialised each symlink as an empty dir. readlink() then
            # raises EINVAL, so the function that exists to provision the toolchain
            # died on the one state it most needed to repair. An EMPTY one is that
            # artifact and is safe to drop; a POPULATED one is somebody's real local
            # toolchain and is left exactly where it is.
            try:
                link.rmdir()
            except OSError:
                return any(link.iterdir())
        else:
            target = link.resolve()
            if target.is_absolute() and str(link.readlink()).startswith("/"):
                return True
    if target is None:
        parent = Path(os.environ.get("VIBEOS_ROOT", repo.resolve().parent))
        hits = sorted(parent.glob("llvm-mingw-*"))
        if not hits:
            return False
        target = hits[0].resolve()
    link.parent.mkdir(parents=True, exist_ok=True)
    tmp = link.with_name("llvm-mingw.tmp")
    tmp.unlink(missing_ok=True)
    tmp.symlink_to(target)
    tmp.rename(link)
    return True


def configure_cmd(arch: str, extra: list[str] | None = None) -> list[str]:
    """The fresh canonical configure, run with cwd=<real build dir> through
    wrap() — all paths reference the canonical tree."""
    canon = canon_path()
    return [
        "cmake",
        "-G",
        "Ninja",
        f"-DCMAKE_TOOLCHAIN_FILE={canon}/toolchain-llvm-mingw.cmake",
        f"-DARCH:STRING={arch}",
        "-DENABLE_CCACHE=1",
        "-DCMAKE_BUILD_TYPE=Debug",  # project standard; carried opts override (last -D wins)
        *(extra or []),
        canon,
    ]
