#!/usr/bin/env python3
"""Emit a crate's `own_exports_decls.rs`: header-exact declarations of functions the crate
DEFINES itself (`#[no_mangle]`) but still CALLS through an extern declaration elsewhere.

WHY THIS EXISTS. gen_driver_types.sh's BLOCKLIST_FUNCS_FROM_CRATE drops every function the
crate defines from its island, because the defining module must not also receive a
declaration of it. Callers in OTHER modules still need one. Those used to be hand-copied into
the prelude after each regeneration (552 of them the first time), so a regen that crossed a
translation milestone — shell32 went to 833 definitions — broke ~76 call sites with no tool
to put them back. This reads the declarations off a full-surface island generated with
KEEP_CRATE_FUNCS=1 into a scratch OUT_CRATE_DIR, so signatures come from bindgen and the
current headers, never from a hand.

    KEEP_CRATE_FUNCS=1 OUT_CRATE_DIR=<scratch> bash tools/rustconv/gen_driver_types.sh <drv>
    python3 tools/rustconv/own_exports_decls.py --full <scratch> --crate-src <src> [--add N ...]

The name set is the file's own existing set plus any `--add`, so a regen preserves it. Every
name must be declared identically on all three arches (the output is arch-independent); a
name that differs or is absent HARD-FAILS rather than being guessed at.
"""

import argparse
import os
import re
import sys
from pathlib import Path

ARCHES = ("i386", "amd64", "arm64")
OUT = "own_exports_decls.rs"


def crate_definitions(crate_src):
    """name -> (abi, arity) for every `#[no_mangle]` fn the crate defines."""
    defs = {}
    for dp, _, fs in os.walk(crate_src):
        for f in fs:
            if not f.endswith(".rs"):
                continue
            t = Path(os.path.join(dp, f)).read_text()
            for m in re.finditer(
                r"#\[no_mangle\]\s*(?:#\[[^\]]*\]\s*)*"
                r'pub (?:unsafe )?extern "(\w+)" fn (\w+)\s*\((.*?)\)\s*(?:->|\{)',
                t,
                re.S,
            ):
                args = [x for x in re.split(r",(?![^<(]*[>)])", m.group(3)) if ":" in x]
                defs[m.group(2)] = (m.group(1), len(args), os.path.join(dp, f))
    return defs


def arity(d):
    inner = re.search(r"\((.*)\)", d, re.S).group(1)
    return len([x for x in re.split(r",(?![^<(]*[>)])", inner) if ":" in x])


def block_abi(src, name):
    """The calling convention of a declaration is its ENCLOSING extern block. Only i386 can
    tell cdecl from stdcall (64-bit targets have one convention, so their blocks all read "C"),
    and normalize_island_abi rewrites fn-pointer TYPES to "system" but leaves the block — so the
    i386 block is the one place the header's real convention survives."""
    i = src.find("pub fn " + name + "(")
    blk = src.rfind('extern "', 0, i)
    abi = re.match(r'extern "(\w+)"', src[blk : blk + 20]).group(1)
    return "system" if abi in ("system", "stdcall") else abi


def decl(src, name):
    m = re.search(
        r"(?:#\[[^\]]*\]\s*)*pub fn " + re.escape(name) + r"\s*\(.*?\)\s*(?:->\s*[^;]+)?;",
        src,
        re.S,
    )
    return m.group(0) if m else None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--full",
        required=True,
        help="scratch dir holding island_<arch>.rs from a KEEP_CRATE_FUNCS=1 run",
    )
    ap.add_argument("--crate-src", required=True)
    ap.add_argument("--stem", default="island")
    ap.add_argument("--add", nargs="*", default=[])
    ap.add_argument(
        "--drop",
        nargs="*",
        default=[],
        help="names whose callers must import the crate's own definition instead "
        "(their contract diverged from the header); recorded in the output",
    )
    a = ap.parse_args()
    out_path = os.path.join(a.crate_src, OUT)
    names = set(a.add)
    if os.path.exists(out_path):
        names |= set(re.findall(r"pub fn (\w+)\s*\(", Path(out_path).read_text()))
    dropped = set(a.drop)
    if os.path.exists(out_path):
        dropped |= set(re.findall(r"^// DROPPED: (\w+)", Path(out_path).read_text(), re.M))
    names -= dropped
    srcs = {ar: Path(os.path.join(a.full, f"{a.stem}_{ar}.rs")).read_text() for ar in ARCHES}
    defs = crate_definitions(a.crate_src)
    body, bad = {}, []
    for n in sorted(names):
        per = {ar: decl(srcs[ar], n) for ar in ARCHES}
        norm = {re.sub(r"\s+", " ", v) if v else None for v in per.values()}
        if None in norm or len(norm) != 1:
            bad.append(
                f"{n}: "
                + (
                    "absent on " + ",".join(ar for ar, v in per.items() if not v)
                    if None in norm
                    else "declared differently per arch"
                )
            )
            continue
        # The header is only the right declaration if the crate's DEFINITION still agrees with
        # it. Upstream can change a contract under a translated function — SHBindToObject went
        # from cdecl (psf, pidl, riid, ppv) to stdcall (psf, pidl, pbc, riid, ppv) in the
        # 2026-09 sync while shell32-rust kept the old definition — and emitting the header's
        # version would declare a 5-arg stdcall extern for a 4-arg cdecl function: it compiles
        # the moment callers are "fixed" to 5 args, and scribbles. Refuse instead; the caller
        # must import the crate's own definition, or the definition must be migrated.
        hdr_abi = block_abi(srcs["i386"], n)
        if n in defs:
            abi, nargs, where = defs[n]
            abi = "system" if abi in ("system", "stdcall") else abi
            if nargs != arity(per["amd64"]) or abi != hdr_abi:
                bad.append(
                    f'{n}: crate defines extern "{abi}" with {nargs} args '
                    f"({where}) but the header declares {arity(per['amd64'])} args, "
                    f'extern "{hdr_abi}" — the contract has diverged'
                )
                continue
        body.setdefault(hdr_abi, []).append("    " + per["amd64"].strip().replace("\n", "\n    "))
    if bad:
        sys.exit("own_exports_decls: refusing to emit —\n  " + "\n  ".join(bad))
    header = (
        "//! GENERATED by tools/rustconv/own_exports_decls.py — do not edit by hand;\n"
        "//! see its docstring.\n"
        "//!\n"
        "//! Header-exact declarations of functions this crate DEFINES (`#[no_mangle]`)\n"
        "//! and also CALLS from other modules. The island leaves them out on purpose\n"
        "//! (BLOCKLIST_FUNCS_FROM_CRATE), so the defining module never sees a declaration\n"
        "//! of its own definition; the prelude re-exports these for everyone else.\n"
        "//! Typed against the island, like the rest of the prelude.\n"
        "#![allow(dead_code)]\n"
        f"use crate::{a.stem}::*;\n"
    )
    drops = "".join(
        f"// DROPPED: {n} — its crate definition no longer matches the header; "
        "callers import the definition.\n"
        for n in sorted(dropped)
    )
    blocks = "".join(
        f"\n// Header convention {abi!r}, read from the i386 island's enclosing block.\n"
        f'extern "{abi}" {{\n' + "\n".join(decls) + "\n}\n"
        for abi, decls in sorted(body.items())
    )
    with open(out_path, "w") as f:
        f.write(header + drops + blocks)
    print(f"own_exports_decls: wrote {sum(map(len, body.values()))} declarations to {out_path}")


if __name__ == "__main__":
    main()
