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

# Crates a difftest harness ALSO compiles for `i686-unknown-linux-gnu`. `target_arch = "x86"`
# is true there, but `"system"` is stdcall only on 32-bit WINDOWS — off Windows it is cdecl,
# while an explicit `extern "stdcall"` stays stdcall on x86 regardless. So in these crates the
# x86 twin's literal spelling is load-bearing and folding it changes the convention IN THE
# HARNESS ONLY: msvcrt-seh-difftest died in `cxx_frame_handler` while all three product arches
# built and booted clean. Field manual: "On a 32-bit HOST harness, `extern "system"` and
# `extern "stdcall"` stop agreeing".
HOST_HARNESS_CRATES = frozenset(
    {"msvcrt-crt-rust", "msvcrt-sys", "cnt-rust", "cnt-ctype", "rtl-strings"}
)

# `unsafe extern "stdcall" fn Name(`, `pub unsafe extern "C" fn Name(`, `extern "C" fn Name(` …
# `system` is in the alternation because a pair that has ALREADY had its ABI reconciled
# (by --abi-only, or by hand) is still a forked pair, and dropping it from the match makes the
# tool blind to exactly the drift it exists to report: after one --abi-only run the repo-wide
# drift count fell 102 -> 2 with nothing fixed, because both sides now said `system`.
_FN = re.compile(r'^(?P<pre>(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?)extern\s+"(?P<abi>stdcall|C|system)"\s+fn\s+(?P<name>\w+)\s*[(<]')

# `extern "stdcall" {` — a DECLARATION block, whose twin declares the same imports under the
# other arch's convention. Same fork, same reason, different syntax; the tool read only the
# `fn` form and so left every one of these standing (po's PopAddRemoveSysCapsCallback, kd's
# screen/serial entry points, classpnp's dispatch imports).
#
# Pairing IS the safety property here. An `extern "C" {` block on its own may be a genuinely
# cdecl import (the CRT), and promoting it to `system` would be a real ABI break on i386 —
# but a block that is the `not(x86)` TWIN of a `stdcall` block is by construction the same
# NTAPI surface spelled per arch, so `system` is what both mean.
_EXTERN_BLOCK = re.compile(r'^(?:unsafe\s+)?extern\s+"(?P<abi>stdcall|C|system)"\s*\{\s*$')
_BLOCK_FIRST_FN = re.compile(r"^\s*(?:pub\s+)?fn\s+(?P<name>\w+)")

# `pub type PKDP_INIT_ROUTINE = Option<unsafe extern "stdcall" fn(..)>;` — the fn-POINTER
# alias a crate writes by hand when it is not taking the type from an island. Same fork
# again; kd, ex, classpnp, kdbg and ntoskrnl all carry these. Unlike the other two shapes an
# alias has no braces, so its extent is the `;`, and the ABI routinely lands on a later line
# than the `type` keyword.
_TYPE_ALIAS = re.compile(r"^(?:pub(?:\([^)]*\))?\s+)?type\s+(?P<name>\w+)\s*=")
_ABI_ANYWHERE = re.compile(r'extern\s+"(?:stdcall|C)"')


def _alias_end(lines: list[str], start: int) -> int:
    """Index one past the `;` ending the type alias that starts at `lines[start]`."""
    for i in range(start, min(start + 12, len(lines))):
        if lines[i].rstrip().endswith(";"):
            return i + 1
    return -1


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


def _attrs_and_sig(lines: list[str], cfg_at: int) -> tuple[int, dict[str, str]] | None:
    """From the cfg attribute line, skip further attributes and return (sig_index, info).

    `info["name"]` identifies the item so the two twins can be matched. For a declaration
    block that is the name of its FIRST import, which is enough: bindgen and the translator
    both emit one block per logical surface, and a mismatch just declines the collapse.
    """
    i = cfg_at + 1
    # Doc comments count as part of the item's header, not as the item — a `///` between the
    # cfg and the signature (ex-rust writes its allocator wrappers that way) used to end the
    # scan and lose the pair.
    while i < len(lines) and (
        lines[i].lstrip().startswith(("#[", "///", "//!")) or not lines[i].strip()
    ):
        i += 1
    if i >= len(lines):
        return None
    m = _FN.match(lines[i])
    if m:
        return i, {"name": m["name"], "abi": m["abi"]}
    b = _EXTERN_BLOCK.match(lines[i])
    if b:
        for k in range(i + 1, min(i + 8, len(lines))):
            f = _BLOCK_FIRST_FN.match(lines[k])
            if f:
                return i, {"name": f"extern {{{f['name']}}}", "abi": b["abi"]}
        return None
    t = _TYPE_ALIAS.match(lines[i])
    if t:
        # Only an alias that actually carries a per-arch ABI is ours; a plain `type X = Y;`
        # under a cfg is arch-specific for some other reason and must be left alone.
        for k in range(i, min(i + 6, len(lines))):
            if _ABI_ANYWHERE.search(lines[k]):
                return i, {"name": f"type {t['name']}", "abi": "alias"}
            if lines[k].rstrip().endswith(";"):
                break
    return None


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


def _target_abi(x86_abi: str | None, other_abi: str | None) -> str | None:
    """What the collapsed item should be spelled, or None if the pair must not be collapsed.

    `stdcall` against `C` is the ABI FORK this tool exists to fold: one convention written
    per arch, which is exactly what `system` means. Collapse it to `system`.

    A pair spelled the SAME on both sides is not an ABI fork — it is one function with two
    per-arch implementations, gated for some unrelated reason. Those still collapse when the
    bodies are identical, but they must KEEP their convention: promoting a cdecl `C` pair to
    `system` makes it stdcall ON i386, which changes the symbol decoration (`_PATH_RoundRect`
    becomes `_PATH_RoundRect@28`) and breaks the link against C callers — and where it does
    NOT break the link it silently mismatches the caller's stack cleanup instead.
    """
    if x86_abi == "stdcall" and other_abi == "C":
        return "system"
    if x86_abi == other_abi and x86_abi in ("C", "stdcall", "system"):
        return x86_abi
    return None


def _normalize(block: list[str], sig_rel: int, abi: str = "system") -> list[str]:
    """The block with its ABI string rewritten to `abi`, for comparison and for output.

    Rewrites the FIRST line carrying an ABI rather than the signature line: a type alias
    routinely wraps, putting `unsafe extern "C" fn` a line or two below the `type` keyword.
    For a fn or an extern block that first line IS the signature, so this is the same edit.
    """
    out = list(block)
    for k in range(sig_rel, len(out)):
        if _ABI_ANYWHERE.search(out[k]):
            out[k] = _ABI_ANYWHERE.sub(f'extern "{abi}"', out[k], count=1)
            break
    return out


def collapse_file(
    path: pathlib.Path,
    apply: bool,
    full_drift: bool = False,
    survivor: str | None = None,
    abi_only: bool = False,
) -> tuple[int, list[str]]:
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
    notes: list[str] = []
    edits: list[tuple[int, int, list[str]]] = []  # (start, end, replacement)

    i = 0
    while i < len(lines):
        # Either order. The translator emits x86-first, but hand-written pairs are as often
        # `not(x86)` first (explorer's window procs are), and reading only one order reported
        # those as "no twin follows" and left them forked.
        if lines[i].strip() not in (X86, NOT_X86):
            i += 1
            continue
        a_is_x86 = lines[i].strip() == X86
        want = NOT_X86 if a_is_x86 else X86
        a = _attrs_and_sig(lines, i)
        if not a:
            i += 1
            continue
        a_sig, a_m = a
        _end = _alias_end if a_m['abi'] == 'alias' else _item_end
        a_end = _end(lines, a_sig)
        if a_end < 0:
            notes.append(f"  ! {path.name}:{i + 1} {a_m['name']}: unbalanced braces, skipped")
            i += 1
            continue

        # The twin follows, possibly after blank lines — and possibly behind its own attributes.
        # `#[no_mangle]` ABOVE `#[cfg(..)]` is what the translator emits for an exported
        # callback, and reading only the cfg line missed every one of them: 71 pairs repo-wide
        # sat uncollapsed for no reason but attribute order, including the afd TDI completions
        # whose `stdcall` spelling then failed to unify with their own normalized island slot.
        j = a_end
        while j < len(lines) and not lines[j].strip():
            j += 1
        b_pre_at = j
        while (
            j < len(lines)
            and lines[j].lstrip().startswith("#[")
            and "]" in lines[j]
            and lines[j].strip() not in (NOT_X86, X86)
        ):
            j += 1
        if j >= len(lines) or lines[j].strip() != want:
            notes.append(
                f"  ! {path.name}:{i + 1} {a_m['name']}: no `{want}` twin follows, skipped"
            )
            i = a_end
            continue
        # Attributes above the X86 cfg sit OUTSIDE the replaced range, so they survive the
        # collapse untouched. The twin's sit inside it and are dropped — which is only correct
        # if they are the same set. A difference is a per-arch attribute, never a formatting
        # artifact, so it is reported rather than guessed at.
        a_pre_at = i
        while (
            a_pre_at > 0
            and lines[a_pre_at - 1].lstrip().startswith("#[")
            and "]" in lines[a_pre_at - 1]
        ):
            a_pre_at -= 1
        a_pre = [ln.strip() for ln in lines[a_pre_at:i]]
        b_pre = [ln.strip() for ln in lines[b_pre_at:j]]
        if a_pre != b_pre:
            notes.append(
                f"  ! {path.name}:{i + 1} {a_m['name']}: the twins carry DIFFERENT attributes "
                f"({'x86' if a_is_x86 else 'not(x86)'} {a_pre or '[]'} vs "
                f"{'not(x86)' if a_is_x86 else 'x86'} {b_pre or '[]'}) — collapsing would drop "
                f"one arch's. Reconcile by hand."
            )
            i = a_end
            continue
        b = _attrs_and_sig(lines, j)
        if not b or b[1]["name"] != a_m["name"]:
            other = b[1]["name"] if b else "?"
            notes.append(f"  ! {path.name}:{j + 1} twin is `{other}`, not `{a_m['name']}`, skipped")
            i = a_end
            continue
        b_sig, _ = b
        b_end = _end(lines, b_sig)
        if b_end < 0:
            notes.append(f"  ! {path.name}:{j + 1} {a_m['name']}: unbalanced braces in twin, skipped")
            i = a_end
            continue

        a_abi, b_abi = a_m.get("abi"), (b[1] if b else {}).get("abi")
        x86_abi, other_abi = (a_abi, b_abi) if a_is_x86 else (b_abi, a_abi)
        target = "system" if a_abi == "alias" else _target_abi(x86_abi, other_abi)
        if target is None:
            notes.append(
                f"  ! {path.name}:{i + 1} {a_m['name']}: the twins disagree on convention in a "
                f"way this tool will not reconcile (x86 {x86_abi}, not(x86) {other_abi}) — "
                f"skipped"
            )
            i = b_end
            continue
        side_a = _normalize(lines[a_sig:a_end], 0, target)
        side_b = _normalize(lines[b_sig:b_end], 0, target)
        # `x86`/`other` name the ARCH, not the source order — the pair may be written either way.
        x86, other = (side_a, side_b) if a_is_x86 else (side_b, side_a)
        drifted = x86 != other

        if abi_only:
            # ONLY a genuine ABI fork — x86 `stdcall` against not-x86 `C`. A cfg-gated pair
            # whose two sides carry the SAME convention is not an ABI twin at all: it is one
            # function with two per-arch IMPLEMENTATIONS (kdbg's KdbpCmdTss is `extern "C"`
            # both sides, real on i386 and a stub elsewhere), and respelling it `system` would
            # promote a genuinely-cdecl function to stdcall ON i386 — a live ABI break, the
            # exact direction normalize_island_abi.py refuses to guess in. Checked here rather
            # than trusted: this pass shipped that break before the check existed.
            a_abi = a_m.get("abi")
            b_abi = (b[1] if b else {}).get("abi")
            x86_abi, other_abi = (a_abi, b_abi) if a_is_x86 else (b_abi, a_abi)
            if not (x86_abi == "stdcall" and other_abi == "C"):
                notes.append(
                    f"  ! {path.name}:{i + 1} {a_m['name']}: cfg pair is not an ABI fork "
                    f"(x86 {x86_abi}, not(x86) {other_abi}) — left alone"
                )
                i = b_end
                continue
            # Re-spell BOTH twins `system` and leave the fork and the two bodies exactly as
            # they are. For a pair whose bodies have genuinely drifted, merging is refused (and
            # should be) — but the ABI half is still mechanical, and until both twins agree the
            # pair cannot be assigned into a normalized island slot at all. This separates the
            # two questions: spelling is fixed here, the drift stays a reported finding.
            for sig in (a_sig, b_sig):
                fixed = _normalize(lines[sig : sig + 1], 0)
                if fixed != lines[sig : sig + 1]:
                    edits.append((sig, sig + 1, fixed))
            notes.append(
                f"  {path.name}:{i + 1} {a_m['name']}: both twins spelled `system`"
                + (" (bodies still drifted — left forked)" if drifted else "")
            )
            i = b_end
            continue
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
    ap.add_argument(
        "--abi-only",
        action="store_true",
        help="do not merge anything: just spell BOTH twins `extern \"system\"`, leaving the "
             "cfg fork and both bodies alone. For pairs whose bodies have genuinely drifted, "
             "which must not be collapsed but still have to agree on ABI to be assignable "
             "into a normalized island slot.",
    )
    args = ap.parse_args(argv)

    repo = pathlib.Path(__file__).resolve().parents[2]
    base = repo / "sdk" / "lib" / "rust"
    crates = [pathlib.Path(c) for c in args.crates] or sorted(
        d for d in base.iterdir() if (d / "src").is_dir()
    )
    grand = 0
    for c in crates:
        if c.name in HOST_HARNESS_CRATES:
            print(
                f"{c.name}: SKIPPED — a difftest harness compiles this crate for "
                f"i686-unknown-linux-gnu, where `system` is cdecl rather than stdcall, so the "
                f"x86 twin's explicit `stdcall` is load-bearing and must not be folded"
            )
            continue
        src = c / "src" if (c / "src").is_dir() else c
        total: list[str] = []
        n = 0
        for f in sorted(src.rglob("*.rs")):
            if f.name.startswith("types_") or f.name.startswith("bindgen"):
                continue  # islands are normalize_island_abi.py's job
            k, notes = collapse_file(
                f, args.apply, args.full_drift, args.survivor, args.abi_only
            )
            n += k
            total += notes
        if total:
            print(f"{c.name}: {n}")
            print("\n".join(total))
        grand += n
    # --abi-only edits one signature LINE at a time, so the tally counts those, not pairs.
    # --abi-only edits one signature LINE at a time, so the tally counts those, not pairs.
    unit = "twin signature(s)" if args.abi_only else "twin pair(s)"
    verb = ("respelled" if args.abi_only else "collapsed") if args.apply else "would fix"
    print(f"\n{verb} {grand} {unit}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
