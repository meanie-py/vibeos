#!/usr/bin/env python3
"""Rewrite an mkisofs graft-point list so PE binaries point at debug-stripped twins.

Why: the CD-staged binaries are 70-92% DWARF debug sections (wdf01000.sys:
13.5MB file, 0.8MB code). The PE loader never maps debug sections, but freeldr
streams every byte off the CD at ~2.2MB/s (ATAPI PIO under TCG) — ~20s of the
pre-kernel boot phase on i386/amd64 is spent reading bytes nobody uses. See
docs/engineering/performance-investigation.md.

The build tree keeps full DWARF (sym.sh / addr2sym / build-index / GDB all read
build-dir binaries, never the ISO); only the media gets stripped twins.

Each stripped twin gets its PE OptionalHeader.CheckSum recomputed:
MmCheckSystemImage (sysldr.c) verifies checksums via
LdrVerifyMappedImageMatchesChecksum for runtime-loaded drivers, and
llvm-objcopy does not update the field.

Per the fail-loud rule: an objcopy or checksum failure on a PE file aborts the
build — we never silently graft the unstripped original.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import struct
import subprocess
import sys
from pathlib import Path

MANIFEST = "strip-manifest.json"


def parse_pe(data: bytes):
    """Return (checksum_offset, [section names]) or None if not a PE image."""
    if len(data) < 0x40 or data[:2] != b"MZ":
        return None
    (e_lfanew,) = struct.unpack_from("<I", data, 0x3C)
    if e_lfanew + 24 > len(data) or data[e_lfanew : e_lfanew + 4] != b"PE\0\0":
        return None
    coff = e_lfanew + 4
    # COFF: Machine(2) NumberOfSections(2) TimeDateStamp(4)
    #       PointerToSymbolTable(4) NumberOfSymbols(4) SizeOfOptionalHeader(2) Characteristics(2)
    nsects, symtab_ptr, nsyms, opt_size = struct.unpack_from("<xxH4xIIH2x", data, coff)
    opt = coff + 20
    # CheckSum is at offset 64 in both PE32 and PE32+ optional headers
    csum_off = opt + 64
    strtab = symtab_ptr + nsyms * 18 if symtab_ptr else 0
    names = []
    sect = opt + opt_size
    for i in range(nsects):
        raw = data[sect + i * 40 : sect + i * 40 + 8]
        if len(raw) < 8:
            break
        name = raw.rstrip(b"\0")
        if name.startswith(b"/") and strtab:
            # long name: '/<decimal offset into COFF string table>'
            try:
                off = strtab + int(name[1:])
                end = data.index(b"\0", off)
                name = data[off:end]
            except (ValueError, IndexError):
                pass
        names.append(name.decode("latin-1"))
    return csum_off, names


def has_debug_sections(names: list[str]) -> bool:
    return any(n.startswith(".debug_") or n == ".debug" for n in names)


def pe_checksum(data: bytearray, csum_off: int) -> int:
    """Standard PE image checksum (16-bit folded dword sum + file length)."""
    struct.pack_into("<I", data, csum_off, 0)
    if len(data) % 4:
        data = data + b"\0" * (4 - len(data) % 4)
    total = 0
    mv = memoryview(data).cast("I")
    for dw in mv:
        total += dw
        total = (total & 0xFFFFFFFF) + (total >> 32)
    total = (total & 0xFFFF) + (total >> 16)
    total = (total + (total >> 16)) & 0xFFFF
    return (total + len(data)) & 0xFFFFFFFF


def strip_one(src: Path, dst: Path, objcopy: str) -> None:
    subprocess.run(
        [objcopy, "--strip-debug", str(src), str(dst)],
        check=True,
        capture_output=True,
        text=True,
    )
    data = bytearray(dst.read_bytes())
    parsed = parse_pe(data)
    if parsed is None:
        raise RuntimeError(f"stripped output is not PE: {dst}")
    csum_off, _ = parsed
    csum = pe_checksum(data, csum_off)
    struct.pack_into("<I", data, csum_off, csum)
    dst.write_bytes(data)


def twin_for(src: Path, cache: Path, objcopy: str, manifest: dict) -> Path | None:
    """Return the stripped twin for src (creating/refreshing it), or None if src
    is not a PE with debug sections. Raises on strip/checksum failure."""
    try:
        head = src.open("rb").read(2)
    except OSError:
        return None
    if head != b"MZ":
        return None
    parsed = parse_pe(src.read_bytes())
    if parsed is None or not has_debug_sections(parsed[1]):
        return None
    st = src.stat()
    key = str(src)
    twin = cache / f"{hashlib.sha1(key.encode()).hexdigest()[:12]}-{src.name}"
    entry = manifest.get(key)
    if not (
        entry
        and entry.get("mtime_ns") == st.st_mtime_ns
        and entry.get("size") == st.st_size
        and twin.is_file()
    ):
        strip_one(src, twin, objcopy)
        manifest[key] = {"mtime_ns": st.st_mtime_ns, "size": st.st_size}
    return twin


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--lst", type=Path, help="input graft list")
    ap.add_argument("--out", type=Path, help="rewritten graft list")
    ap.add_argument("--file", type=Path, help="single-file mode: print twin (or original) path")
    ap.add_argument("--cache", required=True, type=Path, help="stripped-twin cache dir")
    ap.add_argument("--objcopy", required=True, help="llvm-objcopy (or prefixed wrapper)")
    args = ap.parse_args()
    if bool(args.file) == bool(args.lst):
        ap.error("exactly one of --file or (--lst + --out) required")

    args.cache.mkdir(parents=True, exist_ok=True)
    if args.file:
        manifest_path = args.cache / MANIFEST
        try:
            manifest = json.loads(manifest_path.read_text())
        except (OSError, ValueError):
            manifest = {}
        twin = twin_for(args.file, args.cache, args.objcopy, manifest)
        manifest_path.write_text(json.dumps(manifest))
        print(twin if twin else args.file)
        return 0

    manifest_path = args.cache / MANIFEST
    try:
        manifest = json.loads(manifest_path.read_text())
    except (OSError, ValueError):
        manifest = {}

    out_lines = []
    n_stripped = n_cached = 0
    bytes_in = bytes_out = 0
    for line in args.lst.read_text().splitlines():
        graft, sep, src_s = line.partition("=")
        src = Path(src_s) if sep else Path(graft)
        if not (sep and src.is_file()):
            out_lines.append(line)
            continue
        try:
            head = src.open("rb").read(4096)
        except OSError:
            out_lines.append(line)
            continue
        if not head.startswith(b"MZ"):
            out_lines.append(line)
            continue
        data = src.read_bytes()
        parsed = parse_pe(data)
        if parsed is None or not has_debug_sections(parsed[1]):
            out_lines.append(line)
            continue

        st = src.stat()
        key = str(src)
        twin = args.cache / f"{hashlib.sha1(key.encode()).hexdigest()[:12]}-{src.name}"
        entry = manifest.get(key)
        if not (
            entry
            and entry.get("mtime_ns") == st.st_mtime_ns
            and entry.get("size") == st.st_size
            and twin.is_file()
        ):
            strip_one(src, twin, args.objcopy)
            manifest[key] = {"mtime_ns": st.st_mtime_ns, "size": st.st_size}
            n_stripped += 1
        else:
            n_cached += 1
        bytes_in += st.st_size
        bytes_out += twin.stat().st_size
        out_lines.append(f"{graft}={twin}")

    manifest_path.write_text(json.dumps(manifest))

    # --- DOOM (opt-in, VIBEOS_DOOM=1) — x86 twin of the mkramdisk.sh hook ---
    # Foreign-binary demo (docs/engineering/doom-on-vibeos.md). OFF by default:
    # without VIBEOS_DOOM=1 nothing is grafted and the ISO is byte-identical.
    # Runs here (not cmake) because this script already executes on every
    # `livecd` build and inherits the build env. Fail-loud per the house rule:
    # the flag being set with assets missing aborts the build.
    if os.environ.get("VIBEOS_DOOM", "0") == "1":
        repo = Path(__file__).resolve().parent.parent
        # arch from the build dir in the output path (build-amd64/... etc.)
        m = re.search(r"build-(amd64|i386|arm64)", str(args.out))
        arch = m.group(1) if m else None
        doom_exe = Path(
            os.environ.get("VIBEOS_DOOM_EXE", repo / "tools" / "doom" / f"doom-{arch}.exe")
        )
        doom_wad = Path(os.environ.get("VIBEOS_DOOM_WAD", repo / "tools" / "doom" / "doom1.wad"))
        if arch is None or not (doom_exe.is_file() and doom_wad.is_file()):
            print(
                f"strip_cd_media: VIBEOS_DOOM=1 but assets missing "
                f"(arch={arch}, exe={doom_exe}, wad={doom_wad}). "
                f"Build them: tools/doom/build.sh {arch or '<arch>'}",
                file=sys.stderr,
            )
            return 1
        out_lines.append(f"reactos/system32/doom.exe={doom_exe}")
        out_lines.append(f"reactos/system32/doom1.wad={doom_wad}")
        print(f"strip_cd_media: VIBEOS_DOOM=1 — grafting doom.exe + doom1.wad ({arch})")

    args.out.write_text("\n".join(out_lines) + "\n")
    print(
        f"strip_cd_media: {n_stripped} stripped, {n_cached} cached: "
        f"{bytes_in / 1e6:.0f}MB -> {bytes_out / 1e6:.0f}MB"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
