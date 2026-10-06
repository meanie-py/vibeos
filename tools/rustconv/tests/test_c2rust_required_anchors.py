import json
from pathlib import Path

import pytest

from rustconv import c2rust_anchor as anchor


def test_foreign_declaration_cannot_steal_next_anchor():
    text = 'extern "C" {\n fn Missing();\n}\npub unsafe fn Kept() { let s = "}"; }'
    assert set(anchor._parse_functions(text)) == {"Kept"}


def test_requested_arch_does_not_fall_back_to_another_build(tmp_path, monkeypatch):
    monkeypatch.setattr(anchor, "REACTOS_ROOT", tmp_path)
    source = tmp_path / "pilot.c"
    source.write_text("void Kept(void) {}")
    other = tmp_path / "build-amd64"
    other.mkdir()
    (other / "compile_commands.json").write_text(json.dumps([{"file": str(source)}]))
    assert anchor._find_db_entry(source, "i386") is None
    assert anchor._find_db_entry(source, "amd64") is not None


def test_strict_rejects_empty_extracted_map(monkeypatch):
    monkeypatch.setattr(anchor, "_transpile_text", lambda *a, **kw: "pub static X: i32 = 0;")
    with pytest.raises(anchor.C2RustError, match="no extracted function"):
        anchor.build_anchor_map(Path("pilot.c"), strict=True, required_names={"Expected"})


def test_a_data_only_tu_needs_no_function_anchors(monkeypatch):
    # setuplib fileqsup.c: one file-scope global, no functions -- only a preamble to translate
    monkeypatch.setattr(anchor, "_transpile_text", lambda *a, **kw: "pub static X: i32 = 0;")
    assert anchor.build_anchor_map(Path("pilot.c"), strict=True, required_names=set()) == {}


def _elsewhere(found, failures):
    def fake(c_file, names, timeout):
        anchor._ANCHOR_FAILURES[c_file] = failures
        return dict(found), names - set(found)

    return fake


def test_strict_rejects_partial_map(monkeypatch):
    # Another arch could not settle the name (its c2rust and preprocessor both failed).
    monkeypatch.setattr(anchor, "_transpile_text", lambda *a, **kw: "pub unsafe fn Kept() {}")
    monkeypatch.setattr(anchor, "_anchor_elsewhere", _elsewhere({}, ["build-i386: panic"]))
    with pytest.raises(anchor.C2RustError, match="Lost") as e:
        anchor.build_anchor_map(Path("pilot.c"), strict=True, required_names={"Kept", "Lost"})
    assert not isinstance(e.value, anchor.CompiledOut)


def test_a_body_only_another_arch_compiles_is_anchored_from_it(monkeypatch):
    # setuplib IsAcpiComputer: called only under `#ifndef _M_AMD64`
    monkeypatch.setattr(anchor, "_transpile_text", lambda *a, **kw: "pub unsafe fn Kept() {}")
    monkeypatch.setattr(anchor, "_anchor_elsewhere", _elsewhere({"Acpi": "{ x }"}, []))
    got = anchor.build_anchor_map(Path("pilot.c"), strict=True, required_names={"Kept", "Acpi"})
    assert got["Acpi"] == "{ x }"


def test_a_body_no_arch_compiles_is_reported_compiled_out(monkeypatch):
    # partlist.c DumpPartitionTable under `#ifdef DUMP_PARTITION_TABLE`, defined nowhere
    monkeypatch.setattr(anchor, "_transpile_text", lambda *a, **kw: "pub unsafe fn Kept() {}")
    monkeypatch.setattr(anchor, "_anchor_elsewhere", _elsewhere({}, []))
    with pytest.raises(anchor.CompiledOut) as e:
        anchor.build_anchor_map(Path("pilot.c"), strict=True, required_names={"Kept", "Dump"})
    assert e.value.names == {"Dump"}


def test_attached_isystem_is_kept():
    # the arm64 db spells `-isystem/…/crt` under -nostdinc
    flags = anchor._extract_clang_flags(
        {"arguments": ["cc", "-nostdinc", "-isystem/inc/crt", "-c", "x.c"]}
    )
    assert "-isystem/inc/crt" in flags


def test_complete_map_is_accepted(monkeypatch):
    monkeypatch.setattr(anchor, "_transpile_text", lambda *a, **kw: "pub unsafe fn Kept() {}")
    assert set(anchor.build_anchor_map(Path("pilot.c"), strict=True, required_names={"Kept"})) == {
        "Kept"
    }


def test_anchor_flags_preserve_preprocessor_order_and_quoted_paths():
    entry = {
        "arguments": [
            "clang",
            "-D",
            "WIN32",
            "-UWIN32",
            "-I",
            "/source with spaces",
            "-idirafter",
            "/late headers",
            "--target=x86_64-w64-mingw32",
            "-fshort-wchar",
            "-o",
            "out.obj",
            "-c",
            "file.c",
        ]
    }
    assert anchor._extract_clang_flags(entry) == entry["arguments"][1:-4]


def test_nested_sal_does_not_erase_cdfs_functions():
    from rustconv.scanner import scan_file, _strip_sal
    from rustconv.function_coverage import require_c_plan

    source = Path(__file__).resolve().parents[3] / "drivers/filesystems/cdfs/filobsup.c"
    scan = scan_file(source)
    names = {f.name for f in scan.functions}
    assert {"CdSetFileObject", "CdDecodeFileObject", "CdFastDecodeFileObject"} <= names
    require_c_plan(source.read_text(), names)
    diagnostic = 'void f() { log("IN _When_(x, (y))"); }'
    assert _strip_sal(diagnostic) == diagnostic


def test_compiler_panic_reason_keeps_payload_not_just_source_location():
    message = "thread 'main' panicked at convert_type.rs:518:25:\nunsupported calling convention\nnote: trace"
    assert "unsupported calling convention" in anchor._c2rust_reason(message, "")


# --- calling-convention guard ------------------------------------------------
# c2rust 0.22.1 panics in convert_type.rs whenever a pointer is formed to a function
# type carrying a calling-convention attribute. We erase __thiscall to get past it,
# but ONLY on 64-bit targets, where the compiler ignores the keyword anyway. Erasing
# it on i386 would silently change the ABI the anchor teaches.


@pytest.mark.parametrize(
    "flags",
    [
        ["--target=i686-w64-mingw32"],
        ["--target=i386-w64-mingw32"],
        ["--target", "i686-pc-windows-gnu"],
        ["-DFOO", "--target=i586-w64-mingw32", "-I/x"],
    ],
)
def test_targets_i386_true(flags):
    assert anchor._targets_i386(flags)


@pytest.mark.parametrize(
    "flags",
    [
        ["--target=x86_64-w64-mingw32"],
        ["--target=aarch64-w64-mingw32"],
        ["--target", "x86_64-pc-windows-gnu"],
        [],
        ["-DTARGET=i686-w64-mingw32"],  # a -D that merely MENTIONS i686 is not a target
    ],
)
def test_targets_i386_false(flags):
    assert not anchor._targets_i386(flags)


def test_db_entry_order_puts_the_pinned_arch_first(tmp_path, monkeypatch):
    """The fallback exists because a c2rust failure can belong to the ARCH, not the
    file — but the arch the caller asked for must still be tried first."""
    src = tmp_path / "x.c"
    src.write_text("int x;\n")
    for bd in ("build-amd64", "build-arm64", "build-i386"):
        d = tmp_path / bd
        d.mkdir()
        (d / "compile_commands.json").write_text(
            json.dumps([{"file": str(src), "command": "clang -c x.c", "directory": str(d)}])
        )
    monkeypatch.setattr(anchor, "REACTOS_ROOT", tmp_path)
    names = [bd.name for _, bd in anchor._find_db_entries(src, "i386")]
    assert names == ["build-i386", "build-amd64", "build-arm64"]
    assert [bd.name for _, bd in anchor._find_db_entries(src, None)] == [
        "build-amd64",
        "build-arm64",
        "build-i386",
    ]


def test_pseh_variants_are_tried_labelled_first():
    """Order matters: the labelled override is the only one that keeps _SEH2_LEAVE's
    goto target, and it only COMPILES when it is also correct (a second try block in
    the same function is a duplicate label), so trying it first can never anchor a
    file against control flow the C does not have."""
    labelled, labelfree = anchor._PSEH_VARIANTS
    assert any("__seh2_scope_end__:;" in d for d in labelled)
    assert not any("__seh2_scope_end__" in d for d in labelfree)
    for variant in anchor._PSEH_VARIANTS:
        names = {d.split("=", 1)[0] for d in variant}
        assert names == {"-D_SEH2_TRY", "-D_SEH2_FINALLY", "-D_SEH2_EXCEPT(...)", "-D_SEH2_END"}
        # _SEH2_LEAVE is deliberately NOT overridden — it must fail loudly, never quietly.
        assert not any(d.startswith("-D_SEH2_LEAVE") for d in variant)


def test_wide_temporaries_in_a_static_are_hoisted_to_named_statics():
    # setuplib mui.c: `{ L"Arial,186" }` in a file-scope table
    pre = (
        "pub static mut T: [S; 1] = unsafe { [S { F: ::core::mem::transmute::<\n"
        "    [u8; 6],\n    [::core::ffi::c_ushort; 3],\n"
        '>(*b"A\\0,\\0\\0\\0")\n    .as_ptr() as PCWSTR }] };'
    )
    out = anchor.hoist_wide_temporaries(pre)
    assert "transmute" not in out and ".as_ptr() as PCWSTR" in out
    name = out.split("pub static ")[1].split(":")[0]
    assert name.startswith("RUSTCONV_C2R_WSTR_")
    assert "[0x0041u16, 0x002C, 0x0000]" in out
    assert anchor.hoist_wide_temporaries(pre + "\n" + pre).count("pub static RUSTCONV") == 1


def test_type_items_reads_structs_and_aliases_from_c2rust_text():
    text = (
        "pub type UCHAR = ::core::ffi::c_uchar;\n"
        "#[derive(Copy, Clone)]\n#[repr(C)]\npub struct _MBR_PARTITION_TYPE {\n"
        "    pub Type: UCHAR,\n    pub Description: PCSTR,\n}\n"
        "pub type MBR_PARTITION_TYPE = _MBR_PARTITION_TYPE;\n"
    )
    items = anchor._type_items(text)
    assert items["MBR_PARTITION_TYPE"] == "pub type MBR_PARTITION_TYPE = _MBR_PARTITION_TYPE;"
    assert items["_MBR_PARTITION_TYPE"].startswith("#[derive(Copy, Clone)]\n#[repr(C)]\npub struct")
    assert items["_MBR_PARTITION_TYPE"].endswith("pub Description: PCSTR,\n}")
