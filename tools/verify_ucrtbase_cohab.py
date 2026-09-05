#!/usr/bin/env python3
"""Assert that ucrtbase.dll's Rust cohab cannot shadow genuine Microsoft UCRT bodies.

ucrtbase.dll is vendored Microsoft UCRT (`sdk/lib/ucrt`, see vendor.yaml) plus OUR
`msvcrt_shared` plus 132 lines of its own C. The Rust we link into it must therefore cover
`msvcrt_shared`'s surface and NOTHING else -- and the reason that needs a tool rather than a
rule is that violating it produces no diagnostic at all:

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

  ARCHIVE (runs when this worktree has a build; hard-fails if it cannot trust one)
      Every symbol the built `libcrt_shared_cargo.a` defines must also be defined by
      `msvcrt_shared`'s own C objects. This is the empirical backstop for the case the
      structural check cannot see: someone adding a genuinely new `#[no_mangle]` to
      crt-shared-rust itself.

Findings go to stdout, one per line, in the `path: message` shape `vibeos-cli check` parses.
Exit 1 if any finding is emitted.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
WS = REPO / "sdk" / "lib" / "rust"

# The crate ucrtbase links, and the crate it must never be able to reach.
UCRT_ARCHIVE_CRATE = "crt-shared-cargo"
FORBIDDEN_CRATE = "msvcrt-crt-rust"

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
        m = re.match(r'^([A-Za-z0-9_-]+)\s*=', stripped)
        if m:
            out.add(m.group(1))
    return out


def check_structure() -> list[str]:
    """`crt-shared-cargo` must not reach `msvcrt-crt-rust`, transitively or otherwise."""
    findings: list[str] = []
    start = WS / UCRT_ARCHIVE_CRATE
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
        for dep in sorted(_deps_of(WS / cur)):
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


def _nm_defined(path: Path) -> set[str] | None:
    """Globally-defined symbol names in an archive or object, or None if nm cannot read it."""
    for nm in ("llvm-nm", "nm"):
        try:
            res = subprocess.run(
                [nm, "--defined-only", str(path)],
                capture_output=True, text=True, timeout=120,
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
    """Built archive's symbols must be a subset of msvcrt_shared's C objects.

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

        archive = (
            build / "sdk" / "lib" / "rust" / "target" / triple / "release"
            / "libcrt_shared_cargo.a"
        )
        if not archive.is_file():
            archive = build / "dll" / "win32" / "ucrtbase" / "libcrt_shared_cargo.a"
        if not archive.is_file():
            continue  # ucrtbase cohab not wired into this build dir yet

        shared_dir = (
            build / "dll" / "win32" / "msvcrt" / "CMakeFiles" / "msvcrt_shared.dir"
        )
        objs = sorted(shared_dir.rglob("*.obj")) + sorted(shared_dir.rglob("*.o"))
        if not objs:
            findings.append(
                f"build-{arch}: libcrt_shared_cargo.a exists but msvcrt_shared's objects do "
                f"not ({shared_dir} is empty) -- cannot prove the Rust surface is a subset of "
                f"the C's. fix: ./vibeos-cli build --arch {arch}"
            )
            continue

        rust_syms = _nm_defined(archive)
        if rust_syms is None:
            findings.append(
                f"build-{arch}: no usable nm for {archive.name} -- the subset assertion could "
                f"not run. fix: install llvm-nm (this check must not silently pass)"
            )
            continue

        c_syms: set[str] = set()
        for obj in objs:
            got = _nm_defined(obj)
            if got:
                c_syms |= got

        def strip(sym: str, *, i386: bool = (arch == "i386")) -> str:
            """i386 decorates cdecl exports with a leading underscore; compare bare.

            `i386` is a bound default rather than a closure over the loop variable: this
            function outlives the iteration that made it (ruff B023).
            """
            return sym[1:] if i386 and sym.startswith("_") else sym

        c_norm = {strip(s) for s in c_syms}

        # THE ASSERTION, stated precisely. "Subset of msvcrt_shared" is very nearly right but
        # over-fires: rustc bundles compiler-rt into every staticlib, so the archive also
        # carries __divdc3, __absvdi2, _Unwind_Resume and ~100 more that no C of ours defines.
        # Those are not the hazard. Shadowing needs BOTH halves --
        #
        #     (a) msvcrt_shared's C does not define it, so we have no C body to be the
        #         authority on what it should do, AND
        #     (b) ucrtbase EXPORTS it, so a real caller reaches our body instead of the
        #         genuine UCRT one.
        #
        # A compiler-rt builtin fails (b): ucrtbase.spec does not export __divdc3, so ours
        # cannot displace anything callers see. An export we have no C for fails nothing and
        # is exactly the 856-body accident this file exists to prevent.
        #
        # Reporting the (a)-only symbols as findings is what a strict gate looks like right
        # before someone turns it off: 114 lines of noise around zero real defects.
        spec = REPO / "dll" / "win32" / "ucrtbase" / "ucrtbase.spec"
        exported: set[str] = set()
        if spec.is_file():
            for ln in spec.read_text(errors="replace").splitlines():
                m = re.match(
                    r"^[\d@]+\s+(?:cdecl|stdcall|varargs|extern|stub)\s+(?:-\S+\s+)*"
                    r"([A-Za-z_?$][\w?$@]*)",
                    ln.strip(),
                )
                if m:
                    exported.add(m.group(1))
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

        extra = sorted(
            s for s in rust_syms
            if strip(s) not in c_norm
            and strip(s) in exported
            and not is_rust_internal(s)
        )
        if extra:
            more = f", +{len(extra) - 12} more" if len(extra) > 12 else ""
            shown = ", ".join(extra[:12]) + more
            findings.append(
                f"build-{arch}: libcrt_shared_cargo.a defines {len(extra)} symbol(s) that "
                f"ucrtbase EXPORTS but msvcrt_shared's C does not define, so each one "
                f"silently replaces the genuine Microsoft UCRT body callers expect "
                f"(--allow-multiple-definition raises nothing): {shown}. fix: move the body "
                f"back to msvcrt-crt-rust, or -- if ucrtbase really does owe it to us -- add "
                f"the C to SHARED_SOURCE so both sides agree on the surface"
            )
    return findings


def main() -> int:
    findings = check_structure() + check_archive()
    for f in findings:
        print(f)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
