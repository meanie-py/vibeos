"""extern_promote collapses duplicate declarations only when they AGREE semantically."""

from pathlib import Path

from rustconv import extern_promote as EP


def _write(tmp_path: Path, name: str, body: str) -> None:
    (tmp_path / name).write_text(body)


ALIASES = {
    "BOOL": "i32",
    "LPWSTR": "*mut u16",
    "LPCWSTR": "*const u16",
    "DWORD": "u32",
    "INT": "i32",
    "LPCITEMIDLIST": "*const ITEMIDLIST",
}


def test_parameter_names_are_not_part_of_the_signature(tmp_path):
    _write(tmp_path, "a.rs", 'extern "system" {\n    fn F(path: LPWSTR) -> BOOL;\n}\n')
    _write(tmp_path, "b.rs", 'extern "system" {\n    fn F(pszPath: LPWSTR) -> BOOL;\n}\n')
    res = EP.promote(EP.collect(tmp_path), ALIASES)
    assert set(res.agreed) == {"F"} and not res.conflicts


def test_alias_spellings_of_one_type_are_one_signature(tmp_path):
    """LPWSTR/*mut u16 and BOOL/i32 are the same declaration written two ways."""
    _write(tmp_path, "a.rs", 'extern "system" {\n    fn F(p: LPWSTR) -> BOOL;\n}\n')
    _write(tmp_path, "b.rs", 'extern "system" {\n    fn F(p: *mut u16) -> i32;\n}\n')
    res = EP.promote(EP.collect(tmp_path), ALIASES)
    assert set(res.agreed) == {"F"} and not res.conflicts


def test_a_different_return_type_is_a_conflict_not_a_pick(tmp_path):
    """The SHAnsiToUnicode case: one module had it returning a POINTER, others an int."""
    _write(tmp_path, "a.rs", 'extern "system" {\n    fn F(p: LPCWSTR) -> LPWSTR;\n}\n')
    _write(tmp_path, "b.rs", 'extern "system" {\n    fn F(p: LPCWSTR) -> DWORD;\n}\n')
    res = EP.promote(EP.collect(tmp_path), ALIASES)
    assert not res.agreed
    assert len(res.conflicts["F"]) == 2


def test_a_missing_return_type_is_a_conflict(tmp_path):
    """PathUnquoteSpacesW: declared as returning nothing in one module, BOOL elsewhere."""
    _write(tmp_path, "a.rs", 'extern "system" {\n    fn F(p: LPWSTR);\n}\n')
    _write(tmp_path, "b.rs", 'extern "system" {\n    fn F(p: LPWSTR) -> BOOL;\n}\n')
    res = EP.promote(EP.collect(tmp_path), ALIASES)
    assert "F" in res.conflicts


def test_a_different_arity_is_a_conflict(tmp_path):
    _write(tmp_path, "a.rs", 'extern "system" {\n    fn F(a: DWORD) -> BOOL;\n}\n')
    _write(tmp_path, "b.rs", 'extern "system" {\n    fn F(a: DWORD, b: DWORD) -> BOOL;\n}\n')
    res = EP.promote(EP.collect(tmp_path), ALIASES)
    assert "F" in res.conflicts


def test_a_different_abi_is_a_conflict(tmp_path):
    """Same types, different calling convention: an i386 stack imbalance at every call."""
    _write(tmp_path, "a.rs", 'extern "system" {\n    fn F(a: DWORD) -> BOOL;\n}\n')
    _write(tmp_path, "b.rs", 'extern "C" {\n    fn F(a: DWORD) -> BOOL;\n}\n')
    res = EP.promote(EP.collect(tmp_path), ALIASES)
    assert "F" in res.conflicts


def test_signed_and_unsigned_of_the_same_width_still_conflict(tmp_path):
    """GetSystemMetrics(UINT) vs (INT): same register, but the header settles it."""
    _write(tmp_path, "a.rs", 'extern "system" {\n    fn F(a: DWORD) -> INT;\n}\n')
    _write(tmp_path, "b.rs", 'extern "system" {\n    fn F(a: INT) -> INT;\n}\n')
    res = EP.promote(EP.collect(tmp_path), ALIASES)
    assert "F" in res.conflicts


def test_declarations_inside_comments_are_ignored(tmp_path):
    _write(tmp_path, "a.rs", 'extern "system" {\n    fn F(p: LPWSTR) -> BOOL;\n}\n')
    _write(tmp_path, "b.rs", '// extern "C" {\n//     fn F(p: LPWSTR) -> DWORD;\n// }\n')
    res = EP.promote(EP.collect(tmp_path), ALIASES)
    assert set(res.agreed) == {"F"} and not res.conflicts


def test_skip_excludes_a_subtree(tmp_path):
    (tmp_path / "sub").mkdir()
    _write(tmp_path, "a.rs", 'extern "system" {\n    fn F(p: LPWSTR) -> BOOL;\n}\n')
    _write(tmp_path / "sub", "b.rs", 'extern "system" {\n    fn G(p: LPWSTR) -> BOOL;\n}\n')
    assert set(EP.collect(tmp_path, skip="/sub/")) == {"F"}


def test_alias_map_resolves_a_chain(tmp_path):
    _write(tmp_path, "t.rs", "pub type BOOL = i32;\npub type WINBOOL = BOOL;\n")
    aliases = EP.alias_map([tmp_path])
    assert EP.resolve_type("WINBOOL", aliases) == "i32"


def test_alias_map_survives_a_cycle(tmp_path):
    _write(tmp_path, "t.rs", "pub type A = B;\npub type B = A;\n")
    EP.alias_map([tmp_path])  # must terminate


def test_c_int_and_INT_are_the_same_type(tmp_path):
    _write(tmp_path, "a.rs", 'extern "system" {\n    fn F(p: LPCWSTR) -> INT;\n}\n')
    _write(tmp_path, "b.rs", 'extern "system" {\n    fn F(p: LPCWSTR) -> c_int;\n}\n')
    res = EP.promote(EP.collect(tmp_path), ALIASES)
    assert set(res.agreed) == {"F"} and not res.conflicts
