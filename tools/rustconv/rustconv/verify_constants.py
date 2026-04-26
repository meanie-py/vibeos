"""
verify_constants.py — Cross-check Rust constant values against C headers.

Usage:
    python -m rustconv verify-constants [file.rs ...]
    python -m rustconv verify-constants --fix [file.rs ...]

Scans Rust files for `const NAME: TYPE = VALUE;` and compares against
a catalog of C #define values from kernel headers.
"""

import re
import sys
import struct
from pathlib import Path


# ── C Header Catalog Builder ──────────────────────────────────────────

def parse_c_hex(s: str) -> int | None:
    """Parse a C numeric literal to int."""
    s = s.strip()
    # Strip inline C comments and trailing semicolons/suffixes
    if '/*' in s:
        s = s[:s.index('/*')].strip()
    s = s.rstrip(';').rstrip('ULul').strip()
    try:
        if s.startswith('0x') or s.startswith('0X'):
            return int(s, 16)
        if s.startswith('0') and len(s) > 1 and s[1:].isdigit():
            return int(s, 8)
        if s.lstrip('-').isdigit():
            return int(s)
    except ValueError:
        pass
    return None


def parse_c_multichar(s: str) -> int | None:
    """Parse C multi-character literal like 'abcd' to u32."""
    s = s.strip()
    if s.startswith("'") and s.endswith("'"):
        chars = s[1:-1]
        if 1 <= len(chars) <= 4:
            # Pad to 4 chars from left (C pads with nulls on left)
            chars = chars.rjust(4, '\0')
            val = 0
            for c in chars:
                val = (val << 8) | ord(c)
            return val & 0xFFFFFFFF
    return None


def parse_ntstatus_cast(s: str) -> int | None:
    """Parse ((NTSTATUS)0xC0000001) pattern — reject macro calls and expressions."""
    s = s.strip()
    # Reject expressions with operators or macro-call commas
    if any(op in s for op in [',', '|', '+', '<<', '>>']):
        return None
    m = re.search(r'0x[0-9A-Fa-f]+', s)
    if m:
        return int(m.group(), 16)
    return None


def build_c_catalog(reactos_root: Path) -> dict[str, int]:
    """Build name→value mapping from C headers."""
    catalog = {}

    header_files = [
        reactos_root / 'sdk/include/xdk/iotypes.h',
        reactos_root / 'sdk/include/xdk/setypes.h',
        reactos_root / 'sdk/include/psdk/ntstatus.h',
        reactos_root / 'ntoskrnl/include/internal/tag.h',
        reactos_root / 'ntoskrnl/include/internal/io.h',
        reactos_root / 'sdk/include/xdk/ketypes.h',
        reactos_root / 'sdk/include/ndk/iotypes.h',
        reactos_root / 'sdk/include/reactos/drivers/ntddrdsk.h',
    ]

    define_re = re.compile(r'^#define\s+([A-Z_][A-Z0-9_]*)\s+(.+?)(?:\s*//.*)?$')

    for hfile in header_files:
        if not hfile.exists():
            continue
        for line in hfile.read_text(errors='replace').splitlines():
            m = define_re.match(line.strip())
            if not m:
                continue
            name, expr = m.group(1), m.group(2).strip()

            # Try hex/decimal
            val = parse_c_hex(expr)
            if val is not None:
                catalog[name] = val
                continue

            # Try multi-char literal
            val = parse_c_multichar(expr)
            if val is not None:
                catalog[name] = val
                continue

            # Try NTSTATUS cast
            val = parse_ntstatus_cast(expr)
            if val is not None:
                catalog[name] = val
                continue

    return catalog


# ── Rust Constant Parser ─────────────────────────────────────────────

def parse_rust_const(line: str) -> tuple[str, int] | None:
    """Parse a Rust const definition, return (name, value) or None."""
    # Pattern: const NAME: TYPE = VALUE;
    m = re.match(r'const\s+([A-Z_][A-Z0-9_]*)\s*:\s*\w+\s*=\s*(.+?)\s*;', line.strip())
    if not m:
        return None
    name, expr = m.group(1), m.group(2).strip()

    # Hex literal
    hm = re.match(r'(0x[0-9A-Fa-f_]+)(?:_?u\d+)?', expr)
    if hm:
        return name, int(hm.group(1).replace('_', ''), 16)

    # Decimal literal
    if expr.replace('_', '').lstrip('-').isdigit():
        val = int(expr.replace('_', ''))
        if val < 0:
            val = val & 0xFFFFFFFF
        return name, val

    # u32::from_le_bytes(*b"abcd") or from_le_bytes([...])
    m2 = re.search(r'from_le_bytes\(\*?b"(.{4})"\)', expr)
    if m2:
        return name, struct.unpack('<I', m2.group(1).encode('latin-1'))[0]
    m3 = re.search(r'from_le_bytes\(\[(.+)\]\)', expr)
    if m3:
        parts = [p.strip().strip("b'\"").replace("\\", "") for p in m3.group(1).split(',')]
        if len(parts) == 4:
            try:
                bs = bytes(ord(p[0]) if len(p) == 1 else int(p, 0) for p in parts)
                return name, struct.unpack('<I', bs)[0]
            except (ValueError, IndexError):
                pass

    # Cast pattern: 0xXXXX_u32 as i32
    m4 = re.match(r'(0x[0-9A-Fa-f_]+)_?u\d+\s+as\s+i\d+', expr)
    if m4:
        return name, int(m4.group(1).replace('_', ''), 16)

    return None


def fix_const_line(line: str, name: str, c_val: int) -> str:
    """Rewrite a const definition line with the correct C value."""
    # Match the const pattern and replace the value
    m = re.match(r'(const\s+' + re.escape(name) + r'\s*:\s*\w+\s*=\s*)(.+?)(;.*)', line.rstrip())
    if not m:
        return line
    prefix, _old_val, suffix = m.group(1), m.group(2), m.group(3)

    # Determine the best representation
    rust_type = re.search(r':\s*(\w+)', prefix)
    type_name = rust_type.group(1) if rust_type else 'ULONG'

    if type_name in ('NTSTATUS', 'i32') and c_val > 0x7FFFFFFF:
        # NTSTATUS: use unsigned cast pattern
        new_val = f'0x{c_val:08X}_u32 as i32'
    elif c_val > 0xFFFF:
        new_val = f'0x{c_val:08X}'
    elif c_val > 0xFF:
        new_val = f'0x{c_val:04X}'
    else:
        new_val = f'0x{c_val:02X}'

    return prefix + new_val + suffix


# Constants that differ per architecture (#ifdef _WIN64 etc.) — skip auto-fix
ARCH_DEPENDENT_CONSTANTS = {
    'PORT_MAXIMUM_MESSAGE_LENGTH',  # 512 on 64-bit, 256 on 32-bit
}


def verify_file(rs_path: Path, catalog: dict[str, int], fix: bool = False) -> list[dict]:
    """Verify constants in a single Rust file. Returns list of mismatches."""
    mismatches = []
    lines = rs_path.read_text().splitlines()
    modified = False

    for lineno, line in enumerate(lines, 1):
        parsed = parse_rust_const(line)
        if not parsed:
            continue
        name, rust_val = parsed

        if name in ARCH_DEPENDENT_CONSTANTS:
            continue

        if name in catalog:
            c_val = catalog[name]
            if rust_val != c_val:
                mismatches.append({
                    'file': str(rs_path),
                    'line': lineno,
                    'name': name,
                    'rust_val': f'0x{rust_val:08X}',
                    'c_val': f'0x{c_val:08X}',
                })
                if fix:
                    lines[lineno - 1] = fix_const_line(line, name, c_val)
                    modified = True

    if fix and modified:
        rs_path.write_text('\n'.join(lines) + '\n')

    return mismatches


def main():
    import argparse
    parser = argparse.ArgumentParser(description='Verify Rust constants against C headers')
    parser.add_argument('files', nargs='*', help='Rust files to check (default: all)')
    parser.add_argument('--fix', action='store_true', help='Auto-fix mismatches')
    args = parser.parse_args()

    reactos_root = Path(__file__).parent.parent.parent.parent  # tools/rustconv → reactos/

    catalog = build_c_catalog(reactos_root)
    print(f"C catalog: {len(catalog)} constants from headers")

    if args.files:
        rs_files = [Path(f) for f in args.files]
    else:
        rs_files = sorted(
            (reactos_root / 'sdk/lib/rust/ntoskrnl-rust/src').rglob('*.rs')
        )

    total_mismatches = 0
    files_fixed = 0
    for rs_file in rs_files:
        mismatches = verify_file(rs_file, catalog, fix=args.fix)
        if mismatches:
            for m in mismatches:
                action = "FIXED" if args.fix else "MISMATCH"
                print(f"  {action}: {m['file']}:{m['line']}: {m['name']}: "
                      f"Rust={m['rust_val']} → C={m['c_val']}")
            total_mismatches += len(mismatches)
            if args.fix:
                files_fixed += 1

    if args.fix:
        print(f"\nFixed {total_mismatches} constants in {files_fixed} files")
    else:
        print(f"\n{total_mismatches} mismatches found across {len(rs_files)} files")
    return 1 if total_mismatches > 0 and not args.fix else 0


if __name__ == '__main__':
    sys.exit(main())
