"""The .spec is ground truth for an EXPORT's calling convention.

The 2026-07-17 wshtcpip defect: every WSH export is defined in the C as
`EXPORT\\nWSHOpenSocket(...)` with `#define EXPORT WINAPI` (wshtcpip.h:25).
`enforce_abi_from_c` reads the C source for a known convention macro, does not
know the DLL-local `EXPORT` alias, FALLS BACK to cdecl, and emits `extern "C"`
for a `__stdcall` export. On i386 the spec2def .def then demands `_Name@N`, so
the DLL will not link (amd64/arm64 link fine — the ABI is identical there,
which is what made it silent). And `EXPORT` cannot simply be taught to the C
reader: a DDK header spells the same token `#define EXPORT __cdecl`.

`enforce_export_abi_from_spec` pins the fix: the .spec (`@ stdcall WSHOpenSocket`)
decides an export's convention, overriding whatever the C-macro read produced.
"""

from dataclasses import dataclass

from rustconv.postprocess import enforce_export_abi_from_spec


@dataclass
class _Entry:
    """Minimal SpecEntry stand-in — the pass only reads `.convention`."""

    convention: str


def _exports(**kw):
    return {name: _Entry(conv) for name, conv in kw.items()}


def test_stdcall_export_forced_from_c_to_system():
    # exactly the wshtcpip shape: enforce_abi_from_c left the stdcall export as extern "C"
    code = '#[no_mangle]\npub unsafe extern "C" fn WSHOpenSocket(a: i32) -> i32 { 0 }\n'
    out = enforce_export_abi_from_spec(code, _exports(WSHOpenSocket="stdcall"))
    assert 'extern "system" fn WSHOpenSocket(' in out
    assert 'extern "C" fn WSHOpenSocket(' not in out


def test_cdecl_export_stays_c():
    code = 'pub unsafe extern "C" fn some_cdecl_export(a: i32) -> i32 { 0 }\n'
    out = enforce_export_abi_from_spec(code, _exports(some_cdecl_export="cdecl"))
    assert 'extern "C" fn some_cdecl_export(' in out


def test_fastcall_export_left_untouched():
    """fastcall needs the per-arch dual cfg fix_fastcall_exports emits — this pass must
    NOT collapse it to a single ABI."""
    code = (
        '#[cfg(target_arch = "x86")]\npub unsafe extern "fastcall" fn Fc(a: i32) {}\n'
        '#[cfg(not(target_arch = "x86"))]\npub unsafe extern "C" fn Fc(a: i32) {}\n'
    )
    out = enforce_export_abi_from_spec(code, _exports(Fc="fastcall"))
    assert out == code  # no change


def test_import_decl_in_extern_block_not_touched():
    """A same-named IMPORT (`fn WSHOpenSocket(...)` inside `extern "C" { … }`) lacks the
    `pub … extern "…" fn` prefix, so the definition-only regex never rewrites it."""
    code = 'extern "C" {\n    fn WSHOpenSocket(a: i32) -> i32;\n}\n'
    out = enforce_export_abi_from_spec(code, _exports(WSHOpenSocket="stdcall"))
    assert out == code


def test_already_correct_is_noop():
    code = 'pub unsafe extern "system" fn WSHIoctl(a: i32) -> i32 { 0 }\n'
    out = enforce_export_abi_from_spec(code, _exports(WSHIoctl="stdcall"))
    assert out == code


def test_non_exported_fn_untouched():
    """Only names in the exports map are touched; an internal helper keeps its ABI."""
    code = 'pub unsafe extern "C" fn internal_helper(a: i32) -> i32 { 0 }\n'
    out = enforce_export_abi_from_spec(code, _exports(WSHOpenSocket="stdcall"))
    assert out == code
