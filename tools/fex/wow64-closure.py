#!/usr/bin/env python3
"""Compatibility facade for the native SysWOW64 dependency closure."""

import argparse
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parent.parent.parent
if str(_REPO) not in sys.path:
    sys.path.insert(0, str(_REPO))
from tools.build_infra_compat import NativeError, call  # noqa: E402

PE_SUFFIXES = (".dll", ".drv", ".ocx", ".acm", ".cpl")
PRESTAGED = {"ntdll.dll", "kernel32.dll", "user32.dll", "gdi32.dll", "ntdll_vista.dll"}
DYNAMIC_LOADERS = {
    "ws2_32.dll": ("media/inf/nettcpip.inf", ("LibraryPath", "PackedCatalogItem", "AutodialDLL")),
    "msafd.dll": ("media/inf/nettcpip.inf", ("HelperDllName",)),
    "winmm.dll": ("boot/bootdata/hivesft.inf", ("Drivers32",)),
}


def _call(op, kwargs, root=None):
    return call("payload_closure_" + op, kwargs, root=Path(root or _REPO))


def declared_modules(project_dir, inf_rel, kinds):
    try:
        return _call("declared_modules", {"inf": inf_rel, "kinds": kinds}, project_dir)
    except NativeError as error:
        raise SystemExit(str(error)) from error


def find_objdump(project_dir):
    try:
        return _call("find_objdump", {}, project_dir)
    except NativeError as error:
        raise SystemExit(str(error)) from error


def stripped_twins(build_dir):
    return {
        k: Path(v)
        for k, v in _call("stripped_twins", {"build_dir": Path(build_dir).absolute()}).items()
    }


def index_build(build_dir, no_strip=False):
    return {
        k: Path(v)
        for k, v in _call(
            "index_build", {"build_dir": Path(build_dir).absolute(), "no_strip": no_strip}
        ).items()
    }


def string_referenced(pe, index):
    return set(_call("string_referenced", {"pe": Path(pe).absolute(), "index": index}))


def delay_imports_of(pe):
    return _call("delay_imports_of", {"pe": Path(pe).absolute()})


def forward_targets_of(pe):
    return _call("forward_targets_of", {"pe": Path(pe).absolute()})


def imports_of(objdump, pe):
    return _call("imports_of", {"pe": Path(pe).absolute()})


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("roots", nargs="+")
    ap.add_argument("--build-dir", default="build-i386")
    ap.add_argument("--include-prestaged", action="store_true")
    ap.add_argument("--no-strip", action="store_true")
    ap.add_argument("--library-literals", action="store_true")
    ap.add_argument("--emit-roots", action="store_true")
    args = ap.parse_args()
    try:
        result = _call("closure", vars(args))
    except NativeError as error:
        print(error, file=sys.stderr)
        return 1
    paths = result["paths"]
    for name, path in sorted(paths.items()):
        print(f"{name}\t{path}")
    total = sum(Path(path).stat().st_size for path in paths.values())
    print(f"wow64-closure: {len(paths)} module(s), {total / 1e6:.1f} MB", file=sys.stderr)
    for name, decl in sorted(result["dynamic"].items()):
        print(f"wow64-closure: dynamic provider {name} (declared in {decl})", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
