#!/usr/bin/env python3
"""
scan_pack_drift.py — a struct the C declares under `#pragma pack` that its Rust port does not pack.

`#include <pshpack1.h>` ... `<poppack.h>` is invisible at the struct, so a translation gives it
plain `#[repr(C)]` and every field after the first misaligned one moves. All of these shipped:
diskpart's PARTITION_SECTOR read the MBR magic from past the 512-byte sector (no disk ever
looked partitioned); setuplib's FAT/FAT32/BTRFS/NTFS boot sectors had every BPB field from
offset 11 on shifted, and its MountedDevices value was 16 bytes, not 12; oleaut32's APM and icon
headers (field manual, "A `pshpack` region translated as plain `repr(C)`").

For every module a cohab flip names (`# x.c  # replaced by Rust (sdk/lib/rust/<crate>/...)`),
each struct inside a pshpack region of the module's own .c/.h is looked up by its typedef or
tag name in the crate, and its `repr` must say `packed`. A public-header pack (cabinet's
`fdi.h`) is out of reach here: measure those with the module's real flags instead.

Usage:
    python3 tools/scan_pack_drift.py [--lint]
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
_FLIP = re.compile(r"#\s*\S+\.(?:c|cpp)\s+#\s*replaced by Rust \((sdk/lib/rust/[\w.-]+)")
_REGION = re.compile(
    r'#include\s*[<"]pshpack(\d)\.h[>"](.*?)#include\s*[<"]poppack\.h[>"]', re.S
)
_TYPEDEF = re.compile(r"typedef\s+struct\s+(\w+)?\s*\{.*?\}\s*(\w+)", re.S)
_TAGGED = re.compile(r"(?<!typedef )\bstruct\s+(\w+)\s*\{")
_COMMENT = re.compile(r"/\*.*?\*/|//[^\n]*", re.S)


def module_crates() -> dict[Path, set[Path]]:
    out: dict[Path, set[Path]] = {}
    for cml in REPO.glob("**/CMakeLists.txt"):
        rel = cml.relative_to(REPO).parts
        if rel[0].startswith(("build-", ".")) or rel[0] in ("vendor", "output-MinGW-i386"):
            continue
        text = cml.read_text(errors="replace")
        for m in _FLIP.finditer(text):
            crate = REPO / m.group(1)
            if crate.is_dir():
                out.setdefault(cml.parent, set()).add(crate)
    return out


def packed_structs(module: Path) -> list[tuple[str, int, set[str]]]:
    found = []
    for f in sorted(module.rglob("*")):
        if f.suffix not in (".c", ".h") or not f.is_file():
            continue
        text = _COMMENT.sub("", f.read_text(errors="replace"))
        for region in _REGION.finditer(text):
            for s in _TYPEDEF.finditer(region.group(2)):
                found.append((f.name, int(region.group(1)), {n for n in s.groups() if n}))
            for s in _TAGGED.finditer(region.group(2)):
                found.append((f.name, int(region.group(1)), {s.group(1)}))
    return found


def scan() -> list[str]:
    findings = []
    for module, crates in sorted(module_crates().items()):
        structs = packed_structs(module)
        if not structs:
            continue
        rust = "\n".join(
            p.read_text(errors="replace")
            for c in crates
            for p in c.rglob("*.rs")
            if "/target/" not in str(p) and not p.name.startswith(("bindgen_", "island_"))
        )
        for fname, pack, names in structs:
            for name in sorted(names):
                for m in re.finditer(
                    r"#\[repr\(([^)\]]*)\)\]\s*(?:#\[[^\]]*\]\s*)*pub struct " + re.escape(name) + r"\b",
                    rust,
                ):
                    if "packed" not in m.group(1):
                        rel = module.relative_to(REPO)
                        findings.append(
                            f"{rel}/{fname}: pack-drift {name} is pshpack{pack} in the C, "
                            f"#[repr({m.group(1)})] in the Rust"
                        )
    return sorted(set(findings))


def main(argv: list[str] | None = None) -> int:
    findings = scan()
    for f in findings:
        print(f)
    if findings and "--lint" not in (argv if argv is not None else sys.argv[1:]):
        print(f"\n{len(findings)} finding(s): #[repr(C, packed)] plus a size_of/offset_of assert.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
