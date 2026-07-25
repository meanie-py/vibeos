#!/usr/bin/env python3
"""Verify the ARM64 unwind-code decoder semantics against real prologues.

`sdk/lib/rust/rtl-except/src/arm64/unwind_decode.rs` interprets `.xdata` unwind
codes to reconstruct a caller's CONTEXT. It is authored fresh (upstream ReactOS
never implemented an ARM64 unwinder), it runs in BOTH kernel and user mode, and
a wrong decode does not fault -- it silently reloads callee-saved registers from
the wrong stack slots, which surfaces much later as a garbage PC, a wild SP, or
inexplicable corruption after a resume. It has been wrong three times already
(field manual 9.69, 9.92, 9.114) and had no test.

This is that test. It does NOT trust either the Rust decoder or a reading of the
Microsoft spec. For every function in a built ARM64 PE it:

  1. parses .pdata/.xdata and decodes the unwind-code stream,
  2. disassembles the real prologue,
  3. asserts that every register-save code names exactly the register pair the
     prologue actually stores at that stack offset.

The prologue is the ground truth; the decoder gets no vote.

Usage:
    python3 tools/verify_arm64_unwind.py build-arm64            # report
    python3 tools/verify_arm64_unwind.py build-arm64 --check    # exit 1 on any mismatch
    python3 tools/verify_arm64_unwind.py path/to/some.exe       # single PE

Limitation: only pair-saves reachable from `save_next` chains and the explicit
pair/single saves are cross-checked; `save_any_reg` (0xE7) and SVE `alloc_z`
(0xDF) are decoded but not verified against disassembly, and are reported
separately so their appearance in a future toolchain is not silently ignored.
"""

from __future__ import annotations

import argparse
import os
import re
import struct
import subprocess
import sys
from collections import defaultdict

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OBJDUMP_CANDIDATES = [
    os.path.join(HERE, ".build-tools/llvm-mingw/bin/llvm-objdump"),
    "llvm-objdump",
]

# PEs worth checking in a build dir: the two that actually run the unwinder.
DEFAULT_TARGETS = [
    "ntoskrnl/ntoskrnl.exe",
    "dll/ntdll/ntdll.dll",
]


def find_objdump() -> str:
    for cand in OBJDUMP_CANDIDATES:
        if os.path.sep in cand:
            if os.path.exists(cand):
                return cand
        else:
            from shutil import which

            if which(cand):
                return cand
    sys.exit("verify_arm64_unwind: no llvm-objdump found (.build-tools/llvm-mingw/bin)")


# --------------------------------------------------------------------------
# PE parsing
# --------------------------------------------------------------------------


def parse_pe(path):
    with open(path, "rb") as fh:
        data = fh.read()
    if data[:2] != b"MZ":
        return None
    e_lfanew = struct.unpack_from("<I", data, 0x3C)[0]
    if data[e_lfanew : e_lfanew + 4] != b"PE\0\0":
        return None
    fh_off = e_lfanew + 4
    machine, nsec, _, _, _, size_opt, _ = struct.unpack_from("<HHIIIHH", data, fh_off)
    if machine != 0xAA64:  # IMAGE_FILE_MACHINE_ARM64
        return None
    opt = fh_off + 20
    if struct.unpack_from("<H", data, opt)[0] != 0x20B:
        return None
    image_base = struct.unpack_from("<Q", data, opt + 24)[0]
    dd = opt + 112
    pdata_rva, pdata_size = struct.unpack_from("<II", data, dd + 3 * 8)
    sections = []
    sh = opt + size_opt
    for i in range(nsec):
        off = sh + i * 40
        vsize, vaddr, rawsize, rawptr = struct.unpack_from("<IIII", data, off + 8)
        sections.append((vaddr, vsize, rawptr, rawsize))
    return data, image_base, pdata_rva, pdata_size, sections


def rva2off(sections, rva):
    for vaddr, vsize, rawptr, rawsize in sections:
        if vaddr <= rva < vaddr + max(vsize, rawsize):
            return rawptr + (rva - vaddr)
    return None


# --------------------------------------------------------------------------
# Unwind-code decoding -- MUST mirror unwind_decode.rs
# --------------------------------------------------------------------------


def code_len(b0: int) -> int:
    if b0 <= 0xBF:
        return 1  # alloc_s / save_r19r20_x / save_fplr{,_x}
    if b0 <= 0xDF:
        return 2  # alloc_m, the two-byte reg saves, alloc_z
    if b0 == 0xE0:
        return 4  # alloc_l
    if b0 == 0xE2:
        return 2  # add_fp
    if b0 == 0xE7:
        return 3  # save_any_reg
    return 1  # set_fp/nop/end/end_c/save_next


def decode_pair(codes: bytes, j: int):
    """Explicit PAIR save at index j -> (first_reg, sp_offset, is_fp), else None.

    sp_offset is relative to the frame SP the codes are interpreted against,
    which for an offset-based save equals the post-prologue SP -- i.e. exactly
    the literal in the prologue's `[sp, #off]`.
    """
    b0 = codes[j]
    b1 = codes[j + 1] if j + 1 < len(codes) else 0
    if 0x20 <= b0 <= 0x3F:
        return (19, 0, False)  # save_r19r20_x @[sp]
    if 0x40 <= b0 <= 0x7F:
        return (29, (b0 & 0x3F) * 8, False)  # save_fplr
    if 0x80 <= b0 <= 0xBF:
        return (29, 0, False)  # save_fplr_x @[sp]
    if 0xC8 <= b0 <= 0xCB:
        return (19 + (((b0 & 3) << 2) | (b1 >> 6)), (b1 & 0x3F) * 8, False)  # save_regp
    if 0xCC <= b0 <= 0xCF:
        return (19 + (((b0 & 3) << 2) | (b1 >> 6)), 0, False)  # save_regp_x
    if 0xD8 <= b0 <= 0xD9:
        return (8 + (((b0 & 1) << 2) | (b1 >> 6)), (b1 & 0x3F) * 8, True)  # save_fregp
    if 0xDA <= b0 <= 0xDB:
        return (8 + (((b0 & 1) << 2) | (b1 >> 6)), 0, True)  # save_fregp_x
    return None


def unwind_stream(data, sections, unwind_rva):
    """-> (function_length, code_bytes) for an unpacked .xdata record."""
    off = rva2off(sections, unwind_rva)
    if off is None:
        return None, None
    hdr = struct.unpack_from("<I", data, off)[0]
    func_len = (hdr & 0x3FFFF) * 4
    e_bit = (hdr >> 21) & 1
    epilog_count = (hdr >> 22) & 0x1F
    code_words = (hdr >> 27) & 0x1F
    p = off + 4
    if epilog_count == 0 and code_words == 0:  # extended header
        ext = struct.unpack_from("<I", data, p)[0]
        epilog_count = ext & 0xFFFF
        code_words = (ext >> 16) & 0xFF
        p += 4
    if e_bit == 0:
        p += 4 * epilog_count  # epilog scope words
    return func_len, data[p : p + 4 * code_words]


def walk(codes: bytes):
    """Yield code start indices in stream order, stopping at end/end_c."""
    i = 0
    while i < len(codes):
        yield i
        if codes[i] in (0xE4, 0xE5):
            return
        i += code_len(codes[i])


def resolve_save_next(codes: bytes, j: int):
    """THE RULE UNDER TEST (unwind_decode.rs::save_next_target).

    Unwind codes are stored in reverse-prologue order, so the pair a `save_next`
    denotes is derived from the explicit save that appears AFTER the whole
    save_next run in the byte stream. Ordinal 1 is the save_next nearest that
    base pair.
    """
    run_end = j
    while run_end < len(codes) and codes[run_end] == 0xE6:
        run_end += 1
    if run_end >= len(codes):
        return None
    ordinal = run_end - j
    base = decode_pair(codes, run_end)
    if base is None:
        return None
    first, offset, is_fp = base
    return (first + 2 * ordinal, offset + 16 * ordinal, is_fp)


# --------------------------------------------------------------------------
# Prologue ground truth
# --------------------------------------------------------------------------

INSN = re.compile(r"^\s*([0-9a-f]+):\s+(\S+)\s*(.*)$")
STORE = re.compile(r"([xd])(\d+),\s*(?:([xd])(\d+),\s*)?\[sp(?:,\s*#(-?0x[0-9a-f]+|-?\d+))?\](!?)")


def prologue_saves(objdump, path, va, func_len):
    """-> {sp_offset: (reg1, reg2|None, is_fp)} for the prologue's stores.

    Offsets are as written in the instruction, which is what the unwind codes'
    offset-based saves are relative to. A pre-indexed store lands at offset 0.
    """
    span = min(func_len, 128) if func_len else 128
    out = subprocess.run(
        [
            objdump,
            "-d",
            f"--start-address={va}",
            f"--stop-address={va + span}",
            "--no-show-raw-insn",
            path,
        ],
        capture_output=True,
        text=True,
        check=False,
    ).stdout
    saves = {}
    for line in out.splitlines():
        m = INSN.match(line)
        if not m:
            continue
        mnemonic, ops = m.group(2), m.group(3).split("//")[0].strip()
        if mnemonic in ("stp", "str"):
            sm = STORE.match(ops)
            if not sm:
                continue
            is_fp = sm.group(1) == "d"
            r1 = int(sm.group(2))
            r2 = int(sm.group(4)) if sm.group(4) else None
            off = int(sm.group(5), 0) if sm.group(5) else 0
            if sm.group(6) == "!":  # pre-indexed: pair sits at the new sp
                off = 0
            saves[off] = (r1, r2, is_fp)
        elif mnemonic in ("add", "mov") and ops.startswith("x29,"):
            break  # frame pointer set: prologue done
        elif mnemonic in ("bl", "ret", "cbz", "cbnz") and saves:
            break
    return saves


# --------------------------------------------------------------------------


def verify_pe(objdump, path, verbose=False):
    parsed = parse_pe(path)
    if parsed is None:
        return None
    data, image_base, pdata_rva, pdata_size, sections = parsed
    poff = rva2off(sections, pdata_rva)
    if poff is None or pdata_size == 0:
        return None

    stats = defaultdict(int)
    mismatches = []

    for k in range(pdata_size // 8):
        begin, unwind = struct.unpack_from("<II", data, poff + k * 8)
        if unwind & 3:
            stats["packed"] += 1
            continue
        func_len, codes = unwind_stream(data, sections, unwind)
        if not codes:
            continue
        stats["funcs"] += 1

        positions = list(walk(codes))
        targets = []  # (code_index, first, off, is_fp, kind)
        for j in positions:
            b0 = codes[j]
            if b0 == 0xE6:
                stats["save_next"] += 1
                r = resolve_save_next(codes, j)
                if r is None:
                    stats["save_next_unresolved"] += 1
                    continue
                targets.append((j, *r, "save_next"))
            elif b0 == 0xE7:
                stats["save_any_reg"] += 1  # decoded but not disasm-verified
            elif b0 == 0xDF:
                stats["alloc_z"] += 1
            else:
                p = decode_pair(codes, j)
                if p:
                    targets.append((j, *p, "pair"))
        if not targets:
            continue

        real = prologue_saves(objdump, path, image_base + begin, func_len)
        if not real:
            stats["no_prologue"] += 1
            continue

        for j, first, off, is_fp, kind in targets:
            got = real.get(off)
            if got is None:
                stats[f"{kind}_no_slot"] += 1
                continue
            r1, r2, gfp = got
            if r1 == first and r2 == first + 1 and gfp == is_fp:
                stats[f"{kind}_ok"] += 1
            else:
                stats[f"{kind}_MISMATCH"] += 1
                if len(mismatches) < 20:
                    mismatches.append(
                        f"    {image_base + begin:#x} code[{j}] ({kind}): "
                        f"decoder says {'d' if is_fp else 'x'}{first},{first + 1}@sp+{off}"
                        f" but prologue stores "
                        f"{'d' if gfp else 'x'}{r1}"
                        f"{',' + ('d' if gfp else 'x') + str(r2) if r2 is not None else ''}"
                        f"@sp+{off}"
                    )
    return stats, mismatches


def main():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("target", help="build-arm64 directory, or a single ARM64 PE")
    ap.add_argument("--check", action="store_true", help="exit 1 if any decode mismatches")
    ap.add_argument("-v", "--verbose", action="store_true")
    args = ap.parse_args()

    objdump = find_objdump()

    if os.path.isdir(args.target):
        paths = [os.path.join(args.target, rel) for rel in DEFAULT_TARGETS]
        paths = [p for p in paths if os.path.exists(p)]
        if not paths:
            sys.exit(f"verify_arm64_unwind: no ARM64 PEs found under {args.target}")
    else:
        paths = [args.target]

    total_mismatch = 0
    total_checked = 0
    for path in paths:
        result = verify_pe(objdump, path, args.verbose)
        if result is None:
            print(f"{path}: not an ARM64 PE with .pdata -- skipped")
            continue
        stats, mismatches = result
        checked = (
            stats["save_next_ok"]
            + stats["save_next_MISMATCH"]
            + stats["pair_ok"]
            + stats["pair_MISMATCH"]
        )
        bad = stats["save_next_MISMATCH"] + stats["pair_MISMATCH"]
        total_mismatch += bad
        total_checked += checked
        status = "OK" if bad == 0 else f"{bad} MISMATCH"
        print(
            f"{os.path.basename(path)}: {checked} save sites verified against "
            f"disassembly -- {status}"
        )
        print(f"    functions with unwind codes : {stats['funcs']}")
        print(
            f"    save_next codes             : {stats['save_next']}"
            f" ({stats['save_next_ok']} verified)"
        )
        print(f"    explicit pair saves         : {stats['pair_ok'] + stats['pair_MISMATCH']}")
        for key in (
            "save_next_unresolved",
            "save_next_no_slot",
            "pair_no_slot",
            "no_prologue",
            "save_any_reg",
            "alloc_z",
        ):
            if stats[key]:
                print(f"    {key:28}: {stats[key]}")
        for line in mismatches:
            print(line)

    print()
    if total_mismatch:
        print(
            f"FAIL: {total_mismatch}/{total_checked} unwind-code save sites disagree "
            f"with the real prologue"
        )
        if args.check:
            return 1
    else:
        print(f"PASS: {total_checked} unwind-code save sites match the real prologue")
    return 0


if __name__ == "__main__":
    sys.exit(main())
