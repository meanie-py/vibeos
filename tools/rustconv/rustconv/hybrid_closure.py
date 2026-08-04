"""Measure the retained closure of a hybrid C/C++ translation unit, from the link.

A "hybrid" TU is one whose functions are already translated but whose `.cpp` is still
compiled, because *something else in the target still calls into it*. The question that
decides whether it can leave CMake is not how many lines it has -- it is which of its
defined symbols anyone else still needs. CFSFolder.cpp read as 2,283 lines of work and
was 21 symbols.

The measurement is a set intersection over the real link inputs:

    retained = defined(this TU's .obj)  n  undefined(every other .obj + the Rust archive)

Both halves matter. Leaving the Rust archive out of the undefined side understates the
closure, because a `#[no_mangle]` Rust body that calls back into a C++ helper is exactly
the edge that keeps the `.cpp` alive.

Symbols are classified by how they can be re-published from Rust, which is what actually
drives the integration work:

  extern_c       -- already has a C spelling; a `#[no_mangle]` Rust fn replaces it directly.
  cxx_member     -- a C++ class member. It has ONE name and one class scope, so there is no
                    `EXTERN_C` spelling available; the route is an inline forwarder in the
                    class body calling an already-published C bridge.
  cxx_free       -- a mangled free function; give it `EXTERN_C` in the header and it becomes
                    an `extern_c` case.
  data           -- a data symbol, not a function. Usually a vtable or a string table.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

# Toolchain is content-addressed and commit-independent, so it may be shared across
# worktrees; generated artifacts (the .obj tree) must not be -- see AGENTS.md worktree law.
LLVM_NM_REL = Path(".build-tools") / "llvm-mingw" / "bin" / "llvm-nm"

# nm type letters that mean "this object defines it". Everything else is undefined,
# common, or a debug/absolute entry we do not care about.
DEFINED_TYPES = set("TtDdBbRrGgSsWwVi")
UNDEFINED_TYPES = set("Uu")


@dataclass(frozen=True)
class RetainedSymbol:
    name: str
    kind: str  # extern_c | cxx_member | cxx_free | data
    demangled: str
    needed_by: tuple[str, ...]

    def to_dict(self) -> dict[str, object]:
        return {
            "name": self.name,
            "kind": self.kind,
            "demangled": self.demangled,
            "needed_by": list(self.needed_by),
        }


@dataclass(frozen=True)
class ClosureReport:
    tu: str
    obj: str
    arch: str
    defined: int
    retained: tuple[RetainedSymbol, ...]

    def to_dict(self) -> dict[str, object]:
        return {
            "tu": self.tu,
            "obj": self.obj,
            "arch": self.arch,
            "defined": self.defined,
            "retained": [s.to_dict() for s in self.retained],
            "retained_count": len(self.retained),
        }


def _nm(nm_bin: Path, path: Path, extra: list[str]) -> str:
    cmd = [str(nm_bin), *extra, str(path)]
    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
    if proc.returncode != 0:
        raise RuntimeError(f"llvm-nm failed on {path}: {proc.stderr[:400]}")
    return proc.stdout


_NM_LINE = re.compile(r"^(?:[0-9a-fA-F]+)?\s*([A-Za-z?])\s+(\S+)\s*$")


def _parse(out: str, wanted: set[str]) -> set[str]:
    found: set[str] = set()
    for line in out.splitlines():
        m = _NM_LINE.match(line.strip())
        if not m:
            continue
        typ, name = m.group(1), m.group(2)
        if typ in wanted:
            found.add(name)
    return found


def _demangle(nm_bin: Path, names: list[str]) -> dict[str, str]:
    """Demangle via llvm-cxxfilt sitting beside llvm-nm."""
    if not names:
        return {}
    filt = nm_bin.parent / "llvm-cxxfilt"
    if not filt.exists():
        return {n: n for n in names}
    proc = subprocess.run(
        [str(filt)], input="\n".join(names), capture_output=True, text=True, timeout=60
    )
    if proc.returncode != 0:
        return {n: n for n in names}
    out = proc.stdout.splitlines()
    if len(out) != len(names):
        return {n: n for n in names}
    return dict(zip(names, out, strict=True))


def _classify(name: str, demangled: str, is_data: bool) -> str:
    if is_data:
        return "data"
    if not (name.startswith("_Z") or name.startswith("__Z")):
        return "extern_c"
    # A member has a qualified name; `Foo::bar(...)` -- a free function does not.
    head = demangled.split("(", 1)[0]
    return "cxx_member" if "::" in head else "cxx_free"


def _link_objects(link_root: Path) -> list[Path]:
    """Every .obj that participates in the final link under `link_root`.

    A DLL's link unit is wider than one CMake target: shell32 pulls in the shellmenu,
    shelldesktop, and shellrecyclebin static libs, each with its own `*.dir`. Scoping the
    consumers to a single `.dir` understates the closure, which reads as "this TU is free
    to leave" right up until the link fails.
    """
    return sorted(link_root.rglob("*.obj"))


def _find_obj(link_root: Path, tu: Path) -> Path | None:
    want = f"{tu.name}.obj"
    for cand in link_root.rglob(want):
        return cand
    return None


def measure(
    repo: Path,
    tu: str,
    build_dir: Path,
    link_root: Path,
    rust_archive: Path | None,
    arch: str,
) -> ClosureReport:
    nm_bin = repo / LLVM_NM_REL
    if not nm_bin.exists():
        raise FileNotFoundError(
            f"llvm-nm not found at {nm_bin}\nfix: ./vibeos-cli build --arch {arch}"
        )
    if not link_root.is_dir():
        raise FileNotFoundError(
            f"object tree not found: {link_root}\nfix: ./vibeos-cli build --arch {arch}"
        )

    tu_path = Path(tu)
    obj = _find_obj(link_root, tu_path)
    if obj is None:
        raise FileNotFoundError(
            f"no object for {tu} under {link_root} -- is it still in CMake?\n"
            f"fix: ./vibeos-cli build --arch {arch}"
        )

    defined_out = _nm(nm_bin, obj, ["--defined-only"])
    defined = _parse(defined_out, DEFINED_TYPES)
    data_syms = {
        m.group(2)
        for m in (_NM_LINE.match(ln.strip()) for ln in defined_out.splitlines())
        if m and m.group(1) in set("DdBbRrGgSs")
    }

    # Undefined side: every other object in the link, plus the Rust archive. A Rust
    # body calling back into a C++ helper is exactly the edge that keeps the .cpp alive,
    # so omitting the archive would understate the closure.
    others = [p for p in _link_objects(link_root) if p != obj]
    needed_by: dict[str, list[str]] = {}
    for other in others:
        try:
            und = _parse(_nm(nm_bin, other, ["--undefined-only"]), UNDEFINED_TYPES)
        except RuntimeError:
            continue
        for sym in und & defined:
            needed_by.setdefault(sym, []).append(other.name)

    if rust_archive and rust_archive.exists():
        und = _parse(_nm(nm_bin, rust_archive, ["--undefined-only"]), UNDEFINED_TYPES)
        for sym in und & defined:
            needed_by.setdefault(sym, []).append(rust_archive.name)

    names = sorted(needed_by)
    dem = _demangle(nm_bin, names)
    retained = tuple(
        RetainedSymbol(
            name=n,
            kind=_classify(n, dem.get(n, n), n in data_syms),
            demangled=dem.get(n, n),
            needed_by=tuple(sorted(set(needed_by[n]))),
        )
        for n in names
    )
    return ClosureReport(tu=tu, obj=str(obj), arch=arch, defined=len(defined), retained=retained)


def print_report(report: ClosureReport, as_json: bool = False) -> None:
    if as_json:
        print(json.dumps(report.to_dict(), indent=2))
        return
    print(f"{report.tu}  [{report.arch}]")
    print(f"  defines {report.defined} symbols; {len(report.retained)} are still needed elsewhere")
    if not report.retained:
        print("  RETAINED CLOSURE IS EMPTY -- this TU can leave CMake with no bridges.")
        return
    by_kind: dict[str, list[RetainedSymbol]] = {}
    for s in report.retained:
        by_kind.setdefault(s.kind, []).append(s)
    order = ("extern_c", "cxx_free", "cxx_member", "data")
    for kind in order:
        group = by_kind.get(kind)
        if not group:
            continue
        print(f"\n  {kind}  ({len(group)})")
        for s in group:
            print(f"    {s.demangled}")
            print(f"        {s.name}   <- {', '.join(s.needed_by)}")


def build_arg_parser(p: argparse.ArgumentParser) -> argparse.ArgumentParser:
    p.add_argument("tu", help="repo-relative path to the .cpp/.c translation unit")
    p.add_argument("--arch", required=True, choices=("i386", "amd64", "arm64"))
    p.add_argument(
        "--target",
        default="shell32",
        help="CMake target owning the TU (default: shell32)",
    )
    p.add_argument(
        "--target-dir",
        default="dll/win32/shell32",
        help="repo-relative dir whose object tree IS the link unit (default: dll/win32/shell32)",
    )
    p.add_argument("--rust-archive", help="override path to the target's Rust static archive")
    p.add_argument("--json", action="store_true")
    return p


def resolve_paths(repo: Path, args: argparse.Namespace) -> tuple[Path, Path, Path]:
    build_dir = repo / f"build-{args.arch}"
    link_root = build_dir / args.target_dir
    archive = (
        Path(args.rust_archive)
        if args.rust_archive
        else build_dir / args.target_dir / f"lib{args.target}_rust.a"
    )
    return build_dir, link_root, archive


def main(argv: list[str] | None = None) -> int:
    ap = build_arg_parser(argparse.ArgumentParser(prog="rustconv hybrid-closure"))
    args = ap.parse_args(argv)
    repo = Path(__file__).resolve().parents[3]
    build_dir, link_root, archive = resolve_paths(repo, args)
    try:
        report = measure(repo, args.tu, build_dir, link_root, archive, args.arch)
    except (FileNotFoundError, RuntimeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    print_report(report, as_json=args.json)
    return 0


if __name__ == "__main__":
    sys.exit(main())
