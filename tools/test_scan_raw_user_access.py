"""scan_raw_user_access: the callee pass and the rules that keep it quiet on safe code.

Each rule here exists because the first run of the callee pass got it wrong one way
or the other: a flow-insensitive rebind hid IntDefWindowProc's WM_NCCALCSIZE store
behind one `lParam = ..` in another match arm, and without the kernel-arm and
early-return rules every `if PreviousMode != KernelMode { capture } else { raw }`
callee in the tree read as a bug.

The embedded-pointer tests cover the second class the scanner learned to see: a
pointer field read OUT of a `read_back` capture (`UNICODE_STRING.Buffer`,
`OBJECT_ATTRIBUTES.ObjectName`). The scalar-field test is the `NtUserGetIconInfo`
false positive -- `let cb = cap.MaximumLength` must not read as a buffer when it is
handed to a primitive as a length.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import scan_raw_user_access as scan

SYSCALL = (
    'pub unsafe extern "system" fn NtThing(P: *mut ULONG, Mode: KPROCESSOR_MODE) -> NTSTATUS {\n'
)
# A syscall whose pointer parameter addresses a user struct with an embedded buffer.
USTR = (
    'pub unsafe extern "system" fn NtStr(P: *mut _UNICODE_STRING, Mode: KPROCESSOR_MODE)'
    " -> NTSTATUS {\n"
)


def rows(tmp_path: Path, files: dict[str, str]) -> list[str]:
    picked = []
    for rel, text in files.items():
        p = tmp_path / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)
        picked.append((p, rel))
    return sorted(scan.scan_tree(picked))


def test_follows_a_pointer_into_a_callee_and_names_the_chain(tmp_path):
    src = SYSCALL + "    helper(P as *mut u8, 1);\n    0\n}\n"
    src += "unsafe fn helper(q: *mut u8, n: u32) {\n    inner(q);\n}\n"
    src += "unsafe fn inner(r: *mut u8) {\n    *r = 0;\n}\n"
    out = rows(tmp_path, {"x-rust/src/a.rs": src})
    assert out == ["x-rust/src/a.rs: NtThing P  -> helper(q) -> inner(r) @x-rust/src/a.rs"]


def test_a_callee_using_the_fault_tolerant_primitive_is_clean(tmp_path):
    src = SYSCALL + "    helper(P);\n    0\n}\n"
    src += "unsafe fn helper(q: *mut ULONG) {\n    let _ = write_back(q, 0);\n}\n"
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == []


def test_resolves_across_crates_only_when_the_name_is_unique(tmp_path):
    caller = SYSCALL + "    ObThing(P);\n    0\n}\n"
    callee = 'pub unsafe extern "system" fn ObThing(q: *mut ULONG) {\n    *q = 1;\n}\n'
    # A declaration elsewhere is not a second definition.
    decl = 'extern "system" {\n    fn ObThing(q: *mut ULONG);\n}\n'
    out = rows(
        tmp_path,
        {"x-rust/src/a.rs": caller + decl, "ob-rust/src/b.rs": callee},
    )
    assert out == ["x-rust/src/a.rs: NtThing P  -> ObThing(q) @ob-rust/src/b.rs"]
    dup = rows(
        tmp_path,
        {"x-rust/src/a.rs": caller, "ob-rust/src/b.rs": callee, "y-rust/src/c.rs": callee},
    )
    assert dup == []  # ambiguous: not followed


def test_the_kernel_arm_of_a_previous_mode_test_is_not_flagged(tmp_path):
    src = SYSCALL + "    helper(P, Mode);\n    0\n}\n"
    src += (
        "unsafe fn helper(q: *mut ULONG, AccessMode: KPROCESSOR_MODE) {\n"
        "    if AccessMode != 0 {\n        let _ = write_back(q, 0);\n"
        "    } else {\n        *q = 0;\n    }\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == []
    # ...but the USER arm still is.
    bad = src.replace("let _ = write_back(q, 0);", "*q = 1;")
    assert len(rows(tmp_path, {"x-rust/src/a.rs": bad})) == 1


def test_after_a_user_arm_that_returns_the_rest_is_kernel_only(tmp_path):
    src = (
        SYSCALL + "    if Mode != 0 {\n        let v = read_back(P);\n        return v;\n    }\n"
        "    inner(P);\n    0\n}\n"
        "unsafe fn inner(r: *mut ULONG) {\n    *r = 0;\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == []


def test_a_rebind_in_one_match_arm_does_not_hide_the_others(tmp_path):
    src = SYSCALL + (
        "    match Mode {\n"
        "        1 => {\n            P = core::ptr::null_mut();\n        }\n"
        "        _ => {\n            *P = 3;\n        }\n"
        "    }\n    0\n}\n"
    )
    assert len(rows(tmp_path, {"x-rust/src/a.rs": src})) == 1


def test_a_capture_inside_a_null_guard_covers_the_rest(tmp_path):
    src = SYSCALL + (
        "    let mut local: ULONG = 0;\n"
        "    if !P.is_null() {\n        local = read_back(P);\n        P = &mut local;\n    }\n"
        "    inner(P);\n    0\n}\n"
        "unsafe fn inner(r: *mut ULONG) {\n    *r = 0;\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == []


def test_nolint_on_the_call_stops_the_chain(tmp_path):
    src = SYSCALL + (
        "    inner(P); // nolint: raw-user-access (guarded above)\n    0\n}\n"
        "unsafe fn inner(r: *mut ULONG) {\n    *r = 0;\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == []


def test_the_native_entry_point_is_not_a_syscall(tmp_path):
    src = (
        'pub unsafe extern "system" fn NtProcessStartup(Peb: PPEB) {\n'
        "    let x = (*Peb).ImageBaseAddress;\n}\n"
    )
    assert rows(tmp_path, {"u-rust/src/a.rs": src}) == []

def test_a_pointer_field_lifted_from_a_capture_is_tracked(tmp_path):
    # read_back captures the header; .Buffer still points at user memory, so a raw
    # copy from it -- not through a fault-tolerant primitive -- is a bug one hop out.
    src = USTR + (
        "    let hdr = match read_back(P) { Ok(v) => v, Err(s) => return s };\n"
        "    core::ptr::copy_nonoverlapping(hdr.Buffer as *const u8, dst, hdr.Length as usize);\n"
        "    0\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == ["x-rust/src/a.rs: NtStr P"]
    # Lifting the field into a local and dereferencing it is the same bug.
    deref = USTR + (
        "    let hdr = read_back(P).unwrap();\n    let b = hdr.Buffer;\n    *b = 0;\n    0\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": deref}) == ["x-rust/src/a.rs: NtStr P"]


def test_a_capture_copied_through_a_fault_tolerant_primitive_is_clean(tmp_path):
    src = USTR + (
        "    let hdr = match read_back(P) { Ok(v) => v, Err(s) => return s };\n"
        "    let _ = safe_copy_raw(dst, hdr.Buffer as *const _, hdr.Length as usize);\n"
        "    0\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == []


def test_a_scalar_field_of_a_capture_used_as_a_length_is_not_flagged(tmp_path):
    # NtUserGetIconInfo: `let cb = Module.MaximumLength` then RtlZeroMemory(pool, cb)
    # -- cb is the LENGTH, the final argument, not a buffer.
    src = USTR + (
        "    let hdr = read_back(P).unwrap();\n"
        "    let cb = hdr.MaximumLength as usize;\n"
        "    RtlZeroMemory(pool as *mut c_void, cb as _);\n"
        "    0\n}\n"
    )
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == []


def test_an_inline_capture_field_is_followed_into_a_callee(tmp_path):
    src = USTR + (
        "    let oa = read_back(P).unwrap();\n    sink(oa.ObjectName);\n    0\n}\n"
        "unsafe fn sink(us: *mut _UNICODE_STRING) {\n    (*us).Length = 0;\n}\n"
    )
    out = rows(tmp_path, {"x-rust/src/a.rs": src})
    assert out == ["x-rust/src/a.rs: NtStr P  -> sink(us) @x-rust/src/a.rs"]
    # ...and clean when the callee captures it instead of touching it raw.
    safe = src.replace("    (*us).Length = 0;", "    let _ = read_back(us);")
    assert rows(tmp_path, {"x-rust/src/a.rs": safe}) == []


def test_from_raw_parts_on_a_parameter_is_a_raw_access(tmp_path):
    src = SYSCALL + "    let s = core::slice::from_raw_parts(P, 8);\n    let _ = s[0];\n    0\n}\n"
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == ["x-rust/src/a.rs: NtThing P"]


def test_a_store_through_pointer_offset_is_a_raw_access(tmp_path):
    # `*P.offset(i) = v` is the result-list write-back shape (se/audit.c) the base
    # scanner missed because it only recognised `.add(i)`.
    src = SYSCALL + "    *P.offset(2) = 7;\n    let _ = *P.offset(3);\n    0\n}\n"
    assert rows(tmp_path, {"x-rust/src/a.rs": src}) == ["x-rust/src/a.rs: NtThing P"]
