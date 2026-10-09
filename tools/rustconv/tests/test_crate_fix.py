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


def test_a_call_the_build_never_compiles_may_be_dropped(tmp_path: Path):
    # osdetect.c: `#define NDEBUG`, then `#ifndef NDEBUG DumpNTOSInstalls(List); #endif`.
    c = tmp_path / "x.c"
    c.write_text(
        "#define NDEBUG\nint Find(int List)\n{\n#ifndef NDEBUG\n    DumpAll(List);\n#endif\n"
        "    return List;\n}\n"
    )
    cache: dict = {}
    text = cf.c_function_text(c, "Find", cache)
    assert "DumpAll" not in text and "return List;" in text
    assert cf.c_dead_names(c, "Find", cache) == frozenset({"DumpAll"})
    span = cf.function_spans(RUST)[1]
    src = RUST[span.start : span.end].replace(
        "STATUS_SUCCESS\n}", "DumpAll(p);\n    STATUS_SUCCESS\n}"
    )
    dropped = src.replace("DumpAll(p);\n    ", "")
    errs = {"error[E0425]": {"amd64"}}
    assert any("names dropped" in f for f in cf.reply_findings(src, dropped, errs, lambda _: []))
    assert (
        cf.reply_findings(src, dropped, errs, lambda _: [], dead_names=frozenset({"DumpAll"})) == []
    )


def test_a_null_fn_pointer_may_become_none():
    span = cf.function_spans(RUST)[1]
    src = RUST[span.start : span.end].replace(
        "STATUS_SUCCESS\n}", "Set(core::ptr::null_mut());\n    STATUS_SUCCESS\n}"
    )
    fixed = src.replace("core::ptr::null_mut()", "None")
    assert cf.reply_findings(src, fixed, {"error[E0308]": {"amd64"}}, lambda _: []) == []


TWINS = """#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn Cb(a: u32) -> u32 {
    a
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn Cb(a: u32) -> u32 {
    a
}
"""


def test_cfg_twins_are_separate_functions(tmp_path: Path):
    # setuplib: seven NTAPI callbacks written as x86 / not(x86) twins. Keyed by name, a revert
    # put the not(x86) text over the x86 twin and both ended up not(x86).
    spans = cf.function_spans(TWINS)
    assert [cf.span_key(spans, sp) for sp in spans] == ["Cb#1", "Cb#2"]
    (tmp_path / "a.rs").write_text(TWINS)
    second = TWINS[spans[1].start : spans[1].end]
    cf._replace_named(tmp_path, "a.rs", "Cb#2", second.replace("a\n}", "a + 0\n}"), second)
    out = (tmp_path / "a.rs").read_text()
    assert (
        'cfg(target_arch = "x86")]\nunsafe extern "stdcall" fn Cb(a: u32) -> u32 {\n    a\n}' in out
    )
    assert "a + 0" in out
    import pytest

    with pytest.raises(RuntimeError):
        cf._replace_named(tmp_path, "a.rs", "Cb#2", "x", second)  # no longer there as `second`


def test_a_fix_may_not_invent_a_success_value():
    span = cf.function_spans(RUST)[1]
    src = RUST[span.start : span.end]
    c_fn = "{\n    Status = Call(p);\n    if (!x) Status = STATUS_X;\n    return Status;\n}"
    fixed = src.replace("STATUS_SUCCESS\n}", "Status = 0;\n    STATUS_SUCCESS\n}")
    found = cf.reply_findings(src, fixed, {"error[E0308]": {"amd64"}}, lambda _: [], c_fn=c_fn)
    assert any("Status = 0" in f for f in found)


def test_a_banked_fix_follows_its_function_to_new_text():
    original = "fn A() {\n    let x: u32 = f();\n    g(x as u8);\n    h(1);\n}\n"
    fixed = original.replace("g(x as u8);", "g(x as u16);")
    # A deterministic stage later rewrote another line of the same function.
    current = original.replace("h(1);", "h(1 as i32);")
    merged = cf.merge_banked(current, [{"original": original, "fixed": fixed}])
    assert merged == "fn A() {\n    let x: u32 = f();\n    g(x as u16);\n    h(1 as i32);\n}\n"
    conflicting = original.replace("g(x as u8);", "g(x as u32);")
    assert cf.merge_banked(conflicting, [{"original": original, "fixed": fixed}]) is None


def test_a_parameter_type_rustc_cannot_find_may_be_corrected():
    # atapi: `DevExt: *mut ATA_PORT_DEVICE_EXTENSION` (no such type) -> ATAPORT_DEVICE_EXTENSION
    # was rejected twice as "the signature changed"; rustc renders E0412 at the parameter line.
    original = _second().replace("p: *mut u8", "p: *mut ATA_PORT_X")
    fixed = original.replace("ATA_PORT_X", "ATAPORT_X")
    errors = {"error[E0412]: cannot find type `ATA_PORT_X` in this scope": {"amd64"}}
    assert cf.reply_findings(original, fixed, errors, lambda _: []) == []
    # A signature change rustc did not ask for is still refused.
    other = original.replace("p: *mut ATA_PORT_X", "p: *const ATA_PORT_X")
    assert any("signature" in f for f in cf.reply_findings(original, other, errors, lambda _: []))


def test_struct_literal_field_labels_are_not_kept_names():
    # atapi: a zeroed union spelled field by field came back as `{ AsULONG: 0 }`.
    body = "let a = ADDR { u: INNER { Lun: 0, TargetId: 0, IsValid: FLAG_A } };"
    original = _second().replace("let", body + "\n    let", 1) if "let" in _second() else None
    original = original or _second().replace("{\n", "{\n    " + body + "\n", 1)
    fixed = original.replace(body, "let a = ADDR { AsULONG: FLAG_A };")
    assert (
        cf.reply_findings(original, fixed, {"error[E0560]": {"amd64"}}, lambda _: [], {"INNER"})
        == []
    )
    # ...but the VALUE a label carried is still a kept name.
    lost = original.replace(body, "let a = ADDR { AsULONG: 0 };")
    found = cf.reply_findings(original, lost, {"error[E0560]": {"amd64"}}, lambda _: [])
    assert any("FLAG_A" in f for f in found)


def test_an_empty_or_attribute_macro_may_vanish():
    original = cf.function_spans(RUST)[1]
    src = RUST[original.start : original.end].replace(
        "STATUS_SUCCESS", "NOTHING;\n    STATUS_SUCCESS"
    )
    errors = {"error[E0425]: cannot find value `NOTHING`": {"amd64"}}
    assert cf.reply_findings(src, src.replace("NOTHING;\n", ""), errors, lambda _: []) == []
    aligned = src.replace("NOTHING;", "DECLSPEC_ALIGN(16);")
    errors = {"error[E0425]: cannot find function `DECLSPEC_ALIGN`": {"amd64"}}
    dropped = aligned.replace("DECLSPEC_ALIGN(16);", "")
    assert any(
        "names dropped" in f for f in cf.reply_findings(aligned, dropped, errors, lambda _: [])
    )
    kept = aligned.replace("DECLSPEC_ALIGN(16);", "#[repr(align(16))] struct A([u8; 16]);")
    assert not any(
        "names dropped" in f for f in cf.reply_findings(aligned, kept, errors, lambda _: [])
    )
