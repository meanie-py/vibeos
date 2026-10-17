"""A hand edit inside a generated prelude block survives the next `port macros` run."""

from rustconv import generated_items as gi
from rustconv import local_macros as lm
from rustconv import sdk_macros as sm
from rustconv.local_macros import CMacro

WIDE = "#[macro_export] macro_rules! GetDiskSizeInBytes { ($p0:expr $(,)?) => { ((*$p0).A * (*$p0).B) }; }"


def _prelude(tmp_path, block: str):
    p = tmp_path / "prelude.rs"
    p.write_text(f"// head\n\n{block}// tail\n")
    return p


def test_stamp_is_whitespace_blind_and_detects_edits():
    item = gi.stamp(WIDE)
    assert item.endswith(f"// gen:{gi.digest(WIDE)}") and gi.stamp(item) == item
    assert not gi.edited(item, None)
    assert not gi.edited(item.replace(" => ", "=>\n    "), None)  # a reformat
    assert gi.edited(item.replace("(*$p0).B", "((*$p0).B as ULONGLONG)"), None)
    assert gi.edited(gi._STAMP.sub("", item), item.replace("A", "C"))  # unstamped, differs
    assert not gi.edited(gi._STAMP.sub("", item), item)  # unstamped, regenerates the same


def test_sdk_block_keeps_a_hand_edited_macro_and_reexports_it(tmp_path):
    other = "#[macro_export] macro_rules! MAKEWPARAM { ($p0:expr $(,)?) => { $p0 }; }"
    p = _prelude(tmp_path, sm.render_block({"GetDiskSizeInBytes": WIDE, "MAKEWPARAM": other}))
    fixed = "((*$p0).A * ((*$p0).B as ULONGLONG))"
    p.write_text(p.read_text().replace("((*$p0).A * (*$p0).B)", fixed))
    fresh = sm.render_items({"GetDiskSizeInBytes": WIDE, "MAKEWPARAM": other})
    kept = gi.preserve(p, sm.BEGIN, sm.END, sm.split_block, fresh, sm.reexport)
    assert set(kept) == {"MAKEWPARAM"}
    sm.write_block(p, sm.render_block({"MAKEWPARAM": other}))
    text = p.read_text()
    head, block = text.split(sm.BEGIN)
    assert fixed in head and "pub use crate::GetDiskSizeInBytes;" in head and "// gen:" not in head
    assert "GetDiskSizeInBytes" not in block and "pub use crate::{MAKEWPARAM};" in block
    assert "GetDiskSizeInBytes" in sm.outside_block_names(p)  # hand-owned from now on
    # The next run regenerates what it owns and leaves the moved item alone.
    assert gi.preserve(
        p, sm.BEGIN, sm.END, sm.split_block, {"MAKEWPARAM": fresh["MAKEWPARAM"]}, sm.reexport
    ) == {"MAKEWPARAM": fresh["MAKEWPARAM"]}
    assert p.read_text() == text


def test_sdk_split_groups_multiline_and_per_arch_items():
    wrap = (
        '#[cfg(target_arch = "x86")]\n#[inline]\npub unsafe fn IFoo_Bar(This: *mut IFoo) {}\n'
        '#[cfg(target_arch = "x86_64")]\n#[inline]\npub unsafe fn IFoo_Bar(This: *mut IFoo) {}'
    )
    block = sm.render_block(
        {"M": "#[macro_export] macro_rules! M { () => { 1 }; }"},
        {"IFoo_Bar": wrap, "K": "pub const K: u32 = 1;"},
    )
    body = block.split(sm.BEGIN, 1)[1].split(sm.END)[0]
    got = sm.split_block(body)
    assert set(got) == {"M", "IFoo_Bar", "K"}
    assert got["IFoo_Bar"].count("pub unsafe fn") == 2 and not gi.edited(got["IFoo_Bar"], None)


def test_local_block_round_trips_and_keeps_an_edit(tmp_path):
    macros = {
        "SetDlgItemFont": CMacro("SetDlgItemFont", ("h", "f"), "SetWindowFont(h, f)", "r.h:37"),
        "ShowDlgItem": CMacro("ShowDlgItem", ("h",), "ShowWindow(h)", "r.h:34"),
    }
    ported = {
        "SetDlgItemFont": "macro_rules! SetDlgItemFont {\n    ($h:expr, $f:expr) => { SetWindowFont($h, $f) };\n}",
        "ShowDlgItem": "macro_rules! ShowDlgItem {\n    ($h:expr) => { ShowWindow($h) };\n}",
    }
    items = lm.render_items(ported, macros)
    p = _prelude(tmp_path, lm.render_block(items))
    body = p.read_text().split(lm.BEGIN)[1].split(lm.END)[0]
    assert lm.split_block(body) == items
    p.write_text(p.read_text().replace("SetWindowFont($h", "SetWindowFont!($h"))
    # The early pass (nothing regenerated yet) already sees the stamped edit.
    gi.preserve(p, lm.BEGIN, lm.END, lm.split_block, {})
    assert lm.hand_ported(p) == {"SetDlgItemFont"} and lm.existing(p) == {"ShowDlgItem"}
    head, block = p.read_text().split(lm.BEGIN)
    assert "SetWindowFont!($h" in head and "SetDlgItemFont" not in block
