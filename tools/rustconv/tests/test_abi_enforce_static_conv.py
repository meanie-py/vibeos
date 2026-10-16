"""`static` in C is LINKAGE, not calling convention.

The 2026-07 ole32 defect: ~130 COM vtbl methods, every one of them a
`static HRESULT WINAPI Foo(...)` in the C, came out of rustconv as a plain
`unsafe fn` — the **Rust ABI**. On i386 `STDMETHODCALLTYPE`/`WINAPI` is
`__stdcall` (callee pops), so a Rust-ABI fn parked in a C vtbl slot corrupts
the stack at the first call through it; on amd64/arm64 it links and runs
fine, which is what made it silent.

Root cause was `_canonical_block()` throwing the derived convention away for
every `is_static` fn. These tests pin the invariant that `is_static` may only
decide *visibility* (`#[no_mangle]` + `pub`), never the `extern`.

The static+system fn is rendered as a `#[cfg]` dual (`extern "stdcall"` on x86,
`extern "C"` elsewhere) rather than `extern "system"`, because rustc does not
coerce a `"system" fn` into the `"stdcall" fn` slot our COM vtbl bindgen
declares on i386 (`com-sys/src/bindgen_i386.rs`). See the module docstring of
`abi_enforce`.
"""

from pathlib import Path

from rustconv.abi_enforce import audit_abi, build_abi_map_from_c, enforce_abi_from_c

# ---------------------------------------------------------------------------
# fixtures
# ---------------------------------------------------------------------------

# Shape lifted from dll/win32/ole32/storage32.c: a static WINAPI vtbl method,
# an exported WINAPI entry point, a static plain (cdecl) helper, and a
# non-static FASTCALL fn.
C_SRC = """\
#include <windows.h>

static HRESULT WINAPI StorageBaseImpl_QueryInterface(IStorage *iface, REFIID riid, void **obj)
{
    return S_OK;
}

HRESULT WINAPI StgCreateDocfile(LPCOLESTR name, DWORD mode, DWORD reserved, IStorage **out)
{
    return S_OK;
}

static int compare_names(const char *a, const char *b)
{
    return 0;
}

VOID FASTCALL StorageFastHelper(ULONG x)
{
    return;
}
"""


def _write_c(tmp_path: Path, text: str = C_SRC) -> Path:
    p = tmp_path / "storage32.c"
    p.write_text(text)
    build_abi_map_from_c.cache_clear()  # lru_cache is keyed on Path, tmp_path reuse
    return p


RUST_SRC = """\
use crate::prelude::*;

unsafe fn StorageBaseImpl_QueryInterface(
    iface: *mut IStorage,
    riid: REFIID,
    obj: *mut *mut c_void,
) -> HRESULT {
    S_OK
}

unsafe fn StgCreateDocfile(
    name: LPCOLESTR,
    mode: DWORD,
    reserved: DWORD,
    out: *mut *mut IStorage,
) -> HRESULT {
    S_OK
}

unsafe fn compare_names(a: *const c_char, b: *const c_char) -> i32 {
    0
}

unsafe fn StorageFastHelper(x: ULONG) {
}
"""


# ---------------------------------------------------------------------------
# the map itself
# ---------------------------------------------------------------------------


def test_c_derivation_survives_trailing_param_comments(tmp_path: Path):
    """A comment between `)` and `{` must not defeat definition-site detection.

    wine annotates the last parameter (`IEnumSTATSTG** ppenum)   /* [out] */`),
    and the old whitespace-only skip stopped on the `/`, decided "not a
    definition", and silently fell back to **cdecl** — so 17 real
    `static … WINAPI` COM methods in storage32.c derived as cdecl. Found by
    audit_abi(); same i386 callee-pops bug, one layer deeper.
    """
    c = _write_c(
        tmp_path,
        """\
/******************************************************************************
 * StorageBaseImpl_EnumElements (IStorage)
 */
static HRESULT WINAPI StorageBaseImpl_EnumElements(
  IStorage*       iface,
  DWORD           reserved1, /* [in] */
  IEnumSTATSTG**  ppenum)    /* [out] */
{
  return S_OK;
}
""",
    )
    assert build_abi_map_from_c(c)["StorageBaseImpl_EnumElements"] == ("system", True)


def test_c_derivation_knows_callback_and_pascal_are_stdcall(tmp_path: Path):
    """`CALLBACK` is `__stdcall` (minwindef.h:109), `PASCAL` is `WINAPI` (:117).

    These are the WORST ones to miss: they mark the fns C hands to the OS as
    function *pointers* (an EnumFontFamiliesEx enum proc, a DialogBoxParam
    DlgProc), so user32/gdi32 calls them through the pointer with stdcall — a
    Rust-ABI body there corrupts the i386 stack on a call we never make
    ourselves. Missing them derived oleaut32's `font_enum_proc` (olefont.c) and
    `property_sheet_proc` (olepropframe.c) as cdecl.

    `WINAPIV` is `__cdecl` (:114), NOT a WINAPI variant.
    """
    c = _write_c(
        tmp_path,
        "static int CALLBACK font_enum_proc(const LOGFONTW *elf, LPARAM lp)\n{\n    return 1;\n}\n"
        "\nstatic INT_PTR PASCAL dlg_proc(HWND hwnd, UINT msg)\n{\n    return 0;\n}\n"
        "\nint WINAPIV printf_like(const char *fmt, ...)\n{\n    return 0;\n}\n",
    )
    abi = build_abi_map_from_c(c)
    assert abi["font_enum_proc"] == ("system", True)
    assert abi["dlg_proc"] == ("system", True)
    assert abi["printf_like"] == ("cdecl", False)


def test_c_derivation_ignores_keywords_inside_comments(tmp_path: Path):
    """A doc block that merely *mentions* WINAPI must not make a cdecl fn system."""
    c = _write_c(
        tmp_path,
        "/* Unlike the WINAPI entry points, this helper is plain cdecl. */\n"
        "static int compare_names(const char *a, const char *b)\n{\n    return 0;\n}\n",
    )
    assert build_abi_map_from_c(c)["compare_names"] == ("cdecl", True)


def test_abi_map_keeps_convention_for_static_fns(tmp_path: Path):
    abi = build_abi_map_from_c(_write_c(tmp_path))
    # This part was always right — the bug was downstream, in the rendering.
    assert abi["StorageBaseImpl_QueryInterface"] == ("system", True)
    assert abi["StgCreateDocfile"] == ("system", False)
    assert abi["compare_names"] == ("cdecl", True)
    assert abi["StorageFastHelper"] == ("fastcall", False)


# ---------------------------------------------------------------------------
# Task 1 — rendering
# ---------------------------------------------------------------------------


def test_static_winapi_gets_stdcall_cfg_dual_and_is_not_exported(tmp_path: Path):
    """THE regression test. `static HRESULT WINAPI` must NOT become a plain
    `unsafe fn` (Rust ABI). It becomes the stdcall/C cfg dual, file-private."""
    out = enforce_abi_from_c(RUST_SRC, _write_c(tmp_path))

    assert (
        '#[cfg(target_arch = "x86")]\nunsafe extern "stdcall" fn StorageBaseImpl_QueryInterface'
        in out
    )
    assert (
        '#[cfg(not(target_arch = "x86"))]\nunsafe extern "C" fn StorageBaseImpl_QueryInterface'
        in out
    )
    # internal linkage in C: must NOT be exported. (The cfg line sits directly
    # above the fn in the asserts above — i.e. no #[no_mangle] in between.)
    assert 'pub unsafe extern "stdcall" fn StorageBaseImpl_QueryInterface' not in out
    assert 'pub unsafe extern "C" fn StorageBaseImpl_QueryInterface' not in out
    # the Rust ABI is gone
    assert "unsafe fn StorageBaseImpl_QueryInterface" not in out
    # ...and it is NOT `extern "system"`: rustc won't put a "system" fn into an
    # `Option<unsafe extern "stdcall" fn(..)>` vtbl slot on i686.
    assert 'extern "system" fn StorageBaseImpl_QueryInterface' not in out


def test_nonstatic_winapi_is_exported_extern_system(tmp_path: Path):
    out = enforce_abi_from_c(RUST_SRC, _write_c(tmp_path))
    assert '#[no_mangle]\npub unsafe extern "system" fn StgCreateDocfile' in out


def test_static_cdecl_gets_extern_c_without_export(tmp_path: Path):
    out = enforce_abi_from_c(RUST_SRC, _write_c(tmp_path))
    assert 'unsafe extern "C" fn compare_names' in out
    assert 'pub unsafe extern "C" fn compare_names' not in out
    assert "unsafe fn compare_names" not in out


# ---------------------------------------------------------------------------
# com_method_macro="stdmethod" — the ole32/oleaut32 rendering
# ---------------------------------------------------------------------------


def test_macro_mode_wraps_static_system_in_stdmethod(tmp_path: Path):
    """The emitted shape is the INTERSECTION of the two macro definitions this
    can target: single fn per invocation, bare `fn` (not `unsafe fn`), no
    attrs, explicit `-> Ret`.

      * ole32-rust/src/winemacros.rs:99 — `$(#[$m:meta])* fn $name(..) -> $ret`
        (also has a no-return arm)
      * oleaut32-rust/src/oleaut.rs:10  — `fn $name(..) -> $ret` ONLY: no meta
        arm, no no-return arm

    (ole32-rust/src/ring/stg_prop.rs:9 defines a *third*, file-local shadow
    taking a `$(...)* ` repetition of `unsafe fn` — deliberately not targeted;
    a file that shadows the macro keeps its own shape.)
    """
    out = enforce_abi_from_c(RUST_SRC, _write_c(tmp_path), com_method_macro="stdmethod")

    assert "stdmethod! {" in out
    assert "fn StorageBaseImpl_QueryInterface(" in out
    # bare `fn`, NOT `unsafe fn` (winemacros/oleaut both match `fn`)
    assert "unsafe fn StorageBaseImpl_QueryInterface" not in out
    # body written once — no cfg dual for this fn
    assert 'extern "stdcall" fn StorageBaseImpl_QueryInterface' not in out
    # the exported fn is NOT macro-wrapped: it needs #[no_mangle], which
    # stdmethod! does not emit.
    assert '#[no_mangle]\npub unsafe extern "system" fn StgCreateDocfile' in out


def test_macro_mode_emits_explicit_unit_return_for_void_fns(tmp_path: Path):
    """oleaut.rs's macro has NO no-return arm, so a void fn must carry an
    explicit `-> ()` to match `$ret:ty` in both definitions."""
    c = _write_c(tmp_path, "static VOID WINAPI VoidMethod(ULONG x)\n{\n    return;\n}\n")
    out = enforce_abi_from_c(
        "unsafe fn VoidMethod(x: ULONG) {\n}\n", c, com_method_macro="stdmethod"
    )
    assert "stdmethod! {\nfn VoidMethod(x: ULONG) -> () {\n}\n}" in out


def test_macro_mode_is_idempotent(tmp_path: Path):
    c = _write_c(tmp_path)
    once = enforce_abi_from_c(RUST_SRC, c, com_method_macro="stdmethod")
    twice = enforce_abi_from_c(once, c, com_method_macro="stdmethod")
    assert once == twice
    assert once.count("stdmethod! {") == 1  # not re-wrapped


def test_default_mode_still_yields_the_cfg_dual(tmp_path: Path):
    """The default (None) must be unaffected — kernel/HAL/driver targets have
    no such macro."""
    out = enforce_abi_from_c(RUST_SRC, _write_c(tmp_path))
    assert "stdmethod!" not in out
    assert 'unsafe extern "stdcall" fn StorageBaseImpl_QueryInterface' in out


def test_default_mode_does_not_unwrap_a_hand_written_stdmethod(tmp_path: Path):
    """A file already hand-fixed with `stdmethod!` (ole32's ring/*.rs) must not
    churn back into inline duals just because someone re-runs the default."""
    c = _write_c(tmp_path)
    code = (
        "stdmethod! {\n"
        "fn StorageBaseImpl_QueryInterface(iface: *mut IStorage) -> HRESULT {\n"
        "    S_OK\n"
        "}\n"
        "}\n"
    )
    assert enforce_abi_from_c(code, c) == code


def test_macro_mode_falls_back_to_cfg_dual_on_unmatchable_params(tmp_path: Path):
    """`mut x: T` is not an `$arg:ident` — the macro would fail to expand with a
    confusing error. Fall back to the inline dual, which is ABI-identical."""
    c = _write_c(tmp_path)
    code = (
        "unsafe fn StorageBaseImpl_QueryInterface(mut iface: *mut IStorage)"
        " -> HRESULT {\n    S_OK\n}\n"
    )
    out = enforce_abi_from_c(code, c, com_method_macro="stdmethod")
    assert "stdmethod!" not in out
    assert 'unsafe extern "stdcall" fn StorageBaseImpl_QueryInterface' in out
    assert 'unsafe extern "C" fn StorageBaseImpl_QueryInterface' in out


def test_macro_mode_output_audits_clean(tmp_path: Path):
    c = _write_c(tmp_path)
    out = enforce_abi_from_c(RUST_SRC, c, com_method_macro="stdmethod")
    assert audit_abi(out, c, strict_cdecl=True) == []


def test_fastcall_keeps_cfg_dual(tmp_path: Path):
    out = enforce_abi_from_c(RUST_SRC, _write_c(tmp_path))
    assert '#[cfg(target_arch = "x86")]' in out
    assert 'pub unsafe extern "fastcall" fn StorageFastHelper' in out
    assert '#[cfg(not(target_arch = "x86"))]' in out
    assert 'pub unsafe extern "C" fn StorageFastHelper' in out


def test_static_fastcall_gets_cfg_dual_without_export(tmp_path: Path):
    c = _write_c(
        tmp_path,
        "VOID FASTCALL StorageFastHelper(ULONG x)\n{\n    return;\n}\n".replace(
            "VOID FASTCALL", "static VOID FASTCALL"
        ),
    )
    out = enforce_abi_from_c("unsafe fn StorageFastHelper(x: ULONG) {\n}\n", c)
    assert 'unsafe extern "fastcall" fn StorageFastHelper' in out
    assert 'unsafe extern "C" fn StorageFastHelper' in out
    assert '#[cfg(target_arch = "x86")]' in out
    assert "#[no_mangle]" not in out
    assert "pub unsafe" not in out


def test_idempotent(tmp_path: Path):
    c = _write_c(tmp_path)
    once = enforce_abi_from_c(RUST_SRC, c)
    twice = enforce_abi_from_c(once, c)
    assert once == twice


def test_idempotent_on_already_canonical_static_system(tmp_path: Path):
    c = _write_c(tmp_path, C_SRC)
    canonical = (
        '#[cfg(target_arch = "x86")]\n'
        'unsafe extern "stdcall" fn StorageBaseImpl_QueryInterface(x: u32) -> HRESULT {\n'
        "    S_OK\n"
        "}\n\n"
        '#[cfg(not(target_arch = "x86"))]\n'
        'unsafe extern "C" fn StorageBaseImpl_QueryInterface(x: u32) -> HRESULT {\n'
        "    S_OK\n"
        "}\n"
    )
    assert enforce_abi_from_c(canonical, c) == canonical


def test_vtbl_slot_type_shape_is_stdcall_not_system(tmp_path: Path):
    """The rendered x86 arm must be spelled `stdcall`, so its address fits the
    `Option<unsafe extern "stdcall" fn(..)>` slot com-sys generates for i386.
    rustc does not coerce a `"system" fn` into a `"stdcall" fn` slot."""
    out = enforce_abi_from_c(RUST_SRC, _write_c(tmp_path))
    x86_arm = out.split('#[cfg(not(target_arch = "x86"))]')[0]
    assert 'extern "stdcall" fn StorageBaseImpl_QueryInterface' in x86_arm


def test_extern_declaration_is_not_mistaken_for_a_definition(tmp_path: Path):
    """A decl inside an `extern` block has no body — the enforcer must not
    latch onto some unrelated later `{` and rewrite it as if it were a def."""
    c = _write_c(tmp_path)
    code = (
        'extern "system" {\n    fn StgCreateDocfile(name: LPCOLESTR) -> HRESULT;\n}\n\n'
        "fn unrelated() -> u32 {\n    7\n}\n"
    )
    out = enforce_abi_from_c(code, c)
    assert out == code


# ---------------------------------------------------------------------------
# Task 2 — audit_abi
# ---------------------------------------------------------------------------


def test_audit_flags_rust_abi_where_c_says_winapi(tmp_path: Path):
    c = _write_c(tmp_path)
    viols = audit_abi(RUST_SRC, c)
    joined = "\n".join(viols)

    assert any("StorageBaseImpl_QueryInterface" in v for v in viols), joined
    assert any("StgCreateDocfile" in v for v in viols), joined
    assert any("StorageFastHelper" in v for v in viols), joined
    assert "Rust ABI (no `extern`)" in joined
    # cdecl-vs-Rust-ABI is a latent, not a crash: off by default.
    assert not any("compare_names" in v for v in viols), joined


def test_audit_strict_cdecl_also_flags_the_latent(tmp_path: Path):
    c = _write_c(tmp_path)
    viols = audit_abi(RUST_SRC, c, strict_cdecl=True)
    assert any("compare_names" in v for v in viols)


def test_audit_passes_correct_code(tmp_path: Path):
    """The enforcer's own output must audit clean — that's the contract."""
    c = _write_c(tmp_path)
    assert audit_abi(enforce_abi_from_c(RUST_SRC, c), c, strict_cdecl=True) == []


def test_audit_flags_extern_c_where_c_says_winapi(tmp_path: Path):
    """Not just the missing-extern case: `extern "C"` for a WINAPI fn is the
    same i386 stack bug (caller pops instead of callee)."""
    c = _write_c(tmp_path)
    code = 'pub unsafe extern "C" fn StgCreateDocfile(name: LPCOLESTR) -> HRESULT {\n    S_OK\n}\n'
    viols = audit_abi(code, c)
    assert len(viols) == 1
    assert 'extern "C"' in viols[0] and "StgCreateDocfile" in viols[0]


def test_audit_accepts_the_stdmethod_macro_shape(tmp_path: Path):
    """ole32's hand-fix wraps vtbl methods in `stdmethod! { fn Foo(..) {..} }`,
    whose expansion supplies extern "stdcall"/"C". The audit must not read the
    bare `fn` inside the macro as a Rust-ABI violation."""
    c = _write_c(tmp_path)
    code = (
        "stdmethod! {\n"
        "fn StorageBaseImpl_QueryInterface(iface: *mut IStorage) -> HRESULT {\n"
        "    S_OK\n"
        "}\n"
        "}\n"
    )
    assert audit_abi(code, c) == []


def test_audit_ignores_fns_the_c_does_not_define(tmp_path: Path):
    c = _write_c(tmp_path)
    code = "unsafe fn some_rust_only_helper(x: u32) -> u32 {\n    x\n}\n"
    assert audit_abi(code, c) == []


def test_static_system_dual_is_suppressed_once_slots_are_system_spelled(tmp_path):
    """A `static ... CALLBACK` fn needs the per-arch twin ONLY while slots are concrete.

    The dual exists because Rust compares fn-pointer ABIs nominally, so a callback could be
    stored in a bindgen slot spelled `extern "stdcall"` on i386 only if it carried that
    literal convention. `normalize_island_abi.py` respelled the islands `extern "system"`
    everywhere, which retires the premise -- and leaves the dual actively WRONG where the
    consuming typedef is one the generator itself emits as `extern "system"`: dde.cpp's
    `pfnCommandHandler` handler table was E0308 on amd64/arm64, because the non-x86 twin is
    `extern "C"` and Rust types that distinctly even where the ABI coincides.
    """
    c = tmp_path / "t.c"
    c.write_text("static DWORD CALLBACK Handler(int a) { return 0; }\n")
    rust = "unsafe fn Handler(a: i32) -> u32 {\n    0\n}\n"

    dual = enforce_abi_from_c(rust, c, concrete_stdcall_slots=True)
    assert '#[cfg(target_arch = "x86")]' in dual
    assert 'extern "stdcall"' in dual
    assert 'extern "C"' in dual

    single = enforce_abi_from_c(rust, c, concrete_stdcall_slots=False)
    assert "cfg(target_arch" not in single
    assert 'extern "stdcall"' not in single
    assert single.count("fn Handler") == 1
    assert 'unsafe extern "system" fn Handler' in single

    # Default is unchanged, so no existing caller silently changes shape.
    assert enforce_abi_from_c(rust, c) == dual


def test_concrete_stdcall_slots_is_detected_from_the_island_not_the_output(tmp_path):
    """Scanning translated output would make the answer self-perpetuating.

    A previous run's twins are themselves `extern "stdcall"`, so a whole-crate scan would
    keep reporting True forever. Only island_*/bindgen_* files are consulted.
    """
    from rustconv.projects.reactos.targets import _detect_concrete_stdcall_slots

    crate = tmp_path / "c"
    (crate / "src").mkdir(parents=True)
    # No island at all -> conservative True (the dual is wasteful, never incorrect).
    assert _detect_concrete_stdcall_slots(crate) is True

    (crate / "src" / "island_i386.rs").write_text('unsafe extern "system" { fn F(); }\n')
    assert _detect_concrete_stdcall_slots(crate) is False

    # A leftover twin in TRANSLATED output must not flip the answer back.
    (crate / "src" / "some_module.rs").write_text('unsafe extern "stdcall" fn G() {}\n')
    assert _detect_concrete_stdcall_slots(crate) is False

    # Nor does a mention in PROSE. island_extra.rs has exactly one, in a comment explaining
    # that lstrcpynW is now bound from the real header, and it flipped the whole crate.
    (crate / "src" / "island_amd64.rs").write_text(
        '// note: this used to be `unsafe extern "stdcall"` before normalisation\n'
        'unsafe extern "system" { fn H(); }\n'
    )
    assert _detect_concrete_stdcall_slots(crate) is False

    # But a genuinely concrete island does.
    (crate / "src" / "bindgen_i386.rs").write_text(
        'pub type CB = Option<unsafe extern "stdcall" fn()>;\n'
    )
    assert _detect_concrete_stdcall_slots(crate) is True


def test_the_shell32_island_is_system_spelled_so_shell32_takes_the_single_form():
    """The measurement behind the flag, against the real tree rather than a fixture."""
    from rustconv.projects.reactos.targets import TARGETS

    assert TARGETS["shell32"].concrete_stdcall_slots is False


def test_a_gen_driver_types_island_counts_as_an_island(tmp_path):
    # setuplib (scaffolded by `rustconv port`) names its islands types_<arch>.rs; unlisted,
    # it had "no island" and its NTAPI callbacks came back as stdcall/"C" twins.
    from rustconv.projects.reactos.targets import _detect_concrete_stdcall_slots

    crate = tmp_path / "c"
    (crate / "src").mkdir(parents=True)
    (crate / "src" / "types_i386.rs").write_text(
        'pub type CB = Option<unsafe extern "system" fn()>;\n'
    )
    assert _detect_concrete_stdcall_slots(crate) is False


def test_a_same_line_attribute_is_not_taken_as_indentation(tmp_path: Path):
    # storport's StorPortWritePortUshort: the deterministic lowering writes the attribute on
    # the signature's line, and each rewrite prefixed its own with it (three #[no_mangle]).
    c = tmp_path / "stubs.c"
    c.write_text("VOID NTAPI Foo(PVOID p)\n{\n}\n")
    rust = '#[no_mangle] pub unsafe extern "system" fn Foo(p: PVOID) {\n}\n'
    once = enforce_abi_from_c(rust, c)
    assert once.count("#[no_mangle]") == 1
    assert enforce_abi_from_c(once, c) == once
