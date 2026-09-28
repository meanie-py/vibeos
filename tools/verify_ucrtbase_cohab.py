#!/usr/bin/env python3
"""Assert that ucrtbase.dll's Rust cohab cannot shadow genuine Microsoft UCRT bodies.

ucrtbase.dll is vendored Microsoft UCRT (`sdk/lib/ucrt`, see vendor.yaml) plus OUR
`msvcrt_shared`, `crtmath`, `vcruntime` and `chkstk` plus 132 lines of its own C. The Rust we
link into it may cover THOSE surfaces and nothing else -- and the reason that needs a tool
rather than a rule is that violating it produces no diagnostic at all:

    ucrtbase.spec exports        2,584 names
    msvcrt's Rust cohab archive  1,229 names
    they collide on                891, of which only 35 are msvcrt_shared's

Linking msvcrt's archive into ucrtbase under `--whole-archive` would replace the other 856
with bodies written and graded against MSVCRT's C at `_MSVCR_VER=0` -- our malloc, our
_errno, our mbcs tables, our printf engine, standing in for the genuine UCRT's. The module
links with `--allow-multiple-definition`, so it would build clean, boot, and behave subtly
like msvcrt forever. That is the failure this file exists to make impossible.

Two checks, deliberately at different strengths:

  STRUCTURE (always runs, needs no build)
      `crt-shared-cargo` -- the only Rust archive ucrtbase links -- must not reach
      `msvcrt-crt-rust` through any path in the workspace dependency graph. This is the real
      guarantee: a dependency edge that does not exist cannot be got wrong by a build flag,
      which is exactly why the surface was split into its own crate instead of being scoped
      with a cargo feature.

  REACHABILITY (always runs, needs no build)
      Every dependency of `crt-shared-cargo` must be named by an `extern crate` in its
      lib.rs, or its objects never reach the archive at all and the cohab it was added for
      silently does not happen. See `check_extern_crates` for the case that cost a build.

  ARCHIVE (runs when this worktree has a build; hard-fails if it cannot trust one)
      Every symbol the built `libcrt_shared_cargo.a` defines must also be defined by one of
      OUR static libraries on ucrtbase's link line. This is the empirical backstop for the
      case the structural check cannot see: someone adding a genuinely new `#[no_mangle]` to
      crt-shared-rust itself.

      "Ours" is read off `dll/win32/ucrtbase/CMakeLists.txt` -- every static library on that
      link line except `ucrt` -- not msvcrt_shared alone. The Rust archive is whole-archived
      AHEAD of all four, so replacing a body in any of them is ordinary cohab; only a body
      that belongs to the vendored `ucrt` is the shadowing this file exists to stop. Modelling
      it as msvcrt_shared alone made the gate reject the ucrtbase math flip -- 60 findings,
      every one of them a crtmath symbol we own and had graded -- which is a gate lying about
      a correct change, and a false red is how a gate gets turned off.

Findings go to stdout, one per line, in the `path: message` shape `vibeos-cli check` parses.
Exit 1 if any finding is emitted.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent

sys.path.insert(0, str(Path(__file__).resolve().parent))

# After the sys.path.insert above -- tools/ is not a package on the default path.
from source_walk import source_files  # noqa: E402


def _spec_exports(spec: Path, arch: str) -> set[str]:
    """Export NAMES ucrtbase.spec declares for one arch, `-arch=` filters honoured.

    Delegates to export_census.parse_spec, which resolves the filter the way spec2def does
    (including negated tags like `-arch=!i386`) and hard-fails on an unknown tag rather than
    dropping the line silently. Imported lazily so this module still loads if that tool moves.
    """
    from export_census import parse_spec

    return set(parse_spec(spec, arch))


TAKEOVER = REPO / "tools" / "ucrt-takeover.yaml"


def declared_takeovers() -> dict[str, str]:
    """Genuine UCRT exports we have DELIBERATELY replaced -> the family that did it.

    Displacing a ucrt body is the exact shape of the accident this file exists to
    prevent; the only thing separating the two is that somebody declared it and
    named an oracle. `tools/ucrt-takeover.yaml` is that declaration, and it is read
    both ways -- an undeclared displacement fails, and so does a declaration the
    archive no longer backs, because a stale entry is how a list stops being read.
    """
    if not TAKEOVER.is_file():
        return {}
    import yaml

    doc = yaml.safe_load(TAKEOVER.read_text(encoding="utf-8")) or {}
    out: dict[str, str] = {}
    for fam, body in (doc.get("families") or {}).items():
        for sym in body.get("exports") or []:
            out[sym] = fam
    return out


WS = REPO / "sdk" / "lib" / "rust"

# The crate ucrtbase links, and the crate it must never be able to reach.
UCRT_ARCHIVE_CRATE = "crt-shared-cargo"
FORBIDDEN_CRATE = "msvcrt-crt-rust"

# Reached through the `#[panic_handler]` attribute, not by name -- a reference the linker
# honours and a grep for `extern crate` cannot see.
EXTERN_CRATE_EXEMPT = {"panic-common"}

# OUR static libraries on ucrtbase's link line (dll/win32/ucrtbase/CMakeLists.txt), mapped to
# where CMake puts their objects. The Rust archive is whole-archived ahead of all of them, so
# a Rust body replacing one of theirs is ordinary cohab -- these are jointly the authority on
# what we are allowed to define. `ucrt` itself is deliberately absent: it is the vendored
# Microsoft library, and displacing one of ITS bodies is precisely the defect.
OURS_ON_LINK_LINE = {
    "msvcrt_shared": Path("dll/win32/msvcrt/CMakeFiles/msvcrt_shared.dir"),
    "crtmath": Path("sdk/lib/crt/CMakeFiles/crtmath.dir"),
    "vcruntime": Path("sdk/lib/vcruntime/CMakeFiles/vcruntime.dir"),
    # upstream-sync 2026-09-20: upstream split sdk/lib/vcruntime in two and ucrtbase now
    # links BOTH halves. vcstartup holds mainCRTStartup/atexit/tlssup/the security cookie
    # — objects this tree already owned, just under the other target name.
    "vcstartup": Path("sdk/lib/vcstartup/CMakeFiles/vcstartup.dir"),
    "chkstk": Path("sdk/lib/crt/CMakeFiles/chkstk.dir"),
    "wine": Path("sdk/lib/libwine/CMakeFiles/wine.dir"),
}

#: ucrtbase's OWN C -- printf.c, stubs.c and the generated ucrtbase_stubs.c. Kept
#: SEPARATE from OURS_ON_LINK_LINE because that set must stay exactly ucrtbase's
#: link libraries (test_the_authority_set_is_the_whole_link_line_except_ucrt pins
#: it against the CMake, so drift is caught); the DLL's own translation units are
#: not a library and would break that invariant.
#:
#: But they ARE ours, and the assertion is "the Rust surface is a subset of OUR C's
#: surface", so leaving them out under-models the C. They define 12 symbols and
#: ucrtbase EXPORTS six: fma, fmaf, log2, log2f, lrint, lrintf -- stubs.c's
#: placeholders for things spec2def cannot stub because they are intrinsics. Omitted,
#: a Rust twin of any of those six reads as displacing a genuine Microsoft UCRT body
#: when it would be displacing ReactOS's own.
#:
#: This is the same defect the link-line test's docstring records, one target over:
#: modelling the authority as too small turned a correct ucrtbase math flip into 60
#: findings. Found by asking what blocks translating those 132 lines.
OURS_OWN_TUS = {
    "ucrtbase-own": Path("dll/win32/ucrtbase/CMakeFiles/ucrtbase.dir"),
}

#: Everything of OURS that contributes symbols to ucrtbase.dll, which is what the
#: subset assertion must be taken against.
OURS_C_AUTHORITY = {**OURS_ON_LINK_LINE, **OURS_OWN_TUS}

ARCHES = {
    "i386": "i686-pc-windows-gnu",
    "amd64": "x86_64-pc-windows-gnu",
    "arm64": "aarch64-pc-windows-gnullvm",
}


def _deps_of(crate_dir: Path) -> set[str]:
    """Path dependencies named in a crate's Cargo.toml.

    Deliberately a regex over `[dependencies]` rather than a `cargo metadata` call: this runs
    in the lint gate, where shelling out to cargo costs seconds and can block on the package
    cache lock. The manifests here are hand-written and uniform (`name = { path = "..." }`),
    so the parse is exact for the shape the workspace actually uses.
    """
    manifest = crate_dir / "Cargo.toml"
    if not manifest.is_file():
        return set()
    text = manifest.read_text(errors="replace")
    # Keep only dependency tables; a `[features]` entry may legitimately NAME a crate
    # (`cpp = ["crt-shared-rust/cpp"]`) without depending on it from this crate.
    out: set[str] = set()
    in_deps = False
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("["):
            in_deps = stripped.startswith("[dependencies") or (
                stripped.startswith("[target.") and "dependencies" in stripped
            )
            continue
        if not in_deps or not stripped or stripped.startswith("#"):
            continue
        m = re.match(r"^([A-Za-z0-9_-]+)\s*=", stripped)
        if m:
            out.add(m.group(1))
    return out


def check_extern_crates(ws: Path | None = None) -> list[str]:
    """Every dependency of `crt-shared-cargo` must be named by an `extern crate` in its lib.rs.

    A staticlib pulls in a dependency's objects only if something REFERENCES the crate. A
    manifest entry alone compiles the dependency and then discards it, so its `#[no_mangle]`
    bodies never reach the archive -- and nothing says so. The build is green, this file's
    archive check is green (a symbol that is absent cannot shadow anything), and the cohab
    flip has simply not happened.

    That is not hypothetical: enabling `cabi-math` on cnt-rust for the ucrtbase math flip
    landed 268 symbols in `libcrt_shared_cargo.a` and not one of them was `tan`. The archive
    check said OK because its question is "does the archive define too much", and an empty
    answer passes it. This one asks the other question.

    Only `panic-common` is exempt: it is reached through the `#[panic_handler]` attribute
    rather than by name, which is a reference the linker honours and a grep cannot see.
    """
    ws = WS if ws is None else ws
    manifest_deps = _deps_of(ws / UCRT_ARCHIVE_CRATE)
    lib = ws / UCRT_ARCHIVE_CRATE / "src" / "lib.rs"
    if not lib.is_file():
        return [f"sdk/lib/rust/{UCRT_ARCHIVE_CRATE}/src/lib.rs: MISSING"]
    text = lib.read_text(errors="replace")
    declared = set(re.findall(r"^\s*extern\s+crate\s+([A-Za-z0-9_]+)\s*;", text, re.M))
    findings = []
    for dep in sorted(manifest_deps - EXTERN_CRATE_EXEMPT):
        if dep.replace("-", "_") not in declared:
            findings.append(
                f"sdk/lib/rust/{UCRT_ARCHIVE_CRATE}/src/lib.rs: depends on `{dep}` but never "
                f"names it, so NONE of its objects reach libcrt_shared_cargo.a and whatever "
                f"cohab that dependency was added for is silently not happening. "
                f"fix: add `extern crate {dep.replace('-', '_')};`"
            )
    return findings


def check_structure(ws: Path | None = None) -> list[str]:
    """`crt-shared-cargo` must not reach `msvcrt-crt-rust`, transitively or otherwise.

    `ws` overrides the workspace root so the control tests in
    tools/test_verify_ucrtbase_cohab.py can point this at a fixture that DOES have the
    forbidden edge -- a gate nobody has watched fail is a gate nobody knows works.
    """
    ws = WS if ws is None else ws
    findings: list[str] = []
    start = ws / UCRT_ARCHIVE_CRATE
    if not start.is_dir():
        return [
            f"sdk/lib/rust/{UCRT_ARCHIVE_CRATE}: crate is MISSING -- ucrtbase's Rust archive "
            f"has no manifest. fix: restore the crate, or drop the cohab from "
            f"dll/win32/ucrtbase/CMakeLists.txt"
        ]

    # BFS the path-dependency graph, recording how we got there so a violation reports the
    # actual chain rather than just the endpoint.
    parent: dict[str, str | None] = {UCRT_ARCHIVE_CRATE: None}
    queue = [UCRT_ARCHIVE_CRATE]
    while queue:
        cur = queue.pop(0)
        for dep in sorted(_deps_of(ws / cur)):
            if dep in parent:
                continue
            parent[dep] = cur
            queue.append(dep)

    if FORBIDDEN_CRATE in parent:
        chain, node = [], FORBIDDEN_CRATE
        while node is not None:
            chain.append(node)
            node = parent[node]
        findings.append(
            f"sdk/lib/rust/{UCRT_ARCHIVE_CRATE}/Cargo.toml: ucrtbase's Rust archive can reach "
            f"{FORBIDDEN_CRATE} via {' -> '.join(reversed(chain))} -- that pulls msvcrt's "
            f"whole CRT floor into ucrtbase.dll, silently replacing genuine UCRT bodies under "
            f"--allow-multiple-definition. fix: keep the shared surface in crt-shared-rust; "
            f"msvcrt-crt-rust must depend on IT, never the reverse"
        )
    return findings


def _archive_source_crates(ws: Path | None = None) -> tuple[str, ...]:
    """The crates whose Rust actually ends up in `libcrt_shared_cargo.a`: the
    path-dependency closure of crt-shared-cargo, WALKED, not listed.

    This was a hand-written tuple, and it went stale the day ucrt-rust joined the
    archive: edits to the takeover bodies never tripped the staleness note, because
    nobody had added the crate to a list that nothing checked. Deliberately NOT
    "every crate in sdk/lib/rust": a staleness warning that fires whenever any
    unrelated lane edits any .rs is noise, and noise is what gets a check ignored.
    """
    ws = WS if ws is None else ws
    seen = [UCRT_ARCHIVE_CRATE]
    for cur in seen:
        for dep in sorted(_deps_of(ws / cur)):
            if dep not in seen:
                seen.append(dep)
    return tuple(seen)


def _closure(root: str, ws: Path | None = None) -> set[str]:
    """Path-dependency closure of one crate, root included."""
    ws = WS if ws is None else ws
    seen = [root]
    for cur in seen:
        for dep in sorted(_deps_of(ws / cur)):
            if dep not in seen:
                seen.append(dep)
    return set(seen)


#: Each Rust staticlib a CRT DLL links: (its CMakeLists, the `set(...)` naming the
#: crates whose sources trigger its rebuild, the crate cargo builds).
REBUILD_LISTS = (
    ("dll/win32/ucrtbase/CMakeLists.txt", "UCRT_SHARED_RUST_CRATES", "crt-shared-cargo"),
    ("dll/win32/msvcrt/CMakeLists.txt", "MSVCRT_CRT_RUST_CRATES", "msvcrt-crt-cargo"),
)


def check_rebuild_lists(repo: Path | None = None, ws: Path | None = None) -> list[str]:
    """Each archive's CMake rebuild list must name its whole crate closure.

    A crate compiled INTO an archive but missing from the custom command's DEPENDS
    never triggers a rebuild: edit it and the build ships the stale archive, green,
    with no error anywhere. ucrtbase's list named two of its seven crates -- the
    ucrt-rust takeover families and cnt-rust's math were invisible to it -- and
    msvcrt's lacked crt-portable-rust the day the string family moved there.
    """
    repo = REPO if repo is None else repo
    findings = []
    for cmake, var, root in REBUILD_LISTS:
        text = (repo / cmake).read_text(errors="replace")
        m = re.search(rf"set\(\s*{var}\s+([^)]*)\)", text)
        if not m:
            findings.append(f"{cmake}: no `set({var} ...)` -- the rebuild list is missing")
            continue
        listed = set(m.group(1).split())
        want = _closure(root, ws)
        for crate in sorted(want - listed):
            findings.append(
                f"{cmake}: {var} omits `{crate}`, which is compiled into {root}'s archive -- "
                f"an edit to it would ship a STALE archive. fix: add it to {var}"
            )
        for crate in sorted(listed - want):
            findings.append(
                f"{cmake}: {var} names `{crate}`, which {root} no longer depends on. fix: remove it"
            )
    return findings


def _content_changed_at() -> tuple[float, str] | None:
    """`(timestamp, what)` of the last CONTENT change to the archive's sources.

    mtime alone cannot answer this. `git merge` and `git checkout` rewrite a
    file's mtime WITHOUT changing a byte of it, and the conductor merges every
    queued lane at the start of a drain — so in the integration tree every source
    file looks "just modified" on every batch while cargo, correctly, relinks
    nothing. Comparing the archive to mtimes there is comparing it to the clock.

    Git knows the difference. For a tracked, unmodified file the content is
    whatever the last commit touching it says, so the COMMIT time is the real
    change time. Only a file that is dirty in the worktree has content mtime
    cannot account for, and for those mtime is exactly right.

    Returns None when git cannot answer (not a repo, no git) — the caller then
    falls back to mtime, which is wrong in the merge case but is strictly better
    than skipping the assertion entirely.
    """
    paths = [f"sdk/lib/rust/{c}" for c in _archive_source_crates()]
    try:
        committed = subprocess.run(
            ["git", "-C", str(REPO), "log", "-1", "--format=%ct", "--", *paths],
            capture_output=True,
            text=True,
            timeout=60,
        )
        dirty = subprocess.run(
            ["git", "-C", str(REPO), "status", "--porcelain", "--", *paths],
            capture_output=True,
            text=True,
            timeout=60,
        )
    except (OSError, subprocess.SubprocessError):
        return None
    if committed.returncode != 0 or dirty.returncode != 0:
        return None

    best, what = 0.0, ""
    text = committed.stdout.strip()
    if text:
        try:
            best, what = float(text), "the last commit touching them"
        except ValueError:
            return None
    # A locally-edited file is the one case where mtime IS the content clock.
    for line in dirty.stdout.splitlines():
        rel = line[3:].strip().strip('"')
        if not rel:
            continue
        f = REPO / rel
        try:
            if f.is_file() and f.stat().st_mtime > best:
                best, what = f.stat().st_mtime, rel
        except OSError:
            continue
    return (best, what) if best else None


def _ucrt_archive(build: Path, triple: str) -> Path | None:
    """The libcrt_shared_cargo.a a build dir links into ucrtbase, if it has one."""
    for archive in (
        build / "sdk" / "lib" / "rust" / "target" / triple / "release" / "libcrt_shared_cargo.a",
        build / "dll" / "win32" / "ucrtbase" / "libcrt_shared_cargo.a",
    ):
        if archive.is_file():
            return archive
    return None


def burndown_stale(arch: str) -> str | None:
    """Why `arch`'s build cannot be measured for the ratchet right now, or None.

    The ratchet reads the BUILD, and the gate runs check-tooling CONCURRENTLY with
    its builds. A commit that takes a family over and lowers the ceiling in the same
    change is then measured against the previous commit's archive, and reads as a
    regression: the integration tree reported 999 against a recorded 991 and bounced
    the lane that had made the number 991. Same rule as check_archive: an archive
    older than its sources gets no verdict, in either direction.
    """
    archive = _ucrt_archive(REPO / f"build-{arch}", ARCHES[arch])
    if archive is None:
        return "no libcrt_shared_cargo.a in the build"
    stale = _stale_against_sources(archive)
    if stale:
        return stale
    # CONTENT, not the clock. The land queue's integration worktree is shared by
    # consecutive batches with DIFFERENT content, so the archive a previous batch
    # built is newer than this tree's commits and passes every mtime test -- while
    # lacking this tree's takeovers. That measured 999 against a recorded 991 and
    # bounced the lane that earned the drop, gate after gate, after the mtime guard
    # above was already in place. An archive that does not define every export this
    # tree DECLARES taken over was built from some other tree.
    defined = _nm_defined(archive)
    if defined is None:
        return "nm cannot read the archive"
    if arch == "i386":
        defined = {d[1:] if d.startswith("_") else d for d in defined}
    missing = sorted(set(declared_takeovers()) - defined)
    if missing:
        return (
            f"the archive lacks {len(missing)} export(s) this tree declares taken over "
            f"(e.g. {', '.join(missing[:3])}) -- it was built from a different tree"
        )
    return None


def _stale_against_sources(archive: Path) -> str | None:
    """What changed after `archive` was built, or None if it is current.

    Naming the cause rather than counting is what makes the message actionable.
    """
    try:
        archive_mtime = archive.stat().st_mtime
    except OSError:
        return None

    changed = _content_changed_at()
    if changed is not None:
        ts, what = changed
        return what if ts > archive_mtime else None

    # git could not answer — fall back to raw mtimes.
    for crate in _archive_source_crates():
        src = REPO / "sdk" / "lib" / "rust" / crate
        if not src.is_dir():
            continue
        # `source_files` and not `rglob`: no `target/` lives under a crate's `src/`
        # TODAY, but that is a fact about the current tree, not an invariant, and
        # `rglob` has already descended before any filter could exclude one.
        # tools/scan_unpruned_walk.py gates exactly this and caught the rglob here.
        candidates = [
            *source_files(src / "src", "*.rs"),
            src / "Cargo.toml",
            src / "build.rs",
        ]
        for f in candidates:
            try:
                if f.is_file() and f.stat().st_mtime > archive_mtime:
                    return str(f.relative_to(REPO))
            except OSError:
                continue
    return None


def _nm_defined(path: Path | list[Path]) -> set[str] | None:
    """Globally-defined symbol names in an archive or object, or None if nm cannot read it."""
    paths = [path] if isinstance(path, Path) else path
    if not paths:
        return set()
    for nm in (str(REPO / ".build-tools/llvm-mingw/bin/llvm-nm"), "llvm-nm", "nm"):
        try:
            res = subprocess.run(
                [nm, "--defined-only", *(str(p) for p in paths)],
                capture_output=True,
                text=True,
                timeout=120,
            )
        except (OSError, subprocess.TimeoutExpired):
            # OSError, not FileNotFoundError: a present-but-non-executable llvm-nm raises
            # PermissionError, which took this whole check down the first time it ran here.
            # Any failure to EXECUTE means "this nm is unusable, try the next"; a failure to
            # READ is handled by the returncode check below. Never silently pass: if no nm
            # works at all, _nm_defined returns None and the caller emits a finding.
            continue
        if res.returncode != 0:
            continue
        syms: set[str] = set()
        for line in res.stdout.splitlines():
            parts = line.split()
            # `<addr> <type> <name>` for a definition; skip `<type> <name>` (undefined) and
            # archive member headers. Only GLOBAL text/data can collide at link.
            if len(parts) == 3 and parts[1] in "TDBRtdbr" and parts[1].isupper():
                syms.add(parts[2])
        return syms
    return None


def check_archive() -> list[str]:
    """Built archive's symbols must be a subset of what OUR libs on the link line define.

    Fleet isolation law: generated artifacts are per-commit, source-derived state, so they
    come from THIS worktree or not at all. A sibling's build dir at another commit would make
    the verdict depend on which lanes happen to exist -- `verify_signatures` moved 987 -> 9
    findings that way with no source change.
    """
    findings: list[str] = []
    for arch, triple in ARCHES.items():
        build = REPO / f"build-{arch}"
        if not build.is_dir():
            continue  # not built here; the structural check still ran

        archive = _ucrt_archive(build, triple)
        if archive is None:
            continue  # ucrtbase cohab not wired into this build dir yet

        stale = _stale_against_sources(archive)
        if stale:
            # CANNOT RENDER A VERDICT -- and that is NOT the lane's fault, so it
            # must not be a finding. mtime is the only staleness signal available
            # here and it is not sound: `git merge` rewrites a source file's mtime
            # WITHOUT changing its content, while cargo legitimately declines to
            # relink an archive whose inputs are unchanged. The conductor merges
            # every queued lane at batch start, so in the integration tree those
            # two facts combine on every drain and the archive reads "older than
            # its sources" while being perfectly current.
            #
            # Emitted as a finding, this rejected a lane for a merge it did not
            # make -- and had it landed it would have done so to every lane, since
            # this check runs in the SOURCE-lint phase where build state is
            # whatever the last batch left behind. Printed instead: still loud for
            # the human who runs this directly (which is what caught two genuinely
            # stale archives when the guard was written), never a verdict.
            print(
                f"note: build-{arch}: {archive.name} is older than {stale}; skipping the "
                f"subset assertion for this arch rather than grading a possibly-stale "
                f"archive. If you are debugging this locally: rm "
                f"{archive.relative_to(REPO)} && ./vibeos-cli build --arch {arch}",
                file=sys.stderr,
            )
            continue

        objs: list[Path] = []
        missing: list[str] = []
        for target, obj_dir in OURS_C_AUTHORITY.items():
            d = build / obj_dir
            found = sorted(d.rglob("*.obj")) + sorted(d.rglob("*.o"))
            if found:
                objs += found
            else:
                missing.append(f"{target} ({d})")
        if missing:
            # Partial evidence is worse than none: a target whose objects are absent
            # contributes nothing to `c_norm`, so every symbol it defines looks like ours
            # shadowing ucrt. That reads as a wall of false findings, not a soft verdict.
            findings.append(
                f"build-{arch}: libcrt_shared_cargo.a exists but the objects of "
                f"{', '.join(missing)} do not -- cannot prove the Rust surface is a subset "
                f"of the C's. fix: ./vibeos-cli build --arch {arch}"
            )
            continue

        rust_syms = _nm_defined(archive)
        if rust_syms is None:
            findings.append(
                f"build-{arch}: no usable nm for {archive.name} -- the subset assertion could "
                f"not run. fix: install llvm-nm (this check must not silently pass)"
            )
            continue

        # nm accepts many files. One invocation reads the complete C surface;
        # spawning a process per object made a warm check take tens of seconds.
        c_syms = _nm_defined(objs)
        if c_syms is None:
            findings.append(f"build-{arch}: nm could not read the complete C object surface")
            continue

        def strip(sym: str, *, i386: bool = (arch == "i386")) -> str:
            """i386 decorates cdecl exports with a leading underscore; compare bare.

            `i386` is a bound default rather than a closure over the loop variable: this
            function outlives the iteration that made it (ruff B023).
            """
            return sym[1:] if i386 and sym.startswith("_") else sym

        c_norm = {strip(s) for s in c_syms}

        # THE ASSERTION, stated precisely. "Subset of what we define" is very nearly right but
        # over-fires: rustc bundles compiler-rt into every staticlib, so the archive also
        # carries __divdc3, __absvdi2, _Unwind_Resume and ~100 more that no C of ours defines.
        # Those are not the hazard. Shadowing needs BOTH halves --
        #
        #     (a) none of OUR static libs on the link line defines it, so we have no C body
        #         to be the authority on what it should do, AND
        #     (b) ucrtbase EXPORTS it, so a real caller reaches our body instead of the
        #         genuine UCRT one.
        #
        # A compiler-rt builtin fails (b): ucrtbase.spec does not export __divdc3, so ours
        # cannot displace anything callers see. An export we have no C for fails nothing and
        # is exactly the 856-body accident this file exists to prevent.
        #
        # Reporting the (a)-only symbols as findings is what a strict gate looks like right
        # before someone turns it off: 114 lines of noise around zero real defects.
        # `-arch=` is part of (b), not decoration. A spec line is an export only on the
        # arches it names: `@ cdecl -arch=!i386 tanf(float)` means ucrtbase has NO tanf on
        # i386, so a Rust tanf there displaces nothing a caller can reach. Skipping the flags
        # without reading them (`(?:-\S+\s+)*` did exactly that) made the export set
        # arch-blind and reported 11 float bodies -- __logbf, _sqrtf, _tanf and friends, all
        # `-arch=!i386` -- as i386 shadowing. Every one was a false positive, and it bounced a
        # correct lane. `export_census.parse_spec` already resolves this the way spec2def
        # does, negations included; reuse it rather than grow a second parser to drift.
        spec = REPO / "dll" / "win32" / "ucrtbase" / "ucrtbase.spec"
        exported: set[str] = set()
        if spec.is_file():
            exported = set(_spec_exports(spec, arch))
        else:
            findings.append(
                f"dll/win32/ucrtbase/ucrtbase.spec: MISSING -- cannot tell which symbols "
                f"ucrtbase exports, so the shadowing assertion for build-{arch} could not run"
            )
            continue

        def is_rust_internal(sym: str) -> bool:
            """True for a symbol no C declaration could ever name, so it cannot shadow one.

            The check is about SHADOWING, and shadowing needs a name collision: a C caller
            reaches a body by writing its identifier. Rust's mangled items (`_ZN..17h<hash>E`)
            and the `.refptr.` COFF indirection stubs the linker emits for their statics carry
            characters no C identifier has, so no ucrt TU can bind to them however many land in
            the archive. Filtering them is not loosening the assertion -- keeping them in was
            reporting 114 findings that were all noise, which is how a strict gate gets
            disabled by the next person who reads it.
            """
            return (
                sym.startswith((".refptr.", "_ZN", "__rust", "anon.", ".L", ".weak."))
                or "_ZN" in sym
                or not re.fullmatch(r"[A-Za-z_?@$][\w?@$]*", sym)
            )

        # A DECLARED takeover is not shadowing -- it is the quest. Compared on the
        # stripped name so one declaration covers all three arches' decoration.
        declared = declared_takeovers()
        declared_norm = {k.lstrip("_"): v for k, v in declared.items()}
        rust_norm = {strip(s).lstrip("_") for s in rust_syms}
        gone = sorted(k for k in declared_norm if k not in rust_norm)
        if gone:
            findings.append(
                f"build-{arch}: tools/ucrt-takeover.yaml declares {len(gone)} export(s) that "
                f"libcrt_shared_cargo.a does not define: {', '.join(gone[:12])}. Either the "
                f"body was removed and the declaration is stale, or the family's cargo "
                f"feature is not enabled in crt-shared-cargo -- in which case the takeover "
                f"silently did not happen and the Microsoft body is still live"
            )

        extra = sorted(
            s
            for s in rust_syms
            if strip(s) not in c_norm
            and strip(s) in exported
            and not is_rust_internal(s)
            and strip(s).lstrip("_") not in declared_norm
        )
        if extra:
            more = f", +{len(extra) - 12} more" if len(extra) > 12 else ""
            shown = ", ".join(extra[:12]) + more
            findings.append(
                f"build-{arch}: libcrt_shared_cargo.a defines {len(extra)} symbol(s) that "
                f"ucrtbase EXPORTS but none of our own static libs on its link line "
                f"({', '.join(OURS_ON_LINK_LINE)}) defines, so each one "
                f"silently replaces the genuine Microsoft UCRT body callers expect "
                f"(--allow-multiple-definition raises nothing): {shown}. fix: move the body "
                f"back to msvcrt-crt-rust; or, if this is a DELIBERATE takeover of the "
                f"Microsoft body, implement it in ucrt-rust and declare it in "
                f"tools/ucrt-takeover.yaml with the oracle that grades it; or -- if "
                f"ucrtbase really does owe it to us -- add the C to SHARED_SOURCE so both "
                f"sides agree on the surface"
            )
    return findings


def check_burndown() -> tuple[list[str], list[str]]:
    """THE RATCHET: how much of ucrtbase is still answered by vendored Microsoft C.

    `vendor.yaml` flags `sdk/lib/ucrt` UNWELCOME -- ~78K lines carrying no
    licence grant at all. That records the intent; this is what stops the tree
    quietly growing while nobody is looking, and what makes the number march
    down instead of drifting.

    Only an INCREASE fails. A decrease is the entire point of the exercise, and
    it is routinely caused by ANOTHER lane landing a Rust body that happens to
    answer a ucrtbase export -- 234 owed exports became 218 that way in a single
    evening. A gate that failed on "better than recorded" would bounce lanes for
    improving the tree, which is precisely how a gate ends up switched off. A
    decrease prints a note asking for the ceiling to be re-recorded.

    Needs a build: which OBJECT defines a body is not answerable from the .spec,
    and that distinction is the whole point (see check_archive). Arches without
    a build are skipped rather than guessed at.

    Returns (failures, notes).
    """
    if not TAKEOVER.is_file():
        return [], []
    import yaml

    doc = yaml.safe_load(TAKEOVER.read_text(encoding="utf-8")) or {}
    want = {str(k): int(v) for k, v in (doc.get("burndown") or {}).items()}
    if not want:
        return [], []

    try:
        from ucrt_burndown import classify
    except ImportError as e:
        return [f"ucrt burn-down ratchet could not run: {e}"], []

    fails: list[str] = []
    notes: list[str] = []
    for arch, ceiling in sorted(want.items()):
        if not (REPO / f"build-{arch}").is_dir():
            continue
        stale = burndown_stale(arch)
        if stale:
            notes.append(f"note: ucrt burn-down: {arch} not measured -- build is stale ({stale})")
            continue
        try:
            buckets, _ = classify(arch)
        except SystemExit as e:
            notes.append(f"note: ucrt burn-down: {arch} not measurable ({e})")
            continue
        have = len(buckets["MICROSOFT"])
        if have > ceiling:
            fails.append(
                f"ucrt burn-down REGRESSED on {arch}: {have} exports are answered by "
                f"vendored Microsoft C, up from the recorded {ceiling}. Something "
                f"re-enabled a sdk/lib/ucrt body or dropped one of ours. Investigate "
                f"before re-recording: the ceiling in tools/ucrt-takeover.yaml only "
                f"ever goes DOWN. (python3 tools/ucrt_burndown.py --arch {arch})"
            )
        elif have < ceiling:
            notes.append(
                f"note: ucrt burn-down {arch}: {ceiling} -> {have}, {ceiling - have} "
                f"fewer Microsoft-backed exports. Lower the ceiling in "
                f"tools/ucrt-takeover.yaml to keep the ratchet tight."
            )
    return fails, notes


#: Initializers nothing in this tree has ever force-included, so their absence is the
#: baseline, not a loss: fma3 is Microsoft's x64 FMA3 math switch, and ReactOS's
#: ucrtbase never linked it (no `_CRT_LINKER_FORCE_INCLUDE` for it exists anywhere).
UNREFERENCED_INITIALIZERS = frozenset({"__acrt_tran_fma3_initializer"})

_CRTALLOC_SYM = re.compile(r'_CRTALLOC\("\.CRT\$X[^"]*"\)\s*\w+\s+const\s+(\w+)\s*=')


def check_force_includes(repo: Path | None = None) -> list[str]:
    """Every CRT startup/shutdown hook must still be pulled into ucrtbase.

    `sdk/lib/ucrt/initializers/*.cpp` each define a pointer in `.CRT$XIC`/`.CRT$XPX`
    that nothing calls by name; the C file that needs it carries
    `_CRT_LINKER_FORCE_INCLUDE(<sym>)`, a `/include:` directive, instead. Delete that
    file in a takeover and the directive goes with it: the member is never linked and
    the hook silently stops running, with no build or link error. The locale takeover
    dropped two this way (the locale terminator and the multibyte initializer) and
    graded green, because nothing graded runs at process startup or exit.

    So each hook must still be named by a `_CRT_LINKER_FORCE_INCLUDE` in the remaining
    C, or by `force_include!` in ucrt-rust (src/force_include.rs).
    """
    repo = REPO if repo is None else repo
    init_dir = repo / "sdk" / "lib" / "ucrt" / "initializers"
    # Once the initializers themselves are Rust (src/internal/init.rs, `#[used]`
    # statics in the --whole-archive'd crate) there is no member left to fail to
    # pull: every hook is linked unconditionally. Nothing to check.
    if not init_dir.is_dir():
        return []
    hooks = set()
    for f in sorted(init_dir.glob("*.cpp")):
        hooks.update(_CRTALLOC_SYM.findall(f.read_text(errors="replace")))
    text = []
    for root, pattern in (
        (repo / "sdk" / "lib" / "ucrt", "*.cpp"),
        (repo / "sdk" / "lib" / "rust" / "ucrt-rust" / "src", "*.rs"),
    ):
        for f in root.rglob(pattern):
            if init_dir not in f.parents:
                text.append(f.read_text(errors="replace"))
    blob = "\n".join(text)
    findings = []
    for sym in sorted(hooks - UNREFERENCED_INITIALIZERS):
        if not re.search(rf"(_CRT_LINKER_FORCE_INCLUDE|force_include!)\(\s*{sym}\s*\)", blob):
            findings.append(
                f"ucrt initializer `{sym}` is force-included by nothing: its archive member "
                f"will not be linked and the hook will never run. A takeover deleted the C "
                f"file that carried `_CRT_LINKER_FORCE_INCLUDE({sym})`. fix: add "
                f"`crate::force_include!({sym});` to the Rust module that replaced it"
            )
    return findings


_EXTERN_C_BLOCK = re.compile(r'extern\s+"C"\s*\{(.*?)\n\s*\}', re.S)
_PASCAL_FN = re.compile(r"\bfn\s+([A-Z][A-Za-z0-9]*)\s*\(")
_ANY_FN = re.compile(r"\bfn\s+(\w+)\s*\(")
_WINAPI_DECL = re.compile(r"\b(?:WINAPI|APIENTRY|NTAPI|__stdcall)\s+(\w+)\s*\(")


def check_win32_abi(repo: Path | None = None) -> list[str]:
    """No Win32 API may be declared `extern "C"` in ucrt-rust.

    WINAPI is stdcall on i386, so `extern "C" { fn ReadFile(...) }` binds `_ReadFile`
    where kernel32 exports `_ReadFile@20`. It links on amd64 and arm64 (one calling
    convention) and fails only on i386 -- after an arm64 grade has already passed, and
    then only as an undefined symbol that stops the WOW64 closure on amd64 too. Four
    ports have shipped this. Win32 names are PascalCase and CRT names are not, so a
    PascalCase function in an `extern "C"` block is one tell; the CRT's own `__acrt_*`
    Win32 thunks are not PascalCase and are mixed (`__acrt_SetThreadStackGuarantee` is
    WINAPI, `__acrt_MultiByteToWideChar` is __cdecl), so for those the header's
    WINAPI is the other.
    """
    repo = REPO if repo is None else repo
    winapi: set[str] = set()
    for h in sorted((repo / "sdk" / "lib" / "ucrt" / "inc").glob("*.h")):
        winapi.update(_WINAPI_DECL.findall(h.read_text(errors="replace")))
    findings = []
    for f in sorted(source_files(repo / "sdk" / "lib" / "rust" / "ucrt-rust" / "src")):
        for block in _EXTERN_C_BLOCK.finditer(f.read_text(errors="replace")):
            names = set(_PASCAL_FN.findall(block.group(1)))
            names |= {n for n in _ANY_FN.findall(block.group(1)) if n in winapi}
            for name in sorted(names):
                findings.append(
                    f'{f.relative_to(repo)}: `{name}` is declared in an `extern "C"` block; '
                    f'a Win32 API is stdcall on i386. fix: move it to `extern "system"`'
                )
    return findings


#: Exports ucrtbase.dll is ALLOWED to answer with an import of the same name. Every
#: other name it both exports and imports is a body a takeover lost: the string
#: takeover deleted memcpy_s.cpp and strnlen.cpp, their memmove_s and wcslen were
#: never ported, and the link quietly resolved both through the msvcrt/ntdll import
#: libraries -- ucrtbase shipped a forward to msvcrt.dll with no error anywhere.
EXPORT_IMPORT_ALLOWED = {
    # The compiler-intrinsic string primitives: this tree has always taken them
    # from ntdll (add_importlibs(ucrtbase kernel32 ntdll)), which exports the same
    # contracts. Measured 2026-09-28 on all three arches.
    **dict.fromkeys(
        ("memchr", "memcmp", "memcpy", "memmove", "memset", "strchr", "strrchr", "strstr",
         "wcschr", "wcsrchr", "wcsstr"),
        "ntdll.dll",
    ),
    # OWED, arm64 only: the x87/SSE control-word family has no arm64 body in this
    # tree (sdk/lib/crt builds them for i386/amd64 only), so arm64's resolve to
    # msvcrt.dll's. Tracked in the roadmap's known-bugs registry.
    **dict.fromkeys(("_clearfp", "_control87", "_controlfp", "_statusfp"), "msvcrt.dll"),
}


def _pe_exports_imports(dll: Path) -> tuple[set[str], dict[str, str]] | None:
    """(export names, import name -> dll) of a PE, or None if objdump cannot read it."""
    for objdump in (str(REPO / ".build-tools/llvm-mingw/bin/llvm-objdump"), "llvm-objdump"):
        try:
            res = subprocess.run([objdump, "-p", str(dll)], capture_output=True, text=True, timeout=120)
        except (OSError, subprocess.TimeoutExpired):
            continue
        if res.returncode != 0:
            continue
        exports: set[str] = set()
        imports: dict[str, str] = {}
        section, dll_name = None, None
        for line in res.stdout.splitlines():
            m = re.match(r"\s*DLL Name: (\S+)", line)
            if m:
                section, dll_name = "imp", m.group(1)
                continue
            if line.startswith("Export Table"):
                section = "exp"
                continue
            if section == "imp" and dll_name:
                m = re.match(r"\s+\d+\s+(\S+)$", line)
                if m:
                    imports.setdefault(m.group(1), dll_name)
            elif section == "exp":
                m = re.match(r"\s+\d+\s+0x[0-9a-fA-F]+\s+(\S+)$", line)
                if m:
                    exports.add(m.group(1))
        return exports, imports
    return None


def check_export_imports(repo: Path | None = None) -> tuple[list[str], list[str]]:
    """No ucrtbase export may be answered by an import of the same name.

    Reads the LINKED dll, not the sources: the failure it exists for is invisible in
    source (the body is simply gone) and at link time (an import library satisfies
    the reference). Returns (failures, notes); an arch without a current build is a
    note, never a pass or a failure.
    """
    repo = REPO if repo is None else repo
    fails: list[str] = []
    notes: list[str] = []
    for arch in ("i386", "amd64", "arm64"):
        dll = repo / f"build-{arch}" / "dll" / "win32" / "ucrtbase" / "ucrtbase.dll"
        if not dll.is_file():
            notes.append(f"note: export/import check: {arch} not measured -- no {dll.relative_to(repo)}")
            continue
        stale = burndown_stale(arch) if repo == REPO else None
        if stale:
            notes.append(f"note: export/import check: {arch} not measured -- {stale}")
            continue
        got = _pe_exports_imports(dll)
        if got is None:
            fails.append(f"ucrtbase {arch}: no usable llvm-objdump; cannot read {dll}")
            continue
        exports, imports = got
        for name in sorted(exports & imports.keys()):
            if EXPORT_IMPORT_ALLOWED.get(name) == imports[name].lower():
                continue
            fails.append(
                f"ucrtbase {arch}: `{name}` is exported but answered by an import from "
                f"{imports[name]} -- no object on the link line defines it, so the import "
                f"library did. A takeover deleted its body without porting it. fix: port it "
                f"into ucrt-rust (or, if the forward is deliberate, add it to "
                f"EXPORT_IMPORT_ALLOWED with the reason)"
            )
    return fails, notes


def main() -> int:
    findings = (
        check_structure()
        + check_extern_crates()
        + check_rebuild_lists()
        + check_archive()
        + check_force_includes()
        + check_win32_abi()
    )
    bd_fails, bd_notes = check_burndown()
    findings += bd_fails
    ei_fails, ei_notes = check_export_imports()
    findings += ei_fails
    bd_notes += ei_notes
    for f in findings:
        print(f)
    # Notes last, so a green run ends on the number going down -- which is the
    # one line anyone actually wants to see from this tool. On STDERR: the lint
    # harness counts stdout lines as findings against a baseline, so a note there
    # ("not measured, build is stale") failed `check --strict` and blocked a land.
    for n in bd_notes:
        print(n, file=sys.stderr)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
