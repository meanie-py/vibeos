"""Build provenance + staleness model (plan §6/§7, §20.1).

Delegate to git (source content) and ninja (build DAG); the stamp tracks only
the gaps — warm/cold, rust-source state, contributing-binary hashes. Staleness
is tiered cheap-first: a git-fingerprint match means FRESH with no further I/O.
git fingerprints (not mtime) so branch/worktree switches don't churn the verdict.
"""

from __future__ import annotations

import hashlib
import json
import re
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


def _crate_for_stem(repo: Path, stem: str) -> Path | None:
    """Map a staged binary stem (e.g. 'advapi32', 'kernel32_vista') to its
    Rust-cohab crate dir under sdk/lib/rust/, or None if it isn't Rust-cohab.
    Binaries carry '_' in version suffixes (kernel32_vista.dll) while crates use
    '-' (kernel32-vista-rust), so try both separators. By convention a cohab
    crate is `<stem>-rust` and produces a same-named `<stem>.{sys,dll,exe}`."""
    rust = repo / "sdk" / "lib" / "rust"
    for name in (f"{stem}-rust", f"{stem.replace('_', '-')}-rust"):
        c = rust / name
        if (c / "Cargo.toml").is_file():
            return c
    return None


def _newest_rs_mtime(crate: Path) -> float:
    """Newest .rs mtime under a crate — its source-change signal. The cohab
    binary is relinked AFTER cargo rebuilds the rlib from these, so a fresh
    build always leaves the binary newer than every .rs here."""
    newest = 0.0
    root = crate / "src" if (crate / "src").is_dir() else crate
    for p in root.rglob("*.rs"):
        try:
            newest = max(newest, p.stat().st_mtime)
        except OSError:
            continue
    return newest


def _depinfo_newest_rs(cargo_out: Path, repo: Path) -> float | None:
    """Newest mtime of the .rs files cargo ACTUALLY COMPILED into this
    staticlib, from its dep-info (`lib<crate>.d` beside the .a).

    The whole-crate glob above over-approximates: a BANKED-but-unwired
    translation (.rs on disk with no `mod` declaring it — e.g. explorer-rust's
    Tier C files) is invisible to cargo, so cargo is correctly "fresh" forever
    while the glob says "source newer than output" forever — a false RED the
    2026-07-18 master merge triggered by refreshing appbar.rs's mtime. The
    dep-info lists exactly the sources the crate build consumed, so a dead
    file can neither false-red nor mask a real edit to a live one.

    Paths in the dep-info are canonical (/tmp/vibeos/... — builds run under
    tools/canon-run.sh's bind mount), so map that prefix back onto the repo.
    Returns None when the dep-info is absent/unreadable — caller falls back
    to the conservative whole-crate glob."""
    from . import canon

    d = cargo_out.with_suffix(".d")
    try:
        text = d.read_text()
    except OSError:
        return None
    prefix = canon.canon_path() + "/"
    newest = 0.0
    seen = False
    for line in text.splitlines():
        if ":" not in line:
            continue
        _, _, deps = line.partition(":")
        # cargo escapes spaces as '\ '; our tree has none — split plainly.
        for dep in deps.split():
            if not dep.endswith(".rs"):
                continue
            p = Path(repo / dep[len(prefix) :]) if dep.startswith(prefix) else Path(dep)
            try:
                newest = max(newest, p.stat().st_mtime)
                seen = True
            except OSError:
                continue
    return newest if seen else None


def mkramdisk_required_rels(repo: Path) -> list[str]:
    """Build-relative paths of the arm64 ramdisk boot set, parsed from
    mkramdisk.sh's REQUIRED_FILES array (`"$BUILD_DIR/<rel>"`). This is the
    AUTHORITATIVE staged set — a superset of archcfg.staged_bins_rel (the
    advapi32 false-green was staged HERE but absent from archcfg) — and it is
    self-maintaining: a driver added to mkramdisk auto-joins the freshness net."""
    try:
        text = (repo / "qemu" / "mkramdisk.sh").read_text(errors="ignore")
    except OSError:
        return []
    return re.findall(r'"\$BUILD_DIR/([^"]+)"', text)


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
    return sorted(rel for rel in boot_set if _crate_for_stem(repo, Path(rel).stem) is not None)


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
    for rel in boot_set_cohab_rels(repo, arch):
        crate = _crate_for_stem(repo, Path(rel).stem)
        assert crate is not None  # boot_set_cohab_rels only returns mapped rels
        artifact = build_dir / rel
        if not artifact.is_file():
            reasons.append(
                f"{rel}: {crate.name} cohab crate but build artifact absent — never built"
            )
            continue
        newest_rs = _newest_rs_mtime(crate)
        cargo_out, copied = _crate_staticlibs(repo, build_dir, crate)

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
            # Prefer the dep-info's compiled-source set over the whole-crate
            # glob: banked-but-unwired .rs files (no `mod` — cargo never reads
            # them) must not red a build cargo correctly considers fresh.
            compiled_newest = _depinfo_newest_rs(cargo_out, repo)
            effective_rs = compiled_newest if compiled_newest is not None else newest_rs
            if cargo_out.stat().st_mtime < effective_rs:
                reasons.append(
                    f"{rel}: stale — {crate.name} .rs is newer than cargo's own output "
                    f"({cargo_out.name}); the crate was not rebuilt (add to ninja_targets?)"
                )
                continue
        # B. did the link consume the current staticlib?
        if copied is not None and artifact.stat().st_mtime < copied.stat().st_mtime:
            reasons.append(
                f"{rel}: stale — {copied.name} is newer than the built binary; the "
                f"cohab staticlib changed but {Path(rel).name} was never relinked "
                f"(not in ninja_targets?)"
            )
    return reasons


def _crate_staticlibs(repo: Path, build_dir: Path, crate: Path) -> tuple[Path | None, Path | None]:
    """(cargo's own output, the copied staticlib) for a cohab crate, newest of each.

    They are DIFFERENT FILES with deliberately different mtime behaviour and the
    distinction is the whole point of the check above: cargo's output is re-emitted
    on every cargo run, while the copy is touched only when the bytes changed
    (cmake `copy_if_different`). Cargo's lives under a `rust-<x>-target/` (per-module
    custom command) or the shared `sdk/lib/rust/target/<stem>/` (add_cargo_module);
    the copy sits in the module's build dir. Returns None for either if absent."""
    lib = "lib" + crate.name.replace("-", "_") + ".a"
    cands = list(build_dir.rglob(lib))
    ws = repo / "sdk" / "lib" / "rust" / "target"
    if ws.is_dir():
        cands += list(ws.rglob(lib))

    def _in_cargo_target(p: Path) -> bool:
        return any(part == "target" or part.endswith("-target") for part in p.parts)

    def _newest(ps: list[Path]) -> Path | None:
        ps = [p for p in ps if p.is_file()]
        return max(ps, key=lambda p: p.stat().st_mtime) if ps else None

    return (
        _newest([p for p in cands if _in_cargo_target(p)]),
        _newest([p for p in cands if not _in_cargo_target(p)]),
    )


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
    files = _run(
        ["git", "grep", "-lE", r"add_cargo_(module|kernel_dll)\(", "--", "*.txt", "*.cmake"],
        repo,
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


def _newest_ws_src(repo: Path) -> tuple[float, str]:
    """(mtime, repo-rel path) of the newest .rs/.toml in the rust workspace,
    excluding cargo target dirs. This is a cargo-linked module's source-change
    signal: since 2026-07-04 add_cargo_module's DEPENDS is the whole workspace
    glob (one dependency graph — any workspace .rs can be in any module's link
    closure), so a fresh build always leaves every cargo-linked binary newer
    than this."""
    ws = repo / "sdk" / "lib" / "rust"
    newest, newest_rel = 0.0, ""
    for pat in ("*.rs", "*.toml"):
        for p in ws.rglob(pat):
            parts = p.relative_to(ws).parts
            if parts and parts[0] == "target":
                continue
            try:
                mt = p.stat().st_mtime
            except OSError:
                continue
            if mt > newest:
                newest, newest_rel = mt, str(p.relative_to(repo))
    return newest, newest_rel


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
    """The cargo-LINK false-green net (the 2026-07-04 ntoskrnl class) — ALL arches.

    A cargo-linked PE (add_cargo_module) is produced by a custom command; before
    2026-07-04 its DEPENDS globbed only the bin+body crates, so a kernel .rs edit
    rebuilt libkernel_rust.a but never re-ran the LINK: `ninja livecd` re-rolled
    a fresh ISO around a PRE-FIX ntoskrnl.exe and the gate false-greened
    i386+amd64 smoke. This disproved boot_set_cohab_rels' x86 exemption ("a full
    ninja livecd can't ship a stale cohab binary") for cargo-linked modules. The
    build stamp is blind to it (a build RAN and re-stamped; ninja just did no
    work), so — like the cohab net — this is mtime-based and gates both tiers.

    Contract (matches add_cargo_module's workspace-glob DEPENDS): every
    boot-relevant cargo-linked artifact must be NEWER than the newest workspace
    .rs/.toml. Scope = the arch's boot set (arm64) or the CD manifest (x86);
    absent artifacts are skipped (arch-gated/optional modules)."""
    cfg = archcfg.arch_build(arch)
    build_dir = repo / f"build-{archcfg.base_arch(arch)}"
    if cfg.needs_mkramdisk:
        scope = set(mkramdisk_required_rels(repo)) | set(cfg.staged_bins_rel)
    else:
        scope = _media_manifest_rels(repo, arch)
        if scope is None:
            return []  # no manifest yet — media absent, other checks report that
    newest, newest_rel = _newest_ws_src(repo)
    if not newest:
        return []
    reasons = []
    for rel in cargo_module_rels(repo):
        if rel not in scope:
            continue
        artifact = build_dir / rel
        if not artifact.is_file():
            continue
        if artifact.stat().st_mtime < newest:
            reasons.append(
                f"{rel}: stale — {newest_rel} newer than the cargo-linked binary "
                f"(cargo link not re-run; run `build`)"
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
    if rust_src_hash(repo) != stamp.rust_src_hash:
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
    elif build_src_hash(repo) != stamp.build_src_hash:
        reasons.append("build source changed since build (C/asm/cmake/spec/rc)")
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
