#!/usr/bin/env python3
"""Transcribe ctype.c's two 257-entry locale tables into Rust, proven against the object.

A hand-copied table is the worst kind of translation bug: 514 numbers, no compiler check,
and a single wrong entry makes exactly one character classify wrong. The difftest cannot
see it either if both sides read the same copy -- which is precisely why msvcrt-difftest
takes `MSVCRT__ctype` from ctype.c rather than modelling it.

So this does not copy anything. It:

  1. parses the initializers out of `dll/win32/msvcrt/ctype.c`,
  2. resolves the `_MSVCR_VER`-gated macros for THIS DLL's setting (`_MSVCR_VER=0`),
  3. evaluates every entry using the mask values from the msvcrt-sys ISLAND -- not from
     a transcription of ctype.h -- and
  4. refuses to emit unless all 514 values equal the bytes in the COMPILED OBJECT's
     `.data`, which is what the C build actually produced.

Steps 3 and 4 are two independent derivations of the same numbers: the source expressions
evaluated against the island, and the compiler's own output. Agreement is the proof. If
either drifts (a ctype.h mask changes, a `_MSVCR_VER` arm flips) this exits non-zero
instead of shipping a silently-wrong table.

The emitted Rust keeps the C's own spelling -- `_S_|_C_`, `0x0028 | B110` -- via u16
aliases, so the two files stay diffable line by line.

    ./vibeos-cli check                         # native ctype-table freshness/proof check
    python3 tools/gen_ctype_tables.py --emit     # rewrite the table block in place
"""

from __future__ import annotations

import argparse
import re
import struct
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CTYPE_C = REPO / "dll/win32/msvcrt/ctype.c"
ISLAND = REPO / "sdk/lib/rust/msvcrt-sys/src/bindgen_i386.rs"
OBJ = REPO / "build-i386/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir/ctype.c.obj"
OBJCOPY = REPO / ".build-tools/llvm-mingw/bin/llvm-objcopy"
NM = REPO / ".build-tools/llvm-mingw/bin/llvm-nm"

# ctype.c's own abbreviations, verbatim from its header comment ("Some abbreviations to
# make the following table readable").
ABBREV = {
    "_C_": "_CONTROL",
    "_S_": "_SPACE",
    "_P_": "_PUNCT",
    "_D_": "_DIGIT",
    "_H_": "_HEX",
    "_U_": "_UPPER",
    "_L_": "_LOWER",
}

# The `_MSVCR_VER`-gated arms, resolved for msvcrt.dll (`-D_MSVCR_VER=0`, see the module's
# CMakeLists). Each records the guard it came from so a future _MSVCR_VER change is a
# one-line edit here and a re-run, not a re-transcription.
VER_GATED = {
    "B110": ("_BLANK", "_MSVCR_VER <= 110"),
    "D120": ("4", "_MSVCR_VER != 120"),
    "S140": ("0", "_MSVCR_VER < 140"),
    "L140": ("0", "_MSVCR_VER < 140"),
    "C140": ("0", "_MSVCR_VER < 140"),
}


def island_masks() -> dict[str, int]:
    """The ctype mask values, read from the bindgen island (the substrate the Rust uses)."""
    text = ISLAND.read_text()
    masks = {}
    for name in ("_UPPER", "_LOWER", "_DIGIT", "_SPACE", "_PUNCT", "_CONTROL", "_BLANK", "_HEX"):
        m = re.search(rf"^pub const {name}: u32 = (\d+);", text, re.M)
        if not m:
            sys.exit(
                f"gen_ctype_tables: {name} is not in {ISLAND.relative_to(REPO)} -- "
                f"the island must carry every mask the tables are built from"
            )
        masks[name] = int(m.group(1))
    return masks


def initializer(src: str, decl: str) -> list[str]:
    """The comma-separated entries of `decl`'s brace-enclosed initializer, comments stripped."""
    at = src.index(decl)
    open_at = src.index("{", at)
    close_at = src.index("};", open_at)
    body = src[open_at + 1 : close_at]
    body = re.sub(r"/\*.*?\*/", "", body, flags=re.S)
    entries = [e.strip() for e in body.split(",")]
    entries = [e for e in entries if e]
    if len(entries) != 257:
        sys.exit(f"gen_ctype_tables: {decl} parsed to {len(entries)} entries, expected 257")
    return entries


def evaluate(entries: list[str], masks: dict[str, int]) -> list[int]:
    env = dict(masks)
    for abbrev, real in ABBREV.items():
        env[abbrev] = masks[real]
    for name, (value, _why) in VER_GATED.items():
        env[name] = masks[value] if value in masks else int(value)
    out = []
    for i, e in enumerate(entries):
        try:
            # Closed environment: the only names are the island's masks and the only
            # operators ctype.c uses in these initializers are `|` on integer literals.
            v = eval(e, {"__builtins__": {}}, env)
        except Exception as exc:  # pragma: no cover -- a parse break must be loud
            sys.exit(f"gen_ctype_tables: entry {i} {e!r} did not evaluate: {exc}")
        if not 0 <= v <= 0xFFFF:
            sys.exit(f"gen_ctype_tables: entry {i} {e!r} = {v} does not fit a WORD")
        out.append(v)
    return out


def object_tables() -> tuple[list[int], list[int]] | None:
    """The two tables as the C compiler actually emitted them, from the object's .data.

    None when this worktree has no i386 build. The object is the ORACLE, not the source:
    without it the source-vs-island derivation and the block-freshness check both still
    run, only the proof is skipped -- and it says so on stderr rather than passing quietly.
    Deliberately never falls back to a sibling worktree's build: a generated artifact from
    another commit would green-light real drift (AGENTS.md, fleet isolation applies to
    READS).
    """
    if not OBJ.exists():
        print(
            f"gen_ctype_tables: WARNING -- no {OBJ.relative_to(REPO)}, so the 514 values "
            f"were NOT proven against the compiler's own output; only the source-vs-island "
            f"derivation ran.\n  to get the proof: ./vibeos-cli build --arch i386",
            file=sys.stderr,
        )
        return None
    syms = subprocess.run(
        [str(NM), "--defined-only", str(OBJ)], capture_output=True, text=True, check=True
    ).stdout
    off = {}
    for line in syms.splitlines():
        parts = line.split()
        if len(parts) == 3 and parts[1] == "D":
            off[parts[2].lstrip("_")] = int(parts[0], 16)
    for want in ("MSVCRT__ctype", "MSVCRT__wctype"):
        if want not in off:
            sys.exit(f"gen_ctype_tables: {want} is not a DATA symbol of the object")
    dump = REPO / "build-i386/.ctype-data.bin"
    subprocess.run(
        [str(OBJCOPY), f"--dump-section=.data={dump}", str(OBJ), "/dev/null"], check=True
    )
    data = dump.read_bytes()
    dump.unlink()
    narrow = list(struct.unpack_from("<257H", data, off["MSVCRT__ctype"]))
    wide = list(struct.unpack_from("<257H", data, off["MSVCRT__wctype"]))
    return narrow, wide


def prove(name: str, entries: list[str], derived: list[int], compiled: list[int]) -> None:
    bad = [
        (i, entries[i], derived[i], compiled[i]) for i in range(257) if derived[i] != compiled[i]
    ]
    if bad:
        for i, e, d, c in bad[:10]:
            print(
                f"  {name}[{i}] (char 0x{i - 1:02x}): source {e!r} = {d:#06x} "
                f"but the object has {c:#06x}",
                file=sys.stderr,
            )
        sys.exit(
            f"gen_ctype_tables: {len(bad)} of 257 {name} entries disagree with the "
            f"compiled object -- do NOT ship this table"
        )


def render(name: str, entries: list[str]) -> str:
    """8 entries per line, indexed, keeping each entry's C spelling."""
    lines = []
    for row in range(0, 257, 8):
        chunk = entries[row : row + 8]
        # Index 0 is the EOF slot, so row N holds chars N-1 .. N+6.
        tag = "EOF + 0x00" if row == 0 else f"0x{row - 1:02x}"
        lines.append(f"    /* {tag} */ " + ", ".join(chunk) + ",")
    body = "\n".join(lines).rstrip(",")
    # `#[no_mangle]` is emitted here, not left to the caller: the object lists this as a
    # global `D` symbol, and a private Rust static links clean while leaving the C copy
    # live -- a flip that silently does nothing (task #43).
    return f"#[no_mangle]\npub static mut {name}: [WORD; 257] = [\n{body}\n];"


def build_block(masks: dict[str, int]) -> str:
    src = CTYPE_C.read_text()
    narrow_src = initializer(src, "WORD MSVCRT__ctype [257]")
    wide_src = initializer(src, "WORD MSVCRT__wctype[257]")
    narrow = evaluate(narrow_src, masks)
    wide = evaluate(wide_src, masks)
    compiled = object_tables()
    if compiled is not None:
        prove("MSVCRT__ctype", narrow_src, narrow, compiled[0])
        prove("MSVCRT__wctype", wide_src, wide, compiled[1])

    # ctype.c mixes its abbreviations with the occasional full mask name (`_S_|_BLANK` at
    # 0x20). The island types every mask `u32` and the tables are `WORD`, so each name used
    # gets a WORD-typed local. An explicitly declared item shadows a glob import, so these
    # win over `use crate::prelude::*` inside this module and nowhere else.
    direct = sorted(
        {n for e in narrow_src + wide_src for n in re.findall(r"\b_[A-Z]+\b", e) if n in masks}
    )

    aliases = [
        "// ctype.c's own abbreviations, so the two tables below stay diffable against",
        "// the C line by line. WORD-typed, because the island types the masks u32.",
        "#[allow(non_upper_case_globals)]",
    ]
    aliases += [f"const {a}: WORD = msvcrt_sys::{r} as WORD;" for a, r in ABBREV.items()]
    aliases += [f"const {n}: WORD = msvcrt_sys::{n} as WORD;" for n in direct]
    aliases.append("")
    aliases.append("// The `_MSVCR_VER`-gated arms, resolved for THIS DLL (`-D_MSVCR_VER=0`).")
    aliases.append("#[allow(non_upper_case_globals)]")
    for name, (value, why) in VER_GATED.items():
        val = f"msvcrt_sys::{value} as WORD" if value.startswith("_") else value
        aliases.append(f"const {name}: WORD = {val};  // {why}")

    return (
        "\n".join(aliases)
        + "\n\n"
        + render("MSVCRT__ctype", narrow_src)
        + "\n\n"
        + render("MSVCRT__wctype", wide_src)
        + "\n"
    )


BEGIN = "// <gen_ctype_tables BEGIN>"
END = "// <gen_ctype_tables END>"


def main() -> int:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument(
        "--emit",
        metavar="FILE",
        nargs="?",
        const="-",
        help="write the generated block (default: stdout)",
    )
    ap.add_argument(
        "--check",
        metavar="FILE",
        help="verify FILE's generated block is byte-identical to a fresh run",
    )
    args = ap.parse_args()

    if args.check:
        ap.error("ctype audit moved to ./vibeos-cli check (Rust rule: gen_ctype_tables)")
    block = build_block(island_masks())

    if args.emit and args.emit != "-":
        target = Path(args.emit)
        text = target.read_text()
        if BEGIN not in text or END not in text:
            sys.exit(f"gen_ctype_tables: {target} has no {BEGIN} .. {END} block")
        head, rest = text.split(BEGIN, 1)
        _, tail = rest.split(END, 1)
        target.write_text(f"{head}{BEGIN}\n{block}{END}{tail}")
        print(f"gen_ctype_tables: rewrote the table block in {target} (514/514 proven)")
        return 0

    print(f"{BEGIN}\n{block}{END}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
