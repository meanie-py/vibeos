"""
migrate_guards_ts.py — Tree-sitter based guard migration.

Handles three patterns:
  Single:     1 acquire, 1 release → guard + drop()
  EarlyReturn: 1 acquire, N releases → guard only (Drop handles all paths)
  Sequential: N acquires, N releases (non-overlapping) → N guards + N drops

Usage:
    python tools/rustconv/rustconv/migrate_guards_ts.py --scan
    python tools/rustconv/rustconv/migrate_guards_ts.py --fix
    python tools/rustconv/rustconv/migrate_guards_ts.py --fix file.rs
"""

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

import tree_sitter as ts
import tree_sitter_rust as tsrust

RUST_LANGUAGE = ts.Language(tsrust.language())
PARSER = ts.Parser(RUST_LANGUAGE)

LOCK_PATTERNS = [
    {"name": "pfn", "acquire": "MiAcquirePfnLock", "release": "MiReleasePfnLock"},
    {"name": "spinlock", "acquire": "KeAcquireSpinLock", "release": "KeReleaseSpinLock"},
    {"name": "irql", "acquire": "KeRaiseIrql", "release": "KeLowerIrql"},
]


@dataclass
class LockSite:
    line: int  # 1-indexed
    text: str  # stripped line text
    indent: int  # leading spaces


@dataclass
class FunctionMatch:
    file: str
    func_name: str
    pattern_name: str
    pattern_type: str  # 'single', 'early_return', 'sequential'
    acquires: list = field(default_factory=list)  # list of LockSite
    releases: list = field(default_factory=list)  # list of LockSite
    irql_var: str = ""


def find_lock_sites(body_lines, body_start_line, call_name):
    """Find all non-commented call sites of a function in body lines."""
    sites = []
    for i, line in enumerate(body_lines):
        call = call_name + "("
        if call in line:
            before_call = line.split(call)[0]
            if "//" not in before_call and "// [guard]" not in line:
                sites.append(
                    LockSite(
                        line=body_start_line + i + 1,
                        text=line.strip(),
                        indent=len(line) - len(line.lstrip()),
                    )
                )
    return sites


def extract_irql_var(acq_text):
    """Extract the OldIrql variable name from an acquire line."""
    m = re.match(r"(?:let\s+(?:mut\s+)?)?(\w+)\s*=", acq_text)
    return m.group(1) if m else None


def is_var_used_outside(body_lines, body_start_line, irql_var, exclude_lines):
    """Check if irql_var appears on any line not in exclude_lines."""
    if not irql_var:
        return False
    for i, line in enumerate(body_lines):
        line_num = body_start_line + i + 1
        if line_num in exclude_lines:
            continue
        if line.strip().startswith("//"):
            continue
        if irql_var in line:
            return True
    return False


def classify_function(filepath, func_name, body_lines, body_start_line, pat):
    """Classify a function's lock pattern and return a FunctionMatch or None."""
    acquires = find_lock_sites(body_lines, body_start_line, pat["acquire"])
    releases = find_lock_sites(body_lines, body_start_line, pat["release"])

    if not acquires:
        return None

    # Skip pointer-deref acquires (*OldIrql = ...)
    acquires = [a for a in acquires if "*" not in a.text.split("=")[0]]
    if not acquires:
        return None

    acq_count = len(acquires)
    rel_count = len(releases)

    # Extract irql var from first acquire
    irql_var = extract_irql_var(acquires[0].text)

    # Collect all lines involved
    all_lines = set(a.line for a in acquires) | set(r.line for r in releases)

    # Check if irql_var is used outside lock operations
    if is_var_used_outside(body_lines, body_start_line, irql_var, all_lines):
        return None

    # --- Pattern: Single (1 acquire, 1 release) ---
    if acq_count == 1 and rel_count == 1 and acquires[0].line < releases[0].line:
        return FunctionMatch(
            file=str(filepath),
            func_name=func_name,
            pattern_name=pat["name"],
            pattern_type="single",
            acquires=acquires,
            releases=releases,
            irql_var=irql_var or "",
        )

    # --- Pattern B: Early Return (1 acquire, N releases) ---
    # All releases must be after the acquire
    if acq_count == 1 and rel_count >= 2 and all(r.line > acquires[0].line for r in releases):
        return FunctionMatch(
            file=str(filepath),
            func_name=func_name,
            pattern_name=pat["name"],
            pattern_type="early_return",
            acquires=acquires,
            releases=releases,
            irql_var=irql_var or "",
        )

    # --- Pattern A: Sequential (N acquires, N releases, non-overlapping) ---
    if acq_count >= 2 and acq_count == rel_count:
        # Sort by line number
        acq_sorted = sorted(acquires, key=lambda s: s.line)
        rel_sorted = sorted(releases, key=lambda s: s.line)

        # Check each acquire[i] is followed by release[i] before acquire[i+1]
        sequential = True
        for j in range(acq_count):
            if acq_sorted[j].line >= rel_sorted[j].line:
                sequential = False
                break
            if j + 1 < acq_count and rel_sorted[j].line >= acq_sorted[j + 1].line:
                sequential = False
                break

        if sequential:
            # Check each pair's irql var
            all_vars_ok = True
            for j in range(acq_count):
                v = extract_irql_var(acq_sorted[j].text)
                if v != irql_var:
                    all_vars_ok = False
                    break
            if all_vars_ok:
                return FunctionMatch(
                    file=str(filepath),
                    func_name=func_name,
                    pattern_name=pat["name"],
                    pattern_type="sequential",
                    acquires=acq_sorted,
                    releases=rel_sorted,
                    irql_var=irql_var or "",
                )

    return None


def find_matches(filepath: Path) -> list[FunctionMatch]:
    """Find all transformable lock patterns in a file."""
    source = filepath.read_bytes()
    has_any = any(p["acquire"].encode() in source for p in LOCK_PATTERNS)
    if not has_any:
        return []

    tree = PARSER.parse(source)
    matches = []

    def walk(node):
        if node.type == "function_item":
            name_node = node.child_by_field_name("name")
            body_node = node.child_by_field_name("body")
            if not name_node or not body_node:
                return

            func_name = source[name_node.start_byte : name_node.end_byte].decode()
            body_text = source[body_node.start_byte : body_node.end_byte].decode()
            body_lines = body_text.splitlines()
            body_start = body_node.start_point[0]

            for pat in LOCK_PATTERNS:
                m = classify_function(filepath, func_name, body_lines, body_start, pat)
                if m:
                    matches.append(m)

        for child in node.children:
            walk(child)

    walk(tree.root_node)
    return matches


def make_guard_line(m: FunctionMatch, idx: int, acq_line_text: str) -> str | None:
    """Generate the guard creation line."""
    indent = " " * (len(acq_line_text) - len(acq_line_text.lstrip()))
    suffix = f"_{idx}" if len(m.acquires) > 1 else ""

    if m.pattern_name == "pfn":
        return (
            f"{indent}let pfn_guard{suffix} = unsafe "
            "{ crate::ARM3::helpers::PfnLockGuard::acquire() };"
        )
    elif m.pattern_name == "spinlock":
        sm = re.search(r"KeAcquireSpinLock\((.+?),", acq_line_text)
        if not sm:
            return None
        lock_expr = sm.group(1).strip()
        return (
            f"{indent}let spin_guard{suffix} = unsafe "
            f"{{ ntkrnl_sys::guards::SpinLockGuard::acquire({lock_expr}) }};"
        )
    elif m.pattern_name == "irql":
        sm = re.search(r"KeRaiseIrql\((.+?),", acq_line_text)
        if not sm:
            return None
        level = sm.group(1).strip()
        return (
            f"{indent}let irql_guard{suffix} = unsafe "
            f"{{ ntkrnl_sys::guards::IrqlGuard::raise({level}) }};"
        )
    return None


def guard_name(m: FunctionMatch, idx: int) -> str:
    suffix = f"_{idx}" if len(m.acquires) > 1 else ""
    if m.pattern_name == "pfn":
        return f"pfn_guard{suffix}"
    elif m.pattern_name == "spinlock":
        return f"spin_guard{suffix}"
    # pattern_name is always one of pfn/spinlock/irql (set from LOCK_PATTERNS)
    return f"irql_guard{suffix}"


def apply_fix(filepath: Path, matches: list[FunctionMatch]) -> int:
    """Apply guard transforms."""
    if not matches:
        return 0

    lines = filepath.read_text().splitlines()
    count = 0

    # Collect all line edits: {line_num: new_text or None for delete}
    edits = {}

    for m in matches:
        if m.pattern_type == "single":
            acq = m.acquires[0]
            rel = m.releases[0]
            acq_line = lines[acq.line - 1]
            rel_line = lines[rel.line - 1]

            guard_line = make_guard_line(m, 0, acq_line)
            if not guard_line:
                continue

            gn = guard_name(m, 0)
            indent = " " * (len(acq_line) - len(acq_line.lstrip()))
            rel_indent = " " * (len(rel_line) - len(rel_line.lstrip()))

            edits[acq.line] = f"{indent}// [guard] {acq_line.strip()}\n{guard_line}"
            edits[rel.line] = f"{rel_indent}drop({gn});"
            count += 1

        elif m.pattern_type == "early_return":
            acq = m.acquires[0]
            acq_line = lines[acq.line - 1]

            guard_line = make_guard_line(m, 0, acq_line)
            if not guard_line:
                continue

            indent = " " * (len(acq_line) - len(acq_line.lstrip()))
            edits[acq.line] = f"{indent}// [guard] {acq_line.strip()}\n{guard_line}"

            # Remove ALL releases — Drop handles them
            for rel in m.releases:
                rel_line = lines[rel.line - 1]
                rel_indent = " " * (len(rel_line) - len(rel_line.lstrip()))
                edits[rel.line] = f"{rel_indent}// [guard-drop] {rel_line.strip()}"

            count += 1

        elif m.pattern_type == "sequential":
            for idx, (acq, rel) in enumerate(zip(m.acquires, m.releases, strict=False)):
                acq_line = lines[acq.line - 1]
                rel_line = lines[rel.line - 1]

                guard_line = make_guard_line(m, idx, acq_line)
                if not guard_line:
                    continue

                gn = guard_name(m, idx)
                indent = " " * (len(acq_line) - len(acq_line.lstrip()))
                rel_indent = " " * (len(rel_line) - len(rel_line.lstrip()))

                edits[acq.line] = f"{indent}// [guard] {acq_line.strip()}\n{guard_line}"
                edits[rel.line] = f"{rel_indent}drop({gn});"
                count += 1

    # Apply edits
    new_lines = []
    for i, line in enumerate(lines):
        line_num = i + 1
        if line_num in edits:
            new_lines.append(edits[line_num])
        else:
            new_lines.append(line)

    filepath.write_text("\n".join(new_lines) + "\n")
    return count


def main():
    parser = argparse.ArgumentParser(description="Tree-sitter guard migration")
    parser.add_argument("files", nargs="*")
    parser.add_argument("--scan", action="store_true")
    parser.add_argument("--fix", action="store_true")
    args = parser.parse_args()

    if not args.scan and not args.fix:
        args.scan = True

    root = Path(__file__).parent.parent.parent.parent

    if args.files:
        rs_files = [Path(f) for f in args.files]
    else:
        rs_files = sorted((root / "sdk/lib/rust/ntoskrnl-rust/src").rglob("*.rs"))

    by_type = {"single": 0, "early_return": 0, "sequential": 0}
    by_pat = {"pfn": 0, "spinlock": 0, "irql": 0}
    fixed_count = 0

    for f in rs_files:
        matches = find_matches(f)
        for m in matches:
            by_type[m.pattern_type] += 1
            by_pat[m.pattern_name] += 1
            if args.scan:
                pairs = len(m.acquires)
                print(
                    f"  {Path(m.file).relative_to(root)}: {m.func_name} "
                    f"({m.pattern_name}, {m.pattern_type}, {pairs} pair{'s' if pairs > 1 else ''})"
                )

        if args.fix and matches:
            n = apply_fix(f, matches)
            if n > 0:
                print(f"  FIXED {Path(f).relative_to(root)}: {n} transforms")
                fixed_count += n

    total = sum(by_type.values())
    print(f"\nSummary: {total} transformable functions")
    for k, v in by_type.items():
        if v > 0:
            print(f"  {k}: {v}")
    print("By lock type:")
    for k, v in by_pat.items():
        if v > 0:
            print(f"  {k}: {v}")
    if args.fix:
        print(f"Transformed: {fixed_count} pairs")


if __name__ == "__main__":
    sys.exit(main())
