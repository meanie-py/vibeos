#!/usr/bin/env python3
"""
scan_raw_bitfield.py — a C bitfield reached by bit INDEX instead of by NAME.

bindgen packs C bitfields into `_bitfield_N: __BindgenBitfieldUnit<[u8; K]>` and generates a
named accessor per field (`set_Valid`, `DeletePending()`). C names the field, and so must the
Rust: a hand call to `_bitfield_N.set(offset, width, v)` / `.get(offset, width)` is a guess at
the layout, it compiles, and it is wrong in every way a guess can be:

  * atapi's MODE_INFO_EXCEPTIONS `set(0, MODE_PAGE_FAULT_REPORTING, ..)` passed the page code
    as the WIDTH: 28 bits of a 1-byte unit, an index-out-of-bounds panic (BugCheck E0525354)
    the first time Windows asked the CD for its fault-reporting page;
  * atapi's SENSE_DATA put Valid in bit 0 and ErrorCode in bits 1-7 (ErrorCode:7 comes first);
  * ntoskrnl's ARM3 session code swapped Initialized (bit 0) and DeletePending (bit 1) at all
    four sites, so `MmAttachSession` refused every live session as terminating;
  * pnpmgr read DEVICE_CAPABILITIES bits 2-5 (Lock/Eject/Removable/Dock) as WakeFromD0-D3.

Generated files (bindgen's own accessor bodies) are skipped by content, like every scanner.

Usage:
    python3 tools/scan_raw_bitfield.py            # report
    python3 tools/scan_raw_bitfield.py --lint     # terse (CLI baseline gate)

Exit 1 if any finding.
"""

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_bitwise_not_bool import GEN_MARKERS, SKIP_PATH_RE
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"

NOLINT_RE = re.compile(r"//\s*nolint(?::\s*raw-bitfield\b|\s*$|:?\s*\()")
# A call on a bitfield unit that is not bindgen's own `self._bitfield_N` inside its accessor.
RAW_RE = re.compile(
    r"(?<!\bself\.)\b_bitfield_\d+\s*\.\s*(?:set|get|raw_set|raw_get|set_bit|get_bit)\s*\("
)
STRING_RE = re.compile(r'b?"(?:\\.|[^"\\])*"')


def is_generated(text: str) -> bool:
    return any(m in "\n".join(text.split("\n")[:6]) for m in GEN_MARKERS)


def scan_text(text: str) -> list[tuple[int, str]]:
    if is_generated(text):
        return []
    out = []
    for i, line in enumerate(text.split("\n")):
        code = STRING_RE.sub('""', line.split("//", 1)[0])
        if NOLINT_RE.search(line) or not RAW_RE.search(code):
            continue
        out.append((i + 1, line.strip()[:110]))
    return out


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("-")]
    lint = "--lint" in sys.argv
    roots = [Path(a) for a in args] if args else [RUST_ROOT]
    files: list[Path] = []
    for root in roots:
        if root.is_file():
            files.append(root)
        elif root.is_dir():
            files.extend(source_files(root))

    findings = []
    for f in sorted(set(files)):
        if SKIP_PATH_RE.search(str(f)):
            continue
        try:
            text = f.read_text(errors="replace")
        except OSError:
            continue
        for ln, src in scan_text(text):
            try:
                rel = f.relative_to(REPO)
            except ValueError:
                rel = f
            findings.append((str(rel), ln, src))

    for rel, ln, src in findings:
        if lint:
            print(f"{rel}:{ln}: raw-bitfield")
        else:
            print(f"{rel}:{ln}: a bitfield reached by bit index, not by its name")
            print(f"    {src}")
    if not lint:
        print(f"\n{len(findings)} site(s)")
        if findings:
            print("fix: use bindgen's named accessor (`x.set_Field(v)` / `x.Field()`), as C does.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
