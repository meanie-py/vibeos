from rustconv import fn_alias_audit


def _crate(tmp_path, body):
    (tmp_path / "Cargo.toml").write_text("[package]\nname = 'x'\n")
    (tmp_path / "src").mkdir()
    (tmp_path / "src" / "lib.rs").write_text(body)
    (tmp_path / "src" / "types_i386.rs").write_text('pub type ISLAND = unsafe extern "C" fn();\n')
    return tmp_path


def test_a_null_test_or_transmute_marks_an_alias_risky(tmp_path):
    """cabinet's FDICreate null-tests a bare-fn parameter; cmd transmutes GetProcAddress."""
    crate = _crate(
        tmp_path,
        'pub type PFNALLOC = unsafe extern "C" fn(u32) -> *mut u8;\n'
        'pub type MYEX = unsafe extern "system" fn(i32) -> i32;\n'
        'pub type QUIET = unsafe extern "C" fn();\n'
        "unsafe fn f(pfnalloc: PFNALLOC) -> bool { (pfnalloc as *const ()).is_null() }\n"
        "unsafe fn g(p: FARPROC) { let h: MYEX = core::mem::transmute(p); }\n"
        "unsafe fn q(cb: QUIET) { cb() }\n",
    )
    found = {a.name: a for a in fn_alias_audit.audit(crate)}
    assert set(found) == {"PFNALLOC", "MYEX", "QUIET"}  # the island's are bindgen's, not ours
    assert found["PFNALLOC"].null_tests == 1 and found["PFNALLOC"].risky
    assert found["MYEX"].transmutes == 1 and found["MYEX"].risky
    assert not found["QUIET"].risky


def test_apply_makes_every_alias_an_option(tmp_path):
    crate = _crate(
        tmp_path,
        'pub type PFN = unsafe extern "C" fn(u32);\nstatic mut P: Option<PFN> = None;\n',
    )
    assert fn_alias_audit.apply(crate) == ["PFN"]
    text = (crate / "src" / "lib.rs").read_text()
    assert 'pub type PFN = Option<unsafe extern "C" fn(u32)>;' in text
    assert "static mut P: PFN = None;" in text
