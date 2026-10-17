"""Bare `fn` aliases of nullable C function-pointer typedefs, in crates that already ship.

`typedef HRESULT (*PFN)(..)` translated as `pub type PFN = unsafe extern "C" fn(..);` is a
type whose values can never be null; the C's can. Three shapes make that undefined
behaviour rather than a style point, all found in the tree (2026-10-17):

- a NULL test on one -- cabinet's `FDICreate` checks `(pfnalloc as *const ()).is_null()` on
  a bare-fn parameter, so a NULL from a C caller is UB and the check may be folded away
  (the C returns an error there);
- a `transmute` of a pointer that may be NULL into one -- cmd's `MYEX` from
  `GetProcAddress`, ctfmon's `TF_CreateLangProfileUtil` before its own null check;
- a struct field of one in storage that is ever zeroed.

New crates get `Option<fn>` from `port integrate` (`port._option_fn_aliases`). This applies
the same rewrite to a shipped crate and reports what it touched; crate-repair then fixes the
call sites (`option_fn_call`, `option_is_null`, `some_fn_item`). Without `--apply`, a report.

    python -m rustconv fn-alias-audit cabinet-rust            # report
    python -m rustconv fn-alias-audit cabinet-rust --apply    # rewrite, then crate-repair
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path

from .port import _BARE_FN_ALIAS, _option_fn_aliases

RUST = Path(__file__).resolve().parents[3] / "sdk" / "lib" / "rust"


@dataclass
class Alias:
    name: str
    file: Path
    null_tests: int  # `(x as *const ()).is_null()`, `x as usize == 0`
    transmutes: int  # transmute into the alias
    fields: int  # `name: ALIAS,` lines (struct fields and multi-line parameters)

    @property
    def risky(self) -> bool:
        return bool(self.null_tests or self.transmutes)


def _sources(crate: Path) -> list[Path]:
    import sys

    root = str(RUST.parents[2])
    if root not in sys.path:
        sys.path.insert(0, root)  # tools.source_walk: the pruned walk every scanner uses
    from tools.source_walk import source_files

    return [
        p
        for p in sorted(source_files(crate / "src"))
        if not p.name.startswith(("types_", "bindgen_"))
    ]


def audit(crate: Path) -> list[Alias]:
    files = _sources(crate)
    text = "\n".join(p.read_text(errors="replace") for p in files)
    out = []
    for p in files:
        for m in _BARE_FN_ALIAS.finditer(p.read_text(errors="replace")):
            n = m.group(2)
            # The bindings declared with the alias (`pfnalloc: PFNALLOC`), and NULL tests of
            # exactly those names.
            bound = set(re.findall(rf"\b(\w+)\s*:\s*{n}\b", text))
            tests = 0
            for b in bound:
                tests += len(
                    re.findall(
                        rf"\(\s*(?:\w+\s*\.\s*)*{b}\s+as\s+\*const\s*\(\)\s*\)\s*\.\s*is_null\(\)"
                        rf"|\(\s*(?:\w+\s*\.\s*)*{b}\s+as\s+usize\s*\)\s*==\s*0",
                        text,
                    )
                )
            out.append(
                Alias(
                    n,
                    p,
                    null_tests=tests,
                    transmutes=len(
                        re.findall(
                            rf"transmute\s*::\s*<[^>]*,\s*{n}\s*>"
                            rf"|:\s*{n}\s*=\s*(?:\w+::)*transmute",
                            text,
                        )
                    ),
                    fields=len(re.findall(rf"^\s*(?:pub\s+)?\w+\s*:\s*{n}\s*,", text, re.M)),
                )
            )
    return out


def apply(crate: Path) -> list[str]:
    """Rewrite every bare alias to `Option<fn>`, and each NULL test of a binding of one
    (`(pfnalloc as *const ()).is_null()`, which an Option cannot be cast for) to
    `.is_none()`: the test the C makes, now one the compiler must keep. Returns the names."""
    files = _sources(crate)
    names = []
    for p in files:
        names += _option_fn_aliases(p)
    if not names:
        return names
    text = "\n".join(p.read_text(errors="replace") for p in files)
    alt = "|".join(map(re.escape, names))
    bound = set(re.findall(rf"\b(\w+)\s*:\s*(?:{alt})\b", text))
    if bound:
        balt = "|".join(map(re.escape, sorted(bound)))
        test = re.compile(
            rf"\(\s*((?:\(\s*\*\s*\w+\s*\)\s*\.\s*|\w+\s*\.\s*)*(?:{balt}))\s+as\s+"
            rf"\*const\s+(?:\(\s*\)|(?:core::ffi::)?c_void)\s*\)\s*\.\s*is_null\(\)"
        )
        for p in files:
            t = p.read_text(errors="replace")
            new = test.sub(lambda m: f"{m.group(1)}.is_none()", t)
            if new != t:
                p.write_text(new)
    return names


def add_parser(sub) -> None:
    p = sub.add_parser("fn-alias-audit", help="bare fn aliases of nullable C fn pointers")
    p.add_argument("crate", help="crate directory name under sdk/lib/rust (cabinet-rust)")
    p.add_argument("--apply", action="store_true", help="rewrite to Option<fn>, then crate-repair")


def main(args) -> int:
    crate = RUST / args.crate
    if not (crate / "Cargo.toml").is_file():
        print(f"fn-alias-audit: no crate {crate}")
        return 2
    found = audit(crate)
    for a in found:
        tag = "RISK" if a.risky else "    "
        rel = a.file.relative_to(RUST)
        print(
            f"{tag} {rel}: {a.name} null_tests={a.null_tests} transmutes={a.transmutes} "
            f"field_lines={a.fields}"
        )
    print(
        f"{len(found)} bare alias(es), {sum(a.risky for a in found)} with a null test or transmute"
    )
    if not args.apply or not found:
        return 0
    names = apply(crate)
    print(f"rewritten to Option<fn>: {', '.join(names)}")
    from . import crate_repair

    session = crate_repair.repair_crate(
        args.crate,
        root=RUST,
        targets=["i386", "amd64", "arm64"],
    )
    crate_repair.print_session(session)
    return 0 if session.status not in ("regressed", "max_passes") else 1
