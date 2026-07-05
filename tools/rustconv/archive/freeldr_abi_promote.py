#!/usr/bin/env python3
"""freeldr_abi_promote — mechanical ABI fixups for freeldr-rust integration.

The recurring per-file integration pattern (see memory
feedback_freeldr_dll_translation):

  1. FLDRAPI is empty on the non-MSVC build → every freeldr header-declared
     function is plain **cdecl**. rustconv emits some as private `unsafe fn`
     and some as `#[no_mangle] pub unsafe extern "system" fn` (i386 stdcall
     `_Name@N`) — both WRONG. Promote every C-header-declared fn to
     `#[no_mangle] pub unsafe extern "C" fn`. File-static C helpers (NOT in
     any header) are left private so they can't collide across units.
  2. Any leftover `extern "system"` in a fn def → `extern "C"`.
  3. Variadic debug decls `fn TRACE/ERR/WARN/FIXME(<f>: <ty>, ...)` — the
     first (format) param retyped to `*const i8` so the `as *const i8` /
     `as PCHAR` / `as _` call-site casts all typecheck.

Deterministic, idempotent. AI-bug hunting + keystone stay manual.

Usage:
  python3 freeldr_abi_promote.py <file.rs> <header.h> [<header2.h> ...]
"""
import re
import sys
from pathlib import Path


def header_symbols(headers: list[Path]) -> set[str]:
    """Extract function names declared in C headers (`Type Name(...)`)."""
    syms: set[str] = set()
    decl = re.compile(r'\b([A-Za-z_]\w*)\s*\(', re.M)
    for h in headers:
        txt = h.read_text(errors="replace")
        # strip // and /* */ comments and preprocessor lines
        txt = re.sub(r'//[^\n]*', '', txt)
        txt = re.sub(r'/\*.*?\*/', '', txt, flags=re.S)
        txt = re.sub(r'^\s*#[^\n]*', '', txt, flags=re.M)
        # collapse so multi-line prototypes (return type \n Name( \n args )) join
        flat = re.sub(r'\s+', ' ', txt)
        # each `;`-terminated statement that has a (...) and isn't a typedef
        for stmt in flat.split(';'):
            stmt = stmt.strip()
            if not stmt or '(' not in stmt or stmt.startswith('typedef'):
                continue
            # the function name is the identifier immediately before `(`
            mm = re.search(r'\b([A-Za-z_]\w*)\s*\(', stmt)
            if mm:
                name = mm.group(1)
                if name not in {
                    'if', 'while', 'for', 'switch', 'sizeof', 'return',
                    'defined', 'VOID', 'struct', 'union', 'enum',
                }:
                    syms.add(name)
    return syms


def promote(src: str, syms: set[str]) -> tuple[str, int]:
    n = 0
    lines = src.split('\n')
    out: list[str] = []
    i = 0
    fn_def = re.compile(
        r'^(?P<indent>\s*)(?P<pub>pub\s+)?(?P<unsafe>unsafe\s+)?'
        r'(?:extern\s+"(?P<abi>[A-Za-z]+)"\s+)?fn\s+(?P<name>[A-Za-z_]\w*)\s*[(<]'
    )
    in_extern = False
    while i < len(lines):
        line = lines[i]
        st = line.strip()
        # never promote declarations inside an `extern "..." { }` block
        if not in_extern and re.match(r'extern\s+"[^"]+"\s*\{', st) and '}' not in st:
            in_extern = True
            out.append(line)
            i += 1
            continue
        if in_extern:
            if st == '}':
                in_extern = False
            out.append(line)
            i += 1
            continue
        m = fn_def.match(line)
        if m and m.group('name') in syms and 'fn ' in line:
            name = m.group('name')
            indent = m.group('indent')
            # collect any existing attribute lines immediately above
            has_no_mangle = (out and out[-1].strip().startswith('#[') and
                             'no_mangle' in out[-1])
            # rebuild the signature head as the canonical cdecl export
            rest = line[m.end('name'):]  # the '(' or '<' and onward
            opener = line[m.start('name') + len(name):m.end('name')]  # spacing
            new_sig = f'{indent}pub unsafe extern "C" fn {name}{opener}{rest}'
            if not has_no_mangle:
                out.append(f'{indent}#[no_mangle]')
            out.append(new_sig)
            n += 1
            i += 1
            continue
        out.append(line)
        i += 1
    src = '\n'.join(out)

    # 2. any leftover extern "system" in a fn def → extern "C"
    src, c2 = re.subn(r'extern\s+"system"\s+fn', 'extern "C" fn', src)
    n += c2

    # 3. variadic debug-channel decls: first param → *const i8
    def fix_var(mm: re.Match) -> str:
        return f'{mm.group("fn")}(fmt: *const i8, ...)'
    src, c3 = re.subn(
        r'(?P<fn>\bfn\s+(?:TRACE|ERR|WARN|FIXME|DPRINTM|BugCheck))'
        r'\([A-Za-z_]\w*\s*:\s*[^,)]+,\s*\.\.\.\)',
        fix_var, src)
    n += c3
    return src, n


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__)
        return 2
    rs = Path(sys.argv[1])
    headers = [Path(h) for h in sys.argv[2:]]
    syms = header_symbols(headers)
    src = rs.read_text()
    new, n = promote(src, syms)
    if new != src:
        rs.write_text(new)
    print(f"{rs.name}: {n} ABI fixups ({len(syms)} header symbols)")
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
