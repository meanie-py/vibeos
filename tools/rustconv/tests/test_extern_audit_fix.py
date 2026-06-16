"""extern-audit `--fix` rebuild guarantees.

When the fixer rewrites a convention-mismatched extern decl into cfg-gated
blocks, it must preserve the declaration verbatim — including a `pub` visibility
prefix. (A 2026-06 review flagged a suspected `pub`-drop here; it turned out to
be a false alarm — the visibility rides along in the extracted source line — but
the guarantee is worth pinning so it can't regress into a real bug.)
"""

from pathlib import Path

from rustconv.extern_audit import (
    AuditMismatch,
    ExternDecl,
    FnDefinition,
    fix_file,
)


def _mismatch(name: str, decl_conv: str, def_conv: str) -> AuditMismatch:
    return AuditMismatch(
        decl=ExternDecl(name=name, convention=decl_conv, file="x.rs", line=2),
        defn=FnDefinition(name=name, convention=def_conv, file="x.rs", line=2),
        spec_convention=None,
        severity="critical",
        detail="",
    )


def test_fix_preserves_pub_visibility(tmp_path: Path):
    rs = tmp_path / "x.rs"
    rs.write_text('extern "C" {\n    pub fn KeFoo(x: u32);\n}\n')

    # decl says "C", definition says "fastcall" -> target=fastcall -> rewrite.
    applied = fix_file(rs, [_mismatch("KeFoo", "C", "fastcall")])
    out = rs.read_text()

    assert applied == 1
    # cfg-gated rebuild, and `pub` survives in BOTH arms.
    assert out.count("pub fn KeFoo(x: u32);") == 2
    assert 'extern "fastcall"' in out
    assert 'target_arch = "x86"' in out


def test_fix_preserves_non_pub_decl(tmp_path: Path):
    rs = tmp_path / "x.rs"
    rs.write_text('extern "C" {\n    fn KeBar(x: u32);\n}\n')

    applied = fix_file(rs, [_mismatch("KeBar", "C", "fastcall")])
    out = rs.read_text()

    assert applied == 1
    # no spurious `pub` is introduced for a decl that didn't have one.
    assert "pub fn KeBar" not in out
    assert out.count("fn KeBar(x: u32);") == 2
