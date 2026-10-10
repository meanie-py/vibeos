"""verify_ffi's [ABI] check: the .spec governs a call site only when the linker
will actually reach the export.

The suppression added 2026-07-25 took the [ABI] class from 6 findings to 0, and a
class that reports 0 is indistinguishable from a class whose matcher died — the
exact failure mode tools/test_scanner_controls.py exists to prevent. verify_ffi
cannot be driven by a fixture DIRECTORY (it needs the whole workspace to resolve
cross-crate ABI, which is why it sits in that file's UNCOVERED list), so the
positive/negative pair lives here at the function level instead.

Positive: no local definition -> the spec IS the authority, finding stands. That is
the user32 MessageInit class (a FASTCALL export declared "system"), which on i386
made LLVM delete the calling branch outright and cost explorer its desktop icons.

Negative: the crate defines the name itself with #[no_mangle], so the link resolves
in-crate; "should be fastcall per ntoskrnl.spec" would then be advice that BREAKS
the link, since an i386 `extern "system"` decl looks for the decorated `_Foo@N`
that the undecorated shim does not export.
"""

import textwrap

from rustconv import verify_ffi

SPEC = {"InterlockedIncrement": {"conv": "fastcall", "params": 1}}


def _abi_findings(local_defs):
    issues = []
    verify_ffi._check_fn(
        issues,
        "cdrom-rust/src/common.rs",
        130,
        "InterlockedIncrement",
        "InterlockedIncrement",
        "C",
        1,
        SPEC,
        {},
        local_defs=local_defs,
    )
    return [i for i in issues if i["type"] == "ABI"]


def test_spec_is_the_authority_when_nothing_local_defines_the_symbol():
    found = _abi_findings(frozenset())
    assert len(found) == 1, found
    assert "fastcall" in found[0]["detail"]


def test_local_no_mangle_definition_outranks_the_spec():
    assert _abi_findings({"InterlockedIncrement"}) == []


def test_local_definition_scan_sees_the_one_line_shim_form():
    """The shims are written on ONE line. A pattern anchored on a newline after the
    attribute missed 4 of the 5 real cases, so this is not a hypothetical spelling."""
    src = textwrap.dedent(
        """\
        #[no_mangle] pub unsafe extern "C" fn InterlockedIncrement(p: *mut i32) -> i32 {
            (*(p as *const core::sync::atomic::AtomicI32)).fetch_add(1, SeqCst) + 1
        }
        #[no_mangle]
        pub unsafe extern "system" fn MultiLineForm(x: u32) -> u32 { x }
        """
    )
    names = set(verify_ffi._LOCAL_DEF_RE.findall(src))
    assert names == {"InterlockedIncrement", "MultiLineForm"}


def test_a_plain_declaration_is_not_mistaken_for_a_definition():
    """`extern "C" { fn Foo(); }` declares; it must not read as a local definition, or
    every decl would suppress its own ABI check."""
    src = 'unsafe extern "C" {\n    fn InterlockedIncrement(p: *mut i32) -> i32;\n}\n'
    assert verify_ffi._LOCAL_DEF_RE.findall(src) == []


def test_a_block_rejoined_on_one_line_takes_the_new_convention():
    """`} extern "system" {` closes one block and opens the next; its braces net to
    zero. The parser used to stay in the FIRST block's convention for the rest of it, so
    fastfat's verfysup.rs read as correct only while the two conventions happened to agree."""
    spec = {"KeSetEvent": {"conv": "stdcall", "params": 3}}
    src = (
        'extern "C" {\n    fn helper(p: *mut u8);\n'
        '} extern "system" {\n    fn KeSetEvent(e: *mut u8, i: i32, w: u8) -> i32;\n}\n'
    )
    facts = verify_ffi.extern_facts("fastfat-rust/src/verfysup.rs", src, spec, {}, [])
    assert [f for f in facts if f["type"] == "ABI"] == []
    wrong = src.replace('} extern "system" {', '} extern "C" {')
    facts = verify_ffi.extern_facts("fastfat-rust/src/verfysup.rs", wrong, spec, {}, [])
    assert [f["type"] for f in facts if f["type"] == "ABI"] == ["ABI"]
