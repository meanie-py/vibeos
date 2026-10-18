"""The stub TU a fully flipped module keeps, and the export anchors it carries.

A module whose every TU is Rust links its code from one staticlib, and an archive member is
loaded only when something asks for one of its symbols. Its exports are asked for last, by
the .def, after the import libraries have been read. A DELAY import library is one object
holding every thunk of its DLL, so a Rust call to any ole32 function loads all of them,
ole32's own `DllGetClassObject` included. When crt-startup's `DllMain` reference then pulls
the module's real one in, it is the duplicate, and `--allow-multiple-definition` keeps the
first: acppage on i386 exported ole32's `DllGetClassObject` thunk (link_guard: "an import
was kept over a local definition"). The C never had this problem because object files load
eagerly. Whether it happened to the Rust depended on the order its codegen units were
pulled in, so sendmail and mydocs, with the same delay imports, linked by luck.

So the stub references every export the .spec requires, by raw symbol name: an object
file's undefined symbol is resolved from the first archive that defines it, which is the
module's own (it is first on the link line). The references are asm labels, not
prototypes, so they cannot clash with the declarations the module's precompiled header
brings in (objbase.h declares `DllGetClassObject`).
"""

from __future__ import annotations

from pathlib import Path

from .spec_parser import SpecEntry, parse_spec

# i386 stdcall/fastcall decoration counts argument bytes: 8 for these, 4 for the rest.
_WIDE_PARAMS = {"int64", "double"}

# Each arch the tree builds, its preprocessor test, and how it spells a C symbol.
_ARCHES = (
    ("i386", "defined(__i386__)"),
    ("amd64", "defined(__x86_64__)"),
    ("arm64", "defined(__aarch64__)"),
)


_ANCHOR_HEAD = "/* Every export the .spec requires, referenced from this OBJECT so the module's own"


def _i386_symbol(entry: SpecEntry) -> str:
    name = entry.symbol
    size = sum(8 if p in _WIDE_PARAMS else 4 for p in entry.params)
    if entry.convention == "stdcall":
        return f"_{name}@{size}"
    if entry.convention == "fastcall":
        return f"@{name}@{size}"
    return f"_{name}"


def _symbols(spec: Path, arch: str) -> list[str]:
    """The symbols `arch`'s link must find in this module, as that arch spells them. Parsed
    per arch: a name-keyed parse of every row collapses a per-arch pair to the last one."""
    out = []
    for e in parse_spec(spec, arch=arch).values():
        if not e.needs_implementation or e.convention == "extern":
            continue
        out.append(_i386_symbol(e) if arch == "i386" else e.symbol)
    return sorted(set(out))


def export_anchors(stem: str, spec: Path) -> str:
    """The anchor block for `spec`'s exports, or "" when there is nothing to anchor."""
    per_arch = [(test, _symbols(spec, arch)) for arch, test in _ARCHES]
    if not any(syms for _, syms in per_arch):
        return ""
    lines = [
        _ANCHOR_HEAD,
        "   archive defines it before an import library can (rustconv/cohab_stub.py). */",
    ]
    for n, (test, syms) in enumerate(per_arch):
        lines.append(f"#{'if' if n == 0 else 'elif'} {test}")
        for i, sym in enumerate(syms):
            lines.append(f'extern char {stem}_export_anchor_{i} __asm__("{sym}");')
        lines.append(f"__attribute__((used)) static const void *const {stem}_export_anchors[] = {{")
        lines += [f"    &{stem}_export_anchor_{i}," for i in range(len(syms))]
        lines += ["    0,", "};"]
    lines.append("#endif")
    return "\n".join(lines) + "\n"


def stub_text(stem: str, crate: str, spec: Path | None) -> str:
    text = (
        f"/* {stem} is Rust (sdk/lib/rust/{crate}); a CMake target needs one TU. */\n"
        f"typedef int {stem}_rust_cohab_placeholder;\n"
    )
    if spec is not None:
        text += export_anchors(stem, spec)
    return text


def module_spec(module: Path) -> Path | None:
    """The module's own .spec, when it has exactly one."""
    specs = sorted(module.glob("*.spec"))
    return specs[0] if len(specs) == 1 else None


def is_generated(text: str, stem: str, crate: str) -> bool:
    """Whether `text` is a stub as `stub_text` writes it, for any spec (so it may be redone)."""
    bare = stub_text(stem, crate, None)
    rest = text[len(bare) :] if text.startswith(bare) else None
    return rest is not None and (not rest or rest.startswith(_ANCHOR_HEAD))
