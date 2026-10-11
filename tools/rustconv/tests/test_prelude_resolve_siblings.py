"""prelude_resolve.defined_outside_siblings: a sibling import never shadows the prelude."""

from rustconv import prelude_resolve as pr


def test_a_name_the_prelude_defines_is_reported_and_the_import_block_is_not(tmp_path):
    prelude = tmp_path / "prelude.rs"
    prelude.write_text(
        "pub unsafe fn E1000ReadUlong(a: u32) {}\n"
        + pr.render_siblings({"E1000ReadUlong": "nic_h", "MiniportISR": "interrupt"}, "e1000")
    )
    names = pr.defined_outside_siblings(prelude)
    assert "E1000ReadUlong" in names
    assert "MiniportISR" not in names
