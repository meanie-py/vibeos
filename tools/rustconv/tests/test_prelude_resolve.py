"""prelude_resolve: copy only what the tree agrees on, never a guess."""

from rustconv.prelude_resolve import Item, consensus, render, existing_block


def C(text, path="a.rs"):
    return Item("const", text, None, path)


def F(text, abi='"C"', path="a.rs"):
    return Item("fn", text, abi, path)


def test_constants_agree_by_value_not_spelling():
    pick, _ = consensus("FILE_SHARE_READ", [
        C("const FILE_SHARE_READ: DWORD = 0x00000001;"),
        C("const FILE_SHARE_READ: DWORD = 0x00000001;", "b.rs"),
        C("const FILE_SHARE_READ: ULONG = 0x0000_0001;", "c.rs"),
        C("const FILE_SHARE_READ: u32 = 1u32 as u32;", "d.rs"),
    ])
    assert pick is not None and pick.text == "const FILE_SHARE_READ: DWORD = 0x00000001;"


def test_constants_with_different_values_are_refused():
    pick, why = consensus("X", [C("const X: u32 = 1;"), C("const X: u32 = 2;")])
    assert pick is None and "different values" in why


def test_an_unevaluated_expression_neither_vetoes_nor_is_copied():
    pick, _ = consensus("IOCTL", [
        C("const IOCTL: u32 = 0x74004;"),
        C("const IOCTL: DWORD = CTL_CODE(FILE_DEVICE_DISK, 1, METHOD_BUFFERED, FILE_READ_ACCESS);"),
    ])
    assert pick is not None and "0x74004" in pick.text


def test_extern_abi_class_must_agree():
    pick, why = consensus("RtlFreeHeap", [
        F("fn RtlFreeHeap(h: HANDLE, f: ULONG, p: PVOID) -> BOOLEAN;", '"system"'),
        F("fn RtlFreeHeap(h: HANDLE, f: ULONG, p: PVOID) -> BOOLEAN;", '"C"'),
    ])
    assert pick is None and "ABI" in why
    pick, _ = consensus("RtlFreeHeap", [
        F("fn RtlFreeHeap(h: HANDLE, f: ULONG, p: PVOID) -> BOOLEAN;", '"system"'),
        F("fn RtlFreeHeap(h: HANDLE, f: ULONG, p: PVOID) -> BOOLEAN;", '"stdcall"'),
        F("fn RtlFreeHeap(_: usize, _: usize, _: usize);", '"C"'),  # a link stub, not evidence
    ])
    assert pick is not None


def test_types_are_never_copied():
    pick, why = consensus("HANDLE", [Item("type", "type HANDLE = *mut c_void;", None, "a.rs")])
    assert pick is None and "island" in why


def test_rendered_block_round_trips(tmp_path):
    block = render({
        "X": C("const X: u32 = 1;"),
        "_wcsicmp": F("fn _wcsicmp(a: *const u16, b: *const u16) -> c_int;"),
    })
    assert 'extern "C" {' in block and '""C""' not in block
    prelude = tmp_path / "prelude.rs"
    prelude.write_text("pub use crate::types::*;\n\n" + block)
    back = existing_block(prelude)
    assert set(back) == {"X", "_wcsicmp"} and back["_wcsicmp"].abi == '"C"'
