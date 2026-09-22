"""Deterministic COM vtbl-table generation (rustconv.vtbl_gen).

A vtbl table is a positional struct initializer whose slot order we already own
(the bindgen'd com-sys structs), so it must NEVER be handed to a language model:
the paid preamble translation of ole32/storage32.c silently dropped all 10 of its
tables, which does not break the build — it produces COM objects with **no
methods**, discoverable only at runtime. 23 such tables across ole32 had to be
hand-authored.

The strongest test here is `TestStorage32AgainstHandWritten`: it regenerates
storage32.c's tables and compares them slot-for-slot with the ones a human wrote
into `ole32-rust`. Two independent derivations of 149 slots agreeing is a real
oracle, not a snapshot of our own output.
"""

from pathlib import Path

import pytest

from rustconv.config import REACTOS_ROOT
from rustconv.vtbl_gen import (
    FieldOrderResolver,
    VtblArityError,
    VtblResolveError,
    _rust_struct_fields,
    com_method_names,
    find_vtbl_tables,
    render_vtbl_tables,
)

STORAGE32_C = REACTOS_ROOT / "dll" / "win32" / "ole32" / "storage32.c"
OLE32_RUST = REACTOS_ROOT / "sdk" / "lib" / "rust" / "ole32-rust" / "src" / "ring" / "storage32.rs"
OLEAUT32 = REACTOS_ROOT / "dll" / "win32" / "oleaut32"
COM_SYS_SRC = REACTOS_ROOT / "sdk" / "lib" / "rust" / "com-sys" / "src"


# ---------------------------------------------------------------------------
# The Rust struct reader (field order + Option-ness ground truth)
# ---------------------------------------------------------------------------


class TestRustStructFields:
    def test_reads_fields_in_declaration_order(self):
        rs = (
            "pub struct IFooVtbl {\n"
            "    pub QueryInterface: ::core::option::Option<\n"
            '        unsafe extern "stdcall" fn(This: *mut IFoo, riid: *const IID) -> HRESULT,\n'
            "    >,\n"
            "    pub AddRef: ::core::option::Option<\n"
            '        unsafe extern "stdcall" fn(This: *mut IFoo) -> ULONG,\n'
            "    >,\n"
            "}\n"
        )
        fields = _rust_struct_fields(rs, "IFooVtbl")
        assert [f.name for f in fields] == ["QueryInterface", "AddRef"]

    def test_a_multiline_generic_fn_type_is_not_mistaken_for_more_fields(self):
        """The killer case: a slot's `fn(..)` params are themselves `name: type`
        pairs on their own lines. Only nesting depth distinguishes a real field."""
        rs = COM_SYS_SRC.joinpath("bindgen_i386.rs").read_text(encoding="utf-8")
        fields = _rust_struct_fields(rs, "IStorageVtbl")
        # IStorage has exactly 18 slots (IUnknown's 3 + 15 of its own).
        assert len(fields) == 18
        assert [f.name for f in fields][:4] == [
            "QueryInterface",
            "AddRef",
            "Release",
            "CreateStream",
        ]
        assert fields[-1].name == "Stat"

    def test_bindgen_slots_are_option_wrapped(self):
        """Whether we emit `Some(f)` or bare `f` is READ, never assumed."""
        rs = COM_SYS_SRC.joinpath("bindgen_i386.rs").read_text(encoding="utf-8")
        fields = _rust_struct_fields(rs, "IUnknownVtbl")
        assert all(f.is_option for f in fields)

    def test_unknown_struct_returns_none(self):
        assert _rust_struct_fields("pub struct Other { pub a: u32 }", "IFooVtbl") is None


class TestSlotOrderIsArchIndependent:
    """Field ORDER is identical on all three arches — only the ABI on each slot
    differs (stdcall on i386, C on amd64/arm64). vtbl_gen relies on that to read
    slot order from a single bindgen file; pin it so a regen cannot break it."""

    @pytest.mark.parametrize(
        "iface", ["IUnknownVtbl", "IStorageVtbl", "IDispatchVtbl", "IPictureVtbl"]
    )
    def test_all_three_arches_agree(self, iface: str):
        orders = {}
        for arch in ("i386", "amd64", "arm64"):
            text = COM_SYS_SRC.joinpath(f"bindgen_{arch}.rs").read_text(encoding="utf-8")
            fields = _rust_struct_fields(text, iface)
            assert fields, f"{iface} missing from bindgen_{arch}.rs"
            orders[arch] = [f.name for f in fields]
        assert orders["i386"] == orders["amd64"] == orders["arm64"]


# ---------------------------------------------------------------------------
# Finding tables in the C
# ---------------------------------------------------------------------------


class TestFindVtblTables:
    def test_storage32_finds_all_ten_definitions(self):
        tables = find_vtbl_tables(STORAGE32_C)
        assert [t.name for t in tables] == [
            "DirectWriterLockVtbl",
            "IEnumSTATSTGImpl_Vtbl",
            "StorageImpl_BaseVtbl",
            "StorageImpl_Vtbl",
            "StorageInternalImpl_Vtbl",
            "StorageInternalImpl_BaseVtbl",
            "TransactedSnapshotImpl_Vtbl",
            "TransactedSnapshotImpl_BaseVtbl",
            "TransactedSharedImpl_Vtbl",
            "TransactedSharedImpl_BaseVtbl",
        ]

    def test_forward_declarations_are_not_counted_twice(self):
        """storage32.c forward-declares StorageInternalImpl_Vtbl (L84) and
        TransactedSnapshotImpl_Vtbl (L153) before defining them. A forward decl has
        no initializer, so it contributes no table — the naive grep that says
        '12 tables' is counting these."""
        tables = find_vtbl_tables(STORAGE32_C)
        names = [t.name for t in tables]
        assert len(names) == len(set(names)) == 10
        by_name = {t.name: t for t in tables}
        # The DEFINITION line is recorded, not the forward-declaration line.
        assert by_name["StorageInternalImpl_Vtbl"].line == 5487
        assert by_name["TransactedSnapshotImpl_Vtbl"].line == 6397

    def test_internal_non_com_vtbl_resolves_from_the_c_header(self):
        """StorageBaseImplVtbl is ole32-internal (storage32.h), not a COM interface,
        so it is absent from com-sys and its slots are PLAIN C fn pointers — no
        STDMETHODCALLTYPE, hence no `Option`, hence no `Some(..)`."""
        t = {x.name: x for x in find_vtbl_tables(STORAGE32_C)}["StorageImpl_BaseVtbl"]
        assert t.type_name == "StorageBaseImplVtbl"
        assert t.origin == "c-header:storage32.h"
        assert len(t.slots) == 16
        assert not any(s.is_option for s in t.slots)
        assert "Destroy: StorageImpl_Destroy," in t.render()
        assert "Some(" not in t.render()

    def test_com_interface_slots_are_some_wrapped(self):
        t = {x.name: x for x in find_vtbl_tables(STORAGE32_C)}["StorageImpl_Vtbl"]
        assert t.origin.startswith("com-sys/")
        assert all(s.is_option for s in t.slots)
        assert "QueryInterface: Some(StorageBaseImpl_QueryInterface)," in t.render()

    def test_static_without_const_is_found(self):
        """oleaut32/olepropframe.c:142 is `static IPropertyPageSiteVtbl ... = {`
        — no `const`. Matching on line shape would miss it; we parse the AST."""
        tables = find_vtbl_tables(OLEAUT32 / "olepropframe.c")
        assert [t.name for t in tables] == ["PropertyPageSiteVtbl"]
        assert tables[0].type_name == "IPropertyPageSiteVtbl"
        assert len(tables[0].slots) == 7

    def test_type_and_name_split_across_lines_is_found(self):
        """oleaut32/olefont.c:1807 puts the type on one line and the table name on
        the next."""
        names = [t.name for t in find_vtbl_tables(OLEAUT32 / "olefont.c")]
        assert "OLEFontImpl_IConnectionPointContainer_VTable" in names

    def test_file_with_no_tables_yields_nothing(self):
        assert find_vtbl_tables(OLEAUT32 / "varformat.c") == []
        assert render_vtbl_tables(OLEAUT32 / "varformat.c") == ""

    @pytest.mark.parametrize(
        ("c_file", "count"),
        [
            ("connpt.c", 2),
            ("olefont.c", 6),
            ("olepicture.c", 5),
            ("olepropframe.c", 1),
            ("recinfo.c", 1),
            ("varformat.c", 0),
        ],
    )
    def test_oleaut32_table_counts(self, c_file: str, count: int):
        assert len(find_vtbl_tables(OLEAUT32 / c_file)) == count


# ---------------------------------------------------------------------------
# THE ORACLE: regenerated vs. hand-authored
# ---------------------------------------------------------------------------


def _parse_hand_written(text: str) -> dict[str, tuple[str, list[tuple[str, str]]]]:
    """`pub static NAME: TY = TY { field: Some(v), .. };` -> {NAME: (TY, [(field, v)])}."""
    import re

    out: dict[str, tuple[str, list[tuple[str, str]]]] = {}
    for m in re.finditer(r"pub static (\w+):\s*(\w+)\s*=\s*\w+\s*\{(.*?)\n\};", text, re.S):
        slots = [
            (fm.group(1), fm.group(2) or fm.group(3))
            for fm in re.finditer(r"(\w+):\s*(?:Some\(([\w:]+)\)|(\w+))\s*,", m.group(3))
        ]
        out[m.group(1)] = (m.group(2), slots)
    return out


class TestStorage32AgainstHandWritten:
    """Cross-check the generator against the tables a human hand-authored into
    ole32-rust after the LLM dropped them. Two independent derivations agreeing on
    all 149 slots is a genuine correctness oracle."""

    HAND = _parse_hand_written(OLE32_RUST.read_text(encoding="utf-8"))

    def test_the_oracle_itself_parsed(self):
        assert len(self.HAND) == 10, "hand-written ole32 tables moved — update the oracle"

    def test_same_set_of_tables(self):
        gen = {t.name for t in find_vtbl_tables(STORAGE32_C)}
        assert gen == set(self.HAND)

    def test_every_slot_matches_field_and_value(self):
        mismatches: list[str] = []
        total = 0
        for t in find_vtbl_tables(STORAGE32_C):
            hand_ty, hand_slots = self.HAND[t.name]
            gen_slots = [(s.field, s.value) for s in t.slots]
            if hand_ty != t.type_name:
                mismatches.append(f"{t.name}: type {hand_ty} != {t.type_name}")
            if len(hand_slots) != len(gen_slots):
                mismatches.append(
                    f"{t.name}: {len(hand_slots)} hand slots != {len(gen_slots)} generated"
                )
                continue
            for i, (h, g) in enumerate(zip(hand_slots, gen_slots, strict=True)):
                total += 1
                if h != g:
                    mismatches.append(f"{t.name} slot {i}: hand={h} generated={g}")
        assert not mismatches, "generator disagrees with hand-written ole32:\n" + "\n".join(
            mismatches
        )
        assert total == 149, f"expected 149 cross-checked slots, got {total}"


# ---------------------------------------------------------------------------
# Rendering, method names, and the FATAL arity check
# ---------------------------------------------------------------------------


class TestRender:
    def test_renders_every_table_with_provenance(self):
        rust = render_vtbl_tables(STORAGE32_C)
        for t in find_vtbl_tables(STORAGE32_C):
            assert f"pub static {t.name}: {t.type_name} = {t.type_name} {{" in rust
        assert "storage32.c:5209" in rust  # provenance comment

    def test_all_oleaut32_targets_render(self):
        for c in ("connpt.c", "olefont.c", "olepicture.c", "olepropframe.c", "recinfo.c"):
            rust = render_vtbl_tables(OLEAUT32 / c)
            assert rust.strip(), f"{c} rendered empty"
            assert "pub static" in rust


class TestComMethodNames:
    def test_reports_the_functions_occupying_slots(self):
        names = com_method_names(STORAGE32_C)
        # Filled by many different tables; this is the "these are COM methods" set
        # the ABI enforcer needs (they must be `extern "system"`).
        assert "StorageBaseImpl_QueryInterface" in names
        assert "IEnumSTATSTGImpl_Next" in names
        assert "StorageImpl_Destroy" in names  # plain-C baseVtbl slot counts too

    def test_recinfo_has_all_nineteen(self):
        assert len(com_method_names(OLEAUT32 / "recinfo.c")) == 19


C_ARITY_SHORT = """
typedef struct IFoo IFoo;
static const IUnknownVtbl Foo_Vtbl =
{
  Foo_QueryInterface,
  Foo_AddRef
};
"""

C_ARITY_LONG = """
static const IUnknownVtbl Foo_Vtbl =
{
  Foo_QueryInterface,
  Foo_AddRef,
  Foo_Release,
  Foo_Extra
};
"""

C_DESIGNATED_BAD = """
static const IUnknownVtbl Foo_Vtbl =
{
  .QueryInterface = Foo_QueryInterface,
  .AddRef = Foo_AddRef,
  .Nonexistent = Foo_Release
};
"""


class TestArityIsFatal:
    """A mis-zipped vtbl dispatches calls to the WRONG method — silent, and far
    worse than a build break. Never pad, never truncate: raise."""

    def test_too_few_initializers_raises(self, tmp_path: Path):
        c = tmp_path / "short.c"
        c.write_text(C_ARITY_SHORT)
        with pytest.raises(VtblArityError) as e:
            find_vtbl_tables(c)
        msg = str(e.value)
        assert "Foo_Vtbl" in msg and "IUnknownVtbl" in msg
        assert "2 initializer(s)" in msg and "3 field(s)" in msg

    def test_too_many_initializers_raises(self, tmp_path: Path):
        c = tmp_path / "long.c"
        c.write_text(C_ARITY_LONG)
        with pytest.raises(VtblArityError) as e:
            find_vtbl_tables(c)
        assert "4 initializer(s)" in str(e.value) and "3 field(s)" in str(e.value)

    def test_unknown_designated_field_raises(self, tmp_path: Path):
        c = tmp_path / "desig.c"
        c.write_text(C_DESIGNATED_BAD)
        with pytest.raises(VtblArityError, match="Nonexistent"):
            find_vtbl_tables(c)

    def test_unresolvable_vtbl_type_raises_not_guesses(self, tmp_path: Path):
        c = tmp_path / "unknown.c"
        c.write_text("static const IMadeThisUpVtbl X = { a, b, c };\n")
        with pytest.raises(VtblResolveError, match="IMadeThisUpVtbl"):
            find_vtbl_tables(c, c_headers=[])


C_GOOD = """
/* comments between entries must not be counted as slots */
static const IUnknownVtbl Foo_Vtbl =
{
  Foo_QueryInterface,  /* slot 0 */
  /* slot 1 */
  Foo_AddRef,
  NULL
};
"""

C_DESIGNATED = """
static const IUnknownVtbl Bar_Vtbl =
{
  .QueryInterface = Bar_QueryInterface,
  .AddRef = Bar_AddRef,
  .Release = Bar_Release
};
"""


class TestInitializerForms:
    def test_comments_are_not_slots_and_null_becomes_none(self, tmp_path: Path):
        c = tmp_path / "good.c"
        c.write_text(C_GOOD)
        (t,) = find_vtbl_tables(c)
        assert [s.field for s in t.slots] == ["QueryInterface", "AddRef", "Release"]
        assert t.slots[2].is_null
        assert "Release: None," in t.render()
        # A NULL slot is not a method.
        assert t.method_names == ["Foo_QueryInterface", "Foo_AddRef"]

    def test_designated_initializers_map_by_name(self, tmp_path: Path):
        c = tmp_path / "desig.c"
        c.write_text(C_DESIGNATED)
        (t,) = find_vtbl_tables(c)
        assert [(s.field, s.value) for s in t.slots] == [
            ("QueryInterface", "Bar_QueryInterface"),
            ("AddRef", "Bar_AddRef"),
            ("Release", "Bar_Release"),
        ]


class TestResolverLayering:
    def test_com_sys_wins_over_local_sources(self, tmp_path: Path):
        """A DLL-local Rust file must never override the real COM slot order."""
        decoy = tmp_path / "decoy.rs"
        decoy.write_text("pub struct IUnknownVtbl {\n    pub Bogus: u32,\n}\n")
        r = FieldOrderResolver(rust_sources=[decoy])
        fields, origin = r.resolve("IUnknownVtbl")
        assert origin.startswith("com-sys/")
        assert [f.name for f in fields] == ["QueryInterface", "AddRef", "Release"]


class TestModuleIslandAsFieldOrderSource:
    """A module-PRIVATE COM interface lives in neither com-sys nor a sibling header.

    netcfgx's `INetCfgBindingInterface` is declared `DECLARE_INTERFACE_(...)` in
    sdk/include/psdk/netcfgx.h — so com-sys does not carry it, and the C file's own
    directory does not either. The translation died with `VtblResolveError: cannot
    resolve field order` on interfaces that were sitting, fully expanded, in the
    module's own `types_<arch>.rs`: bindgen had already turned the DECLARE_INTERFACE_
    macro into an exact struct, which a C-header regex cannot do reliably.
    """

    NETCFGX_C = REACTOS_ROOT / "dll" / "win32" / "netcfgx"
    ISLAND_DIR = REACTOS_ROOT / "sdk" / "lib" / "rust" / "netcfgx-rust" / "src"

    @pytest.mark.skipif(
        not (ISLAND_DIR / "types_i386.rs").is_file(), reason="netcfgx island not generated"
    )
    def test_a_private_interface_resolves_from_the_modules_own_island(self):
        c = self.NETCFGX_C / "netcfgbindinginterface_iface.c"
        islands = sorted(self.ISLAND_DIR.glob("types*.rs"))
        tables = {t.name: t for t in find_vtbl_tables(c, rust_sources=islands)}
        t = tables["vt_NetCfgBindingInterface"]
        assert t.type_name == "INetCfgBindingInterfaceVtbl"
        assert t.origin.startswith("rust:types")
        # IUnknown's three, then the interface's own
        assert [s.field for s in t.slots][:3] == ["QueryInterface", "AddRef", "Release"]
        assert len(t.slots) == 6

    @pytest.mark.skipif(
        not (ISLAND_DIR / "types_i386.rs").is_file(), reason="netcfgx island not generated"
    )
    def test_without_the_island_it_REFUSES_rather_than_guessing(self):
        """The point of the resolver: no field order, no table. Never a guess."""
        c = self.NETCFGX_C / "netcfgbindinginterface_iface.c"
        with pytest.raises(VtblResolveError, match="cannot resolve field order"):
            find_vtbl_tables(c, rust_sources=[], c_headers=[])
