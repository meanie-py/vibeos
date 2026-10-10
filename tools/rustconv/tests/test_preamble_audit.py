"""The preamble-coverage audit (rustconv.preamble_audit).

`translate` sends the file-scope preamble to the LLM as one unit and pastes back
whatever returns. Nothing checked the output covered the input — and on
ole32/storage32.c the model was handed a 49,870-char preamble containing 10 COM
vtable tables and returned 143 lines containing NONE of them. That does not break
the build; it produces COM objects with no methods.

The regression test that matters is `TestAgainstTheRealPaidPreamble`: the actual
`.responses/` artifact from that run, audited, must name all 10 dropped tables —
and must NOT cry wolf about the structs/typedefs/#defines the model *did*
translate (measured: zero false positives, zero false negatives on that file).
"""

from pathlib import Path

import pytest

from rustconv.config import REACTOS_ROOT
from rustconv.preamble_audit import audit_preamble, preamble_decls

STORAGE32_C = REACTOS_ROOT / "dll" / "win32" / "ole32" / "storage32.c"
# The preamble a paid storage32.c translation returned (2026-07-12), kept as a
# committed fixture: the .responses/ bank entry it came from was deleted once ole32
# was fully integrated, and the bank is gitignored, so it never reached CI anyway.
PAID_PREAMBLE = Path(__file__).parent / "fixtures" / "storage32_paid_preamble.rs"


# ---------------------------------------------------------------------------
# What the C defines
# ---------------------------------------------------------------------------


class TestPreambleDecls:
    def test_storage32_inventory(self):
        kinds: dict[str, int] = {}
        for d in preamble_decls(STORAGE32_C):
            kinds[d.kind] = kinds.get(d.kind, 0) + 1
        assert kinds["table"] == 10
        assert kinds["struct"] == 9
        assert kinds["enum"] == 1
        assert kinds["variable"] == 2  # STORAGE_magic, STORAGE_oldmagic

    def test_bare_file_scope_struct_definitions_are_found(self, tmp_path: Path):
        """`struct Bare { int b; };` at file scope parses as a top-level
        struct_specifier, NOT as a `declaration`. Handling only `declaration` would
        silently drop 4 of storage32.c's 9 structs."""
        c = tmp_path / "bare.c"
        c.write_text("struct Bare { int b; };\n")
        assert [(d.kind, d.name) for d in preamble_decls(c)] == [("struct", "Bare")]
        # The real file: these four are bare top-level struct_specifiers.
        names = {d.name for d in preamble_decls(STORAGE32_C) if d.kind == "struct"}
        assert {
            "BlockChainRun",
            "BlockChainStream",
            "SmallBlockChainStream",
            "IEnumSTATSTGImpl",
        } <= names

    def test_vtbl_tables_are_classified_as_tables_not_variables(self):
        """The `table` kind is what makes the finding say 'NO METHODS' instead of a
        generic 'missing global'."""
        tables = {d.name for d in preamble_decls(STORAGE32_C) if d.kind == "table"}
        assert "StorageImpl_Vtbl" in tables
        assert "StorageImpl_BaseVtbl" in tables
        assert len(tables) == 10

    def test_a_forward_declared_table_is_not_also_a_variable(self):
        """storage32.c forward-declares two tables before defining them; the decl and
        the definition are one symbol, so it must not be reported twice."""
        names = [d.name for d in preamble_decls(STORAGE32_C)]
        assert names.count("StorageInternalImpl_Vtbl") == 1
        assert names.count("TransactedSnapshotImpl_Vtbl") == 1

    def test_extern_declarations_are_not_definitions(self, tmp_path: Path):
        """storage32.c:64 `extern const IPropertySetStorageVtbl IPropertySetStorage_Vtbl;`
        lives in another TU — the Rust preamble is RIGHT not to define it."""
        assert "IPropertySetStorage_Vtbl" not in {d.name for d in preamble_decls(STORAGE32_C)}

    def test_function_prototypes_are_not_declarations(self, tmp_path: Path):
        c = tmp_path / "p.c"
        c.write_text("static HRESULT Foo_AddRef(IFoo *iface);\nstatic int g = 3;\n")
        names = {d.name for d in preamble_decls(c)}
        assert names == {"g"}


# ---------------------------------------------------------------------------
# THE REGRESSION: the real paid artifact
# ---------------------------------------------------------------------------


@pytest.mark.skipif(not PAID_PREAMBLE.is_file(), reason="storage32 preamble fixture missing")
class TestAgainstTheRealPaidPreamble:
    RUST = PAID_PREAMBLE.read_text(encoding="utf-8") if PAID_PREAMBLE.is_file() else ""

    def test_reports_all_ten_dropped_vtbl_tables(self):
        findings = audit_preamble(STORAGE32_C, self.RUST)
        vtbl = [f for f in findings if f.startswith("MISSING COM VTABLE")]
        assert len(vtbl) == 10
        for name in (
            "DirectWriterLockVtbl",
            "IEnumSTATSTGImpl_Vtbl",
            "StorageImpl_Vtbl",
            "StorageImpl_BaseVtbl",
            "StorageInternalImpl_Vtbl",
            "StorageInternalImpl_BaseVtbl",
            "TransactedSnapshotImpl_Vtbl",
            "TransactedSnapshotImpl_BaseVtbl",
            "TransactedSharedImpl_Vtbl",
            "TransactedSharedImpl_BaseVtbl",
        ):
            assert any(f"'{name}'" in f for f in vtbl), f"{name} not reported missing"

    def test_the_finding_says_what_actually_breaks(self):
        findings = audit_preamble(STORAGE32_C, self.RUST)
        assert any("NO METHODS" in f for f in findings)

    def test_no_false_positives(self):
        """The model DID translate all 5 structs, 8 typedefs, 2 #defines and both
        file-scope variables. Only the 10 tables are missing — so the audit must
        report exactly 10 findings and nothing else, even in strict mode."""
        assert len(audit_preamble(STORAGE32_C, self.RUST)) == 10
        assert len(audit_preamble(STORAGE32_C, self.RUST, strict=True)) == 10


# ---------------------------------------------------------------------------
# Coverage detection + false-positive suppression
# ---------------------------------------------------------------------------

C_MIXED = """
#define STG_MAGIC 0xd0cf11e0
typedef struct TagFoo { int a; } Foo;
struct Bare { int b; };
static const unsigned char magic[2] = { 1, 2 };
static const IUnknownVtbl Impl_Vtbl = { Impl_QI, Impl_AddRef, Impl_Release };
static int helper(int x) { return x + 1; }
"""


def _c(tmp_path: Path, text: str = C_MIXED) -> Path:
    p = tmp_path / "mixed.c"
    p.write_text(text)
    return p


class TestAuditCoverage:
    def test_a_complete_preamble_produces_no_findings(self, tmp_path: Path):
        rust = """
        pub const STG_MAGIC: u32 = 0xd0cf11e0;
        pub struct Foo { pub a: i32 }
        pub struct Bare { pub b: i32 }
        pub static magic: [u8; 2] = [1, 2];
        pub static Impl_Vtbl: IUnknownVtbl = IUnknownVtbl {
            QueryInterface: Some(Impl_QI), AddRef: Some(Impl_AddRef), Release: Some(Impl_Release),
        };
        """
        assert audit_preamble(_c(tmp_path), rust, strict=True) == []

    def test_a_dropped_table_is_reported_even_if_everything_else_is_present(self, tmp_path: Path):
        rust = """
        pub const STG_MAGIC: u32 = 0xd0cf11e0;
        pub struct Foo { pub a: i32 }
        pub struct Bare { pub b: i32 }
        pub static magic: [u8; 2] = [1, 2];
        """
        findings = audit_preamble(_c(tmp_path), rust)
        assert len(findings) == 1
        assert findings[0].startswith("MISSING COM VTABLE 'Impl_Vtbl'")

    def test_a_dropped_struct_is_reported(self, tmp_path: Path):
        rust = """
        pub const STG_MAGIC: u32 = 0xd0cf11e0;
        pub struct Foo { pub a: i32 }
        pub static magic: [u8; 2] = [1, 2];
        pub static Impl_Vtbl: IUnknownVtbl = IUnknownVtbl { QueryInterface: Some(Impl_QI) };
        """
        findings = audit_preamble(_c(tmp_path), rust)
        assert [f for f in findings if "'Bare'" in f]

    def test_a_dropped_static_table_is_never_suppressed_by_a_mere_mention(self, tmp_path: Path):
        """Referencing `Impl_Vtbl` (e.g. `iface.lpVtbl = &Impl_Vtbl`) is NOT defining
        it. The critical kinds require a real definition."""
        rust = "let x = &Impl_Vtbl;\npub struct Foo { pub a: i32 }\npub struct Bare {}\n"
        findings = audit_preamble(_c(tmp_path), rust)
        assert any(f.startswith("MISSING COM VTABLE 'Impl_Vtbl'") for f in findings)


class TestFalsePositiveSuppression:
    def test_typedef_folded_into_one_struct_is_not_reported(self, tmp_path: Path):
        """C `typedef struct TagFoo { .. } Foo;` becomes ONE `pub struct Foo` — the
        tag `TagFoo` never appears, and that is correct, not a drop."""
        c = _c(tmp_path, "typedef struct TagFoo { int a; } Foo;\n")
        assert audit_preamble(c, "pub struct Foo { pub a: i32 }", strict=True) == []

    def test_self_named_typedef_is_not_double_reported(self, tmp_path: Path):
        c = _c(tmp_path, "typedef struct Foo Foo;\nstruct Foo { int a; };\n")
        assert audit_preamble(c, "pub struct Foo { pub a: i32 }", strict=True) == []

    def test_an_inlined_define_is_quiet_by_default(self, tmp_path: Path):
        """A `#define` the function bodies inlined rather than emitting as a const is
        a legitimate translation. Quiet by default; visible under strict."""
        c = _c(tmp_path, "#define SOME_LIMIT 32\n")
        assert audit_preamble(c, "") == []
        assert audit_preamble(c, "", strict=True) == [
            "missing #define 'SOME_LIMIT' (= 32) — mixed.c:1."
        ]

    def test_include_guards_are_never_reported(self, tmp_path: Path):
        c = _c(tmp_path, "#define __STORAGE32_H 1\n#define _FOO_INCLUDED 1\n")
        assert audit_preamble(c, "", strict=True) == []

    def test_an_enum_emitted_as_consts_is_covered(self, tmp_path: Path):
        """storage32.c:10609 `enum stream_1ole_flags { OleStream_LinkedObject = 1,
        OleStream_Convert = 4 };` was translated as two `pub const`s. A C enum's
        payload IS its variants, so nothing was lost — do not cry wolf."""
        c = _c(tmp_path, "enum flags { A_FLAG = 1, B_FLAG = 4 };\n")
        rust = "pub const A_FLAG: u32 = 1;\npub const B_FLAG: u32 = 4;"
        assert audit_preamble(c, rust, strict=True) == []

    def test_an_enum_missing_a_variant_is_still_reported(self, tmp_path: Path):
        """The suppression is 'all variants present', not 'enums are ignored'."""
        c = _c(tmp_path, "enum flags { A_FLAG = 1, B_FLAG = 4 };\n")
        findings = audit_preamble(c, "pub const A_FLAG: u32 = 1;")
        assert [f for f in findings if "'flags'" in f]

    def test_rust_full_covers_a_const_the_preamble_omitted(self, tmp_path: Path):
        c = _c(tmp_path, "static const int Threshold = 5;\n")
        assert audit_preamble(c, "") != []
        assert audit_preamble(c, "", rust_full="pub const Threshold: i32 = 5;") == []


# ---------------------------------------------------------------------------
# Parse robustness: locals must never be reported as file-scope state
# ---------------------------------------------------------------------------


class TestUnparseableTopLevel:
    """tree-sitter recovers from a bad parameter list by HOISTING the function
    body's statements to the root, where every local looks like a file-scope
    definition. rasadhlp/winsock.c reported its locals `Ip`, `NetBios`,
    `AutodialAddress`, `Name` and `AutodialConnection` as dropped file-scope
    state; all five were present in the translation, as locals. Five confident,
    specific, wrong findings is how an oracle stops being read."""

    SRC = (
        "#include <foo.h>\n"
        "BOOL\n"
        "WINAPI\n"
        "DoThing(IN CONST SOCKADDR FAR *Name,\n"
        "        IN INT NameLength)\n"
        "{\n"
        "    PSOCKADDR_IN Ip = (PSOCKADDR_IN)Name;\n"
        "    AUTODIAL_ADDR AutodialAddress;\n"
        "    return Send(&AutodialAddress, Ip);\n"
        "}\n"
    )

    def test_far_qualified_params_do_not_hoist_locals(self, tmp_path: Path):
        c = tmp_path / "winsock.c"
        c.write_text(self.SRC)
        # the locals are NOT file-scope, so there is nothing to report and
        # nothing to define in the preamble
        assert preamble_decls(c) == []
        assert audit_preamble(c, "", strict=True) == []

    def test_real_file_scope_state_beside_a_far_param_is_still_caught(self, tmp_path: Path):
        """The fix must restore coverage, not suppress the check."""
        c = tmp_path / "winsock.c"
        c.write_text("static const int RealGlobal = 7;\n" + self.SRC)
        findings = audit_preamble(c, "")
        assert len(findings) == 1
        assert "MISSING file-scope definition 'RealGlobal'" in findings[0]

    def test_a_genuinely_unparseable_top_level_says_so_instead_of_guessing(self, tmp_path: Path):
        """Silence would read as 'audited, clean'. It must read as 'not audited'."""
        c = tmp_path / "broken.c"
        c.write_text("static int Real = 1;\n}\n)\n")
        findings = audit_preamble(c, "")
        assert len(findings) == 1
        assert findings[0].startswith("PREAMBLE AUDIT INCOMPLETE")
        assert "SKIPPED, not passed" in findings[0]


class TestACppTranslationUnitIsParsedAsCpp:
    """A `.cpp` parsed with the C grammar is INCOMPLETE by construction, not by defect.

    `class X : public Y {` is not C, so the C grammar makes it a top-level ERROR and the
    audit — correctly refusing to mine parse debris — skipped every file-scope check.
    That hit 53 of shell32's 84 `.cpp`, including the two hand-rolled COM classes in
    shellrecyclebin, i.e. exactly the files most likely to carry a vtable to drop.
    """

    CPP = """\
struct IThing { virtual long Query(void**) = 0; };
static const unsigned int g_Table[3] = { 1, 2, 3 };
enum Mode { ModeA = 1, ModeB = 2 };
class CFoo : public IThing {
public:
    CFoo();
    long Query(void**);
private:
    int m_member;
};
"""

    def test_a_cpp_class_no_longer_defeats_the_audit(self, tmp_path: Path):
        source = tmp_path / "thing.cpp"
        source.write_text(self.CPP)
        findings = audit_preamble(source, "")
        assert not any(f.startswith("PREAMBLE AUDIT INCOMPLETE") for f in findings)

    def test_file_scope_state_beside_a_cpp_class_is_still_caught(self, tmp_path: Path):
        """Parsing further must widen coverage, not quietly find nothing."""
        source = tmp_path / "thing.cpp"
        source.write_text(self.CPP)
        names = {decl.name for decl in preamble_decls(source)}
        assert {"g_Table", "Mode", "IThing"} <= names
        # An instance member is not file-scope state; reporting it would be the
        # parse-debris false positive this audit refuses to produce.
        assert "m_member" not in names

    def test_the_shellrecyclebin_tu_that_motivated_this_now_audits(self):
        source = (
            REACTOS_ROOT
            / "dll"
            / "win32"
            / "shell32"
            / "shellrecyclebin"
            / "recyclebin_v5_enumerator.cpp"
        )
        findings = audit_preamble(source, "")
        assert not any(f.startswith("PREAMBLE AUDIT INCOMPLETE") for f in findings)
