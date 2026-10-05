"""`L"AddReg.NT" INF_ARCH`: a wide run pasted against a string-literal macro is folded."""

from rustconv.wide_seal import (
    KIND_POINTER,
    render_wide_statics,
    seal_wide_literals,
    unseal_wide_literals,
)

PER_ARCH = {"INF_ARCH": {"i386": 'L"x86"', "amd64": 'L"amd64"', "arm64": 'L"arm64"'}}
SAME = {"SUFFIX": {"i386": 'L"\\\\Run"', "amd64": 'L"\\\\Run"', "arm64": 'L"\\\\Run"'}}


def _units(s):
    return tuple(ord(c) for c in s) + (0,)


def test_without_macro_values_the_run_is_skipped_as_before():
    sealed = seal_wide_literals('f(h, L"AddReg.NT" INF_ARCH, 0);')
    assert not sealed.wides and "pasted against a macro" in sealed.skipped[0].reason


def test_an_arch_independent_macro_folds_into_one_literal():
    sealed = seal_wide_literals('p = L"Software" SUFFIX L"Once";', SAME)
    (wide,) = sealed.wides
    assert wide.kind == KIND_POINTER and not wide.arch_units
    assert wide.units == _units("Software\\RunOnce")
    assert sealed.text == f"p = {wide.placeholder};"


def test_an_arch_dependent_macro_becomes_cfg_statics():
    sealed = seal_wide_literals('f(h, L"AddReg.NT" INF_ARCH, 0);', PER_ARCH)
    (wide,) = sealed.wides
    assert dict(wide.arch_units)["i386"] == _units("AddReg.NTx86")
    assert dict(wide.arch_units)["arm64"] == _units("AddReg.NTarm64")
    rust = unseal_wide_literals(f"f(h, {wide.placeholder}, 0);", sealed)
    statics = render_wide_statics([sealed], rust)
    assert statics.count(f"pub static {wide.static_name}:") == 3
    assert '#[cfg(target_arch = "x86")]' in statics and "[u16; 13]" in statics


def test_a_macro_missing_on_one_arch_is_not_guessed():
    partial = {"INF_ARCH": {"i386": 'L"x86"', "amd64": 'L"amd64"'}}
    assert not seal_wide_literals('f(L"A" INF_ARCH);', partial).wides
