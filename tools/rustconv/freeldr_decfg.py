#!/usr/bin/env python3
"""freeldr_decfg — resolve C-#ifdef-derived `#[cfg(feature="X")]` attributes
that rustconv emitted in *expression* position (illegal on stable rustc:
"attributes on expressions are experimental").

For the freeldr smoke path these features are OFF:
  * NTLDR_PROGRESSBAR — only the ntldr/setupldr bootmgr variant defines it;
    plain freeldr.sys / uefildr (what smoke boots) does not.
  * UEFIBOOT — i386/amd64 livecd boot BIOS freeldr (feature off); the arm64
    uefildr difference here is cosmetic progress-bar geometry, so resolving
    to the BIOS variant keeps all 3 arches booting to `shell ready`.

So the correct, deterministic collapse is:
  `#[cfg(feature = "X")] <lineA>`            → delete attr + lineA (off)
  `#[cfg(not(feature = "X"))] <lineB>`       → delete just the attr, keep lineB
This works whether the guarded thing is a statement or a call argument
(both are the positions rustconv produced). Idempotent.

Usage: python3 freeldr_decfg.py <file.rs> [<file2.rs> ...]
       [--features NTLDR_PROGRESSBAR,UEFIBOOT]   (default: those two)
"""
import re
import sys

DEFAULT = ["NTLDR_PROGRESSBAR", "UEFIBOOT"]


def fix(path, feats):
    s = open(path).read()
    orig = s
    for f in feats:
        fe = re.escape(f)
        # 1. `#[cfg(feature = "F")]` + its single following physical line: drop both
        s = re.sub(
            r'[ \t]*#\[cfg\(feature = "' + fe + r'"\)\]\n[^\n]*\n',
            '', s)
        # 2. `#[cfg(not(feature = "F"))]` attribute line only: drop the attr,
        #    keep the line it guards (un-guarded = the active variant)
        s = re.sub(
            r'[ \t]*#\[cfg\(not\(feature = "' + fe + r'"\)\)\]\n',
            '', s)
    if s != orig:
        open(path, "w").write(s)
    print(f"{path}: decfg {'changed' if s != orig else 'noop'}")


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    feats = DEFAULT
    for a in sys.argv[1:]:
        if a.startswith("--features"):
            feats = a.split("=", 1)[1].split(",") if "=" in a else DEFAULT
    for p in args:
        fix(p, feats)
