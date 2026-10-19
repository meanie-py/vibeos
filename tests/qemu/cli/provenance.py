"""Build provenance + staleness model (plan §6/§7, §20.1).

Delegate to git (source content) and ninja (build DAG); the stamp tracks only
the gaps — warm/cold, rust-source state, contributing-binary hashes. Staleness
is tiered cheap-first: a git-fingerprint match means FRESH with no further I/O.
git fingerprints (not mtime) so branch/worktree switches don't churn the verdict.
"""

from __future__ import annotations

import functools
import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

from . import archcfg

STAMP_NAME = ".vibeos-build-stamp.json"


class ProvenanceError(RuntimeError):
    """A command this oracle's verdict depends on did not run successfully.

    Raised instead of degrading, because a staleness oracle that answers from
    PARTIAL output is worse than one that refuses to answer: both directions are
    live. This helper used to `return ""` on any exception AND ignore the exit
    code entirely, so a degraded `git` call either shifted the fingerprint (false
    RED — the BUILD_INCOMPLETE that halts the merge train, with no evidence left
    behind to diagnose it) or emptied `cargo_module_rels` (false GREEN — the
    cargo-link staleness net silently switched off). Neither logged anything."""


def _run_bytes(
    cmd: list[str], cwd: Path, *, ok_codes: tuple[int, ...] = (0,), env: dict | None = None
) -> bytes:
    """Run `cmd` and return its stdout as RAW BYTES, or raise ProvenanceError
    naming the command, rc and stderr.

    Bytes, not text, because the repo tracks files that are not UTF-8 and git
    echoes their content verbatim: `git diff HEAD` over a latin-1 file emitted
    0xe9 and a strict decode took down the whole CLI with a UnicodeDecodeError
    from inside stamp-making, so every build and land failed for as long as
    that file was dirty. Decoding is the caller's problem, and a fingerprint
    should not decode at all.

    `ok_codes` widens the accepted set for commands whose non-zero exit is
    meaningful rather than a failure — `git grep` exits 1 for "no matches",
    which is a legitimate empty result, not a broken oracle."""
    try:
        cp = subprocess.run(cmd, cwd=cwd, capture_output=True, timeout=120, env=env)
    except (OSError, subprocess.SubprocessError) as e:
        raise ProvenanceError(f"{' '.join(cmd)} (cwd={cwd}): {type(e).__name__}: {e}") from e
    if cp.returncode not in ok_codes:
        err = cp.stderr.decode("utf-8", "replace").strip().splitlines()
        raise ProvenanceError(
            f"{' '.join(cmd)} (cwd={cwd}) exited {cp.returncode}: "
            + (err[0][:200] if err else "<no stderr>")
        )
    return cp.stdout


def _run(cmd: list[str], cwd: Path, *, ok_codes: tuple[int, ...] = (0,)) -> str:
    """`_run_bytes`, decoded leniently. Undecodable bytes become U+FFFD rather
    than an exception: every text caller here parses git's own framing (paths,
    porcelain status, ref names), which is ASCII, and a mangled character
    inside a payload line must not be able to abort a build."""
    return _run_bytes(cmd, cwd, ok_codes=ok_codes).decode("utf-8", "replace")


def git_head(repo: Path) -> str:
    """The HEAD sha. Recorded in the stamp ALONGSIDE the fingerprint so that a
    later mismatch can say WHICH input moved: a HEAD change (a merge/fetch landed
    under the build) reads completely differently from a working-tree change, and
    the fingerprint alone conflates them into one useless "source changed"."""
    return _run(["git", "rev-parse", "HEAD"], repo).strip()


def git_fingerprint(repo: Path, paths: list[str] | None = None) -> str:
    """Content-true fingerprint of the (optionally path-scoped) source state:
    committed content + working-tree diff + untracked. Immune to mtime churn
    from git ops.

    **Scoped calls must not fold in the HEAD SHA, and this is the whole reason
    the scoped and unscoped branches differ.** A commit sha names the whole
    tree, so hashing it defeats every exclusion in the pathspec: a docs-only
    commit moves HEAD, the fingerprint moves, and `build_src_hash` reports
    build-relevant change for a file it was explicitly told to ignore. Every
    exclusion in NON_BUILD_PATHSPEC was added to stop a false STALE_MEDIA, and
    every one of them worked only while the change sat UNCOMMITTED — committing
    it brought the false positive straight back. Measured on this repo: a
    docs-only commit leaves `git diff --name-only HEAD~1 HEAD -- <scope>` empty
    and still moved the fingerprint, which is how it was found; the oracle's own
    explanation string had been saying "HEAD MOVED SINCE THE BUILD ... NOT a
    stale artifact" the entire time.

    So a scoped call hashes the INDEX ENTRIES in scope (`ls-files -s` gives mode
    + blob sha per path) instead. That is content-true, respects exclusions, and
    never mentions a commit. 21ms for 38,845 files here. The unscoped call keeps
    the HEAD sha: it is the coarse tier-1 "did anything at all change" gate, for
    which whole-tree identity is exactly the right question, and it is cheap.

    Raises ProvenanceError if any of its git calls fails — see _run."""
    scope = paths or []
    # Hash the RAW bytes: a lossy decode would map distinct non-UTF-8 contents
    # onto the same U+FFFD run, i.e. two different working trees onto one
    # fingerprint — a false GREEN in the staleness net.
    if scope:
        committed = _run_bytes(["git", "ls-files", "-s", "--", *scope], repo)
    else:
        committed = git_head(repo).encode()
    diff = _run_bytes(["git", "diff", "HEAD", "--", *scope], repo)
    untracked = _run_bytes(["git", "status", "--porcelain", "--", *scope], repo)
    h = hashlib.sha256()
    h.update(committed)
    h.update(b"\0")
    h.update(diff)
    h.update(b"\0")
    h.update(untracked)
    return h.hexdigest()


def tree_content_hash(repo: Path, paths: list[str]) -> str:
    """Hash of the path-scoped WORKING-TREE content alone: the index entries (mode + blob
    sha per path) that `git add -A` would record, built in a scratch copy of the index.

    `git_fingerprint`'s scoped branch is not this. It hashes the CURRENT index plus the text
    of `git diff HEAD` plus `git status`, so the same bytes hash differently committed and
    uncommitted. Build a dirty tree, commit exactly what was built, and every smoke said
    STALE_MEDIA ("119 path(s) vs the build's commit") for a tree identical to the one
    compiled, per the build's own warm-receipt. A content question needs an answer that
    does not depend on git bookkeeping, and this is that answer. ~0.1s here.

    Raises ProvenanceError if any of its git calls fails -- see _run."""
    index = Path(_run(["git", "rev-parse", "--git-path", "index"], repo).strip())
    index = index if index.is_absolute() else repo / index
    with tempfile.TemporaryDirectory(prefix="vibeos-prov-") as tmp:
        scratch = Path(tmp) / "index"
        if index.is_file():
            shutil.copy2(index, scratch)
        env = dict(os.environ, GIT_INDEX_FILE=str(scratch))
        # Stage the WHOLE tree and scope only the listing: `git add -A -- <pathspec>`
        # exits 1 when an exclude names a gitignored path (`.claude`), and that took a
        # build down in make_stamp.
        _run_bytes(["git", "add", "-A"], repo, env=env)
        entries = _run_bytes(["git", "ls-files", "-s", "--", *paths], repo, env=env)
    return hashlib.sha256(entries).hexdigest()


def _sha256_file(p: Path) -> str:
    h = hashlib.sha256()
    try:
        with open(p, "rb") as f:
            for chunk in iter(lambda: f.read(1 << 20), b""):
                h.update(chunk)
    except OSError:
        return ""
    return h.hexdigest()


#: How many times a racing walk is redone before we call it a real fault.
_RGLOB_RETRIES = 5


def rglob_stable(root: Path, pattern: str) -> list[Path]:
    """`root.rglob(pattern)`, redone when a CONCURRENT build mutates the tree mid-walk.

    A parallel `gate` builds all three arches inside ONE worktree against ONE shared cargo
    target dir, so an arm64 smoke's staleness check walks `sdk/lib/rust/target/` at the very
    moment the i386 `--clean` build is deleting `i686-kernel/` out of it. pathlib's generator
    turns the vanished directory into a FileNotFoundError and dies mid-iteration, which took
    out a whole gate's arm64 smoke and bounced the batch.

    Redo the walk rather than swallow the error and return what we had: a silently short list
    reads as "this crate has no staticlib", which is a staleness verdict, not a missing one.
    The walk is cheap and the race window is milliseconds; if it still races
    `_RGLOB_RETRIES` times the tree is genuinely broken and the OSError should surface.
    """
    for attempt in range(_RGLOB_RETRIES):
        try:
            return list(root.rglob(pattern))
        except OSError:
            if attempt == _RGLOB_RETRIES - 1:
                raise
    raise AssertionError("unreachable")  # pragma: no cover


def discover_rust_targets(build_dir: Path) -> list[Path]:
    """Every cargo target dir under build-<arch>/ — ntoskrnl, hal, hal-mini,
    freeldr, rtl, win32k, per-DLL (build-*/dll/**), etc. Dynamic so new crates
    auto-expand the nuke/staleness set (plan §6 'Dynamic Target Discovery')."""
    if not build_dir.is_dir():
        return []
    found = set()
    for pat in ("rust-*-target", "rust-target"):
        for p in rglob_stable(build_dir, pat):
            if p.is_dir():
                found.add(p)
    return sorted(found)


def discover_rust_archives(build_dir: Path) -> list[Path]:
    """Copied `lib*_rust.a` staticlibs — these are the outputs that link into
    consumers; a stale one is a classic false-green (rtl-priv/win32k/ntdll)."""
    if not build_dir.is_dir():
        return []
    return sorted(rglob_stable(build_dir, "lib*_rust.a"))


# arch -> the cargo workspace target stem (add_cargo_module's custom-target json
# stem == cargo's CARGO_TARGET_DIR subdir; gcc.cmake:add_cargo_module).
_CARGO_STEM = {
    "i386": "i686-kernel",
    "amd64": "x86_64-pc-windows-gnu",
    "arm64": "aarch64-pc-windows-gnullvm",
}


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


def workspace_cargo_targets(repo: Path, arch: str) -> list[Path]:
    """Outer and nested staticlib targets for THIS arch, including storage pilots.

    Do not remove whole inner target directories: they contain other arches and
    host build scripts that a concurrent architecture build may be using.
    """
    base = archcfg.base_arch(arch)
    stem = _CARGO_STEM.get(base)
    if stem is None:
        return []
    root = repo / "sdk/lib/rust/target"
    if root.resolve() != repo.resolve() / "sdk/lib/rust/target":
        raise ValueError(f"refusing symlinked workspace target: {root}")
    stems = [stem, "i686-pc-windows-gnu"] if base == "i386" else [stem]
    parents = [
        root,
        *sorted(root.glob("*-rustlib")),
        *sorted(root.glob("*-cargo-link")),
        *sorted(root.glob("cell-link-v1-*")),
    ]
    result = []
    for parent in parents:
        for target in stems:
            path = parent / target
            if parent.is_symlink() or path.is_symlink():
                raise ValueError(f"refusing symlinked nested Cargo target: {path}")
            if path.is_dir():
                result.append(path)
    return sorted(result)


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


@functools.cache
def _workspace_members(repo_str: str) -> frozenset[str] | None:
    """Crate dir names listed in sdk/lib/rust/Cargo.toml's `members`.

    A crate that is NOT a member is built by nothing, so none of its .rs can be
    inside any binary. That is exactly how a TRANSLATED-BUT-NOT-YET-FLIPPED module
    is parked (netcfgx set the rule: a crate with a red `cargo check` stays out of
    `members` so it cannot take the workspace check down for every other lane).

    Returns None when the manifest is absent or carries no `members`: membership is a
    REFINEMENT of the naming convention, not a replacement, so an unreadable manifest
    falls back to convention rather than silently declaring nothing cohab — which would
    disable the whole staleness net. The real tree always has it; fixtures may not.
    """
    try:
        text = (Path(repo_str) / "sdk" / "lib" / "rust" / "Cargo.toml").read_text()
    except OSError:
        return None
    block = re.search(r"members\s*=\s*\[(.*?)\]", text, re.S)
    if block is None:
        return None
    return frozenset(re.findall(r'"([^"]+)"', block.group(1)))


def _crate_for_stem(repo: Path, stem: str) -> Path | None:
    """Map a staged binary stem (e.g. 'advapi32', 'kernel32_vista') to its
    Rust-cohab crate dir under sdk/lib/rust/, or None if it isn't Rust-cohab.
    Binaries carry '_' in version suffixes (kernel32_vista.dll) while crates use
    '-' (kernel32-vista-rust), so try both separators. By convention a cohab
    crate is `<stem>-rust` and produces a same-named `<stem>.{sys,dll,exe}`.

    The convention alone is not enough, and the gap halted the merge train on
    2026-08-04: `msv1_0-rust` existed (translated, committed, PARKED out of
    `members` pending its grind) while dll/win32/msv1_0 still built from C, so the
    staleness net paired the fresh .rs against the C-built msv1_0.dll and reported
    `cohab flip not rebuilt` for a flip that had not happened. No rebuild can clear
    that. Requiring workspace membership is the discriminator: a parked crate is
    not a member, so it is not cohab-linked and its DLL is legitimately the C one.
    """
    rust = repo / "sdk" / "lib" / "rust"
    members = _workspace_members(str(repo))
    linked = _cmake_named_crates(str(repo))
    for name in (f"{stem}-rust", f"{stem.replace('_', '-')}-rust"):
        c = rust / name
        if (
            (c / "Cargo.toml").is_file()
            and (members is None or name in members)
            and (linked is None or name in linked)
        ):
            return c
    return None


def _crate_for_rel(repo: Path, rel: str) -> Path | None:
    """`_crate_for_stem` for a built binary, with its extension as the tiebreak.

    The stem alone is ambiguous when two binaries share one: wdmaud.sys is wdmaud-rust
    while wdmaud.drv is wdmaud_drv-rust. By stem both mapped to the KERNEL crate, so the
    staleness net judged wdmaud.drv against libwdmaud_rust.a, which it never links, and a
    cold arm64 gate whose kernel lib happened to finish after the .drv link went
    BUILD_INCOMPLETE on a current tree (integration batch 4, 2026-10-17). It also meant
    wdmaud.drv's real crate was never checked. `<stem>_<ext>-rust` wins when it is a
    linked member crate; otherwise the stem convention stands.
    """
    path = Path(rel)
    ext = path.suffix.lower().lstrip(".")
    if ext:
        exact = _crate_for_stem(repo, f"{path.stem}_{ext}")
        if exact is not None:
            return exact
    return _crate_for_stem(repo, path.stem)


@functools.lru_cache(maxsize=4)
def _cmake_named_crates(repo: str) -> frozenset[str] | None:
    """Every `<name>-rust` crate some CMake file names (a cohab flip's
    `rust_staticlib_variant(..)`, a cargo module's BODY_CRATE).

    Membership alone stopped being enough once `rustconv port` arrived: it makes a crate a
    workspace member when work on it STARTS (cargo check needs that), long before the flip.
    So spoolsv.exe, still built from C, was paired with spoolsv-rust mid-integration and the
    arm64 build reported `cohab flip not rebuilt` for a flip that had not happened. A crate
    no CMake file names is not linked into anything yet. None when git cannot answer.
    """
    try:
        out = subprocess.run(
            [
                "git",
                "-C",
                repo,
                "grep",
                "-ohE",
                r"[A-Za-z0-9_-]+-rust\b",
                "--",
                "*CMakeLists.txt",
                "*.cmake",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError:
        return None
    names = frozenset(out.stdout.split()) if out.returncode == 0 else frozenset()
    # A tree whose CMake names no crate at all cannot answer this question.
    return names or None


def _newest_rs_mtime(crate: Path) -> float:
    """Newest .rs mtime under a crate — its source-change signal. The cohab
    binary is relinked AFTER cargo rebuilds the rlib from these, so a fresh
    build always leaves the binary newer than every .rs here.

    CAVEAT (why check A prefers _depinfo_rs_mtime when cargo's output exists):
    this rglob counts ORPHAN .rs files — parked translations on disk that no
    `mod` declaration reaches (e.g. explorer-rust's appbar.rs, folded into
    traywnd.rs but kept per the never-delete-translations rule). Cargo
    correctly ignores them, so a sweep that touches an orphan makes this
    signal newer than cargo's output FOREVER — a false RED no rebuild can
    clear (2026-07-18 arm64 gate). Cargo's dep-info is the authoritative
    input list; fall back here only when it's unavailable."""
    newest = 0.0
    root = crate / "src" if (crate / "src").is_dir() else crate
    for p in root.rglob("*.rs"):
        try:
            newest = max(newest, p.stat().st_mtime)
        except OSError:
            continue
    return newest


def _depinfo_rs_mtime(repo: Path, cargo_out: Path, crate: Path) -> float | None:
    """Newest mtime of the CRATE'S OWN .rs inputs cargo ITSELF recorded for
    this output — parsed from the make-style dep-info (.d) cargo writes beside
    every artifact. Unlike the rglob it excludes ORPHAN .rs by construction
    (files no `mod` reaches — parked translations cargo never compiles).

    Scoped to the crate dir ON PURPOSE: the .d lists the full transitive
    closure (ntkrnl-sys etc.), and comparing against that flags every cohab
    crate as stale whenever a shared dep changes — TRUE in principle, but the
    cohab custom commands' cmake GLOB deps only cover each crate's own src, so
    the build graph cannot clear that staleness and the check would red every
    gate unfixably (2026-07-18: 12 crates flagged after a tree-wide sweep
    touched ntkrnl-sys). The cross-crate-deps graph gap is tracked separately;
    this check keeps its original per-crate semantic, minus the orphans.

    Paths inside are CANONICAL (/tmp/vibeos/..., the canon-run bind mount) and
    must be remapped to the repo before stat'ing — the mount only exists
    during a build. Returns None when the .d is missing or lists none of this
    crate's .rs (caller falls back to the rglob)."""
    from . import canon

    d = cargo_out.with_suffix(".d")
    try:
        text = d.read_text()
    except OSError:
        return None
    prefix = canon.CANON.rstrip("/") + "/"
    crate_res = crate.resolve()
    newest = None
    for line in text.splitlines():
        _target, sep, deps = line.partition(": ")
        if not sep:
            continue
        # Escaped-space tolerant tokenization ("\ " inside a path).
        for tok in deps.replace("\\ ", "\x00").split():
            tok = tok.replace("\x00", " ")
            if not tok.endswith(".rs"):
                continue
            if tok.startswith(prefix):
                tok = str(repo / tok[len(prefix) :])
            p = Path(tok)
            try:
                if not p.resolve().is_relative_to(crate_res):
                    continue
                mt = p.stat().st_mtime
            except OSError:
                continue
            newest = mt if newest is None else max(newest, mt)
    return newest


def mkramdisk_required_rels(repo: Path) -> list[str]:
    """Build-relative paths of the arm64 ramdisk boot set: every livecd-list source
    under build-arm64/ (mkramdisk.sh lays that list out, tools/ramdisk_tree.py). It is
    the AUTHORITATIVE staged set and self-maintaining -- an add_cd_file(FOR livecd)
    joins the freshness net with no edit here. Empty before the first configure."""
    lst = repo / "build-arm64" / "boot" / "livecd.Debug.lst"
    try:
        text = lst.read_text(errors="ignore")
    except OSError:
        return []
    rels = []
    for line in text.splitlines():
        src = line.split("=", 1)[-1]
        for root in ("/tmp/vibeos/build-arm64/", f"{repo}/build-arm64/"):
            if src.startswith(root):
                rels.append(src[len(root) :])
    return list(dict.fromkeys(rels))


def boot_set_cohab_rels(repo: Path, arch: str) -> list[str]:
    """Build-relative paths in the arm64 boot set that map to a `<stem>-rust`
    cohab crate — i.e. the Rust binaries that actually boot. Used by both the
    staleness net (gate) and `build` (so they're force-rebuilt regardless of
    whether anyone remembered to add them to archcfg.ninja_targets). Empty for
    x86, where `ninja livecd` rebuilds every CMAKE-linked cohab binary. (That
    exemption is NOT safe for cargo-LINKED modules — `ninja livecd` only re-runs
    the cargo link if its declared deps changed, which false-greened a stale
    ntoskrnl.exe on 2026-07-04; cargo_link_staleness_reasons covers those on
    all arches.)"""
    cfg = archcfg.arch_build(arch)
    if not cfg.needs_mkramdisk:
        return []
    boot_set = set(mkramdisk_required_rels(repo)) | set(cfg.staged_bins_rel)
    # Only linked images: the livecd list also carries acpi.inf, cdrom.inf ... whose
    # stems match a cohab crate but which no staticlib ever feeds.
    return sorted(
        rel
        for rel in boot_set
        if Path(rel).suffix.lower() in _PE_SUFFIXES and _crate_for_rel(repo, rel) is not None
    )


_PE_SUFFIXES = frozenset(
    {".exe", ".dll", ".sys", ".drv", ".cpl", ".ocx", ".efi", ".acm", ".ax", ".tlb", ".mui", ".scr"}
)


def cohab_staleness_reasons(repo: Path, arch: str) -> list[str]:
    """The arm64 staged-build false-green net (the fastfat/ramdisk/advapi32 class).

    arm64 builds a curated staged set + ramdisk instead of a full `ninja livecd`,
    so a Rust-cohab driver whose .sys/.dll is NOT in archcfg.ninja_targets is
    never rebuilt — mkramdisk then stages the STALE pre-flip C binary and the gate
    boots it, false-greening the Rust translation (bitten 6x: fastfat->ramdisk->
    pl011->gdi32->user32->advapi32, field-manual §9.6c.1). It's invisible to the
    other staleness checks: the stale C binary is OLDER than boot.img (so the
    re-stage trap doesn't fire) and isn't ninja_targets[0] (so the ninja dry-run
    misses it).

    Self-maintaining net (no archcfg list to forget): for every boot-set binary
    mapping to a `<stem>-rust` crate, check the TWO links in the chain that can
    actually break. Run unconditionally (a flip that wasn't rebuilt leaves the
    build-stamp fingerprint matching, so the tier-1 fast path can't be trusted for
    it); the gate flow is build->smoke back-to-back so mtimes are authoritative.

        .rs --cargo--> <target>/release/lib<c>.a --copy_if_different--> lib<c>.a --link--> artifact
                       [A: touched on every cargo run]                  [B: touched only on CHANGE]

      A. cargo's own output must be at least as new as the newest .rs
         => "cargo has run since the sources changed". Catches the original bug:
            a crate that is never built for this arch at all (gdi32/user32/...).
      B. the artifact must be at least as new as the COPIED staticlib it links
         => "the link consumed the current .a". Catches the other half: the crate
            rebuilt and changed, but the .dll was not in the staged target set so
            ninja never relinked it, and mkramdisk staged the old binary.

    WHY NOT `artifact.mtime >= newest .rs` (the obvious rule — it false-RED the
    arm64 gate, field-manual §9.81): that is a proxy, and it is wrong whenever an
    edit is CODEGEN-NEUTRAL. cargo rebuilds, emits a byte-identical .a,
    copy_if_different deliberately does not touch the staged copy, nothing
    relinks (correctly — there is nothing to relink), and the artifact stays older
    than the .rs FOREVER. The ws2_32 `WPUCompleteOverlappedRequest` signature fix
    (DWORD_PTR->DWORD on an unimplemented stub with unused params) did exactly
    this and red the arm64 gate on every run while the build was perfectly current.
    A false RED is not the safe direction: it is the mirror of the false green this
    check exists to catch, and it teaches people to `touch` binaries to shut it up.
    Splitting the chain at the copy is what fixes it — B compares against the file
    whose mtime moves ONLY when the content really changed, so an identical
    rebuild is silent, while A still proves the rebuild happened at all.

    WHY NOT `ninja -n` (the other obvious rule — it is BLIND, §9.81): CMake's
    CONFIGURE_DEPENDS globs make the glob-verify edge dirty by design on every
    build, so under -n (which executes nothing) ninja assumes the cmake stamp
    changes, marks the regen edge dirty, and STOPS PLANNING — it never evaluates
    the target you asked about. `_ninja_has_work` returns False for `livecd` on
    amd64 with a rebuild genuinely pending. Do not route this check through it."""
    build_dir = repo / f"build-{archcfg.base_arch(arch)}"
    reasons = []
    index = None  # built on the first crate that needs it, then shared by all of them
    for rel in boot_set_cohab_rels(repo, arch):
        crate = _crate_for_rel(repo, rel)
        assert crate is not None  # boot_set_cohab_rels only returns mapped rels
        artifact = build_dir / rel
        if not artifact.is_file():
            reasons.append(
                f"{rel}: {crate.name} cohab crate but build artifact absent — never built"
            )
            continue
        newest_rs = _newest_rs_mtime(crate)
        if index is None:
            index = _StaticlibIndex(repo, build_dir)
        cargo_out, copied = _crate_staticlibs(repo, build_dir, crate, index)

        if cargo_out is None and copied is None:
            # No staticlib anywhere in this crate's chain => it is not a
            # staticlib-cohab at all but a cargo-LINKED module (ntoskrnl-cargo /
            # win32k produce the binary directly through reactos-link, consuming
            # the crate as an rlib). "Staticlib absent" means "different build
            # shape", NOT "never built" — reading it as the latter reported
            # ntoskrnl.exe and win32k.sys as never-compiled on a green tree. Keep
            # the original artifact-vs-source rule for these; cargo_link_staleness
            # _reasons is the check that actually models that shape.
            if newest_rs and artifact.stat().st_mtime < newest_rs:
                reasons.append(
                    f"{rel}: stale — {crate.name} has .rs newer than the built binary "
                    f"(cohab flip not rebuilt; `build` force-rebuilds it, or add to ninja_targets)"
                )
            continue

        # A. did cargo run since the sources changed?
        if newest_rs and cargo_out is None:
            reasons.append(
                f"{rel}: stale — {crate.name} has a staged staticlib but cargo's own "
                f"output is missing for this arch (cohab crate never compiled here)"
            )
            continue
        if newest_rs and cargo_out is not None:
            # Prefer cargo's own dep-info over the rglob: the rglob counts
            # orphan .rs files cargo never compiles (see _newest_rs_mtime
            # CAVEAT), which made this arm false-RED unfixably.
            depinfo_rs = _depinfo_rs_mtime(repo, cargo_out, crate)
            effective_rs = depinfo_rs if depinfo_rs is not None else newest_rs
            if cargo_out.stat().st_mtime < effective_rs:
                reasons.append(
                    f"{rel}: stale — {crate.name} .rs is newer than cargo's own output "
                    f"({cargo_out.name}); the crate was not rebuilt (add to ninja_targets?)"
                )
                continue
        # B. did the link consume the current staticlib?
        if copied is not None and _belongs_to_sibling(copied, artifact):
            continue
        if copied is not None and artifact.stat().st_mtime < copied.stat().st_mtime:
            reasons.append(
                f"{rel}: stale — {copied.name} is newer than the built binary; the "
                f"cohab staticlib changed but {Path(rel).name} was never relinked "
                f"(not in ninja_targets?)" + _lag(artifact, copied)
            )
    return reasons


def _belongs_to_sibling(copied: Path, artifact: Path) -> bool:
    """The copied staticlib sits beside ANOTHER binary of the same stem, so that binary links
    it and this one does not. The stem convention pairs both `wdmaud.drv` (user mode, links
    libwdmaud_drv_rust.a) and the kernel's `wdmaud.sys` with wdmaud-rust; when the kernel
    crate rebuilt, rule B reported wdmaud.drv "never relinked" on every arm64 build, a false
    RED no rebuild could clear (2026-10-17)."""
    if copied.parent == artifact.parent:
        return False
    stem = artifact.stem.lower()
    return any(
        p.is_file() and p.stem.lower() == stem and p.suffix.lower() in _PE_SUFFIXES
        for p in copied.parent.iterdir()
    )


def _lag(older: Path, newer: Path) -> str:
    """ " (input Ns newer)" — the discriminator between the two ways an
    artifact-older-than-input verdict happens, and the one fact these messages
    never carried.

    A gap of minutes or hours means the input really did change in an earlier
    run and nothing relinked — a missing dependency edge, i.e. a true positive.
    Both used to print the identical sentence, which is why 5 occurrences a day
    could not be triaged.

    A SUB-SECOND gap was originally read as a harmless same-run ordering race.
    That reading was wrong, and it cost the merge train ~an hour of BUILD_INCOMPLETE
    before anyone chased it: the copy is `copy_if_different`, so a fresh mtime
    means the BYTES CHANGED, which the already-finished link cannot have seen.
    The real cause is build.py's post-main `ninja-optional:<target>` passes —
    `ntkrnlmp` rebuilds `-p kernel-rust` into the same CARGO_TARGET_DIR and the
    same copied `libkernel_rust.a` that `ntoskrnl` links, so the optional pass
    re-emits the archive after the main pass has stopped relinking. Measured on
    amd64: lib at 02:08:11, ntkrnlmp.exe at 02:08:23, ntoskrnl.exe still 00:27:35.
    A `ninja-reaffirm` pass now closes that window; a sub-second gap surviving it
    means a cell is re-emitting on EVERY pass, which is a wiring bug to chase."""
    try:
        d = newer.stat().st_mtime - older.stat().st_mtime
    except OSError:
        return ""
    if d < 2.0:
        return (
            f" [input {d * 1000:.0f}ms newer — bytes changed AFTER the link; a later pass "
            f"(ninja-optional:*?) re-emitted this staticlib. NOT a benign ordering race]"
        )
    if d < 120.0:
        return f" [input {d:.1f}s newer]"
    return f" [input {d / 60:.0f}min newer — a genuinely missed rebuild]"


class _StaticlibIndex:
    """Every `lib*.a` under the build dir and the shared cargo target, and this lane's newest
    artifact-store reference per (arch, body), each read ONCE per staleness check.

    `_crate_staticlibs` used to walk the whole build dir and parse every store reference in
    the fleet for each of ~330 crates: 5-8 minutes of `check_staleness` on a loaded host, the
    window in which a source edit made a queued smoke refuse media that was already built."""

    def __init__(self, repo: Path, build_dir: Path):
        self.libs: dict[str, list[Path]] = {}
        roots = [build_dir, repo / "sdk" / "lib" / "rust" / "target"]
        for root in roots:
            if root.is_dir():
                for p in rglob_stable(root, "lib*.a"):
                    self.libs.setdefault(p.name, []).append(p)
        self.refs = _store_refs(repo, build_dir.name.removeprefix("build-"))


def _crate_staticlibs(
    repo: Path, build_dir: Path, crate: Path, index: _StaticlibIndex | None = None
) -> tuple[Path | None, Path | None]:
    """(cargo's own output, the copied staticlib) for a cohab crate, newest of each.

    They are DIFFERENT FILES with deliberately different mtime behaviour and the
    distinction is the whole point of the check above: cargo's output is re-emitted
    on every cargo run, while the copy is touched only when the bytes changed
    (cmake `copy_if_different`). Cargo's lives under a `rust-<x>-target/` (per-module
    custom command) or the shared `sdk/lib/rust/target/<stem>/` (add_cargo_module);
    the copy sits in the module's build dir. Returns None for either if absent."""
    lib = "lib" + crate.name.replace("-", "_") + ".a"
    if index is None:
        index = _StaticlibIndex(repo, build_dir)
    cands = list(index.libs.get(lib, []))

    def _in_cargo_target(p: Path) -> bool:
        return any(part == "target" or part.endswith("-target") for part in p.parts)

    def _newest(ps: list[Path]) -> Path | None:
        ps = [p for p in ps if p.is_file()]
        return max(ps, key=lambda p: p.stat().st_mtime) if ps else None

    cargo_out = _newest([p for p in cands if _in_cargo_target(p)])
    # A body the artifact store served is linked from a shared immutable blob, not a
    # file under this tree, and any private copy left here is an old one. Its
    # "cargo ran" clock is the lane's store reference for it, rewritten on every
    # request, hit or miss: a blob's own mtime is not, since an edit that rebuilds to
    # the same bytes reuses it.
    ref = index.refs.get(crate.name)
    if ref is not None and (cargo_out is None or ref.stat().st_mtime > cargo_out.stat().st_mtime):
        cargo_out = ref
    return cargo_out, _newest([p for p in cands if not _in_cargo_target(p)])


def _store_ref(repo: Path, arch: str, body: str) -> Path | None:
    """This lane's newest artifact-store reference for `body` built for `arch`."""
    return _store_refs(repo, arch).get(body)


def _store_refs(repo: Path, arch: str) -> dict[str, Path]:
    """This lane's newest artifact-store reference per body built for `arch`, in one pass."""
    root = os.environ.get("VIBEOS_ARTIFACT_STORE_ROOT") or str(
        Path.home() / "vibeos-artifact-store"
    )
    refs = Path(root) / "v1" / "refs"
    if not refs.is_dir():
        return {}
    lane = str(repo.resolve())
    markers = (f"-{arch}-cargo-link/", f"/cell-link-v1-{arch}-")
    newest: dict[str, tuple[float, Path]] = {}
    for path in refs.glob("*.json"):
        try:
            record = json.loads(path.read_text())
        except (OSError, ValueError):
            continue
        consumer = record.get("consumer", "")
        fields = consumer.split("|")
        if (
            record.get("lane") == lane
            and consumer.startswith("cargo:")
            and len(fields) > 2
            and any(m in fields[1] for m in markers)
        ):
            try:
                mtime = path.stat().st_mtime
            except OSError:
                continue
            if fields[2] not in newest or mtime > newest[fields[2]][0]:
                newest[fields[2]] = (mtime, path)
    return {body: path for body, (_, path) in newest.items()}


_CARGO_CALL_RE = re.compile(
    r"^\s*add_cargo_(?:module|kernel_dll)\(\s*(\S+)\s+(\S+)\s+(\S+)", re.MULTILINE
)


def cargo_module_rels(repo: Path) -> list[str]:
    """Build-relative artifact paths of every cargo-LINKED module, parsed from
    the `add_cargo_module`/`add_cargo_kernel_dll` call sites in the tree
    (self-maintaining, like mkramdisk_required_rels: a module graduated to the
    cargo link auto-joins the freshness net). The custom command's OUTPUT is
    `${CMAKE_CURRENT_BINARY_DIR}/<output>`, i.e. the call-site dir mirrored
    into the build dir — true for the two .cmake-included call sites too
    (rosload.cmake/uefi.cmake live in their includer's dir). Call sites whose
    output is a `${var}` (bootvid) are skipped — unresolvable statically."""
    # ok_codes includes 1: `git grep` exits 1 for "no matches", which is a real
    # empty result rather than a broken oracle. Every OTHER non-zero rc now
    # raises instead of yielding [] — an empty list here silently disables the
    # whole cargo-link staleness net, which is the false-GREEN direction.
    files = _run(
        ["git", "grep", "-lE", r"add_cargo_(module|kernel_dll)\(", "--", "*.txt", "*.cmake"],
        repo,
        ok_codes=(0, 1),
    ).splitlines()
    rels = []
    for rel_file in files:
        p = repo / rel_file
        if p.name != "CMakeLists.txt" and p.suffix != ".cmake":
            continue
        try:
            text = p.read_text(errors="ignore")
        except OSError:
            continue
        for m in _CARGO_CALL_RE.finditer(text):
            out = m.group(3).rstrip(")")
            if "$" in m.group(1) or "$" in out:
                continue  # variable target/output (bootvid) — not resolvable here
            rels.append(str(Path(rel_file).parent / out))
    return sorted(set(rels))


def _cargo_link_inputs(repo: Path, build_dir: Path, artifact: Path) -> list[Path]:
    """Read the copied PE's compiler dependency rule in this checkout's namespace."""
    depfile = artifact.with_suffix(artifact.suffix + ".d")
    try:
        text = depfile.read_text().replace("\\\n", " ")
        rule = text.splitlines()[0]
        targets = (
            artifact,
            Path("/tmp/vibeos") / artifact.relative_to(repo),
            artifact.relative_to(build_dir),
        )
        for target in targets:
            prefix = str(target).replace(" ", "\\ ") + ": "
            if rule.startswith(prefix):
                words = shlex.split(rule[len(prefix) :], comments=False)
                break
        else:
            raise ValueError("missing expected output rule")
        if not words:
            raise ValueError("empty input closure")
        inputs = []
        for word in words:
            path = Path(word.replace("$$", "$"))
            if path.is_relative_to("/tmp/vibeos"):
                path = repo / path.relative_to("/tmp/vibeos")
            elif not path.is_absolute():
                path = build_dir / path
            if not path.exists():
                raise ValueError(f"missing dependency {path}")
            inputs.append(path)
            if path.is_dir():
                # Cargo rerun-if-changed on a directory watches its whole tree.
                inputs.extend(rglob_stable(path, "*"))
        return inputs
    except (OSError, ValueError, IndexError) as error:
        raise ProvenanceError(
            f"{depfile}: cannot verify cargo link inputs: {error}; run `build`"
        ) from error


def _media_manifest_rels(repo: Path, arch: str) -> set[str] | None:
    """Build-relative source paths staged onto the x86 boot CD, parsed from the
    generated boot/<media>.cmake.lst manifest (`cdpath=srcpath` lines, srcpath
    under the canonical /tmp/vibeos build dir). None if the manifest is absent
    (never configured) — caller treats that as 'no scope known, skip'."""
    base = archcfg.base_arch(arch)
    media_stem = Path(archcfg.arch_build(arch).boot_media_rel).stem  # e.g. livecd
    lst = repo / f"build-{base}" / "boot" / f"{media_stem}.cmake.lst"
    try:
        text = lst.read_text(errors="ignore")
    except OSError:
        return None
    prefix = f"/tmp/vibeos/build-{base}/"
    rels = set()
    for line in text.splitlines():
        if "=" not in line:
            continue
        src = line.split("=", 1)[1].strip()
        if src.startswith(prefix):
            rels.add(src[len(prefix) :])
    return rels


def cargo_link_staleness_reasons(repo: Path, arch: str) -> list[str]:
    """Check each boot-relevant Cargo PE against its actual emitted input closure.

    A workspace-wide maximum falsely rejects unrelated binaries after selective
    rebuilds. Missing dependency evidence fails loudly; optional outputs skip.
    """
    cfg = archcfg.arch_build(arch)
    build_dir = repo / f"build-{archcfg.base_arch(arch)}"
    if cfg.needs_mkramdisk:
        scope = set(mkramdisk_required_rels(repo)) | set(cfg.staged_bins_rel)
    else:
        scope = _media_manifest_rels(repo, arch)
        if scope is None:
            return []  # no manifest yet — media absent, other checks report that
    reasons = []
    for rel in cargo_module_rels(repo):
        if rel not in scope:
            continue
        artifact = build_dir / rel
        if not artifact.is_file():
            continue
        inputs = _cargo_link_inputs(repo, build_dir, artifact)
        newest_path = max(inputs, key=lambda p: p.stat().st_mtime_ns)
        newest = newest_path.stat().st_mtime
        newest_rel = str(
            newest_path.relative_to(repo) if newest_path.is_relative_to(repo) else newest_path
        )
        if artifact.stat().st_mtime < newest:
            lag = newest - artifact.stat().st_mtime
            note = (
                # Source-vs-binary, so unlike _lag's staticlib comparison a
                # sub-second gap really can be same-run ordering (a generated .rs
                # rewritten while the build ran). Say "check", not "ignore" —
                # calling a gap benign by default is what hid a real stale kernel.
                " [<2s — possibly same-run ordering; confirm the binary is newer"
                " than the SOURCE it compiled, not merely close]"
                if lag < 2.0
                else f" [{lag / 60:.0f}min newer — a genuinely missed rebuild]"
                if lag >= 120.0
                else f" [{lag:.1f}s newer]"
            )
            reasons.append(
                f"{rel}: stale — {newest_rel} newer than the cargo-linked binary "
                f"(cargo link not re-run). ninja's graph can MISS this edge, in which "
                f"case `build` reports the same failure forever and `ninja <target>` "
                f"says 'no work to do'. If a rebuild does not clear it: "
                f"`rm {rel}` and build again. That works PRECISELY BECAUSE the "
                f"edge is missing — deleting the output is the only way to make "
                f"ninja rebuild something it has no reason to rebuild — so if you "
                f"need it, the cargo module is probably missing a BODY_CRATES "
                f"declaration. Once that is declared, the rm is a ONE-TIME cleanup "
                f"for an already-stale tree, not a standing remedy." + note
            )
    return reasons


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
    # Defaulted so a stamp written before this field existed still loads. An EMPTY value
    # therefore means "this stamp cannot answer the question", NOT "nothing changed" —
    # check_staleness must say so out loud rather than let it read as clean.
    build_src_hash: str = ""
    # The HEAD sha at stamp time. Recorded so a later fingerprint mismatch can name
    # WHICH input moved instead of reporting an undiagnosable "source changed":
    # a HEAD move (a merge landed under the build) and a working-tree edit are very
    # different failures, and the fingerprint alone cannot tell them apart. Empty on
    # stamps written before this field existed.
    head_sha: str = ""
    # tree_content_hash() of the same two scopes. The fingerprints above are not
    # commit-invariant (see tree_content_hash), so committing exactly what was built read
    # as stale; these are compared instead whenever the stamp has them. Empty on stamps
    # written before they existed, which fall back to the fingerprints.
    rust_tree_hash: str = ""
    build_tree_hash: str = ""

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


# Paths that CANNOT change the boot media. Deliberately an EXCLUDE-list, not an
# include-list: an unknown path must land on the build-RELEVANT side, because the two
# errors are not symmetric. Over-including costs a no-op rebuild; under-including boots
# stale media and reports a pass — which is precisely what the oracle this replaces did
# (field-manual §9.81b). So this stays as short as the evidence allows: `docs/` holds no
# build inputs at all, and markdown is never compiled. Everything else — including tests/
# and tools/ — is treated as build-relevant, because it can be: archcfg.py decides which
# ninja targets get staged into the arm64 ramdisk, qemu/mkramdisk.sh assembles it, and
# tools/ carries real compiled sources (tools/libschily/*.c, tools/gen-delaylib.py).
# This is exactly the set the old code meant by "nothing build-relevant (docs, this plan)".
NON_BUILD_PATHSPEC = (
    ":(exclude)docs",
    ":(exclude)*.md",
    ":(exclude).github",
    ":(exclude).claude",
    # tools/lint-baseline/ is accepted-findings bookkeeping: text files that only
    # lints read, never compiled and never staged into media. Including them made
    # the documented loop (build -> `check --update-baseline <tool>` -> smoke)
    # ALWAYS report STALE_MEDIA and demand a second, no-op rebuild -- so the fix
    # was to stop re-running the build, not to remember to reorder the steps.
    # Deliberately narrow: the rest of tools/ carries real compiled sources
    # (tools/libschily/*.c, tools/gen-delaylib.py) and stays in the fingerprint.
    ":(exclude)tools/lint-baseline",
    # The cohab census snapshot is the same kind of file: `cohab_census --write`
    # re-records it after a build, and that alone made the next smoke STALE_MEDIA.
    ":(exclude)tools/cohab-census.txt",
    # tools/pi4/ drives a physical Raspberry Pi bench (serial, TFTP, power) from
    # the host. Nothing in it is compiled or staged into media, and editing a
    # bench script while an arm64 build ran made that build's smoke STALE_MEDIA
    # twice in a row. tools/pi5/ is the same for the Pi 5 (its card templates
    # made the first `pi5 boot` refuse a fresh build as STALE_MEDIA).
    ":(exclude)tools/pi4",
    ":(exclude)tools/pi5",
    # pytest modules are test-time consumers of the media, never build inputs:
    # only other tests import them. Adding a regression test to test_xhci.py made
    # that very test refuse the build it was written against.
    ":(exclude,glob)tests/**/test_*.py",
    ":(exclude,glob)tools/test_*.py",
    # tests/qemu/vm.py is the QEMU launcher: it decides how the media boot (devices,
    # PCI slots, patched COPIES in a work dir), never what they contain, and nothing
    # in build/archcfg/mkramdisk imports it. Pinning a USB controller's PCI slot in it
    # made every arch's fresh media STALE_MEDIA for the very boot that needed the fix.
    ":(exclude)tests/qemu/vm.py",
)


def build_src_hash(repo: Path) -> str:
    """Fingerprint of every source that can change the boot media.

    This is the tier-2 answer to "the git fingerprint moved — was it build-relevant?".
    It replaced a `ninja -n` dry-run that was structurally incapable of answering:
    CMake's CONFIGURE_DEPENDS globs make the glob-verify edge dirty by design on every
    build, so under -n (which executes nothing) ninja assumes the cmake stamp changes,
    marks the regen edge dirty, and STOPS PLANNING — it never evaluates the targets it
    was asked about, and reported "no work" for `livecd` on amd64 with a rebuild pending.
    Since rust_src_hash covered only sdk/lib/rust, that made the dry-run the ONLY thing
    standing between a C edit and "clean-diff (non-build change) -> FRESH": editing a
    tracked, compiled .c and running smoke booted the STALE iso and called it a pass.
    Content, not mtime and not -n, is the only thing that answers this honestly."""
    return git_fingerprint(repo, [".", *NON_BUILD_PATHSPEC])


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
        build_src_hash=build_src_hash(repo),
        head_sha=git_head(repo),
        rust_tree_hash=tree_content_hash(repo, list(archcfg.RUST_SRC_DIRS)),
        build_tree_hash=tree_content_hash(repo, [".", *NON_BUILD_PATHSPEC]),
        boot_media=cfg.boot_media_rel,
        boot_media_sha=_sha256_file(media),
        cold=cold,
        contributing=contributing_hashes(repo, arch),
        rust_targets=targets,
    )


def _src_delta(repo: Path, stamp: Stamp) -> str:
    """WHAT changed, appended to the bare "build source changed" reason.

    That reason is the single least actionable line this oracle emits: it fires
    milliseconds after make_stamp computed the very same hash, and then says
    nothing about which of the three fingerprint inputs (HEAD, tracked diff,
    untracked set) moved. It has halted the merge train with no evidence left to
    diagnose it. Best-effort by construction — a diagnostic that raises while
    explaining a failure would be worse than the failure."""
    try:
        now_head = git_head(repo)
        moved = bool(stamp.head_sha) and now_head != stamp.head_sha
        # Diff against the STAMP'S commit, not HEAD, whenever we have it: the
        # question is what changed since the build, and after a commit "vs HEAD"
        # reports nothing while the build is genuinely out of date. This used to
        # return early on a moved HEAD and never list a path at all, which made
        # a real in-scope change and a false positive print the SAME line --
        # measured cost: one run where "HEAD MOVED ... NOT a stale artifact" was
        # the whole explanation and the actual culprit was an edited .py.
        base = stamp.head_sha if moved else "HEAD"
        names = _run(
            ["git", "diff", "--name-only", base, "--", ".", *NON_BUILD_PATHSPEC], repo
        ).split()
        dirty = [
            ln[3:]
            for ln in _run(
                ["git", "status", "--porcelain", "--", ".", *NON_BUILD_PATHSPEC], repo
            ).splitlines()
        ]
        changed = sorted(set(names) | set(dirty))
        head_note = f" (HEAD also moved: {stamp.head_sha[:12]} -> {now_head[:12]})" if moved else ""
        if not changed:
            if moved:
                # The case the scoped-fingerprint fix exists for. If this still
                # fires, the stamp predates that fix and a rebuild re-stamps it.
                return (
                    f" — but NO build-relevant path differs from the build's commit"
                    f"{head_note}. Nothing in scope changed, so this is a STALE STAMP, "
                    "not stale media: rebuild once to re-stamp."
                )
            return (
                " — but NOTHING is currently modified vs HEAD, and HEAD did not move. "
                "The fingerprint moved without a source change: suspect the oracle, "
                "not the tree (see ProvenanceError / _run)"
            )
        return (
            f" — {len(changed)} path(s) vs the build's commit{head_note}: "
            + ", ".join(changed[:6])
            + (" …" if len(changed) > 6 else "")
        )
    except (ProvenanceError, OSError) as e:
        return f" — (could not explain: {e})"


@dataclass
class Staleness:
    fresh: bool
    reasons: list  # human strings, e.g. ["rust source changed", ...]
    warm: bool  # cold==False on the stamp (advisory)
    media_missing: bool

    @property
    def stale(self) -> bool:
        return (not self.fresh and not self.media_missing) or self.media_missing


# DELETED 2026-07-14: `_ninja_has_work(build_dir, targets)` — a `ninja -n` dry-run that
# was the tier-2 oracle for "did a non-Rust build input change?". It NEVER worked, on any
# arch, and it failed silently in the false-GREEN direction (field-manual §9.81b):
#
#   $ ninja -n dll/win32/ws2_32/ws2_32.dll
#   [0/2] Re-checking globbed directories...
#   [1/2] Re-running CMake...          <- and nothing else, EVER
#
# CMake's CONFIGURE_DEPENDS globs install a glob-verify edge that is dirty by design on
# every build. Under -n no command executes, so ninja must assume that edge changes the
# cmake stamp, marks the regen edge dirty, and STOPS PLANNING — regenerating build.ninja
# would invalidate the rest of the graph, so it never evaluates the targets you asked
# about. It printed only those two lines, which the parser skipped as "not real work",
# and returned False = clean. Verified: False for `livecd` on amd64 with a rebuild
# genuinely pending. Running `cmake .` or `ninja build.ninja` first does NOT help; the
# glob edge is always-dirty by construction.
#
# It is deleted rather than left unused: a plausible-looking helper named "has work" is
# exactly what the next person reaches for. Use build_src_hash() — content, not a
# dry-run. Do not reintroduce a `ninja -n` staleness probe in a CONFIGURE_DEPENDS tree.


def external_media_problems(repo: Path, arch: str, image: Path) -> list[str]:
    """Why `image` (tools/media_manifest.py --assemble output) may NOT be booted as this
    tree's `arch` livecd; empty when it may. Its stamp `<cd>.provenance.json` must name the
    arch and the livecd, hash the image as it is now, and record the source this tree has
    now: an image mastered from another tree, or from cells built from one, is refused
    (--assemble refuses cells that do not match the source it masters from). The entries
    that are not cells come from build-<arch>, which the ordinary staleness check covers."""
    stamp = image.parent / f"{image.stem}.provenance.json"
    if not image.is_file():
        return [f"{image}: no such image"]
    if not stamp.is_file():
        return [f"{stamp} missing: master it with tools/media_manifest.py --assemble"]
    st = json.loads(stamp.read_text())
    problems = []
    if st.get("arch") != arch:
        problems.append(f"{image} is {st.get('arch')} media, not {arch}")
    if st.get("cd") != "livecd":
        problems.append(f"{image} is the {st.get('cd')} image: smoke boots the livecd")
    if st.get("sha256") != _sha256_file(image):
        problems.append(f"{image} changed since it was mastered (sha256 differs from its stamp)")
    if st.get("source") != tree_content_hash(repo, [".", *NON_BUILD_PATHSPEC]):
        problems.append(f"{image} was mastered from another source tree: master it again")
    return problems


def check_staleness(repo: Path, arch: str) -> Staleness:
    """Fresh/stale verdict for the built media. Never raises: a ProvenanceError
    is reported as its OWN failure ("the oracle could not answer") rather than
    escaping as a traceback or — the old behaviour — being laundered into a
    confident but wrong "source changed since build"."""
    try:
        return _check_staleness(repo, arch)
    except ProvenanceError as e:
        return Staleness(
            fresh=False,
            reasons=[f"provenance oracle FAILED (verdict unknown, not stale): {e}"],
            warm=False,
            media_missing=False,
        )


def _check_staleness(repo: Path, arch: str) -> Staleness:
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
    # The arm64 cohab false-green is invisible to the fingerprint: a flip that
    # was never rebuilt leaves the stamp matching, so this check must gate BOTH
    # tiers (it cannot live behind the tier-2 escalation like the re-stage trap).
    # Same for the cargo-link net (stale cargo-linked PE inside a fresh ISO).
    cohab = cohab_staleness_reasons(repo, arch) + cargo_link_staleness_reasons(repo, arch)

    # Tier 1: cheap fingerprint compare → FRESH (unless a cohab binary is stale).
    if git_fingerprint(repo) == stamp.git_fingerprint:
        if cohab:
            return Staleness(fresh=False, reasons=cohab, warm=warm, media_missing=False)
        return Staleness(fresh=True, reasons=[], warm=warm, media_missing=False)

    # Tier 2: source changed — escalate to authoritative per-layer checks.
    reasons = list(cohab)
    if stamp.rust_tree_hash and stamp.build_tree_hash:
        rust_moved = tree_content_hash(repo, list(archcfg.RUST_SRC_DIRS)) != stamp.rust_tree_hash
        build_moved = tree_content_hash(repo, [".", *NON_BUILD_PATHSPEC]) != stamp.build_tree_hash
    else:
        rust_moved = rust_src_hash(repo) != stamp.rust_src_hash
        build_moved = bool(stamp.build_src_hash) and build_src_hash(repo) != stamp.build_src_hash
    if rust_moved:
        reasons.append(
            "rust source changed since build" + (" (last build WARM — may mask)" if warm else "")
        )
    # Everything that is NOT Rust: C, asm, cmake, .spec, .rc, mkramdisk, archcfg…
    # rust_src_hash above covers only sdk/lib/rust, so without this a C edit reads as a
    # non-build change and smoke boots stale media (§9.81b).
    if not stamp.build_src_hash:
        reasons.append(
            "stamp predates build-source fingerprinting — cannot verify non-Rust sources; "
            "run `build` to re-stamp"
        )
    elif build_moved:
        reasons.append(
            "build source changed since build (C/asm/cmake/spec/rc)" + _src_delta(repo, stamp)
        )
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


# --- probe presence in the MEDIA -------------------------------------------
#
# `strings build-<arch>/ntoskrnl/ntoskrnl.exe | grep -c '<literal>'` (field
# manual §9.87) answers a DIFFERENT question from "is my probe in the image
# qemu is about to open". On 2026-07-26 a 12-round arm64 soak ran for ~9 hours
# against media frozen four hours before the two probes it existed to exercise
# were written: the binary check passed, the media carried zero of either
# literal, and every probe-derived conclusion was vacuous. A rule with no
# command behind it is a rule that gets broken, so this is the command.
#
# `arm64` deliberately checks BOTH boot.img and ramdisk.img: the kernel lives in
# the ramdisk, and checking only the outer image would reproduce the same
# right-check-wrong-artifact mistake one level down.
#
# KNOWN LIMIT — this checks for a literal in the image, so it only sees probes
# whose text actually survives into .rdata. That covers the normal forms
# (`ARM64_UART_TRACE*(b"…".as_ptr())`, `dbg_print!`), which pass the string by
# pointer. It does NOT cover a hand-rolled byte loop:
#
#     let msg = b"[IO] PsLocateSystemDll\r\n";
#     for &ch in msg.iter() { *uart = ch as ULONG; }
#
# LLVM unrolls that into immediate stores and the string never appears in the
# binary at all, so this check reports it MISSING while it executes fine.
# Measured on iomgr.rs 2026-07-26: "[IO] PsLocateSystemDll" -> 0 hits in a media
# image that contains the code. A missing verdict on a loop-written probe is a
# false alarm; rewrite the probe to pass a pointer (which is better anyway —
# that is the form proven to reach the serial) rather than trusting either
# answer.

#: Extra media that must ALSO carry the literal, beyond `boot_media_rel`.
_EXTRA_MEDIA: dict[str, tuple[str, ...]] = {"arm64": ("qemu/ramdisk.img",)}


def media_paths(repo: Path, arch: str) -> list[Path]:
    """Every image a boot of `arch` actually reads, newest-build first."""
    cfg = archcfg.arch_build(arch)
    rels = [cfg.boot_media_rel, *_EXTRA_MEDIA.get(archcfg.base_arch(arch), ())]
    return [repo / r for r in rels if r]


def _count_in_file(path: Path, needles: list[bytes]) -> dict[bytes, int]:
    """Count each needle in `path`, streaming with an overlap so a literal
    straddling a chunk boundary is still found."""
    found = {n: 0 for n in needles}
    overlap = max((len(n) for n in needles), default=1) - 1
    try:
        with path.open("rb") as fh:
            tail = b""
            while True:
                chunk = fh.read(8 << 20)
                if not chunk:
                    break
                buf = tail + chunk
                for n in needles:
                    found[n] += buf.count(n)
                tail = buf[-overlap:] if overlap else b""
    except OSError:
        return found
    return found


def probes_missing_from_media(repo: Path, arch: str, literals: list[str]) -> list[str]:
    """Reasons why `literals` are NOT all present in every boot image.

    Empty list == every literal is in every image, i.e. the experiment you
    think you are running is the one that will run.
    """
    reasons: list[str] = []
    # .strip(), not truthiness: a whitespace-only literal would otherwise become
    # a live needle and fail the run on a "probe" that names nothing.
    needles = [s.encode() for s in literals if s.strip()]
    if not needles:
        return reasons
    for media in media_paths(repo, arch):
        if not media.is_file():
            reasons.append(f"{media} does not exist — build {arch} before requiring a probe")
            continue
        counts = _count_in_file(media, needles)
        for n in needles:
            if counts[n] == 0:
                reasons.append(
                    f"{n.decode()!r} appears 0 times in {media.name} "
                    f"(built {_mtime_str(media)}) — the probe is NOT in the media this boot "
                    f"would open, so its silence would prove nothing"
                )
    return reasons


def _mtime_str(p: Path) -> str:
    import datetime

    try:
        return datetime.datetime.fromtimestamp(p.stat().st_mtime).strftime("%Y-%m-%d %H:%M:%S")
    except OSError:
        return "?"
