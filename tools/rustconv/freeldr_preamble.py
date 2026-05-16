#!/usr/bin/env python3
"""freeldr_preamble — normalize a rustconv-translated freeldr file's preamble
for crate::frldr keystone integration. Recurring across mm/fs/ui units.

  * Drop bogus ntkrnl_sys umbrella globs (globals/constants/mm/ex/rtl) — these
    are ntoskrnl surfaces, irrelevant to the freeldr freestanding binary.
  * Ensure `use ntkrnl_sys::base::*; use core::ffi::c_void; use crate::frldr::*;`.
  * In `extern "C"` blocks, delete declarations for symbols crate::frldr
    *exports* (would clash with the glob): FrLdrTempAlloc/Free, min, and the
    keystone struct/type names. KEEP the variadic debug decls (TRACE/ERR/
    WARN/FIXME/BugCheck/DbgDumpBuffer) — frldr provides those as non-pub
    #[no_mangle] symbols, so the decl is needed for call typecheck and the
    link resolves to the stub.
  * Normalize the first param of variadic debug decls to `*const i8`.

Idempotent. Usage: python3 freeldr_preamble.py <file.rs> [<file2.rs> ...]
"""
import re
import sys
from pathlib import Path

# Symbols crate::frldr re-exports (pub) → must NOT also be `extern`-declared.
FRLDR_EXPORTS = {
    "FrLdrTempAlloc", "FrLdrTempFree", "min",
}
DEBUG_VARIADIC = {"TRACE", "ERR", "WARN", "FIXME", "DPRINTM",
                  "BugCheck", "printf", "DbgPrint", "DbgPrint2"}


def fix(path: Path) -> None:
    s = path.read_text()
    lines = s.split("\n")
    out: list[str] = []
    in_extern = False
    have_base = have_cvoid = have_frldr = False

    for ln in lines:
        st = ln.strip()

        # drop bogus ntkrnl umbrella imports
        if re.match(r'use ntkrnl_sys::(globals|constants|mm|ex|rtl|ke|ob|io|ps|se|cc|cm)::\*;', st):
            continue
        if st == 'use ntkrnl_sys::base::*;':
            have_base = True
        if st.startswith('use core::ffi::c_void'):
            have_cvoid = True
        if st.startswith('use crate::frldr::'):
            have_frldr = True

        if st.startswith('extern "C"') and '{' in st and '}' not in st:
            in_extern = True
            out.append(ln)
            continue
        if in_extern:
            if st == '}':
                in_extern = False
                out.append(ln)
                continue
            m = re.match(r'(?:pub\s+)?fn\s+([A-Za-z_]\w*)\s*\(', st)
            ms = re.match(r'(?:pub\s+)?static\s+(?:mut\s+)?([A-Za-z_]\w*)', st)
            nm = m.group(1) if m else (ms.group(1) if ms else None)
            if nm and nm in FRLDR_EXPORTS:
                continue  # frldr exports it — drop the clashing decl
            # normalize variadic debug first-arg → *const i8
            if m and nm in DEBUG_VARIADIC and '...' in st:
                ln = re.sub(
                    r'(fn\s+' + re.escape(nm) + r'\()\s*[A-Za-z_]\w*\s*:\s*[^,)]+,',
                    r'\1fmt: *const i8,', ln)
            out.append(ln)
            continue
        out.append(ln)

    s = "\n".join(out)

    # ensure the three needed imports, right after the first `use` line
    inject = []
    if not have_base:
        inject.append("use ntkrnl_sys::base::*;")
    if not have_cvoid:
        inject.append("use core::ffi::c_void;")
    if not have_frldr:
        inject.append("use crate::frldr::*;")
    if inject:
        m = re.search(r'^use [^\n]*\n', s, re.M)
        at = m.end() if m else 0
        s = s[:at] + "\n".join(inject) + "\n" + s[at:]

    # ensure a variadic decl exists for every debug fn the file *calls*
    # (frldr provides the non-pub #[no_mangle] symbol; the decl is only for
    # call-site typecheck). DbgPrint returns i32, the rest are VOID.
    missing = []
    for nm in ("TRACE", "ERR", "WARN", "FIXME", "DPRINTM", "DbgPrint"):
        if re.search(r'\b' + nm + r'\s*\(', s) and not re.search(
                r'\bfn\s+' + nm + r'\s*\(', s):
            ret = " -> i32" if nm == "DbgPrint" else ""
            missing.append(f"    fn {nm}(fmt: *const i8, ...){ret};")
    if missing:
        blk = "extern \"C\" {\n" + "\n".join(missing) + "\n}\n"
        m = re.search(r'(^use [^\n]*\n)(?!use )', s, re.M)
        # insert after the contiguous run of `use` lines
        mu = list(re.finditer(r'^use [^\n]*\n', s, re.M))
        at = mu[-1].end() if mu else 0
        s = s[:at] + blk + s[at:]

    path.write_text(s)
    print(f"{path.name}: preamble normalized"
          f"{' (+'+','.join(x.split('::')[1] for x in inject)+')' if inject else ''}")


if __name__ == "__main__":
    for f in sys.argv[1:]:
        fix(Path(f))
