"""Regression tests for the comment-eating bug in the extern-block dedup
helpers in `postprocess.py`.

The original join loop accumulated every non-empty stripped line into a
"current declaration" until one ended with `;`. A standalone `// comment`
line therefore got concatenated with the next `fn ...;` declaration and was
re-emitted as a single line — which made the function disappear behind the
`//` once Rust parsed it.

These tests pin the comment-aware behaviour we want.
"""

from __future__ import annotations

import sys
from pathlib import Path

# Make the package importable when tests run directly.
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from rustconv.postprocess import (
    _split_extern_decls,
    fix_duplicate_extern_decls,
    fix_extern_vs_local_dups,
)


def test_split_preserves_standalone_line_comment():
    inner = [
        "    // Defined in `mm/rmap.rs` (Rust-side, not in bindgen).",
        "    fn MmDeleteSectionAssociation(Page: PFN_NUMBER);",
    ]
    out = _split_extern_decls(inner)
    assert out == [
        ("comment", "// Defined in `mm/rmap.rs` (Rust-side, not in bindgen)."),
        ("decl", "fn MmDeleteSectionAssociation(Page: PFN_NUMBER);"),
    ]


def test_split_joins_multi_line_decl():
    inner = [
        "    fn Foo(",
        "        Arg: PVOID,",
        "    ) -> NTSTATUS;",
    ]
    out = _split_extern_decls(inner)
    assert len(out) == 1
    kind, text = out[0]
    assert kind == "decl"
    assert text.startswith("fn Foo(")
    assert text.endswith(";")


def test_dedup_does_not_eat_comments():
    code = (
        'extern "system" {\n    // standalone comment between decls\n    fn A();\n    fn B();\n}\n'
    )
    out = fix_duplicate_extern_decls(code)
    assert "fn A()" in out, "fn A was eaten by the comment merge"
    assert "fn B()" in out
    assert "// standalone comment between decls" in out


def test_dedup_drops_real_duplicates_across_blocks():
    code = (
        'extern "system" {\n'
        "    fn A();\n"
        "}\n"
        'extern "system" {\n'
        "    // duplicate of A below\n"
        "    fn A();\n"
        "    fn B();\n"
        "}\n"
    )
    out = fix_duplicate_extern_decls(code)
    # First occurrence kept, second occurrence dropped, comment preserved.
    assert out.count("fn A()") == 1
    assert "fn B()" in out
    assert "// duplicate of A below" in out


def test_extern_vs_local_dup_does_not_eat_comments():
    code = (
        'pub unsafe extern "system" fn LocalImpl() -> NTSTATUS { 0 }\n'
        "\n"
        'extern "system" {\n'
        "    // shared with rmap.rs\n"
        "    fn Other(p: PVOID);\n"
        "    fn LocalImpl() -> NTSTATUS;\n"
        "}\n"
    )
    out = fix_extern_vs_local_dups(code)
    # The locally-defined `LocalImpl` extern decl is removed (correct), but the
    # standalone comment must survive and `Other` must still be declared.
    assert "fn LocalImpl() -> NTSTATUS;" not in out, "Local-duplicate extern decl was not removed"
    assert "fn Other(" in out, "Comment merge ate fn Other"
    assert "// shared with rmap.rs" in out


def test_extern_vs_local_dup_drops_block_with_only_comments():
    # If the only remaining content after stripping local dups is a comment,
    # the now-empty extern block should be removed entirely (no `extern { } `
    # carcass with just a comment inside).
    code = (
        'pub unsafe extern "system" fn LocalImpl() -> NTSTATUS { 0 }\n'
        "\n"
        'extern "system" {\n'
        "    // only this — and the dup below\n"
        "    fn LocalImpl() -> NTSTATUS;\n"
        "}\n"
    )
    out = fix_extern_vs_local_dups(code)
    assert 'extern "system" {' not in out


if __name__ == "__main__":
    # Allow running without pytest: `python tests/test_postprocess_extern_comments.py`
    failures = 0
    for name, fn in list(globals().items()):
        if not name.startswith("test_") or not callable(fn):
            continue
        try:
            fn()
            print(f"PASS  {name}")
        except AssertionError as e:
            failures += 1
            print(f"FAIL  {name}: {e}")
    sys.exit(1 if failures else 0)
