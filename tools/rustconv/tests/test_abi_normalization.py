"""Regression tests for the two ABI-normalization passes.

Both bugs pinned here were SILENT — they produced code that compiled on the arch under test
and misbehaved (or vanished) elsewhere, which is precisely the class `cargo check` cannot see.

1. `collapse_abi_twins._target_abi` — a cfg pair spelled the SAME on both sides is not an ABI
   fork, and folding it to `system` makes a cdecl function stdcall ON i386. That shipped four
   times in one pass: `PATH_RoundRect` failed at link, `GetW32ThreadInfo` folded a caller to
   `ud2` (a BUGCHECK 0x1E at boot), and `KdbpCmdTss` / `MiDecrementPageTableReferences` failed
   nothing at all.

2. `normalize_island_abi` witness collection — i386 entries already rewritten to `system` are
   still stdcall witnesses, because on i386 `system` IS stdcall. Excluding them made the pass
   lose coverage on its SECOND run rather than converge, hiding 1,218 slots including
   `KIPI_BROADCAST_WORKER`, which stayed `extern "C"` on amd64 while its i386 twin said
   `system` and broke pcix's `KeIpiGenericCall` on that arch only.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))


def _load(name: str):
    spec = importlib.util.spec_from_file_location(name, TOOLS / f"{name}.py")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


collapse = _load("collapse_abi_twins")
normalize = _load("normalize_island_abi")


# --------------------------------------------------------------------------- _target_abi


def test_a_stdcall_c_pair_is_the_abi_fork_and_folds_to_system():
    assert collapse._target_abi("stdcall", "C") == "system"


def test_a_cdecl_pair_keeps_cdecl_because_system_would_be_stdcall_on_i386():
    # PATH_RoundRect: C callers reference `_PATH_RoundRect`; `system` emits `_PATH_RoundRect@28`.
    assert collapse._target_abi("C", "C") == "C"


def test_a_stdcall_pair_keeps_stdcall():
    assert collapse._target_abi("stdcall", "stdcall") == "stdcall"


def test_an_already_reconciled_pair_stays_system():
    assert collapse._target_abi("system", "system") == "system"


def test_a_backwards_fork_is_refused_rather_than_guessed():
    # `C` on x86 against `stdcall` on the 64-bit pair is not a shape we produce; folding it
    # either way would pick a convention for one arch by accident.
    assert collapse._target_abi("C", "stdcall") is None


def test_fastcall_is_never_folded():
    # fastcall passes in ecx/edx; `system` would move those arguments to the stack.
    assert collapse._target_abi("fastcall", "C") is None
    assert collapse._target_abi("fastcall", "fastcall") is None


# ------------------------------------------------------------------- island witness / chains


def test_system_on_i386_still_counts_as_a_stdcall_witness():
    # The idempotence bug: after one run the i386 island says `system`, and if that stops being
    # a witness the pass silently stops projecting onto 64-bit entries it never reached.
    i386 = 'pub type PFOO = ::core::option::Option<unsafe extern "system" fn(x: u32) -> u32>;\n'
    assert normalize.scan(i386)["PFOO"] == "system"


def test_cdecl_on_i386_is_not_a_witness():
    i386 = 'pub type PBAR = ::core::option::Option<unsafe extern "C" fn(x: u32) -> u32>;\n'
    assert normalize.scan(i386)["PBAR"] == "C"


def test_alias_chain_is_followed_to_the_type_that_carries_the_abi():
    # amd64 forwards the pointer alias to a named type; the ABI lives on the target, so a
    # witness keyed on the forwarder must reach it.
    amd64 = (
        'pub type KIPI_BROADCAST_WORKER =\n'
        '    ::core::option::Option<unsafe extern "C" fn(Argument: u32) -> u32>;\n'
        "pub type PKIPI_BROADCAST_WORKER = KIPI_BROADCAST_WORKER;\n"
    )
    assert normalize.alias_chains(amd64) == {
        "PKIPI_BROADCAST_WORKER": "KIPI_BROADCAST_WORKER"
    }
    new, n = normalize.rewrite(amd64, {"PKIPI_BROADCAST_WORKER", "KIPI_BROADCAST_WORKER"}, "C")
    assert n == 1
    assert 'extern "system" fn(Argument: u32)' in new


def test_rewrite_leaves_a_slot_whose_key_is_not_a_witness():
    amd64 = 'pub type PBAZ = ::core::option::Option<unsafe extern "C" fn(x: u32) -> u32>;\n'
    new, n = normalize.rewrite(amd64, set(), "C")
    assert (n, new) == (0, amd64)


def test_declaration_blocks_are_projected_too():
    # The shape that let afd's TDI completions and smss's SmpApiLoop keep a `stdcall`
    # declaration that no `system` definition could unify with.
    i386 = 'unsafe extern "stdcall" {\n    pub fn SmpApiLoop(Parameter: *mut u8) -> u32;\n}\n'
    assert normalize.scan(i386)["fn SmpApiLoop"] == "stdcall"
    new, n = normalize.rewrite(i386, {"fn SmpApiLoop"}, "stdcall")
    assert n == 1
    assert 'unsafe extern "system" {' in new


def test_nested_callback_parameters_are_projected_too():
    # IViewObject::Draw's pfnContinue: normalizing the outer slot while this stayed literal
    # left the two signatures differing in one buried argument.
    i386 = (
        "    pub Draw: ::core::option::Option<\n"
        '        unsafe extern "stdcall" fn(\n'
        "            pfnContinue: ::core::option::Option<\n"
        '                unsafe extern "stdcall" fn(dwContinue: usize) -> i32,\n'
        "            >,\n"
        "        ) -> i32,\n"
        "    >,\n"
    )
    assert normalize.scan(i386)["param pfnContinue"] == "stdcall"


def test_host_harness_crates_are_skipped_by_both_tools(tmp_path):
    """`system` is stdcall only on 32-bit WINDOWS.

    A difftest harness builds these crates for `i686-unknown-linux-gnu`, where `target_arch`
    is still "x86" but `system` means cdecl — so the x86 twin's explicit `stdcall` is
    load-bearing. Normalizing it turned msvcrt-sys's Win32 imports cdecl in the harness,
    leaving 4 bytes on the stack per call, and msvcrt-seh-difftest died in `cxx_frame_handler`
    while all three PRODUCT arches built, linked and booted clean.
    """
    assert "msvcrt-sys" in normalize.HOST_HARNESS_CRATES
    assert "msvcrt-crt-rust" in normalize.HOST_HARNESS_CRATES
    # The two lists must agree, or one tool re-breaks what the other refuses to touch.
    assert normalize.HOST_HARNESS_CRATES == collapse.HOST_HARNESS_CRATES

    crate = tmp_path / "msvcrt-sys"
    (crate / "src").mkdir(parents=True)
    (crate / "src" / "bindgen_i386.rs").write_text(
        "/* automatically generated by rust-bindgen 0.72.1 */\n"
        'pub type PFOO = ::core::option::Option<unsafe extern "stdcall" fn(x: u32)>;\n'
    )
    n, notes = normalize.normalize_crate(crate / "src", apply=False)
    assert n == 0
    assert notes and "SKIPPED" in notes[0]


def test_generated_banner_is_what_identifies_an_island(tmp_path):
    # Discovery used to be an allowlist of filename stems, which skipped win_sys_*, island_*,
    # hal_bindgen_* and commacros_* entirely — silently, since a skipped island just stays
    # literal until a consumer fails to compile.
    src = tmp_path / "src"
    src.mkdir()
    (src / "win_sys_i386.rs").write_text("/* automatically generated by rust-bindgen 0.72.1 */\n")
    (src / "win_sys_amd64.rs").write_text("/* automatically generated by rust-bindgen 0.72.1 */\n")
    (src / "win_sys_arm64.rs").write_text("/* automatically generated by rust-bindgen 0.72.1 */\n")
    # A hand-written per-arch module sharing the naming convention carries no banner.
    (src / "seh_i386.rs").write_text("pub fn hand_written() {}\n")

    found = normalize.island_paths(src)
    assert [p.name for p in found["i386"]] == ["win_sys_i386.rs"]
    assert [p.name for p in found["amd64"]] == ["win_sys_amd64.rs"]
