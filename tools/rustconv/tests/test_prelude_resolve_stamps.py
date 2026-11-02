"""port resolve's tree-consensus block keeps hand edits (generated_items stamps)."""

from rustconv import prelude_resolve as pr
from rustconv.prelude_resolve import Item


def _block(tmp_path):
    prelude = tmp_path / "prelude.rs"
    copied = {
        "MAX_X": Item("const", "const MAX_X: u32 = 4;", None, "a-rust/src/x.rs"),
        "Foo": Item("fn", "fn Foo(a: u32) -> u32;", '"system"', "b-rust/src/y.rs"),
        "GetW": Item("fn", "fn GetW(a: *const u16) -> u32;", '"C"', "c-rust/src/z.rs"),
    }
    prelude.write_text("// head\n\n" + pr.render(copied))
    return prelude, copied


def test_render_stamps_and_existing_block_reads_through_the_stamps(tmp_path):
    prelude, copied = _block(tmp_path)
    text = prelude.read_text()
    assert text.count("// gen:") == 3
    back = pr.existing_block(prelude)
    assert {n: (i.text, i.path) for n, i in back.items()} == {
        n: (i.text, i.path) for n, i in copied.items()
    }
    # A re-render of what was read back is byte-identical: stamps never accumulate.
    assert pr.render(back) == pr.render(copied)
    assert pr.preserve_hand_edits(prelude) == [] and prelude.read_text() == text


def test_a_hand_edited_item_moves_out_and_keeps_its_extern_wrapper(tmp_path):
    prelude, _ = _block(tmp_path)
    text = prelude.read_text()
    text = text.replace("const MAX_X: u32 = 4;", "const MAX_X: u32 = 5;")
    text = text.replace("fn Foo(a: u32) -> u32;", "fn Foo(a: u32, b: u32) -> u32;")
    prelude.write_text(text)
    assert pr.preserve_hand_edits(prelude) == ["Foo", "MAX_X"]
    out = prelude.read_text()
    head, _, block = out.partition(pr.BEGIN)
    assert "pub const MAX_X: u32 = 5;" in head and "MAX_X" not in block
    assert 'extern "system" {\n    pub fn Foo(a: u32, b: u32) -> u32;' in head
    assert "gen:" not in head
    assert set(pr.existing_block(prelude)) == {"GetW"}


def test_an_unstamped_legacy_block_is_left_alone(tmp_path):
    prelude, _ = _block(tmp_path)
    legacy = "\n".join(line.split(" // gen:")[0] for line in prelude.read_text().splitlines())
    prelude.write_text(legacy + "\n")
    assert pr.preserve_hand_edits(prelude) == []
    assert set(pr.existing_block(prelude)) == {"MAX_X", "Foo", "GetW"}
