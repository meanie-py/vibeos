#!/usr/bin/env python3
"""
scan_variadic_array_arg.py — an array FIELD passed by value to a C variadic function.

    C:            note("DHCPDISCOVER on %s ...", ip->name);     // char name[256] decays
    translated:   note(b"DHCPDISCOVER on %s ...\0".as_ptr(), (*ip).name, ...);
    faithful:     note(b"DHCPDISCOVER on %s ...\0".as_ptr(), (*ip).name.as_ptr(), ...);

In C an array argument decays to a pointer. In Rust `(*ip).name` is the `[c_char; 256]`
VALUE, and rustc accepts it in a variadic call (E0617 covers floats, small integers, bool
and fn items, not arrays). What the callee then sees depends on the ABI. amd64 and arm64
pass a large aggregate by reference, so the callee happens to get a pointer to a copy and
the bug is invisible. i386 copies the 256 bytes onto the stack, so `%s` reads the
string's first four bytes as the pointer.

Found 2026-10-04: the i386 DHCP client service died in msvcrt's printf engine reading
address 0x7374656e, which is "nets", the start of "netstack smoltcp adapter". It had
crashed that way on every i386 boot that ran long enough to start Dhcp.

A bare name match cannot find this: in dhcpcsvc `name` is an array in `interface_info`
but a pointer in two other structs. So each argument `(*v).f` / `v.f` is resolved to v's
declared type (the nearest `v: T` / `v: *mut T` before the call, through `pub type`
aliases), and flagged only when that struct's field `f` is an array.

Usage:
    python3 tools/scan_variadic_array_arg.py [root ...] # report
    python3 tools/scan_variadic_array_arg.py --lint     # terse (CLI baseline gate)

Exit 1 if any finding.
"""

import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO))
from tools.source_walk import source_files  # noqa: E402

RUST_ROOT = REPO / "sdk" / "lib" / "rust"
SKIP_PATH_RE = re.compile(r"(/target/|/\.responses/)")

STRUCT_RE = re.compile(r"\bstruct\s+(\w+)\s*\{([^{}]*)\}", re.S)
FIELD_RE = re.compile(r"(?:^|[,{\n])\s*(?:pub(?:\([^)]*\))?\s+)?(\w+)\s*:\s*(\S)")
PTR_ALIAS_RE = re.compile(r"\btype\s+(\w+)\s*=\s*\*(?:mut|const)\s+([\w:]+)\s*;")
ALIAS_RE = re.compile(r"\btype\s+(\w+)\s*=\s*([\w:]+)\s*;")
VARIADIC_RE = re.compile(r"\bfn\s+(\w+)\s*\(([^;{]*?)\.\.\.\s*\)", re.S)
LINK_NAME_RE = re.compile(r'#\[link_name\s*=\s*"(\w+)"\]')
PLACE_RE = re.compile(r"^(?:\(\*(\w+)\)|(\w+))\.(\w+)$")
STRING_RE = re.compile(r'b?"(?:\\.|[^"\\])*"')


def rust_files(roots: list[Path]):
    for root in roots:
        found = [root] if root.is_file() else source_files(root)
        for p in found:
            if not SKIP_PATH_RE.search(str(p)):
                yield p


def last_segment(t: str) -> str:
    return t.split("::")[-1]


def index(files: list[Path]):
    structs: dict[str, dict[str, bool]] = {}
    ptr_alias: dict[str, str] = {}
    alias: dict[str, str] = {}
    variadic: set[str] = set()
    for p in files:
        text = p.read_text(errors="replace")
        for m in STRUCT_RE.finditer(text):
            fields = {f.group(1): f.group(2) == "[" for f in FIELD_RE.finditer(m.group(2))}
            prev = structs.setdefault(m.group(1), {})
            for name, is_array in fields.items():
                prev[name] = prev.get(name, False) or is_array
        for m in PTR_ALIAS_RE.finditer(text):
            ptr_alias[m.group(1)] = last_segment(m.group(2))
        for m in ALIAS_RE.finditer(text):
            alias[m.group(1)] = last_segment(m.group(2))
        for m in VARIADIC_RE.finditer(text):
            variadic.add(m.group(1))
    return structs, ptr_alias, alias, variadic


def resolve(t: str, ptr_alias: dict[str, str], alias: dict[str, str]) -> str:
    seen = set()
    while t not in seen:
        seen.add(t)
        if t in ptr_alias:
            t = ptr_alias[t]
        elif t in alias and alias[t] != t:
            t = alias[t]
        else:
            break
    return t


def split_args(text: str, open_idx: int) -> tuple[list[str], int]:
    """Top-level comma-separated args of the call whose '(' is at open_idx."""
    args, depth, start, i = [], 0, open_idx + 1, open_idx
    while i < len(text):
        c = text[i]
        if c == '"':
            m = STRING_RE.match(text, i if text[i - 1] != "b" else i - 1)
            i = m.end() if m else i + 1
            continue
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0:
                args.append(text[start:i])
                return [a.strip() for a in args], i
        elif c == "," and depth == 1:
            args.append(text[start:i])
            start = i + 1
        i += 1
    return [], -1


def var_type(text: str, var: str, before: int) -> str | None:
    fn_start = text.rfind("\nfn ", 0, before)
    fn_start = max(fn_start, text.rfind(" fn ", 0, before))
    window = text[max(fn_start, 0) : before]
    decl = None
    v = re.escape(var)
    # `v: T` / `v: *mut T` (params and annotated lets), or `let v = expr as *mut T;`.
    pat = (
        r"\b" + v + r"\s*:\s*(?:\*(?:mut|const)\s+|&(?:mut\s+)?)?([\w:]+)"
        r"|\blet\s+(?:mut\s+)?" + v + r"\s*=[^;]*?\bas\s+\*(?:mut|const)\s+([\w:]+)\s*;"
    )
    for m in re.finditer(pat, window):
        decl = m.group(1) or m.group(2)
    return last_segment(decl) if decl else None


def scan_file(path: Path, structs, ptr_alias, alias, variadic) -> list[tuple[int, str]]:
    text = path.read_text(errors="replace")
    if "automatically generated by rust-bindgen" in text[:600]:
        return []
    out = []
    names = "|".join(sorted(map(re.escape, variadic), key=len, reverse=True))
    if not names:
        return out
    for m in re.finditer(r"(?<![\w.:])(" + names + r")\s*\(", text):
        line_start = text.rfind("\n", 0, m.start()) + 1
        if text[line_start : m.start()].lstrip().startswith("//"):
            continue
        if re.search(r"\bfn\s*$", text[line_start : m.start()]):
            continue  # the declaration itself
        args, _ = split_args(text, m.end() - 1)
        for arg in args[1:]:
            pm = PLACE_RE.match(arg)
            if not pm:
                continue
            var, field = pm.group(1) or pm.group(2), pm.group(3)
            t = var_type(text, var, m.start())
            if not t:
                continue
            struct = resolve(t, ptr_alias, alias)
            fields = structs.get(struct)
            if fields and fields.get(field):
                lineno = text.count("\n", 0, m.start()) + 1
                why = f"`{field}` is an array in {struct}; pass {arg}.as_ptr()"
                out.append((lineno, f"{m.group(1)}(.., {arg}, ..): {why}"))
    return out


def main() -> int:
    lint = "--lint" in sys.argv
    roots = [Path(a) for a in sys.argv[1:] if not a.startswith("-")] or [RUST_ROOT]
    files = sorted(set(rust_files(roots)))
    structs, ptr_alias, alias, variadic = index(files)
    findings = 0
    for p in files:
        for lineno, msg in scan_file(p, structs, ptr_alias, alias, variadic):
            findings += 1
            try:
                rel = p.relative_to(REPO)
            except ValueError:
                rel = p
            print(f"{rel}:{lineno}: variadic-array-arg" if lint else f"{rel}:{lineno}: {msg}")
    if not lint:
        print(f"{findings} finding(s) over {len(variadic)} variadic functions")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
