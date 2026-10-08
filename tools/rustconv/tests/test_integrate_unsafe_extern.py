"""crate_integrate handles the Rust 2024 `unsafe extern` block spelling."""

from rustconv.crate_integrate import fix_macro_calls, integrate


def _crate(tmp_path):
    (tmp_path / "src").mkdir()
    (tmp_path / "src" / "prelude.rs").write_text(
        "#[macro_export] macro_rules! DPRINT1 { ($($a:tt)*) => {{}}; }\n"
        "extern \"system\" { pub fn NtClose(h: HANDLE) -> NTSTATUS; }\n"
    )
    return tmp_path


def test_a_dropped_unsafe_extern_block_leaves_no_bare_unsafe(tmp_path):
    src = (
        "use ntdll_sys::nt::*;\n"
        "unsafe extern \"system\" {\n    fn NtClose(h: HANDLE) -> NTSTATUS;\n}\n"
        "pub unsafe extern \"C\" fn f() { NtClose(0 as _); }\n"
    )
    out, report = integrate(src, _crate(tmp_path), module="m")
    assert "NtClose" in report.dropped_provided
    assert not any(line.strip() == "unsafe" for line in out.splitlines())
    assert "unsafe unsafe" not in out


def test_a_declaration_of_a_prelude_macro_is_dropped_not_banged(tmp_path):
    src = "use ntdll_sys::nt::*;\nextern \"C\" {\n    fn DPRINT1(format: *const i8, ...);\n}\n"
    out, _ = integrate(src, _crate(tmp_path), module="m")
    assert "DPRINT1!" not in out and "fn DPRINT1" not in out
    called, _ = fix_macro_calls("fn g() { DPRINT1(x); }", {"DPRINT1"}, set())
    assert "DPRINT1!(x)" in called
