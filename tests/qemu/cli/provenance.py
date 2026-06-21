"""Build provenance + staleness model (plan §6/§7, §20.1).

Delegate to git (source content) and ninja (build DAG); the stamp tracks only
the gaps — warm/cold, rust-source state, contributing-binary hashes. Staleness
is tiered cheap-first: a git-fingerprint match means FRESH with no further I/O.
git fingerprints (not mtime) so branch/worktree switches don't churn the verdict.
"""

from __future__ import annotations

import hashlib
import json
import subprocess
from dataclasses import dataclass, field
from pathlib import Path

from . import archcfg

STAMP_NAME = ".vibeos-build-stamp.json"


def _run(cmd: list[str], cwd: Path) -> str:
    try:
        return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, timeout=120).stdout
    except (OSError, subprocess.SubprocessError):
        return ""


def git_fingerprint(repo: Path, paths: list[str] | None = None) -> str:
    """Content-true fingerprint of the (optionally path-scoped) source state:
    HEAD + working-tree diff + untracked. Immune to mtime churn from git ops."""
    scope = paths or []
    head = _run(["git", "rev-parse", "HEAD"], repo).strip()
    diff = _run(["git", "diff", "HEAD", "--", *scope], repo)
    untracked = _run(["git", "status", "--porcelain", "--", *scope], repo)
    h = hashlib.sha256()
    h.update(head.encode())
    h.update(b"\0")
    h.update(diff.encode())
    h.update(b"\0")
    h.update(untracked.encode())
    return h.hexdigest()


def _sha256_file(p: Path) -> str:
    h = hashlib.sha256()
    try:
        with open(p, "rb") as f:
            for chunk in iter(lambda: f.read(1 << 20), b""):
                h.update(chunk)
    except OSError:
        return ""
    return h.hexdigest()


def discover_rust_targets(build_dir: Path) -> list[Path]:
    """Every cargo target dir under build-<arch>/ — ntoskrnl, hal, hal-mini,
    freeldr, rtl, win32k, per-DLL (build-*/dll/**), etc. Dynamic so new crates
    auto-expand the nuke/staleness set (plan §6 'Dynamic Target Discovery')."""
    if not build_dir.is_dir():
        return []
    found = set()
    for pat in ("rust-*-target", "rust-target"):
        for p in build_dir.rglob(pat):
            if p.is_dir():
                found.add(p)
    return sorted(found)


def discover_rust_archives(build_dir: Path) -> list[Path]:
    """Copied `lib*_rust.a` staticlibs — these are the outputs that link into
    consumers; a stale one is a classic false-green (rtl-priv/win32k/ntdll)."""
    if not build_dir.is_dir():
        return []
    return sorted(build_dir.rglob("lib*_rust.a"))


# arch -> the cargo workspace target stem (add_cargo_module's custom-target json
# stem == cargo's CARGO_TARGET_DIR subdir; gcc.cmake:add_cargo_module).
_CARGO_STEM = {"i386": "i686-kernel", "amd64": "x86_64-kernel", "arm64": "aarch64-kernel"}


def workspace_cargo_target(repo: Path, arch: str) -> Path | None:
    """The SHARED cargo cache the cargo-driven links (ntoskrnl.exe, win32k.sys,
    kernel32.dll, the *-cargo modules) build into: `sdk/lib/rust/target/<stem>`.

    It lives OUTSIDE build-<arch>, so discover_rust_targets (which only globs
    build-<arch>) never sees it — that's why a `--clean` left cargo serving a
    warm-cached object and agents had to `rm -rf sdk/lib/rust/target/<stem>` by
    hand (field-manual §9.6b). Returns the per-arch stem dir if present."""
    stem = _CARGO_STEM.get(archcfg.base_arch(arch))
    if not stem:
        return None
    p = repo / "sdk" / "lib" / "rust" / "target" / stem
    return p if p.is_dir() else None


def discover_rust_module_outputs(repo: Path, arch: str) -> list[Path]:
    """Copied cargo-module output binaries under build-<arch>. add_cargo_module
    builds each into the workspace cargo cache then COPIES it in, so nuking the
    target dirs alone leaves these copies NEWER than their .rs inputs → ninja
    sees "no work" and serves a stale binary (deterministic-builds-plan.md
    §stale-cargo / task #41). `--clean` removes them so ninja MUST re-run the
    cargo link. Boot-critical set: the kernel(s) + the two non-standard cargo
    links (win32k/kernel32) + the arm64 staged set (archcfg). Source-unchanged
    modules not listed here rebuild byte-identically anyway (the determinism
    invariant), so a stale copy of one is harmless; the listed ones are exactly
    the frequently-edited boot-critical links whose staleness is a false-green."""
    cfg = archcfg.arch_build(arch)
    build_dir = repo / f"build-{archcfg.base_arch(arch)}"
    rels = set(cfg.staged_bins_rel)  # arm64: ntoskrnl/hal/uefildr/win32k/kernel32/input/vid
    rels.update(
        (
            "ntoskrnl/ntoskrnl.exe",
            "ntoskrnl/ntkrnlmp.exe",  # amd64 MP kernel
            "win32ss/win32k.sys",
            "dll/win32/kernel32/kernel32.dll",
        )
    )
    return [build_dir / r for r in sorted(rels) if (build_dir / r).is_file()]


@dataclass
class Stamp:
    arch: str
    ts: int
    git_fingerprint: str
    rust_src_hash: str
    boot_media: str
    boot_media_sha: str
    cold: bool
    contributing: dict = field(default_factory=dict)  # rel path -> sha256
    rust_targets: list = field(default_factory=list)

    def write(self, build_dir: Path) -> Path:
        path = build_dir / STAMP_NAME
        tmp = build_dir / (STAMP_NAME + ".tmp")
        tmp.write_text(json.dumps(self.__dict__, indent=2))
        tmp.replace(path)  # atomic
        return path

    @staticmethod
    def read(build_dir: Path) -> Stamp | None:
        path = build_dir / STAMP_NAME
        try:
            d = json.loads(path.read_text())
            return Stamp(**d)
        except (OSError, ValueError, TypeError):
            return None


def rust_src_hash(repo: Path) -> str:
    return git_fingerprint(repo, list(archcfg.RUST_SRC_DIRS))


def contributing_hashes(repo: Path, arch: str) -> dict:
    """Content hash of each binary that gets packaged into the boot media, so
    preflight can detect a warm-cached/un-grafted contributor even when the
    whole-media hash 'passes' (ntdll #1, rtl-priv #2)."""
    cfg = archcfg.arch_build(arch)
    build_dir = repo / f"build-{archcfg.base_arch(arch)}"
    out = {}
    for rel in cfg.staged_bins_rel:
        f = build_dir / rel
        if f.is_file():
            out[rel] = _sha256_file(f)
    # x86: hash the key binaries the iso carries even though ninja tracks the
    # iso DAG — catches a warm-cached contributor whose content differs cold.
    if archcfg.base_arch(arch) in ("i386", "amd64"):
        for rel in (
            "ntoskrnl/ntoskrnl.exe",
            "hal/halx86/hal.dll",
            "boot/freeldr/freeldr/freeldr.sys",
        ):
            f = build_dir / rel
            if f.is_file():
                out[rel] = _sha256_file(f)
    return out


def make_stamp(repo: Path, arch: str, cold: bool, ts: int) -> Stamp:
    cfg = archcfg.arch_build(arch)
    build_dir = repo / f"build-{archcfg.base_arch(arch)}"
    media = repo / cfg.boot_media_rel
    targets = [str(p.relative_to(repo)) for p in discover_rust_targets(build_dir)]
    return Stamp(
        arch=archcfg.base_arch(arch),
        ts=ts,
        git_fingerprint=git_fingerprint(repo),
        rust_src_hash=rust_src_hash(repo),
        boot_media=cfg.boot_media_rel,
        boot_media_sha=_sha256_file(media),
        cold=cold,
        contributing=contributing_hashes(repo, arch),
        rust_targets=targets,
    )


@dataclass
class Staleness:
    fresh: bool
    reasons: list  # human strings, e.g. ["rust source changed", ...]
    warm: bool  # cold==False on the stamp (advisory)
    media_missing: bool

    @property
    def stale(self) -> bool:
        return (not self.fresh and not self.media_missing) or self.media_missing


def _ninja_has_work(build_dir: Path, target: str) -> bool:
    """Reconfigure-then-dry-run: cmake re-runs (cheap/idempotent) so the dirty
    set is evaluated against the *updated* graph, then `ninja -n` reports work.
    We treat ONLY a 'no work to do' as clean (plan §5/§20 'don't filter naively')."""
    out = _run(["ninja", "-n", target], build_dir)
    # ninja prints "ninja: no work to do." when clean. The reconfigure line
    # ("Re-running CMake...") is expected and not by itself staleness; we look
    # for actual build edges.
    if not out:
        return False
    if "no work to do" in out:
        return False
    # Any non-cmake build edge line means real work pending.
    for line in out.splitlines():
        s = line.strip()
        if not s or s.startswith("ninja:"):
            continue
        if "Re-running CMake" in s or "Regenerating ninja files" in s:
            continue
        return True
    return False


def check_staleness(repo: Path, arch: str) -> Staleness:
    cfg = archcfg.arch_build(arch)
    build_dir = repo / f"build-{archcfg.base_arch(arch)}"
    media = repo / cfg.boot_media_rel
    stamp = Stamp.read(build_dir)

    if not media.is_file():
        return Staleness(
            fresh=False, reasons=["boot media absent — run `build`"], warm=False, media_missing=True
        )
    if stamp is None:
        return Staleness(
            fresh=False,
            reasons=["no build stamp — never built via vibeos-cli"],
            warm=False,
            media_missing=False,
        )

    warm = not stamp.cold
    # Tier 1: cheap fingerprint compare → FRESH, done.
    if git_fingerprint(repo) == stamp.git_fingerprint:
        return Staleness(fresh=True, reasons=[], warm=warm, media_missing=False)

    # Tier 2: source changed — escalate to authoritative per-layer checks.
    reasons = []
    if rust_src_hash(repo) != stamp.rust_src_hash:
        reasons.append(
            "rust source changed since build" + (" (last build WARM — may mask)" if warm else "")
        )
    if _ninja_has_work(build_dir, cfg.ninja_targets[0]):
        reasons.append(f"ninja reports work for `{cfg.ninja_targets[0]}`")
    # arm64 re-stage trap: a contributing binary newer than boot.img.
    if cfg.needs_mkramdisk:
        media_mtime = media.stat().st_mtime
        for rel in cfg.staged_bins_rel:
            f = build_dir / rel
            if f.is_file() and f.stat().st_mtime > media_mtime:
                reasons.append(f"{rel} newer than boot.img — ramdisk not re-staged")
                break
    # media swapped out of band
    if _sha256_file(media) != stamp.boot_media_sha:
        reasons.append("boot media changed since stamp (rebuilt out of band?)")

    if not reasons:
        # Source changed but nothing build-relevant (docs, this plan).
        return Staleness(
            fresh=True, reasons=["clean-diff (non-build change)"], warm=warm, media_missing=False
        )
    return Staleness(fresh=False, reasons=reasons, warm=warm, media_missing=False)
