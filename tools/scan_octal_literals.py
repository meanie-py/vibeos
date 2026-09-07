#!/usr/bin/env python3
"""scan_octal_literals.py — a C octal literal that became a Rust DECIMAL one.

THE INCIDENT
    `dll/win32/msvcrt/file.c` computes a regular file's mode as

        buf->st_mode = _S_IFREG | 0444;
        if (!(basic_info.FileAttributes & FILE_ATTRIBUTE_READONLY))
            buf->st_mode |= 0222;

    and the translation carried both literals across unchanged:

        (*buf).st_mode = (_S_IFREG | 0444) as u16;
        (*buf).st_mode |= 0222 as u16;

    C reads a leading zero as OCTAL. Rust has no C-style octal at all — a leading zero is
    simply a leading zero, and `0444` is a perfectly ordinary way to write four hundred and
    forty-four. So there is no error, no warning, and no lint anywhere in rustc that has an
    opinion about it. msvcrt.dll shipped `st_mode` 444|222 = 0o776 where msvcrt reports
    0o444|0o222 = 0o666 — the execute bits set on every regular file, on all three arches,
    through six `_fstat*` exports, until a differential row diverged on byte 14 of the
    compared struct.

WHY A SCANNER AND NOT "the difftest will catch it"
    The difftest caught THIS one because msvcrt is one of the three modules that has a
    differential harness at all (msvcrt, cnt, rtl). The other ~40 translated crates have no
    oracle of any kind, rustconv keeps producing new ones, and every C source we translate
    is full of octal: mode bits, ioctl numbers, terminal flags, permission masks. The bug
    is invisible to the compiler, invisible to review (the literal is three characters from
    correct and looks *more* right in its C form), and free to detect with a text scan.

WHAT COUNTS AS A FINDING
    A bare `0[0-7]+` in numeric position WHOSE TWO READINGS DISAGREE — that is,
    `int(text, 10) != int(text, 8)`. The value test is the discriminator that makes the
    finding worth acting on rather than merely worth looking at:

      * `0`, `00`, `07`, `007` are IDENTICAL in both bases and are never reported. A
        scanner that flagged them would report a "bug" with no observable difference, and
        would be tuned out inside a week.
      * `010` reads 10 to Rust and 8 to C, so it IS reported.

    Everything else is excluded by construction rather than by allowlist:

      * float exponents — `1.66e-01`, `7.85e+00`. The `01` is preceded by `-`/`+`/`e`,
        all of which the lookbehind rejects. This is load-bearing, not cosmetic: without
        it cnt-rust's math tables alone produce 78 findings and the scan is worthless.
      * `0x1FE`, `0o444`, `0b0101` — the digits are preceded by `x`/`o`/`b` (word
        characters), and the leading `0` is not followed by an octal digit.
      * `x.0`, `1.5`, `tuple.01` — preceded or followed by `.`.
      * anything inside a string, byte string, char literal, `//` comment or `/* */`
        comment. Stripped before matching, because `b"0640\\0"` (atapi's PCI device id) and
        `"Type 000"` (advapi32's comment) are exactly the shapes a naive grep drowns in.

    A suffixed literal (`0644u32`) IS matched — the suffix does not change the base, and
    excluding it would leave the most explicitly-typed instances unchecked.

    Findings go to stdout for the baseline comparison; the tally goes to STDERR, because
    `lints.py` reads stdout as the finding list and would otherwise count the tally line
    as a finding (which is how this scanner's neighbour reported `1 new` on a clean tree).

BASELINE
    Empty, and it should stay that way. The tree was swept when this landed and the
    `_fstat` pair was the only instance in 3,064 files. A legitimate leading-zero decimal
    is possible in hand-written Rust — put it in the baseline with a comment saying why.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.source_walk import source_files

# A leading-zero literal in NUMERIC position.
#
# The lookbehind rejects `e`/`E`/`+`/`-` so a float exponent is not a candidate, and every
# other word character so `0x…`/`0o…`/`0b…` digit runs and identifier tails are not either.
# The lookahead allows an integer suffix but nothing else, so `0.5` and `x.01` are out.
OCTALISH = re.compile(r"(?<![\w.eE+-])0[0-7]+(?:[iu](?:8|16|32|64|128|size))?(?![\w.])")

SUFFIX = re.compile(r"[iu](?:8|16|32|64|128|size)$")

# A char literal is a tick, one character or escape, and a closing tick. Anything else
# beginning with a tick is a LIFETIME (`&'static str`, `Foo<'a>`), which has no closing
# tick at all — the distinction the regex approach cannot make.
CHAR_LIT = re.compile(r"'(?:\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f]+\}|.)|[^'\\])'")


def strip_noise(text: str) -> str:
    """Blank out comments and literals, PRESERVING newlines so line numbers survive.

    A left-to-right scan rather than a sequence of regex passes, because the passes are
    order-dependent and every order is wrong somewhere: strip `//` first and `"http://x"`
    loses its tail; strip strings first and a `//` comment containing an apostrophe pairs
    with a later tick. Rust makes it worse by spelling lifetimes with the same character as
    char literals, so `&'static` opens a quote that regexes then close on the next tick in
    the file. The first version of this scanner did exactly that and reported `"0777"` —
    inside a string, in a scanf test corpus — as a finding.

    Newlines are preserved rather than collapsed: a multi-line block comment replaced by
    one space renumbers every line after it, and a finding naming the wrong line sends the
    reader to innocent code and teaches them the scanner is unreliable.
    """
    out: list[str] = []
    i, n = 0, len(text)

    def blank(s: str) -> str:
        return re.sub(r"[^\n]", " ", s)

    while i < n:
        c = text[i]

        # Raw string: r"...", r#"..."#, br##"..."##. No escapes inside.
        m = re.compile(r"b?r(#*)\"").match(text, i)
        if m:
            close = '"' + m.group(1)
            end = text.find(close, m.end())
            end = n if end < 0 else end + len(close)
            out.append(blank(text[i:end]))
            i = end
            continue

        if text.startswith("//", i):
            end = text.find("\n", i)
            end = n if end < 0 else end
            out.append(blank(text[i:end]))
            i = end
            continue

        if text.startswith("/*", i):
            # Rust block comments NEST, so a plain `find("*/")` stops too early.
            depth, j = 1, i + 2
            while j < n and depth:
                if text.startswith("/*", j):
                    depth += 1
                    j += 2
                elif text.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            out.append(blank(text[i:j]))
            i = j
            continue

        if c == '"' or (c == "b" and text.startswith('b"', i)):
            j = i + (2 if c == "b" else 1)
            while j < n:
                if text[j] == "\\":
                    j += 2
                elif text[j] == '"':
                    j += 1
                    break
                else:
                    j += 1
            out.append(blank(text[i:j]))
            i = j
            continue

        if c == "'" or (c == "b" and text.startswith("b'", i)):
            m = CHAR_LIT.match(text, i + 1 if c == "b" else i)
            if m:
                out.append(blank(text[i : m.end()]))
                i = m.end()
                continue
            # A lifetime tick. Emit it and move on by ONE, so the rest of the line is
            # still scanned for literals.
            out.append(c)
            i += 1
            continue

        out.append(c)
        i += 1

    return "".join(out)


def scan_text(text: str) -> list[tuple[int, str, int, int]]:
    """Return (line, literal, decimal_value, octal_value) for each disagreeing literal."""
    out: list[tuple[int, str, int, int]] = []
    for lineno, line in enumerate(strip_noise(text).split("\n"), 1):
        for m in OCTALISH.finditer(line):
            digits = SUFFIX.sub("", m.group(0))
            dec, oct_ = int(digits, 10), int(digits, 8)
            # The whole point: only report where the two readings actually differ.
            if dec != oct_:
                out.append((lineno, m.group(0), dec, oct_))
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--root",
        action="append",
        default=None,
        help="directory to scan (repeatable); defaults to sdk/",
    )
    args = ap.parse_args()

    repo = Path(__file__).resolve().parent.parent
    roots = [repo / r for r in (args.root or ["sdk"])]

    findings = 0
    files = 0
    for root in roots:
        for path in sorted(source_files(root)):
            text = path.read_text(encoding="utf-8", errors="replace")
            files += 1
            for lineno, lit, dec, oct_ in scan_text(text):
                rel = path.relative_to(repo)
                print(
                    f"{rel}:{lineno}: OCTAL_AS_DECIMAL `{lit}` — Rust reads {dec}, "
                    f"C reads 0o{oct_:o} ({oct_}). If this came from C, write 0o{oct_:o}."
                )
                findings += 1

    print(f"scanned {files} .rs files, {findings} findings", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
