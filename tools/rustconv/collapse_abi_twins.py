#!/usr/bin/env python3
"""collapse_abi_twins.py — fold a per-arch `#[cfg(target_arch)]` pair of IDENTICAL function
bodies into one `extern "system"` definition.

WHY THEY EXIST. Before the islands were normalized (see normalize_island_abi.py), assigning a
translated callback into a bindgen slot only type-checked if the function was spelled with the
arch's literal convention — `extern "stdcall"` on i386, `extern "C"` on amd64/arm64 — because
Rust compares function-pointer ABIs NOMINALLY: `extern "system" fn` unifies with neither, even
where the ABI is byte-identical. So the translator emitted the same body twice under a cfg:

    #[cfg(target_arch = "x86")]        unsafe extern "stdcall" fn WndProc(..) { <body> }
    #[cfg(not(target_arch = "x86"))]   unsafe extern "C"       fn WndProc(..) { <body> }

Now that the slots are spelled `system`, that duplication is pure cost, and the cost is not
just line count:

  * ONLY HALF THE CODE IS EVER COMPILED. An i386 build type-checks the x86 twin and never
    looks at the other; amd64 checks the other and never looks at this one. A fix applied to
    one twin and not the other is INVISIBLE until someone builds the third arch, and
    `cargo check` on any single target is structurally incapable of catching it.
  * It defeats crate-repair's cross-target agreement rule. A width-sensitive numeric cast is
    only applied when the SAME repair is independently required on 2+ targets — and two
    targets can never agree about a site that exists on exactly one of them. On aclui this
    withheld 119 of 119 repairs: every one of them sat inside a twin.

THE RULE. Collapse only when the two bodies are byte-identical after normalizing away the cfg
attribute and the ABI string. If they have drifted, that drift is a REAL finding (someone
patched one arch only) — it is reported and never guessed at.
"""

from __future__ import annotations

import argparse
import difflib
import pathlib
import re
import sys

X86 = '#[cfg(target_arch = "x86")]'
NOT_X86 = '#[cfg(not(target_arch = "x86"))]'

# `unsafe extern "stdcall" fn Name(`, `pub unsafe extern "C" fn Name(`, `extern "C" fn Name(` …
_FN = re.compile(r'^(?P<pre>(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?)extern\s+"(?P<abi>stdcall|C)"\s+fn\s+(?P<name>\w+)\s*[(<]')


def _item_end(lines: list[str], start: int) -> int:
    """Index one past the closing `}` of the item whose signature starts at `lines[start]`.

    Brace-counts with a lexer just good enough for translated C: line/block comments and
    string, byte-string and char literals all hide braces. Returns -1 if unbalanced.
    """
    depth = 0
    seen = False
    in_block = False
    for i in range(start, len(lines)):
        line = lines[i]
        j = 0
        while j < len(line):
            c = line[j]
            if in_block:
                if line.startswith("*/", j):
                    in_block = False
                    j += 2
                    continue
                j += 1
                continue
            if line.startswith("//", j):
                break
            if line.startswith("/*", j):
                in_block = True
                j += 2
                continue
            if c == '"':
                j += 1
                while j < len(line):
                    if line[j] == "\\":
                        j += 2
                        continue
                    if line[j] == '"':
                        break
                    j += 1
                j += 1
                continue
            if c == "'":
                # A char literal is `'x'` or `'\x'`; anything else here is a lifetime, which
                # carries no braces and can be stepped over one character at a time.
                m = re.match(r"'(?:\\.|[^\\'])'", line[j:])
                j += len(m.group(0)) if m else 1
                continue
            if c == "{":
                depth += 1
                seen = True
            elif c == "}":
                depth -= 1
                if seen and depth == 0:
                    return i + 1
            j += 1
    return -1


def _attrs_and_sig(lines: list[str], cfg_at: int) -> tuple[int, re.Match[str]] | None:
    """From the cfg attribute line, skip further attributes and return (sig_index, match)."""
    i = cfg_at + 1
    while i < len(lines) and (lines[i].lstrip().startswith("#[") or not lines[i].strip()):
        i += 1
    if i >= len(lines):
        return None
    m = _FN.match(lines[i])
    return (i, m) if m else None


_INT = r"(?:[ui](?:8|16|32|64|128|size))"


def strip_cast_decoration(text: str) -> str:
    """Erase every `(EXPR) as <primitive int>` wrapper, leaving `EXPR`.

    Used ONLY as the safety proof behind `--survivor`: if the two twins become identical once
    this decoration is gone, the drift was a repair pass that ran on one arch and not the
    other, and picking a survivor discards nothing but casts. If ANY real difference remains,
    the pair is still refused — that would be a hand-fix, and losing one is a silent bug.

    Matches the one shape crate-repair emits: the operand is always parenthesized, so the
    parens can be found by balance and no precedence question arises. A bare `x as u32` is
    left alone, which is the conservative direction (it can only make the proof fail).
    """
    while True:
        m = re.search(rf"\)\s+as\s+{_INT}\b", text)
        if not m:
            return text
        depth = 0
        for k in range(m.start(), -1, -1):
            if text[k] == ")":
                depth += 1
            elif text[k] == "(":
                depth -= 1
                if depth == 0:
                    text = text[:k] + text[k + 1 : m.start()] + text[m.end() :]
                    break
        else:
            return text  # unbalanced; stop rather than corrupt


def _normalize(block: list[str], sig_rel: int) -> list[str]:
    """The block with its ABI string rewritten to `system`, for comparison and for output."""
    out = list(block)
    out[sig_rel] = re.sub(r'extern\s+"(?:stdcall|C)"', 'extern "system"', out[sig_rel], count=1)
    return out


def collapse_file(
    path: pathlib.Path, apply: bool, full_drift: bool = False, survivor: str | None = None
) -> tuple[int, list[str]]:
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
    notes: list[str] = []
    edits: list[tuple[int, int, list[str]]] = []  # (start, end, replacement)

    i = 0
    while i < len(lines):
        if lines[i].strip() != X86:
            i += 1
            continue
        a = _attrs_and_sig(lines, i)
        if not a:
            i += 1
            continue
        a_sig, a_m = a
        a_end = _item_end(lines, a_sig)
        if a_end < 0:
            notes.append(f"  ! {path.name}:{i + 1} {a_m['name']}: unbalanced braces, skipped")
            i += 1
            continue

        # The twin follows, possibly after blank lines.
        j = a_end
        while j < len(lines) and not lines[j].strip():
            j += 1
        if j >= len(lines) or lines[j].strip() != NOT_X86:
            notes.append(f"  ! {path.name}:{i + 1} {a_m['name']}: no `not(x86)` twin follows, skipped")
            i = a_end
            continue
        b = _attrs_and_sig(lines, j)
        if not b or b[1]["name"] != a_m["name"]:
            other = b[1]["name"] if b else "?"
            notes.append(f"  ! {path.name}:{j + 1} twin is `{other}`, not `{a_m['name']}`, skipped")
            i = a_end
            continue
        b_sig, _ = b
        b_end = _item_end(lines, b_sig)
        if b_end < 0:
            notes.append(f"  ! {path.name}:{j + 1} {a_m['name']}: unbalanced braces in twin, skipped")
            i = a_end
            continue

        x86 = _normalize(lines[a_sig:a_end], 0)
        other = _normalize(lines[b_sig:b_end], 0)
        drifted = x86 != other
        if drifted and survivor:
            # Only cast decoration may differ; anything else is a hand-fix we refuse to drop.
            if strip_cast_decoration("".join(x86)) == strip_cast_decoration("".join(other)):
                keep_side, lost = (x86, "not(x86)") if survivor == "x86" else (other, "x86")
                n_cast = sum(1 for a, b in zip(x86, other) if a != b)
                notes.append(
                    f"  {path.name}:{i + 1} {a_m['name']}: cast-only drift, kept the "
                    f"{survivor} body ({n_cast} line(s) of {lost} cast decoration dropped; "
                    f"re-run crate-repair to redo them against all three targets)"
                )
                keep_a = [ln for ln in lines[i + 1 : a_sig] if ln.strip()]
                edits.append((i, b_end, keep_a + keep_side))
                i = b_end
                continue
        if drifted:
            d = [
                ln.rstrip()
                for ln in difflib.unified_diff(other, x86, "not(x86)", "x86", n=0, lineterm="")
            ][2:]
            shown = d if full_drift else d[:6]
            more = "" if len(shown) == len(d) else f"\n      … {len(d) - len(shown)} more (--full-drift)"
            notes.append(
                f"  ! {path.name}:{i + 1} {a_m['name']}: TWINS HAVE DRIFTED — one arch was "
                f"patched and the other was not. Reconcile by hand, never collapse blind:\n"
                + "\n".join(f"      {ln}" for ln in shown)
                + more
            )
            i = b_end
            continue

        # Keep any non-cfg attributes that sat between the cfg line and the signature.
        keep = [ln for ln in lines[i + 1 : a_sig] if ln.strip()]
        edits.append((i, b_end, keep + x86))
        notes.append(f"  {path.name}:{i + 1} {a_m['name']} ({a_end - a_sig} lines x2 -> x1)")
        i = b_end

    if edits and apply:
        for start, end, repl in reversed(edits):
            lines[start:end] = repl
        path.write_text("".join(lines), encoding="utf-8")
    return len(edits), notes


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("crates", nargs="*", help="crate dirs (default: every sdk/lib/rust/*)")
    ap.add_argument("--apply", action="store_true", help="write (default: report only)")
    ap.add_argument("--full-drift", action="store_true", help="show every drifted line, not the first 6")
    ap.add_argument(
        "--survivor",
        choices=("x86", "other"),
        help="also collapse pairs whose ONLY drift is `(x) as <int>` decoration, keeping "
             "this side. Refused if any non-cast difference remains.",
    )
    args = ap.parse_args(argv)

    repo = pathlib.Path(__file__).resolve().parents[2]
    base = repo / "sdk" / "lib" / "rust"
    crates = [pathlib.Path(c) for c in args.crates] or sorted(
        d for d in base.iterdir() if (d / "src").is_dir()
    )
    grand = 0
    for c in crates:
        src = c / "src" if (c / "src").is_dir() else c
        total: list[str] = []
        n = 0
        for f in sorted(src.rglob("*.rs")):
            if f.name.startswith("types_") or f.name.startswith("bindgen"):
                continue  # islands are normalize_island_abi.py's job
            k, notes = collapse_file(f, args.apply, args.full_drift, args.survivor)
            n += k
            total += notes
        if total:
            print(f"{c.name}: {n}")
            print("\n".join(total))
        grand += n
    verb = "collapsed" if args.apply else "would collapse"
    print(f"\n{verb} {grand} twin pair(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
