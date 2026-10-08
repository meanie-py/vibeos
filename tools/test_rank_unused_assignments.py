"""Pins rank_unused_assignments' C liveness: each case is a shape that once ranked wrong."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import rank_unused_assignments as r


def verdict(c: str, rust: str, marker: str) -> str:
    """Verdict for the Rust write on the line containing `marker`, against C function `f`."""
    src = rust.encode()
    line = next(i for i, ln in enumerate(rust.splitlines(), 1) if marker in ln)
    col = rust.splitlines()[line - 1].index(marker.split()[0]) + 1
    got = r.rust_site(src, line, col)
    assert not isinstance(got, str), got
    tree = r.parser("c").parse(c.encode())
    fn = next(n for n in tree.root_node.named_children if n.type == "function_definition")
    return r.c_verdict(fn, got[1])[0]


RUST_TWO_CALLS = """
fn f() -> i32 {
    let mut Status: i32;
    Status = Foo();
    Status = Bar();
    return Status;
}
"""


def test_a_dropped_error_check_is_c_reads():
    c = (
        "int f(void) { int Status; Status = Foo(); "
        "if (!NT_SUCCESS(Status)) return Status; Status = Bar(); return Status; }"
    )
    assert verdict(c, RUST_TWO_CALLS, "Status = Foo()") == "C_READS"


def test_the_same_dead_store_in_the_c_is_c_dead():
    c = "int f(void) { int Status; Status = Foo(); Status = Bar(); return Status; }"
    assert verdict(c, RUST_TWO_CALLS, "Status = Foo()") == "C_DEAD"


def test_unreferenced_parameter_is_a_hush_not_a_read():
    """cdrom DeviceEvtD0Exit: `status = ..; UNREFERENCED_PARAMETER(status); status = OK;`"""
    c = (
        "int f(void) { int Status; Status = Foo(); UNREFERENCED_PARAMETER(Status); "
        "Status = Bar(); return Status; }"
    )
    assert verdict(c, RUST_TWO_CALLS, "Status = Foo()") == "C_DEAD"


def test_a_read_only_in_an_assert_is_graded_debug():
    """RtlDefaultNpAcl: every `Status = RtlAdd..Ace()` is followed by ASSERT(NT_SUCCESS(Status))."""
    c = (
        "int f(void) { int Status; Status = Foo(); ASSERT(NT_SUCCESS(Status)); "
        "Status = Bar(); return Status; }"
    )
    assert verdict(c, RUST_TWO_CALLS, "Status = Foo()") == "C_READS_DEBUG"


def test_a_write_inside_an_if_condition_kills():
    """ole32 render_embed_source_hack: `if (FAILED(hr = GetDataHere(..))) return hr;`"""
    c = (
        "int f(void) { int Status; Status = Foo(); "
        "if (FAILED(Status = Bar())) return Status; return 0; }"
    )
    assert verdict(c, RUST_TWO_CALLS, "Status = Foo()") == "C_DEAD"


def test_a_conditional_kill_is_not_a_kill():
    c = (
        "int f(int x) { int Status; Status = Foo(); if (x && (Status = Bar())) return 1; "
        "return Status; }"
    )
    assert verdict(c, RUST_TWO_CALLS, "Status = Foo()") == "C_READS"


def test_goto_reaches_the_label():
    c = (
        "int f(int x) { int Status; Status = Foo(); if (x) goto Exit; Status = Bar();"
        " Exit: return Status; }"
    )
    assert verdict(c, RUST_TWO_CALLS, "Status = Foo()") == "C_READS"


def test_a_store_at_the_loop_tail_is_read_next_iteration():
    c = (
        "int f(int n) { int Status = 0; while (n--) { Use(Status); Status = Foo(); } "
        "Status = Bar(); return Status; }"
    )
    assert verdict(c, RUST_TWO_CALLS, "Status = Foo()") == "C_READS"


def test_break_leaves_the_loop():
    c = (
        "int f(int n) { int Status; for (;;) { Status = Foo(); if (n) break; Use(Status); }"
        " Status = Bar(); return Status; }"
    )
    # on the `break` path the value is dead, on the other it is read: may-live
    assert verdict(c, RUST_TWO_CALLS, "Status = Foo()") == "C_READS"
    c2 = (
        "int f(int n) { int Status; for (;;) { Status = Foo(); break; } Status = Bar(); "
        "return Status; }"
    )
    assert verdict(c2, RUST_TWO_CALLS, "Status = Foo()") == "C_DEAD"


def test_switch_fallthrough_and_break():
    c = (
        "int f(int k) { int Status; Status = Foo(); switch (k) { case 1: Status = Bar(); break;"
        " case 2: Use(1); default: return Status; } return 0; }"
    )
    assert verdict(c, RUST_TWO_CALLS, "Status = Foo()") == "C_READS"


RUST_INIT = """
fn f() -> i32 {
    let mut entry: *mut list = core::ptr::null_mut();
    entry = next();
    return 0;
}
"""


def test_a_foreach_macro_assigns_its_cursor():
    """acpi_bus_find_driver: `entry = NULL;` then `list_for_each(entry, &head)`."""
    c = (
        "int f(void) { struct list *entry = NULL; list_for_each(entry, &head) { "
        "Use(entry); } return 0; }"
    )
    assert verdict(c, RUST_INIT, "core::ptr::null_mut()") == "C_DEAD"


def test_an_out_param_is_graded_addr():
    """cdrom GetConfigurationData..: `totalLength = 0; REVERSE_BYTES(&totalLength, ..);`
    then `totalLength += ..` -- strictly an out-param write, so not C_READS."""
    c = "int f(void) { struct list *entry = NULL; Fill(&entry, 4); Use(entry); return 0; }"
    assert verdict(c, RUST_INIT, "core::ptr::null_mut()") == "C_READS_ADDR"


def test_the_c_declaring_without_a_value_is_c_no_init():
    c = "int f(void) { struct list *entry; entry = next(); return 0; }"
    assert verdict(c, RUST_INIT, "core::ptr::null_mut()") == "C_NO_INIT"


def test_a_hoisted_rust_write_is_not_paired_with_the_real_one():
    """cdrom ScratchBuffer_SetupSrb: the Rust calls IoGetNextIrpStackLocation twice, the C once;
    the dead Rust one must not inherit the C one's liveness."""
    rust = """
fn f() -> i32 {
    let mut s: i32;
    s = IoGetNext(irp);
    s = IoGetNext(irp);
    return s;
}
"""
    c = "int f(void) { int s = 0; s = IoGetNext(irp); return s; }"
    assert verdict(c, rust, "s = IoGetNext") == "NO_MATCH"


def test_a_fn_inside_a_macro_body_is_found():
    rust = """
com_vtbl! {
    fn f() -> i32 {
        let mut Status: i32;
        Status = Foo();
        Status = Bar();
        return Status;
    }
}
"""
    c = "int f(void) { int Status; Status = Foo(); Status = Bar(); return Status; }"
    assert verdict(c, rust, "Status = Foo()") == "C_DEAD"


def test_rustc_truncated_snippets_relocate():
    lines = ["a", "    let x = 1;", "    busDataRead = HalGetBusData(PCIConfiguration, 1, 2);"]
    assert r.relocate(lines, 2, "/ ...   busDataRead = HalGetBusData(") == 3
    assert r.relocate(lines, 1, "nowhere") is None


def test_sizeof_is_not_a_read():
    """partmgr VolumeDeleteMountPoints: `InputSize = sizeof(*InputBuffer) + ..` precedes the
    allocation that first gives InputBuffer a value."""
    c = "int f(void) { struct list *entry = NULL; n = sizeof(*entry); entry = Alloc(n); return 0; }"
    assert verdict(c, RUST_INIT, "core::ptr::null_mut()") == "C_DEAD"


def test_a_type_spelled_like_the_variable_is_not_a_read():
    """cdrom GetConfigurationData..: `sizeof(FEATURE_HEADER)` against `featureHeader`."""
    rust = RUST_INIT.replace("entry", "featureHeader")
    c = (
        "int f(int n) { struct list *featureHeader = NULL; "
        "if (n >= Size(FEATURE_HEADER)) { featureHeader = next(); Use(featureHeader); } return 0; }"
    )
    assert verdict(c, rust, "core::ptr::null_mut()") == "C_DEAD"


def test_a_hoisted_c99_declaration_is_never_called_harmless_blind():
    """scsiport RegistryInitLunKey: the C declares `PWCHAR typeName = Get(..)` mid-function
    and the Rust hoisted it with a null default. A differing C initializer that is READ
    could be the same value spelled differently (`= STATUS_SUCCESS` vs `= 0`), so it is
    NO_MATCH; only a dead one makes the Rust default harmless."""
    c = "int f(void) { Other(); struct list *entry = next(); Use(entry); return 0; }"
    assert verdict(c, RUST_INIT, "core::ptr::null_mut()") == "NO_MATCH"
    c2 = "int f(void) { struct list *entry = Make(); entry = next(); return 0; }"
    assert verdict(c2, RUST_INIT, "core::ptr::null_mut()") == "C_NO_INIT"


def test_c_brace_zero_pairs_with_rust_zeroed():
    rust = RUST_INIT.replace("core::ptr::null_mut()", "core::mem::zeroed()")
    c = "int f(void) { struct list *entry = {0}; Use(entry); entry = next(); return 0; }"
    assert verdict(c, rust, "core::mem::zeroed()") == "C_READS"


def test_while_true_never_exits_through_its_condition():
    """cdfs CdLookupNextInitialFileDirent: the FALSE default is overwritten on every path
    out of `while (TRUE)`."""
    c = (
        "int f(void) { int Status = 0; while (TRUE) { Status = Foo(); if (!Status) break; }"
        " return Status; }"
    )
    rust = RUST_TWO_CALLS.replace("let mut Status: i32;", "let mut Status: i32 = 0;")
    assert verdict(c, rust, "let mut Status") == "C_DEAD"


def test_the_reactos_arm_of_an_ifdef_is_the_code():
    """actctx RtlFindActivationContextSectionString: only the Wine `#else` read the default."""
    c = (
        "int f(int x) { int Status = 7;\n#ifdef __REACTOS__\nStatus = Check(x);\n#else\n"
        "if (x) return 1;\n#endif\n return Status; }"
    )
    rust = RUST_TWO_CALLS.replace("let mut Status: i32;", "let mut Status: i32 = 7;")
    assert verdict(c, rust, "let mut Status") == "C_DEAD"


def test_an_if_guarding_only_an_assert_is_debug():
    """heappage RtlpPageHeapReAllocate: `if (!OldBlockPageHeap) ASSERT(FALSE);`"""
    c = (
        "int f(void) { int Status; Status = Foo(); if (!Status) ASSERT(FALSE);"
        " Status = Bar(); return Status; }"
    )
    assert verdict(c, RUST_TWO_CALLS, "Status = Foo()") == "C_READS_DEBUG"


def test_a_debug_print_by_naming_convention_is_debug():
    """afd AfdCreateSocket: `AFD_DbgPrint(MID_TRACE, (.., EaInfoValue))`."""
    c = (
        "int f(void) { int Status; Status = Foo(); AFD_DbgPrint(1, (Status));"
        " Status = Bar(); return Status; }"
    )
    assert verdict(c, RUST_TWO_CALLS, "Status = Foo()") == "C_READS_DEBUG"
