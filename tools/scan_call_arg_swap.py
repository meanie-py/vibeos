#!/usr/bin/env python3
"""
scan_call_arg_swap.py — a call passes a different local variable than its own C does.

    C:     LoadStringW(hInstance, IDS_PROFILEPATH, szBuffer, ARRAYSIZE(szBuffer));
           ExpandEnvironmentStringsW(szBuffer, szProfilesPath, ARRAYSIZE(szProfilesPath));
    Rust:  LoadStringW(hInstance, IDS_PROFILEPATH, szProfilesPath.as_mut_ptr(), MAX_PATH);
           ExpandEnvironmentStringsW(szBuffer.as_mut_ptr(), szProfilesPath.as_mut_ptr(), ..);

Two buffers of one type, and the translation wrote the wrong one: userenv's
InitializeProfiles expanded an empty string over the path it had just loaded, and GUI setup
stopped at "InitializeProfiles() failed / Rebooting now!" on every install. Two lines below,
CreateStandardProfile copied the profile name into the buffer the next line overwrites.
Everything type-checks.

The oracle is the C the translator left in the file ("ORIGINAL C SOURCE"). For each Rust fn
whose C function of the same name is there, the k-th call of a callee (same arity) is paired
with the k-th such call in the C, and an argument is reported when both sides pass a bare
identifier that is a LOCAL of the C function and the two differ. Functions whose C uses
`goto` are skipped (cleanup restructured into early returns does not keep call order), as
is a Rust local declared as an alias of the C's variable (`let k = InputBuffer`).
Baseline: the reviewed look-alikes.

Usage:
    python3 tools/scan_call_arg_swap.py [paths...]   # default: sdk/lib/rust
    python3 tools/scan_call_arg_swap.py --lint
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.scan_call_literal_vs_c import C_CALL_RE, _calls
from tools.scan_wide_byte_copy import SKIP_FILE_RE
from tools.source_walk import source_files

REPO = Path(__file__).resolve().parent.parent
RUST_ROOT = REPO / "sdk" / "lib" / "rust"
MARKER = "ORIGINAL C SOURCE"
ID_RE = re.compile(r"^[A-Za-z_]\w*$")
RS_FN_RE = re.compile(r"\bfn\s+(\w+)\s*(?:<[^>]*>)?\(")
C_FN_RE = re.compile(r"^(\w+)\s*\(", re.M)
C_LOCAL_RE = re.compile(r"\b(\w+)\s*(?:\[[^\]]*\])?\s*;")


def _blank_comments(text: str) -> str:
    return re.sub(
        r"//[^\n]*|/\*.*?\*/", lambda m: re.sub(r"[^\n]", " ", m.group(0)), text, flags=re.S
    )


def _brace_body(text: str, start: int) -> tuple[str, int] | None:
    i = text.find("{", start)
    if i < 0:
        return None
    depth = 0
    for j in range(i, len(text)):
        if text[j] == "{":
            depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0:
                return text[i:j], i
    return None


def _rust_arg(a: str) -> str:
    a = re.sub(r"\bas\s+[\w:*\s]+?(?=$|\))", "", a.strip())
    a = re.sub(r"^(&raw\s+(mut|const)\s+|&mut\s+|&|core::ptr::addr_of(_mut)?!\()", "", a.strip())
    a = re.sub(r"\.(as_mut_ptr|as_ptr|cast)\(\)", "", a)
    return a.strip("() ")


def _c_arg(a: str) -> str:
    a = re.sub(r"\((?:[A-Z_][A-Z0-9_]*\s*\*?)\)", "", a.strip())
    return a.lstrip("&").strip("() ")


def _by_callee(body: str, call_re=None) -> dict[tuple[str, int], list[list[str]]]:
    out: dict[tuple[str, int], list[list[str]]] = {}
    calls = _calls(body) if call_re is None else _calls(body, call_re)
    for _pos, fn, args in calls:
        out.setdefault((fn, len(args)), []).append(args)
    return out


def scan(paths: list[Path]) -> list[tuple[Path, int, str]]:
    findings = []
    for root in paths:
        files = [root] if root.is_file() else list(source_files(root))
        for f in files:
            if f.suffix != ".rs" or SKIP_FILE_RE.search(str(f)):
                continue
            try:
                text = f.read_text(errors="replace")
            except OSError:
                continue
            cut = text.find(MARKER)
            if cut < 0:
                continue
            rust = _blank_comments(text[:cut])
            c = "\n".join(re.sub(r"^\s*//\s?", "", ln) for ln in text[cut:].splitlines())
            c = re.sub(r"/\*.*?\*/", "", c, flags=re.S)
            c_fns = {}
            for m in C_FN_RE.finditer(c):
                head_end = c.find("{", m.end())
                if head_end < 0 or ";" in c[m.end() : head_end]:
                    continue
                got = _brace_body(c, m.end())
                if got:
                    c_fns.setdefault(m.group(1), got[0])
            for m in RS_FN_RE.finditer(rust):
                cbody = c_fns.get(m.group(1))
                got = _brace_body(rust, m.end())
                if cbody is None or got is None or re.search(r"\bgoto\b", cbody):
                    continue
                rbody = got[0]
                c_locals = set(C_LOCAL_RE.findall(cbody))
                c_calls = _by_callee(cbody, C_CALL_RE)
                for key, r_list in _by_callee(rbody).items():
                    c_list = c_calls.get(key)
                    if not c_list or len(c_list) != len(r_list):
                        continue
                    for k, (ra, ca) in enumerate(zip(r_list, c_list, strict=True)):
                        for i, (x, y) in enumerate(zip(ra, ca, strict=True)):
                            xs, ys = _rust_arg(x), _c_arg(y)
                            if (
                                not (ID_RE.match(xs) and ID_RE.match(ys))
                                or xs.lower() == ys.lower()
                            ):
                                continue
                            if xs not in c_locals or ys not in c_locals:
                                continue
                            alias = (
                                rf"\blet\s+(mut\s+)?{re.escape(xs)}\b[^;=]*=\s*{re.escape(ys)}\b"
                            )
                            if re.search(alias, rbody):
                                continue
                            line = text.count("\n", 0, m.start()) + 1
                            findings.append(
                                (
                                    f,
                                    line,
                                    f"{m.group(1)}: {key[0]} call#{k} arg{i}: {xs}"
                                    f" where C passes {ys}",
                                )
                            )
    return sorted(findings)


def main(argv: list[str] | None = None) -> int:
    args = list(sys.argv[1:] if argv is None else argv)
    lint = "--lint" in args
    paths = [Path(a).resolve() for a in args if not a.startswith("--")] or [RUST_ROOT]
    findings = scan(paths)
    for f, lineno, what in findings:
        try:
            rel = f.relative_to(REPO)
        except ValueError:
            rel = f
        tag = "call-arg-swap" if lint else "argument differs from the C's:"
        print(f"{rel}:{lineno}: {tag} {what}")
    if findings and not lint:
        print(f"\n{len(findings)} finding(s): pass the variable the C passes.")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
