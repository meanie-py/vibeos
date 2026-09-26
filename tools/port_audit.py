#!/usr/bin/env python3
"""Tree-wide translation status: how much C the OS still COMPILES, by surface.

WHY THIS EXISTS
---------------
`rust-port-status.md` is the single source of truth for Rust-vs-C coverage, and its
headline ledger is prose: per-surface claims a human wrote after a hand audit. It was
re-measured on 2026-07-03 and not again, so by the time anyone read it the numbers were
twelve weeks old and there was no command to refresh them. That is the same shape the
repo has been bitten by repeatedly — a number that lived in a message or a document
instead of in a tool, re-derived from scratch by whoever looked next and landing
somewhere different each time.

So this computes the ledger instead of asserting it.

THE ORACLE IS THE LINK, NOT THE SOURCE TREE
-------------------------------------------
Convention here is that a translated `.c` is never deleted: it is commented out of its
CMakeLists (`# replaced by Rust`) and left on disk as dead source while `#[no_mangle]`
Rust takes the symbol. So `find -name '*.c' | wc -l` answers a question nobody asked.
Reading CMakeLists for commented-out entries is not much better: it parses a comment,
misses `if(ARCH)` guards, and cannot see a file dropped by a variable.

The ninja graph knows exactly which translation units the build compiles, after every
conditional. A source is LIVE if some arch's graph has a compile edge for it, and
OUT-OF-BUILD otherwise. Both halves are reported, because their ratio is the coverage
number and neither alone is meaningful.

THREE ARCHES, UNIONED
---------------------
i386, amd64 and arm64 do not compile the same set: `hal/halx86/` is i386+amd64, the
arm64 board support is arm64-only, and a file live on ONE arch is live C. Taking the
union is therefore not a convenience, it is the definition. It also makes the tool
refuse to run on a partial set of build dirs rather than quietly report a smaller tree
as more translated than it is.

WHAT "OUT OF BUILD" DOES AND DOES NOT MEAN
------------------------------------------
It means the link does not contain it. It does NOT mean "translated to Rust" — upstream
ReactOS carries `.c` this port never built (an unreferenced sample, a driver for a
device we do not enable), and those land in the same bucket as a cohab flip. So the
out-of-build figure alone is an UPPER BOUND on progress, and reporting it as coverage
would be the tree's favourite error: a predicate that is TRUE carrying a label that is
FALSE.

The `Rust twin` column is what narrows it. A file counts as twinned when its module
directory (or a parent, up to three levels) has a crate of that name in
`sdk/lib/rust/`, which is the naming the cohab flip actually uses — `cdfs` -> `cdfs-rust`,
`basesrv` -> `basesrv-cargo`. That is still a statement about the MODULE and not about
the individual file, so it too is an upper bound; it is simply a much tighter one, and
the gap between the two columns is exactly the C that left the build with nothing in
Rust standing behind it. At the time of writing that gap is 8% of the out-of-build core,
and it is mostly code this port never compiled: `drivers/bus/acpi_new/uacpi`, the
unbuilt DirectX cluster, and freeldr's i386/pc arch on a UEFI-only boot.

For a claim about a specific export, ask `export_census.py`; for whether a flip actually
took at the symbol level, ask `verify_cohab_coverage.py`, which reads object files.

WHAT COUNTS AS CORE
-------------------
Owner's definition (2026-09-25): core is the base Windows XP/Vista OS -- the kernel,
HAL, drivers, the in-box DLLs, services, shell and control panel, everything a user
expects out of the box. PROVENANCE DOES NOT DECIDE IT. Wine-derived code is ours: once
imported it is translated like everything else, so ole32 and msvcrt are core exactly as
kernel32 is. An earlier cut of this tool excluded Wine as "a separate tier", which hid
876,000 live lines of core DLLs from the core number and made the OS look 60% done when
the honest answer was about half that.

What does come out, each with a source of truth that is not this file:

  * VENDORED trees (`vendor.yaml` `trees:`) -- third-party libraries we keep in C and
    bump as versioned dependencies (ACPICA, FreeType, Mesa, ...). Not ours to translate.
  * APPLICATIONS -- calc, mspaint, regedit. Ours and shipped, reported in their own
    table with the same columns, never folded into the core headline.
  * TESTS and HOST TOOLS -- never ship.

Provenance is still REPORTED (the `Wine` column), because it predicts cost: a Wine
module has an upstream to diff against and a winetest suite to grade it.

Everything else is classified by path, and `--check` FAILS on a compiled directory that
matches no rule. That is deliberate: a new top-level area must be classified before it
can pass, the same ratchet `vendored.py --check` applies to a new vendor drop. A silent
default is how 53,679 lines of Mesa sat in the core bucket unnoticed: `--suspects` could
not see it, because that scan matches CORPORATE copyright holders and Mesa's headers
name Brian Paul, and `dll/opengl` is not one of vendor.yaml's three containers either.
This ratchet asks a different question -- what does the LINK contain -- and caught what
a copyright grep structurally cannot. It only sees UNCLASSIFIED top-level areas,
though: third-party code nested inside a classified directory passes silently.

LINES OF C ARE NOT LINES OF RUST
--------------------------------
Every column above counts C. The Rust that replaced it is roughly twice as long
(explicit casts, unsafe blocks, bindgen-shaped FFI, no macros doing the work), so
"3M lines of Rust" and "1.4M lines of C covered" describe the same progress. The
`--rust` inventory reports the Rust side separately and splits out generated bindings
and host test harnesses, which are neither translation nor shipped code.
"""

from __future__ import annotations

import argparse
import collections
import json
import os
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import vendored

REPO = Path(__file__).resolve().parents[1]
ARCHES = ("i386", "amd64", "arm64")
SOURCE_SUFFIXES = (".c", ".cpp", ".cc", ".cxx")


class PortAuditError(RuntimeError):
    """A measurement that cannot be trusted is not reported as a smaller number."""


# ── surface classification ───────────────────────────────────────────────────────────
#
# Longest prefix wins, so a specific rule can carve an exception out of a general one.
# `core` is the OS itself; `app` and `excluded` are reported separately and never folded
# into the headline. The note is the REASON, and it is what a reader disputes.

CORE = "core"
APP = "app"
EXCLUDED = "excluded"

SURFACES: list[tuple[str, str, str, str]] = [
    # prefix,                 bucket,   surface,                 note
    ("ntoskrnl", CORE, "Kernel (ntoskrnl.exe)", "the kernel proper"),
    ("hal", CORE, "HAL", "hardware abstraction layer"),
    ("boot", CORE, "Boot (freeldr)", "boot loader + its RTL"),
    ("subsystems", CORE, "CSR subsystem", "the subsystem server spine"),
    ("win32ss/user", CORE, "win32k: USER", "window manager + user32"),
    ("win32ss/gdi", CORE, "win32k: GDI", "graphics engine + gdi32"),
    ("win32ss/win32u", CORE, "win32k: syscall stubs", ""),
    ("win32ss/reactx", CORE, "win32k: ReactX", "DirectDraw kernel side"),
    ("win32ss/drivers", CORE, "win32k: display drivers", "videoprt/framebuf"),
    ("win32ss/printing", CORE, "Print spooler", "localspl/winspool"),
    ("drivers", CORE, "Driver fleet", "every kernel driver"),
    ("dll/ntdll", CORE, "ntdll.dll", "native API"),
    ("dll/win32", CORE, "Win32 DLLs", "kernel32/advapi32/shell32/..."),
    ("dll/directx", CORE, "DirectX", "ddraw/d3d8/d3d9/dsound"),
    ("dll/keyboard", CORE, "Keyboard layouts", "per-locale scancode tables"),
    ("dll/nls", CORE, "NLS codepages", ""),
    ("dll/appcompat", CORE, "AppCompat", "shim engine"),
    ("dll/shellext", CORE, "Shell extensions", "in-box property sheets + namespace ext"),
    ("dll/cpl", CORE, "Control Panel applets", "in-box .cpl"),
    ("base/system", CORE, "System spine", "smss/csrss/winlogon/services/lsass"),
    ("base/services", CORE, "Services", "svchost + the in-box service set"),
    ("base/setup", CORE, "Setup", "usetup/syssetup/first-boot"),
    ("base/shell", CORE, "Shell", "explorer/cmd/rshell"),
    ("base/ctf", CORE, "Text services (CTF/IME)", ""),
    ("sdk/lib", CORE, "Support libraries", "RTL, CRT, cmlib, fslib, driver libs"),
    ("sdk/include", CORE, "SDK inline TUs", "guid/version TUs compiled into modules"),
    ("sdk/tools", EXCLUDED, "Host tools", "build-time only, never ships"),
    ("base/applications", APP, "In-box applications", "calc/mspaint/regedit/notepad/cmd tools"),
    ("modules/rostests", EXCLUDED, "Tests", "apitests + winetests"),
    # rosapps is not in the build at all, so a coverage ratio for it is noise (0 live
    # lines make any stray twin read as 100%). Listed so it is visibly accounted for.
    ("modules/rosapps", EXCLUDED, "rosapps", "optional extras, not configured into the build"),
    # Mesa and its GLU are vendored and fall out above; what is left under dll/opengl is
    # opengl32 itself -- the WGL/ICD layer ReactOS wrote -- which is ours and core.
    ("dll/opengl", CORE, "OpenGL (opengl32)", "WGL + ICD dispatch; Mesa is vendored"),
]


@dataclass
class Row:
    surface: str
    bucket: str
    note: str
    live_tus: int = 0
    live_lines: int = 0
    dead_tus: int = 0
    dead_lines: int = 0
    twin_lines: int = 0
    wine_live_lines: int = 0
    wine_twin_lines: int = 0
    live_dirs: collections.Counter = field(default_factory=collections.Counter)

    @property
    def total_lines(self) -> int:
        return self.live_lines + self.dead_lines

    @property
    def out_of_build_pct(self) -> float:
        return 100.0 * self.dead_lines / self.total_lines if self.total_lines else 0.0

    @property
    def coverage_pct(self) -> float:
        """Out-of-build C that a Rust crate stands behind, over all C in the surface."""
        denom = self.live_lines + self.twin_lines
        return 100.0 * self.twin_lines / denom if denom else 0.0


WINESYNC = Path("media") / "doc" / "WINESYNC.txt"


def winesync_paths(repo: Path = REPO) -> tuple[str, ...]:
    """Every tree path upstream ReactOS records as shared with Wine.

    `media/doc/WINESYNC.txt` is ReactOS's own ledger of Wine imports, maintained by the
    people who did the syncing: ~200 modules and files, each with the Wine version it
    tracks. vendor.yaml's Wine entry lists 24 of them, so reading only the registry
    labelled gdiplus, shlwapi, riched20, dbghelp and comdlg32 -- 150K live lines --
    "ReactOS-native". A line counts when its first token is a path that exists here
    (`dll/win32/msi   # Synced to Wine-9.8`, and the indented per-file lines under
    `kernel32 -`); prose and missing paths are ignored. "Forked" entries count too:
    shell32 forked from Wine in 2007 is still Wine-derived, however rewritten.
    """
    path = repo / WINESYNC
    if not path.is_file():
        raise PortAuditError(f"{WINESYNC} missing -- it is upstream's Wine-import ledger")
    out: list[str] = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        if "#" not in line:
            continue
        token = line.split("#", 1)[0].strip().rstrip("/")
        if "/" in token and " " not in token and (repo / token).exists():
            out.append(token)
    return tuple(out)


def provenance(rel: str, registry: vendored.Registry, winesync: tuple[str, ...] = ()) -> str:
    """ "vendored", "wine" or "native" -- reported, and it never moves a file OUT of core."""
    if registry.is_vendored(rel):
        return "vendored"
    # A `wine/` DIRECTORY is provenance the registry cannot carry per-file: kernel32 and
    # gdi32 are native, but `dll/win32/kernel32/wine/actctx.c` and the gdi32 metafile
    # cluster are synced from Wine (sdk/tools/winesync/*.cfg names the exact files).
    if "/wine/" in f"/{rel}":
        return "wine"
    for our in registry.ours:
        # `.paths`, never `.path`: one `ours:` entry carries ONE decision that may span
        # many directories, and `.path` is only the first of them. The Wine entry alone
        # covers 24 shipped modules; reading the singular mislabelled 23 of them.
        if any(rel == p or rel.startswith(p + "/") for p in our.paths):
            return "wine" if "Wine" in (our.origin or "") else "native"
    if any(rel == p or rel.startswith(p + "/") for p in winesync):
        return "wine"
    return "native"


def classify(rel: str, registry: vendored.Registry) -> tuple[str, str, str]:
    """(bucket, surface, note) for a repo-relative source path.

    Vendored is the ONLY provenance that removes a file from its surface. Wine-derived
    code is classified by where it lives, like everything else we own.
    """
    if registry.is_vendored(rel):
        return EXCLUDED, "Vendored (vendor.yaml)", "third-party, kept in C as a versioned dep"
    best: tuple[str, str, str] | None = None
    best_len = -1
    for prefix, bucket, surface, note in SURFACES:
        if (rel == prefix or rel.startswith(prefix + "/")) and len(prefix) > best_len:
            best, best_len = (bucket, surface, note), len(prefix)
    if best is None:
        raise PortAuditError(
            f"unclassified compiled source `{rel}` — add a rule to SURFACES in "
            f"tools/port_audit.py saying which bucket it belongs to and why"
        )
    return best


# ── the build graph ──────────────────────────────────────────────────────────────────


def _rebase(path: Path, repo: Path = REPO) -> Path:
    """Map a canonical-path source back onto THIS worktree, textually.

    Builds run at `/tmp/vibeos` (tools/canon-run.sh bind-mounts the worktree there so a
    build dir carries no worktree-specific paths and the object cache hits across lanes),
    so every source in the graph is spelled under that mount point. The substitution is
    deliberately a string operation and never touches `/tmp`: the mount belongs to
    whichever lane is building RIGHT NOW, so resolving through it would read a sibling
    worktree's tree at a different commit — the exact failure that moved
    verify_signatures' mismatch count 987 -> 9 with no source change.
    """
    canon = "/tmp/vibeos"
    text = str(path)
    if text == canon or text.startswith(canon + "/"):
        return repo / text[len(canon) :].lstrip("/")
    return path


def _newest_cmake_mtime(repo: Path) -> float:
    newest = 0.0
    for p in repo.rglob("CMakeLists.txt"):
        parts = p.parts
        if any(x.startswith("build-") or x in (".git", "target") for x in parts):
            continue
        newest = max(newest, p.stat().st_mtime)
    return newest


def compiled_sources(repo: Path, arches=ARCHES) -> dict[str, set[str]]:
    """Repo-relative sources with a compile edge, per arch.

    HARD FAILS on a missing or stale graph rather than reporting the smaller tree that a
    partial answer would produce. This reads only THIS worktree's build dirs: a sibling's
    `build-*` is another commit's generated state, and a verdict that depends on which
    siblings happen to exist is not a verdict (AGENTS.md, fleet isolation).
    """
    newest_cmake = _newest_cmake_mtime(repo)
    out: dict[str, set[str]] = {}
    for arch in arches:
        bd = repo / f"build-{arch}"
        ninja = bd / "build.ninja"
        if not ninja.is_file():
            raise PortAuditError(
                f"no build graph for {arch} ({ninja} missing)\n"
                f"  fix: ./vibeos-cli build --arch {arch}"
            )
        if ninja.stat().st_mtime < newest_cmake:
            raise PortAuditError(
                f"the {arch} build graph predates a CMakeLists edit, so it would report "
                f"a flip that has not happened or miss one that has\n"
                f"  fix: (cd build-{arch} && bash ../tools/canon-run.sh ninja build.ninja)"
            )
        proc = subprocess.run(["ninja", "-t", "compdb"], cwd=bd, capture_output=True, text=True)
        if proc.returncode != 0:
            raise PortAuditError(f"ninja -t compdb failed for {arch}: {proc.stderr[:400]}")
        srcs: set[str] = set()
        for entry in json.loads(proc.stdout):
            src = entry.get("file", "")
            if not entry.get("command") or not src.endswith(SOURCE_SUFFIXES):
                continue
            path = _rebase(Path(entry["directory"], src))
            try:
                rel = path.relative_to(repo)
            except ValueError:
                continue  # generated into the build dir, or outside the tree
            if rel.parts and rel.parts[0].startswith("build-"):
                continue
            srcs.add(str(rel))
        out[arch] = srcs
    return out


def _line_count(path: Path) -> int:
    try:
        with path.open("rb") as fh:
            return sum(1 for _ in fh)
    except OSError:
        return 0


def rust_crates(repo: Path = REPO) -> set[str]:
    root = repo / "sdk" / "lib" / "rust"
    return {p.name for p in root.iterdir() if p.is_dir()} if root.is_dir() else set()


def _twin_of(rel: str, crates: set[str]) -> str | None:
    """The Rust crate that owns this file's module, by the cohab naming convention.

    Walks up three levels because a module's `.c` are often one or two directories below
    the name the crate takes (`drivers/filesystems/cdfs/create.c` -> `cdfs-rust`).
    """
    probe = Path(rel).parent
    for _ in range(3):
        name = probe.name
        if not name:
            break
        for cand in (f"{name}-rust", f"{name}-cargo", f"{name}_rust", name):
            if cand in crates:
                return cand
        probe = probe.parent
    return None


def survey(repo: Path = REPO, arches=ARCHES) -> tuple[dict[str, Row], dict[str, set[str]]]:
    per_arch = compiled_sources(repo, arches)
    live = set().union(*per_arch.values())
    registry = vendored.load(repo)
    crates = rust_crates(repo)
    winesync = winesync_paths(repo)
    rows: dict[str, Row] = {}

    def row_for(rel: str) -> Row:
        bucket, surface, note = classify(rel, registry)
        return rows.setdefault(surface, Row(surface=surface, bucket=bucket, note=note))

    for rel in sorted(live):
        row = row_for(rel)
        n = _line_count(repo / rel)
        row.live_tus += 1
        row.live_lines += n
        if provenance(rel, registry, winesync) == "wine":
            row.wine_live_lines += n
        row.live_dirs[str(Path(rel).parent)] += 1

    skip = {".git", "target", "obj-i386", "obj-amd64", "obj-arm64"}
    for path in repo.rglob("*"):
        if path.suffix.lower() not in SOURCE_SUFFIXES or not path.is_file():
            continue
        parts = path.relative_to(repo).parts
        if any(p in skip or p.startswith("build-") or p.startswith(".") for p in parts):
            continue
        rel = str(path.relative_to(repo))
        if rel in live:
            continue
        try:
            row = row_for(rel)
        except PortAuditError:
            continue  # on-disk-only areas need no bucket; only the LINK ratchets
        lines = _line_count(path)
        row.dead_tus += 1
        row.dead_lines += lines
        if _twin_of(rel, crates):
            row.twin_lines += lines
            if provenance(rel, registry, winesync) == "wine":
                row.wine_twin_lines += lines
    return rows, per_arch


# ── reporting ────────────────────────────────────────────────────────────────────────


def _pct(twin: int, live: int) -> str:
    return f"{100.0 * twin / (twin + live):.0f}%" if twin + live else "–"


def _ledger(rows: list[Row], total_label: str) -> list[str]:
    out = [
        "| Surface | Live C (TUs) | Live C (lines) | of which Wine | Out of build "
        "| Rust twin | Covered |",
        "|---|---:|---:|---:|---:|---:|---:|",
    ]
    for r in rows:
        out.append(
            f"| {r.surface} | {r.live_tus:,} | {r.live_lines:,} | {r.wine_live_lines:,} | "
            f"{r.dead_lines:,} | {r.twin_lines:,} | {_pct(r.twin_lines, r.live_lines)} |"
        )
    tl = sum(r.live_lines for r in rows)
    tw = sum(r.twin_lines for r in rows)
    out.append(
        f"| **{total_label}** | **{sum(r.live_tus for r in rows):,}** | **{tl:,}** | "
        f"**{sum(r.wine_live_lines for r in rows):,}** | "
        f"**{sum(r.dead_lines for r in rows):,}** | **{tw:,}** | **{_pct(tw, tl)}** |"
    )
    return out


def render(rows: dict[str, Row], per_arch: dict[str, set[str]]) -> str:
    def by_live(bucket: str) -> list[Row]:
        return sorted((r for r in rows.values() if r.bucket == bucket), key=lambda r: -r.live_lines)

    core, apps = by_live(CORE), by_live(APP)
    out = _ledger(core, "CORE TOTAL")
    wl = sum(r.wine_live_lines for r in core)
    ww = sum(r.wine_twin_lines for r in core)
    nl = sum(r.live_lines for r in core) - wl
    nw = sum(r.twin_lines for r in core) - ww
    out += [
        "",
        "| Core by provenance | Live C (lines) | Rust twin | Covered |",
        "|---|---:|---:|---:|",
        f"| ReactOS-native | {nl:,} | {nw:,} | {_pct(nw, nl)} |",
        f"| Wine-derived | {wl:,} | {ww:,} | {_pct(ww, wl)} |",
        "",
    ]
    out += _ledger(apps, "APPS TOTAL")
    out += [
        "",
        "| Not ours to translate | Live C (TUs) | Live C (lines) | Why |",
        "|---|---:|---:|---|",
    ]
    for r in by_live(EXCLUDED):
        out.append(f"| {r.surface} | {r.live_tus:,} | {r.live_lines:,} | {r.note} |")
    out.append("")
    counts = ", ".join(f"{a} {len(per_arch[a]):,}" for a in per_arch)
    out.append(f"*Compile edges per arch: {counts}; the tables are their union.*")
    return "\n".join(out)


# ── the Rust side ────────────────────────────────────────────────────────────────────

_HARNESS_SUFFIXES = ("-difftest", "-test", "-tests", "-harness", "-fuzz")


def rust_inventory(repo: Path = REPO) -> dict[str, int]:
    """Lines of `.rs` under sdk/lib/rust, split so the headline is not inflated.

    `generated` is checked-in bindgen output (a `bindgen_*.rs` name, or bindgen's own
    "automatically generated" banner): millions of lines nobody wrote or paid for.
    `harness` is host-side difftest/test crates: real work, but they grade the port
    rather than being it. `translation` is everything else -- hand-written or paid
    translation output that ships.
    """
    root = repo / "sdk" / "lib" / "rust"
    inv = {"translation": 0, "generated": 0, "harness": 0}
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d != "target" and not d.startswith(".")]
        crate = Path(dirpath).relative_to(root).parts[:1]
        harness = bool(crate) and crate[0].endswith(_HARNESS_SUFFIXES)
        for name in filenames:
            if not name.endswith(".rs"):
                continue
            path = Path(dirpath, name)
            with path.open("rb") as fh:
                head = fh.read(400)
                n = head.count(b"\n") + sum(
                    chunk.count(b"\n") for chunk in iter(lambda: fh.read(1 << 20), b"")
                )
            if "bindgen" in name or b"automatically generated" in head:
                inv["generated"] += n
            elif harness:
                inv["harness"] += n
            else:
                inv["translation"] += n
    return inv


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(
        description="Tree-wide translation status: how much C the OS still COMPILES."
    )
    ap.add_argument("--json", action="store_true", help="machine-readable rows")
    ap.add_argument(
        "--check",
        action="store_true",
        help="exit non-zero if any compiled source matches no SURFACES rule",
    )
    ap.add_argument("--detail", metavar="SURFACE", help="live directories for one surface")
    ap.add_argument("--rust", action="store_true", help="lines of Rust, split by kind")
    args = ap.parse_args(argv)
    if args.rust:
        inv = rust_inventory()
        for k, v in inv.items():
            print(f"{k:12} {v:>11,}")
        print(f"{'total':12} {sum(inv.values()):>11,}")
        return 0
    try:
        rows, per_arch = survey()
    except PortAuditError as exc:
        sys.stderr.write(f"port_audit: {exc}\n")
        return 2
    if args.check:
        print(f"port_audit: every compiled source is classified ({len(rows)} surfaces)")
        return 0
    if args.detail:
        row = rows.get(args.detail)
        if row is None:
            sys.stderr.write(f"no surface named {args.detail!r}; have: {sorted(rows)}\n")
            return 2
        for d, n in row.live_dirs.most_common():
            print(f"{n:5d}  {d}")
        return 0
    if args.json:
        print(
            json.dumps(
                {
                    r.surface: {
                        "bucket": r.bucket,
                        "live_tus": r.live_tus,
                        "live_lines": r.live_lines,
                        "dead_tus": r.dead_tus,
                        "dead_lines": r.dead_lines,
                        "twin_lines": r.twin_lines,
                        "wine_live_lines": r.wine_live_lines,
                    }
                    for r in rows.values()
                },
                indent=2,
                sort_keys=True,
            )
        )
        return 0
    print(render(rows, per_arch))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
