"""port resolve's UNICODE TCHAR struct family: generic names re-pointed at W, never A names."""

from rustconv import port


def test_an_explicit_ansi_name_is_never_repointed(tmp_path):
    src = tmp_path / "src"
    (src / "m").mkdir(parents=True)
    (src / "types_amd64.rs").write_text(
        "pub type WNDCLASSEX = WNDCLASSEXA;\n"
        "pub type WNDCLASSEXA = _WNDCLASSEXA;\n"
        "pub type WNDCLASSEXW = _WNDCLASSEXW;\n"
        "pub struct _WNDCLASSEXA {}\npub struct _WNDCLASSEXW {}\n"
    )
    (src / "m" / "a.rs").write_text(
        "// the island spells m_wc as WNDCLASSEXA\nlet w: WNDCLASSEX;\n"
    )
    out = port._tchar_structs({"stem": "m"}, src / "prelude.rs")
    assert out == {"WNDCLASSEX": "WNDCLASSEXW"}
