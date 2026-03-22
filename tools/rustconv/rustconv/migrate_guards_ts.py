"""
migrate_guards_ts.py — Tree-sitter based guard migration for single-pair functions.

Only transforms functions with EXACTLY 1 acquire + 1 release of a given lock type.
This guarantees the OldIrql variable is exclusive to the pair — no sharing issues.

Usage:
    python tools/rustconv/rustconv/migrate_guards_ts.py --scan
    python tools/rustconv/rustconv/migrate_guards_ts.py --fix
    python tools/rustconv/rustconv/migrate_guards_ts.py --fix file.rs
"""

import re
import sys
from pathlib import Path
from dataclasses import dataclass

import tree_sitter_rust as tsrust
import tree_sitter as ts

RUST_LANGUAGE = ts.Language(tsrust.language())
PARSER = ts.Parser(RUST_LANGUAGE)

LOCK_PATTERNS = [
    {
        'name': 'pfn',
        'acquire': 'MiAcquirePfnLock',
        'release': 'MiReleasePfnLock',
        'guard': 'crate::ARM3::helpers::PfnLockGuard::acquire()',
    },
    {
        'name': 'spinlock',
        'acquire': 'KeAcquireSpinLock',
        'release': 'KeReleaseSpinLock',
        'guard': None,  # needs lock expr — handled specially
    },
    {
        'name': 'irql',
        'acquire': 'KeRaiseIrql',
        'release': 'KeLowerIrql',
        'guard': None,  # needs level — handled specially
    },
]


@dataclass
class FunctionMatch:
    file: str
    func_name: str
    pattern_name: str
    acquire_line: int  # 1-indexed
    release_line: int
    acquire_text: str
    release_text: str


def find_single_pair_functions(filepath: Path) -> list[FunctionMatch]:
    """Find functions with exactly 1 acquire + 1 release of a lock pattern."""
    source = filepath.read_bytes()
    matches = []

    # Quick text check — skip files without any lock calls
    has_any = False
    for pat in LOCK_PATTERNS:
        if pat['acquire'].encode() in source:
            has_any = True
            break
    if not has_any:
        return []

    tree = PARSER.parse(source)

    def walk_functions(node):
        if node.type == 'function_item':
            name_node = node.child_by_field_name('name')
            body_node = node.child_by_field_name('body')
            if not name_node or not body_node:
                return

            func_name = source[name_node.start_byte:name_node.end_byte].decode()
            body_text = source[body_node.start_byte:body_node.end_byte].decode()
            body_lines = source[body_node.start_byte:body_node.end_byte].decode().splitlines()
            body_start_line = body_node.start_point[0]  # 0-indexed

            for pat in LOCK_PATTERNS:
                acq_call = pat['acquire'] + '('
                rel_call = pat['release'] + '('

                acq_count = body_text.count(acq_call)
                rel_count = body_text.count(rel_call)

                if acq_count == 1 and rel_count == 1:
                    # Find exact lines
                    acq_line = None
                    rel_line = None
                    acq_text = None
                    rel_text = None

                    for i, line in enumerate(body_lines):
                        if acq_call in line and '//' not in line.split(acq_call)[0]:
                            acq_line = body_start_line + i + 1  # 1-indexed
                            acq_text = line.strip()
                        if rel_call in line and '//' not in line.split(rel_call)[0]:
                            rel_line = body_start_line + i + 1
                            rel_text = line.strip()

                    if acq_line and rel_line:
                        # Extract the irql variable name from the acquire
                        irql_var = None
                        m = re.match(r'(?:let\s+(?:mut\s+)?)?(\w+)\s*=', acq_text)
                        if m:
                            irql_var = m.group(1)

                        # Safety: check irql var isn't used outside acquire/release
                        skip = False
                        if irql_var:
                            for bi, bline in enumerate(body_lines):
                                line_num = body_start_line + bi + 1
                                if line_num == acq_line or line_num == rel_line:
                                    continue
                                if bline.strip().startswith('//'):
                                    continue
                                if irql_var in bline:
                                    skip = True
                                    break

                        if not skip and acq_line < rel_line and 'pushlock.rs' not in str(filepath) and 'context.rs' not in str(filepath) and 'except.rs' not in str(filepath) and 'section.rs' not in str(filepath) and 'exp.rs' not in str(filepath) and "*" not in acq_text.split("=")[0]:
                            matches.append(FunctionMatch(
                                file=str(filepath),
                                func_name=func_name,
                                pattern_name=pat['name'],
                                acquire_line=acq_line,
                                release_line=rel_line,
                                acquire_text=acq_text,
                                release_text=rel_text,
                            ))

        for child in node.children:
            walk_functions(child)

    walk_functions(tree.root_node)
    return matches


def apply_fix(filepath: Path, matches: list[FunctionMatch]) -> int:
    """Apply guard transforms to matched functions."""
    if not matches:
        return 0

    lines = filepath.read_text().splitlines()
    count = 0

    # Process in reverse line order to preserve line numbers
    matches.sort(key=lambda m: m.release_line, reverse=True)

    for m in matches:
        acq_idx = m.acquire_line - 1
        rel_idx = m.release_line - 1
        acq_line = lines[acq_idx]
        rel_line = lines[rel_idx]
        indent = ' ' * (len(acq_line) - len(acq_line.lstrip()))

        # Build guard name and acquire replacement
        if m.pattern_name == 'pfn':
            guard_name = 'pfn_guard'
            guard = f'{indent}let {guard_name} = unsafe {{ crate::ARM3::helpers::PfnLockGuard::acquire() }};'
        elif m.pattern_name == 'spinlock':
            sm = re.search(r'KeAcquireSpinLock\((.+?),', acq_line)
            if not sm:
                continue
            lock_expr = sm.group(1).strip()
            guard_name = 'spin_guard'
            guard = f'{indent}let {guard_name} = unsafe {{ ntkrnl_sys::guards::SpinLockGuard::acquire({lock_expr}) }};'
        elif m.pattern_name == 'irql':
            sm = re.search(r'KeRaiseIrql\((.+?),', acq_line)
            if not sm:
                continue
            level = sm.group(1).strip()
            guard_name = 'irql_guard'
            guard = f'{indent}let {guard_name} = unsafe {{ ntkrnl_sys::guards::IrqlGuard::raise({level}) }};'
        else:
            continue

        # Replace release with drop(guard) — preserves exact lock duration
        rel_indent = ' ' * (len(rel_line) - len(rel_line.lstrip()))
        lines[rel_idx] = f'{rel_indent}drop({guard_name});'
        # Comment out acquire, add guard after it
        lines[acq_idx] = f'{indent}// [guard] {acq_line.strip()}\n{guard}'
        count += 1

    filepath.write_text('\n'.join(lines) + '\n')
    return count


def main():
    import argparse
    parser = argparse.ArgumentParser(description='Tree-sitter guard migration (single-pair functions)')
    parser.add_argument('files', nargs='*')
    parser.add_argument('--scan', action='store_true')
    parser.add_argument('--fix', action='store_true')
    args = parser.parse_args()

    if not args.scan and not args.fix:
        args.scan = True

    root = Path(__file__).parent.parent.parent.parent

    if args.files:
        rs_files = [Path(f) for f in args.files]
    else:
        rs_files = sorted((root / 'sdk/lib/rust/ntoskrnl-rust/src').rglob('*.rs'))

    totals = {'pfn': 0, 'spinlock': 0, 'irql': 0}
    fixed_count = 0

    for f in rs_files:
        matches = find_single_pair_functions(f)
        for m in matches:
            totals[m.pattern_name] += 1
            if args.scan:
                print(f'  {Path(m.file).relative_to(root)}: {m.func_name} ({m.pattern_name}) '
                      f'L{m.acquire_line}→L{m.release_line}')

        if args.fix and matches:
            n = apply_fix(f, matches)
            if n > 0:
                print(f'  FIXED {Path(f).relative_to(root)}: {n} functions')
                fixed_count += n

    total = sum(totals.values())
    print(f'\nSummary: {total} single-pair functions')
    for k, v in totals.items():
        if v > 0:
            print(f'  {k}: {v}')
    if args.fix:
        print(f'  Transformed: {fixed_count}')


if __name__ == '__main__':
    sys.exit(main())
