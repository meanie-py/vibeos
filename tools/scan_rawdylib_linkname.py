#!/usr/bin/env python3
"""scan_rawdylib_linkname.py — a raw-dylib link name that resolves to a file that isn't there.

THE INCIDENT
    `ks.sys`, `pcix.sys` and `ramdisk.sys` shipped an import descriptor naming
    `ntoskrnl.dll`, a module that does not exist. Every boot, FreeLDR's PE loader logged

        PeLdrLoadImage('...\\drivers\\ntoskrnl.dll') failed
        PeLdrpLoadAndScanReferencedDll() failed
        PeLdrScanImportDescriptorTable() failed

    three times, and nobody saw it because freeldr's debug channel was itself silent.
    The cause is one line:

        #[link(name = "ntoskrnl", kind = "raw-dylib")]

    rustc appends `.dll` to a raw-dylib link name **unconditionally**. It does not look
    for an existing extension, and it does not check that the result exists. The kernel
    is `ntoskrnl.exe`.

WHY A SCANNER
    Because the obvious fix is also wrong, and shipped. The first repair changed the
    name to `"ntoskrnl.exe"` — which reads correctly, matches the 134 bindgen-emitted
    sites when you grep only the `name =` field, and produces `ntoskrnl.exe.dll`. It was
    caught by `llvm-readobj --coff-imports` on the rebuilt binary, not by review, not by
    the compiler, and not by the link (raw-dylib invents the descriptor, so there is no
    unresolved symbol to trip over). A class that survives its own first fix is a class
    that needs a machine watching it.

    The correct spelling is `name = "ntoskrnl.exe"` **plus** `modifiers = "+verbatim"`.

WHAT COUNTS AS A FINDING
    Resolve each attribute the way rustc does — `name` verbatim if `modifiers` contains
    `+verbatim`, otherwise `name + ".dll"` — and report three shapes:

      DOUBLE_EXT   the name already ends in .dll/.exe/.sys and +verbatim is absent, so
                   the resolved name carries two extensions (`ntoskrnl.exe.dll`). Always
                   wrong; there is no module in any Windows tree shaped like that.
      WRONG_EXT    a bare name whose real module is NOT a .dll (`ntoskrnl` -> the loader
                   asks for `ntoskrnl.dll`, the file is `ntoskrnl.exe`). This is the
                   original bug.
      NO_EXT       +verbatim on a name with no extension at all, so nothing appends one
                   and the descriptor names an extensionless file.

    A bare name whose module really is a .dll (`ntdll` -> `ntdll.dll`) is CORRECT and is
    not reported — the rule is "the resolved name must match the real filename", not
    "always pass +verbatim". Getting that backwards would flag the one healthy site in
    the tree and teach the next agent to "fix" it into `ntdll.dll.dll`.
"""

import argparse
import re
import sys
from pathlib import Path

# Modules imported from Rust whose on-disk name is NOT <name>.dll. Keep this list
# short and evidence-based: every entry is a module something in sdk/lib/rust
# actually imports, spelled as its real filename.
NON_DLL_MODULES = {
    "ntoskrnl": "ntoskrnl.exe",
    "win32k": "win32k.sys",
    "classpnp": "classpnp.sys",
    "scsiport": "scsiport.sys",
    "storport": "storport.sys",
    "videoprt": "videoprt.sys",
    "ndis": "ndis.sys",
    "usbport": "usbport.sys",
    "usbd": "usbd.sys",
    "ks": "ks.sys",
    "portcls": "portcls.sys",
    "bootvid": "bootvid.dll",  # really is a .dll; listed so nobody "fixes" it
}
NON_DLL_MODULES = {k: v for k, v in NON_DLL_MODULES.items() if not v.endswith(".dll")}

KNOWN_EXTS = (".dll", ".exe", ".sys", ".drv", ".ocx", ".cpl")

# `#[link(...)]` with a raw-dylib kind, across line breaks.
LINK_RE = re.compile(r"#\s*\[\s*link\s*\((?P<body>[^]]*?)\)\s*\]", re.S)
NAME_RE = re.compile(r'name\s*=\s*"(?P<name>[^"]*)"')
KIND_RE = re.compile(r'kind\s*=\s*"raw-dylib"')
MODS_RE = re.compile(r'modifiers\s*=\s*"(?P<mods>[^"]*)"')


def resolve(name: str, verbatim: bool) -> str:
    """What rustc will actually put in the import descriptor."""
    return name if verbatim else name + ".dll"


def scan_text(text: str):
    """Yield (lineno, kind, name, resolved, hint) for each bad raw-dylib attribute."""
    for m in LINK_RE.finditer(text):
        body = m.group("body")
        if not KIND_RE.search(body):
            continue
        nm = NAME_RE.search(body)
        if not nm:
            continue
        name = nm.group("name")
        mods = MODS_RE.search(body)
        verbatim = bool(mods and "+verbatim" in mods.group("mods"))
        lineno = text.count("\n", 0, m.start()) + 1
        resolved = resolve(name, verbatim)
        lower = name.lower()

        if lower.endswith(KNOWN_EXTS) and not verbatim:
            yield (
                lineno,
                "DOUBLE_EXT",
                name,
                resolved,
                'add modifiers = "+verbatim" — rustc appends .dll unconditionally',
            )
        elif not verbatim and lower in NON_DLL_MODULES:
            real = NON_DLL_MODULES[lower]
            yield (
                lineno,
                "WRONG_EXT",
                name,
                resolved,
                f'the module is {real} — write name = "{real}", modifiers = "+verbatim"',
            )
        elif verbatim and not lower.endswith(KNOWN_EXTS):
            yield (
                lineno,
                "NO_EXT",
                name,
                resolved,
                "+verbatim suppresses the .dll rustc would append, so the name "
                "must already carry the real extension",
            )


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--root", action="append", help="roots to scan (default: sdk)")
    ap.add_argument("--self-test", action="store_true", help="run the built-in cases")
    args = ap.parse_args()

    if args.self_test:
        return self_test()

    repo = Path(__file__).resolve().parent.parent
    roots = [repo / r for r in (args.root or ["sdk"])]

    findings = files = 0
    for root in roots:
        for path in sorted(root.rglob("*.rs")):
            if "/target/" in str(path):
                continue
            try:
                text = path.read_text(encoding="utf-8", errors="replace")
            except OSError:
                continue
            files += 1
            try:
                rel = path.relative_to(repo)
            except ValueError:
                rel = path  # --root outside the repo (used to replay a historical tree)
            for lineno, kind, name, resolved, hint in scan_text(text):
                print(
                    f'{rel}:{lineno}: {kind} raw-dylib name="{name}" resolves to '
                    f"'{resolved}' — {hint}"
                )
                findings += 1

    print(f"scanned {files} .rs files, {findings} findings", file=sys.stderr)
    return 0


def self_test() -> int:
    cases = [
        ('#[link(name = "ntoskrnl", kind = "raw-dylib")]', "WRONG_EXT"),
        ('#[link(name = "ntoskrnl.exe", kind = "raw-dylib")]', "DOUBLE_EXT"),
        ('#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]', None),
        ('#[link(name = "ntdll", kind = "raw-dylib")]', None),  # ntdll.dll is real
        ('#[link(name = "hal", kind = "raw-dylib")]', None),  # hal.dll is real
        ('#[link(name = "win32k", kind = "raw-dylib")]', "WRONG_EXT"),
        ('#[link(name = "foo", kind = "raw-dylib", modifiers = "+verbatim")]', "NO_EXT"),
        ('#[link(name = "ntoskrnl", kind = "static")]', None),  # not raw-dylib
        ('#[link(\n    name = "ntoskrnl.exe",\n    kind = "raw-dylib"\n)]', "DOUBLE_EXT"),
    ]
    bad = 0
    for src, want in cases:
        got = [k for _, k, *_ in scan_text(src)]
        got1 = got[0] if got else None
        ok = got1 == want
        bad += not ok
        print(f"{'ok  ' if ok else 'FAIL'} want={want!r:12} got={got1!r:12} {src[:58]!r}")
    print(f"\n{'ALL PASS' if not bad else 'FAILED'} ({bad} failures)")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
