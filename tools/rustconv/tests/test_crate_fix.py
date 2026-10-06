from pathlib import Path

from rustconv import crate_fix as cf

RUST = """use crate::prelude::*;
#[no_mangle]
pub unsafe extern "system" fn First(a: u32) -> u32 {
    let s = "}{";
    let c = '}';
    'outer: loop { break 'outer; }
    /* } */ a
}

pub unsafe fn decl_only(x: u32);

#[no_mangle]
pub unsafe extern "C" fn Second(p: *mut u8) -> NTSTATUS {
    fn goto_Quit() {}
    if CALL(p, FLAG_A) != 0 { return STATUS_X; }
    STATUS_SUCCESS
}
"""


def test_function_spans_skip_literals_and_decls():
    spans = cf.function_spans(RUST)
    assert [s.name for s in spans] == ["First", "Second"]
    assert RUST[spans[0].start :].startswith("#[no_mangle]")
    assert RUST[: spans[0].end].endswith("a\n}")
    assert RUST[: spans[1].end].endswith("STATUS_SUCCESS\n}")


def test_c_function_text_reads_raw_source(tmp_path: Path):
    c = tmp_path / "x.c"
    c.write_text(
        "int g;\n/* static */ PCSTR\nFoo(\n    IN PCSTR a)\n{\n#if 0\n  x();\n#endif\n"
        "  return a; /* } */\n}\n\nVOID Bar(VOID);\n"
    )
    text = cf.c_function_text(c, "Foo", {})
    assert text.startswith("/* static */ PCSTR\nFoo(")
    assert text.endswith("return a; /* } */\n}")
    assert cf.c_function_text(c, "Bar", {}) is None


def _second():
    s = cf.function_spans(RUST)[1]
    return RUST[s.start : s.end]


def _check(fixed, errors=None):
    return cf.reply_findings(_second(), fixed, errors or {"error[E0308]": {"amd64"}}, lambda _: [])


def test_reply_guards_accept_a_local_fix():
    fixed = _second().replace("CALL(p, FLAG_A) != 0", "CALL(p, FLAG_A as u32) != 0")
    assert _check(fixed) == []


def test_reply_guards_refuse_dropped_constant_call_and_stub():
    assert any("names dropped" in f for f in _check(_second().replace("FLAG_A", "1")))
    assert any("names dropped" in f for f in _check(_second().replace("CALL(p, FLAG_A)", "0")))
    stub = _second().replace("fn goto_Quit() {}", "fn goto_Quit() {}\n    const FLAG_B: u32 = 1;")
    assert any("declares an item" in f for f in _check(stub))
    unwrap = _second().replace("p, FLAG_A", "p.unwrap(), FLAG_A")
    assert any("forbidden" in f for f in _check(unwrap))
    assert any("signature" in f for f in _check(_second().replace("p: *mut u8", "p: *const u8")))


def test_an_unresolved_value_is_never_substituted():
    # setuplib: ERROR_SYSTEM_PARTITION_NOT_FOUND came back as ERROR_SYSTEM_DEVICE_NOT_FOUND,
    # and NTSTATUS names as bare hex. rustc cannot find it -> resolve supplies it, not the model.
    errors = {"error[E0425]: cannot find value `STATUS_X` in this scope": {"amd64"}}
    assert any(
        "names dropped" in f for f in _check(_second().replace("STATUS_X", "STATUS_Y"), errors)
    )
    hexed = _second().replace("STATUS_X", "0xC000017D")
    found = _check(hexed, errors)
    assert any("names dropped" in f for f in found)
    assert any("numeric literals" in f for f in found)


def test_a_function_reference_is_kept():
    # InstallFat12BootCode came back as InstallFat32BootCode -- a reference, not a call.
    src = _second().replace("STATUS_SUCCESS", "Some(Fat12 as fn())")
    s = cf.function_spans(RUST)[1]
    original = src
    fixed = src.replace("Fat12", "Fat32")
    assert any(
        "names dropped" in f
        for f in cf.reply_findings(original, fixed, {"error[E0308]": {"amd64"}}, lambda _: [])
    )
    assert s.name == "Second"


def test_a_type_rename_rustc_suggests_is_allowed():
    errors = {
        "error[E0412]: cannot find type `FAT32_BOOTSECTOR` in this scope\n"
        "help: a struct with a similar name exists: `_FAT32_BOOTSECTOR`": {"amd64"}
    }
    original = _second().replace(
        "p: *mut u8) -> NTSTATUS {",
        "p: *mut u8) -> NTSTATUS {\n    let q = p as *mut FAT32_BOOTSECTOR;",
    )
    fixed = original.replace("*mut FAT32_BOOTSECTOR", "*mut _FAT32_BOOTSECTOR")
    assert cf.reply_findings(original, fixed, errors, lambda _: []) == []


def test_extract_reply_takes_the_single_fence():
    assert cf.extract_reply("x\n```rust\nfn a() {}\n```\n") == "fn a() {}\n"
    assert cf.extract_reply("```rust\nfn a() {}\n```\n```rust\nfn b() {}\n```") is None


def test_bank_key_depends_on_errors_and_mode():
    a = cf.bank_key("fn a() {}", {"e1": {"amd64"}}, "off")
    assert a != cf.bank_key("fn a() {}", {"e1": {"i386"}}, "off")
    assert a != cf.bank_key("fn a() {}", {"e1": {"amd64"}}, "on")


def test_a_control_flow_macro_may_become_its_construct():
    original = cf.function_spans(RUST)[1]
    src = RUST[original.start : original.end].replace(
        "STATUS_SUCCESS", "_SEH2_LEAVE;\n    STATUS_SUCCESS"
    )
    labeled = src.replace("_SEH2_LEAVE;", "break 'try_;")
    errors = {"error[E0425]: cannot find value `_SEH2_LEAVE`": {"amd64"}}
    assert cf.reply_findings(src, labeled, errors, lambda _: []) == []
    deleted = src.replace("_SEH2_LEAVE;", "")
    assert any("names dropped" in f for f in cf.reply_findings(src, deleted, errors, lambda _: []))


def test_an_error_inside_a_macro_body_goes_to_the_calling_function(tmp_path):
    (tmp_path / "a.rs").write_text(RUST)
    (tmp_path / "lib.rs").write_text("macro_rules! M { () => { 1u8 } }\n")
    call_line = RUST.splitlines().index("    STATUS_SUCCESS") + 1
    diag = {
        "level": "error",
        "message": "mismatched types",
        "rendered": "error[E0308]: mismatched types",
        "spans": [
            {
                "is_primary": True,
                "file_name": "lib.rs",
                "line_start": 1,
                "column_start": 25,
                "expansion": {
                    "span": {"file_name": "a.rs", "line_start": call_line, "column_start": 5}
                },
            }
        ],
    }
    cf.forget_file_cache()
    groups, outside = cf.group_by_function({"amd64": [diag]}, tmp_path, is_error=lambda d: True)
    assert list(groups) == [("a.rs", "Second")] and outside == {"amd64": 0}


def test_a_second_use_does_not_hide_a_dropped_one():
    span = cf.function_spans(RUST)[1]
    src = RUST[span.start : span.end].replace(
        "STATUS_SUCCESS", "MK!(a);\n    MK!(b);\n    STATUS_SUCCESS"
    )
    one_expanded = src.replace("MK!(b);", "(b as usize);")
    found = cf.reply_findings(src, one_expanded, {"error[E0308]": {"amd64"}}, lambda _: [])
    assert any("names dropped: MK" in f for f in found)


def test_transmute_and_intrinsics_are_refused():
    span = cf.function_spans(RUST)[1]
    src = RUST[span.start : span.end]
    abi = src.replace("CALL(p", "core::mem::transmute::<usize, usize>(0); CALL(p")
    assert any("forbidden" in f for f in cf.reply_findings(src, abi, {}, lambda _: []))
