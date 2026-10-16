"""kbd_layouts writes each layout's .spec mapping, not just its tables (2026-10-11).

Five layouts from an upstream sync arrived with a plain `@ stdcall KbdLayerDescriptor()`;
their tables generated, their DLLs failed to link.
"""

from pathlib import Path

import pytest

from rustconv import kbd_layouts as kl

MOD = (
    '#[export_name = "kbdxx_KbdLayerDescriptor"]\n'
    'pub unsafe extern "system" fn KbdLayerDescriptor() -> PKBDTABLES { todo!() }\n'
)
NLS = MOD.replace("KbdLayerDescriptor", "KbdNlsLayerDescriptor")


def _spec(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, text: str) -> Path:
    monkeypatch.setattr(kl, "KEYBOARD", tmp_path)
    (tmp_path / "kbdxx").mkdir()
    spec = tmp_path / "kbdxx" / "kbdxx.spec"
    spec.write_text(text)
    return spec


def test_upstream_plain_names_are_pointed_at_the_accessors(tmp_path, monkeypatch):
    spec = _spec(
        tmp_path, monkeypatch, "@ stdcall KbdLayerDescriptor()\n@ stdcall KbdNlsLayerDescriptor()\n"
    )
    assert kl.wire_spec("kbdxx", {"mod.rs": MOD + NLS})
    assert spec.read_text() == (
        "@ stdcall KbdLayerDescriptor() kbdxx_KbdLayerDescriptor\n"
        "@ stdcall KbdNlsLayerDescriptor() kbdxx_KbdNlsLayerDescriptor\n"
    )


def test_an_already_mapped_spec_is_left_alone(tmp_path, monkeypatch):
    _spec(tmp_path, monkeypatch, "@ stdcall KbdLayerDescriptor() kbdxx_KbdLayerDescriptor\n")
    assert not kl.wire_spec("kbdxx", {"mod.rs": MOD})


def test_an_export_the_c_does_not_define_is_an_error(tmp_path, monkeypatch):
    _spec(
        tmp_path, monkeypatch, "@ stdcall KbdLayerDescriptor()\n@ stdcall KbdNlsLayerDescriptor()\n"
    )
    with pytest.raises(kl.LayoutError, match="KbdNlsLayerDescriptor"):
        kl.wire_spec("kbdxx", {"mod.rs": MOD})


CELL = """# header

[package]
name = "kbdlayouts-cargo"

[[bin]]
name = "kbda"
path = "src/main.rs"

[[bin]]
name = "kbdc"
path = "src/main.rs"
"""


def test_the_cell_gets_a_bin_per_layout_in_list_order(tmp_path):
    cell = tmp_path / "Cargo.toml"
    cell.write_text(CELL)
    assert kl.wire_bins(["kbda", "kbdb", "kbdc"], cell)
    assert cell.read_text() == CELL.replace(
        '[[bin]]\nname = "kbdc"', '[[bin]]\nname = "kbdb"\npath = "src/main.rs"\n\n[[bin]]\nname = "kbdc"'
    )
    assert not kl.wire_bins(["kbda", "kbdb", "kbdc"], cell)
