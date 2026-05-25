#!/usr/bin/env python3
"""freeldr_largeint — normalize rustconv's LARGE_INTEGER union mistranslation.

bindgen's `_LARGE_INTEGER` is a union with a direct `QuadPart: i64` field
and a nested struct field `u { LowPart, HighPart }` (memory
feedback_i64_vs_largeinteger_abi / fs-unit gotcha: it is `.u.LowPart`,
NOT `.__bindgen_anon_1.LowPart`, and QuadPart is a plain field, not a
method). rustconv emits the old bindgen accessor-method shapes:

  *X.QuadPart_mut() = V      (*X.QuadPart_mut()) = V   →  X.QuadPart = V
  X.QuadPart()                                          →  X.QuadPart
  *X.__bindgen_anon_1.HighPart()  X.__bindgen_anon_1.HighPart()
                                                        →  X.u.HighPart
  (likewise LowPart)

`X` is any receiver path (ident, `(*p)`, `a.b.c`). Field access on a
union needs `unsafe`, but every freeldr translation body is already an
`unsafe fn` / `unsafe extern`, so bare field syntax type-checks.

Deterministic, idempotent. Reusable across every freeldr unit that
touches FILEINFORMATION / position seeking.

Usage: python3 freeldr_largeint.py <file.rs> [<file2.rs> ...]
"""
import re
import sys

# receiver = greedy-but-balanced-ish path up to the accessor; we only need
# it to stop at the `.QuadPart`/`.__bindgen_anon_1` token, so match the
# minimal run of path chars / deref-parens preceding it.
RECV = r'([A-Za-z_]\w*(?:\s*\.\s*\w+)*|\([^()]*\))'


def fix(path: str) -> None:
    s = open(path).read()
    o = s

    # 1. assignment via *X.QuadPart_mut() = V  /  (*X.QuadPart_mut()) = V
    s = re.sub(r'\(\*\s*' + RECV + r'\s*\.\s*QuadPart_mut\(\)\s*\)\s*=', r'\1.QuadPart =', s)
    s = re.sub(r'\*\s*' + RECV + r'\s*\.\s*QuadPart_mut\(\)\s*=', r'\1.QuadPart =', s)
    # any stray non-deref X.QuadPart_mut() → X.QuadPart
    s = re.sub(RECV + r'\s*\.\s*QuadPart_mut\(\)', r'\1.QuadPart', s)

    # 2. read accessor  X.QuadPart()  → X.QuadPart
    s = re.sub(RECV + r'\s*\.\s*QuadPart\(\)', r'\1.QuadPart', s)

    # 3. nested struct  *X.__bindgen_anon_1.{Low,High}Part()  → X.u.<>
    for part in ("LowPart", "HighPart"):
        s = re.sub(r'\*\s*' + RECV + r'\s*\.\s*__bindgen_anon_1\s*\.\s*' + part + r'\(\)',
                   r'\1.u.' + part, s)
        s = re.sub(RECV + r'\s*\.\s*__bindgen_anon_1\s*\.\s*' + part + r'\(\)',
                   r'\1.u.' + part, s)
        # already-deprefixed-but-method form: X.u.LowPart() → X.u.LowPart
        s = re.sub(RECV + r'\s*\.\s*u\s*\.\s*' + part + r'\(\)', r'\1.u.' + part, s)

    if s != o:
        open(path, "w").write(s)
    print(f"{path}: largeint {'changed' if s != o else 'noop'}")


if __name__ == "__main__":
    for p in sys.argv[1:]:
        fix(p)
