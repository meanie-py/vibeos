"""cohab_stub: the flipped module's stub references every export, decorated per arch."""

from pathlib import Path

from rustconv.cohab_stub import export_anchors, is_generated, stub_text

SPEC = """\
@ stdcall -private DllGetClassObject(ptr ptr ptr)
@ stdcall -private DllCanUnloadNow()
@ cdecl Helper(long)
@ fastcall Fast(long int64)
@ stdcall -arch=i386 OnlyX86(ptr)
@ stub Unimplemented
@ extern DataItem
@ stdcall Forwarded(ptr) kernel32.Forwarded
"""


def _spec(tmp_path: Path) -> Path:
    p = tmp_path / "mod.spec"
    p.write_text(SPEC)
    return p


def test_i386_names_are_decorated_and_64_bit_are_plain(tmp_path):
    text = export_anchors("mod", _spec(tmp_path))
    i386, rest = text.split("#elif defined(__x86_64__)")
    assert '"_DllGetClassObject@12"' in i386 and '"_DllCanUnloadNow@0"' in i386
    assert '"_Helper"' in i386 and '"@Fast@12"' in i386 and '"_OnlyX86@4"' in i386
    assert '"DllGetClassObject"' in rest and "OnlyX86" not in rest


def test_stubs_data_and_forwards_are_not_anchored(tmp_path):
    text = export_anchors("mod", _spec(tmp_path))
    for name in ("Unimplemented", "DataItem", "Forwarded"):
        assert name not in text


def test_only_a_generated_stub_counts_as_regenerable(tmp_path):
    full = stub_text("mod", "mod-rust", _spec(tmp_path))
    assert is_generated(stub_text("mod", "mod-rust", None), "mod", "mod-rust")
    assert is_generated(full, "mod", "mod-rust")
    assert not is_generated("/* mine */\n", "mod", "mod-rust")
    assert not is_generated(stub_text("mod", "mod-rust", None) + "int x;\n", "mod", "mod-rust")
