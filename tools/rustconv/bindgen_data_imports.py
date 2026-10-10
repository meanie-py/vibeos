#!/usr/bin/env python3
"""Give a kernel island's ntoskrnl/hal DATA imports their real linkage.

A driver header declares an exported variable `extern __declspec(dllimport) ULONG
InitSafeBootMode;`. bindgen drops the dllimport and emits a plain `pub static mut
InitSafeBootMode: ULONG;`, which is an undefined symbol at the driver's link: the
driver has no such symbol, only the import slot `__imp_InitSafeBootMode` that the
C compiler reads through. The workaround was a hand-written accessor per variable
(ntkrnl-sys's `InitSafeBootModeValue()`) and a rewritten use site.

This pass puts each such static in its own extern block with
`#[link(name = "<image>", kind = "raw-dylib", modifiers = "+verbatim")]`, which is
the dllimport: rustc then reads the variable through the import slot, the C's own
access, and the translation keeps the C's plain `InitSafeBootMode`.

Only DATA exports are touched (`@ extern` lines of ntoskrnl.spec / hal.spec, honouring
`-arch=`), and only plain statics: a declaration bindgen already emitted against an
`__imp_` name (the headers' __CREATE_NTOS_DATA_IMPORT_ALIAS form) carries a link_name
and is already the import slot.

usage: bindgen_data_imports.py <island.rs> <arch: i386|amd64|arm64> <repo root>
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

# The spec's -arch= spellings per build arch.
_SPEC_ARCHES = {
    "i386": {"i386"},
    "amd64": {"x86_64", "amd64"},
    "arm64": {"arm64", "aarch64"},
}
# (spec, the image a driver imports it from)
_SPECS = (("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe"), ("hal/hal.spec", "hal.dll"))

_BLOCK_RE = re.compile(
    r"^(?P<attr>#\[link\([^\n]*\)\]\n)?unsafe extern \"C\" \{\n(?P<body>(?:    .*\n)*?)\}\n", re.M
)
_STATIC_RE = re.compile(r"^    pub static (?:mut )?(?P<name>\w+): [^;]+;\n", re.M)


def data_exports(repo_root: Path, arch: str) -> dict[str, str]:
    """`{variable: image}` for every DATA export a driver of `arch` can import."""
    wanted = _SPEC_ARCHES[arch]
    out: dict[str, str] = {}
    for spec, image in _SPECS:
        for line in (repo_root / spec).read_text().splitlines():
            words = line.split("#", 1)[0].split()
            if len(words) < 3 or words[0] != "@" or words[1] != "extern":
                continue
            arches = None
            for word in words[2:-1]:
                if word.startswith("-arch="):
                    arches = set(word.split("=", 1)[1].split(","))
            if arches is not None and not arches & wanted:
                continue
            out.setdefault(words[-1].split("=", 1)[0], image)
    return out


def rewrite(text: str, exports: dict[str, str]) -> tuple[str, list[str]]:
    routed: list[str] = []

    def block(match: re.Match[str]) -> str:
        body = match.group("body")
        if match.group("attr") or "#[link_name" in body:
            return match.group(0)  # already the import slot (`__imp_X`)
        pieces: list[str] = []
        rest = body
        for static in _STATIC_RE.finditer(body):
            image = exports.get(static.group("name"))
            if image is None:
                continue
            routed.append(static.group("name"))
            rest = rest.replace(static.group(0), "", 1)
            pieces.append(
                f'#[link(name = "{image}", kind = "raw-dylib", modifiers = "+verbatim")]\n'
                f'unsafe extern "C" {{\n{static.group(0)}}}\n'
            )
        if not pieces:
            return match.group(0)
        if rest.strip():
            pieces.insert(0, f'unsafe extern "C" {{\n{rest}}}\n')
        return "".join(pieces)

    return _BLOCK_RE.sub(block, text), routed


def main(argv: list[str]) -> int:
    if len(argv) != 4 or argv[2] not in _SPEC_ARCHES:
        print(__doc__, file=sys.stderr)
        return 2
    path, arch, root = Path(argv[1]), argv[2], Path(argv[3])
    text = path.read_text()
    new, routed = rewrite(text, data_exports(root, arch))
    if new != text:
        path.write_text(new)
    print(f"  data imports via raw-dylib: {', '.join(sorted(routed)) or 'none'}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
