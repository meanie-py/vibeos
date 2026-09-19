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
        anchor.build_anchor_map(Path("pilot.c"), strict=True)


def test_strict_rejects_partial_map(monkeypatch):
    monkeypatch.setattr(anchor, "_transpile_text", lambda *a, **kw: "pub unsafe fn Kept() {}")
    with pytest.raises(anchor.C2RustError, match="Lost"):
        anchor.build_anchor_map(Path("pilot.c"), strict=True, required_names={"Kept", "Lost"})


def test_complete_map_is_accepted(monkeypatch):
    monkeypatch.setattr(anchor, "_transpile_text", lambda *a, **kw: "pub unsafe fn Kept() {}")
    assert set(anchor.build_anchor_map(Path("pilot.c"), strict=True, required_names={"Kept"})) == {
        "Kept"
    }


def test_anchor_flags_preserve_preprocessor_order_and_quoted_paths():
    entry = {"arguments": ["clang", "-D", "WIN32", "-UWIN32", "-I", "/source with spaces",
                            "-idirafter", "/late headers", "--target=x86_64-w64-mingw32",
                            "-fshort-wchar", "-o", "out.obj", "-c", "file.c"]}
    assert anchor._extract_clang_flags(entry) == entry["arguments"][1:-4]


def test_nested_sal_does_not_erase_cdfs_functions():
    from rustconv.scanner import scan_file, _strip_sal
    from rustconv.function_coverage import require_c_plan
    source = Path(__file__).resolve().parents[3] / 'drivers/filesystems/cdfs/filobsup.c'
    scan = scan_file(source)
    names = {f.name for f in scan.functions}
    assert {'CdSetFileObject', 'CdDecodeFileObject', 'CdFastDecodeFileObject'} <= names
    require_c_plan(source.read_text(), names)
    diagnostic = 'void f() { log("IN _When_(x, (y))"); }'
    assert _strip_sal(diagnostic) == diagnostic


def test_compiler_panic_reason_keeps_payload_not_just_source_location():
    message = "thread 'main' panicked at convert_type.rs:518:25:\nunsupported calling convention\nnote: trace"
    assert 'unsupported calling convention' in anchor._c2rust_reason(message, '')


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
