"""scan_safe_probe_deref: a user pointer validated by a non-raising range probe
(`SafeProbeForRead`/`SafeProbeForWrite`, or a crate's `Ks*` mirror) and then touched.

A range probe proves WHERE the buffer is, never that it stays mapped, so a raw
dereference after it is an any-process BUGCHECK 0x1E under DUMMY_PSEH. Each SAFE case
is a shape that must stay quiet: the touch routed through a fault-tolerant copy, the
trusted kernel-mode arm, the capture-shadow (`let p = p_safe`), and a probe of a
field pointer (which belongs to the capture class, not this one).
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import scan_safe_probe_deref as scan


def rows(tmp_path: Path, src: str) -> list[str]:
    p = tmp_path / "x-rust/src/a.rs"
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(src)
    return sorted(scan.scan_tree([(p, "x-rust/src/a.rs")]))


def test_a_raw_deref_after_a_probe_is_flagged(tmp_path):
    src = (
        "pub unsafe fn f(P: *const RECTL) -> i32 {\n"
        "    if SafeProbeForRead(P as *const c_void, 16, 4) < 0 { return -1; }\n"
        "    let rc = *(P as *const RECTL);\n"
        "    rc.left\n}\n"
    )
    assert rows(tmp_path, src) == ["x-rust/src/a.rs: f"]


def test_a_raw_copy_from_a_probed_pointer_is_flagged(tmp_path):
    src = (
        "pub unsafe fn f(P: *const u8, dst: *mut u8) -> i32 {\n"
        "    if SafeProbeForRead(P as *const c_void, 16, 4) < 0 { return -1; }\n"
        "    core::ptr::copy_nonoverlapping(P, dst, 16);\n"
        "    0\n}\n"
    )
    assert rows(tmp_path, src) == ["x-rust/src/a.rs: f"]


def test_a_probe_then_fault_tolerant_copy_is_clean(tmp_path):
    # The probe stays for its range check; the copy goes through safe_copy_raw, which
    # is not a raw primitive, so _walk reports a forwarding call this scanner ignores.
    src = (
        "pub unsafe fn f(P: *const c_void) -> i32 {\n"
        "    if SafeProbeForRead(P as *const c_void, 16, 4) < 0 { return -1; }\n"
        "    let mut rc: RECTL = core::mem::zeroed();\n"
        "    safe_copy_raw(&mut rc as *mut _ as *mut c_void, P, 16);\n"
        "    rc.left\n}\n"
    )
    assert rows(tmp_path, src) == []


def test_the_trusted_kernel_mode_arm_is_clean_and_the_user_arm_flags(tmp_path):
    # ks.sys's KsFastPropertyHandler shape: a raw copy on the trusted KernelMode arm,
    # a probe + copy on the UserMode arm. Only the user arm is a boundary crossing.
    bad = (
        "pub unsafe fn f(P: *const u8, dst: *mut u8, Mode: i8) -> i32 {\n"
        "    if Mode == 0i8 {\n"
        "        core::ptr::copy_nonoverlapping(P, dst, 16);\n"
        "    } else {\n"
        "        if SafeProbeForRead(P as *const c_void, 16, 4) < 0 { return -1; }\n"
        "        core::ptr::copy_nonoverlapping(P, dst, 16);\n"
        "    }\n    0\n}\n"
    )
    assert rows(tmp_path, bad) == ["x-rust/src/a.rs: f"]
    # With the user-arm copy also routed through safe_copy_raw, nothing is left.
    good = bad.replace(
        "        core::ptr::copy_nonoverlapping(P, dst, 16);\n    }",
        "        safe_copy_raw(dst as *mut c_void, P as *const c_void, 16);\n    }",
    )
    assert rows(tmp_path, good) == []


def test_a_capture_shadow_ends_tracking(tmp_path):
    # The probed `pbmi` is captured into a kernel pool and `pbmi` is rebound to the
    # safe copy; every deref afterwards reads kernel memory, so it must not flag.
    src = (
        "pub unsafe fn f(pbmi: *const BMI) -> i32 {\n"
        "    if SafeProbeForRead(pbmi as *const c_void, 40, 1) < 0 { return -1; }\n"
        "    let pbmiSafe = alloc(40) as *const BMI;\n"
        "    safe_copy_raw(pbmiSafe as *mut c_void, pbmi as *const c_void, 40);\n"
        "    let pbmi = pbmiSafe;\n"
        "    let h = (*pbmi).biHeight;\n"
        "    h\n}\n"
    )
    assert rows(tmp_path, src) == []


def test_a_crate_house_mirror_is_recognised(tmp_path):
    # ks.sys names its mirror KsSafeProbeForRead precisely so the name scanners see it.
    src = (
        "pub unsafe fn f(P: *const RECTL) -> i32 {\n"
        "    if KsSafeProbeForRead(P as *const c_void, 16, 4) < 0 { return -1; }\n"
        "    let rc = *(P as *const RECTL);\n"
        "    rc.left\n}\n"
    )
    assert rows(tmp_path, src) == ["x-rust/src/a.rs: f"]


def test_a_probe_of_a_capture_field_is_not_this_class(tmp_path):
    # A field pointer of an already-captured struct belongs to scan_raw_user_access's
    # read_back taint, not here; its non-bare arg is not tracked by name.
    src = (
        "pub unsafe fn f(cap: &UNICODE_STRING) -> i32 {\n"
        "    if SafeProbeForRead(cap.Buffer as *const c_void, 16, 4) < 0 { return -1; }\n"
        "    let w = *cap.Buffer;\n"
        "    w as i32\n}\n"
    )
    assert rows(tmp_path, src) == []


def test_nolint_suppresses(tmp_path):
    src = (
        "pub unsafe fn f(P: *const RECTL) -> i32 {\n"
        "    if SafeProbeForRead(P as *const c_void, 16, 4) < 0 { return -1; }\n"
        "    let rc = *(P as *const RECTL); // nolint: safe-probe-deref reviewed\n"
        "    rc.left\n}\n"
    )
    assert rows(tmp_path, src) == []
