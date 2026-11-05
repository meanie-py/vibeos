"""port integrate: a keyword-named C field the model declared `r#ref` but read `.ref_`."""

from rustconv import port


def _state(tmp_path, monkeypatch, files):
    paths = {}
    for name, text in files.items():
        p = tmp_path / f"{name}.rs"
        p.write_text(text)
        paths[name] = p
    monkeypatch.setattr(port, "_units", lambda state: list(paths))
    monkeypatch.setattr(port, "_tu_output", lambda state, tu: paths[tu])
    return paths


def test_a_field_only_read_escaped_is_renamed(tmp_path, monkeypatch):
    """dsound duplex.c: `LONG ref;` declared `pub r#ref: LONG`, read `(*This).ref_` (E0609)."""
    paths = _state(
        tmp_path,
        monkeypatch,
        {
            "duplex": "pub struct Impl {\n    pub r#ref: LONG,\n    pub r#type: u32,\n}\n",
            "user": "fn f(t: *mut Impl) { let _ = (*t).ref_; }\n",
        },
    )
    assert port._unify_keyword_fields({}) == (["ref"], [])
    text = paths["duplex"].read_text()
    assert "pub ref_: LONG," in text
    assert "pub r#type: u32," in text  # never read as `.type_`: untouched


def test_a_field_read_both_ways_is_reported_not_renamed(tmp_path, monkeypatch):
    paths = _state(
        tmp_path,
        monkeypatch,
        {"m": "pub struct S {\n    pub r#ref: i32,\n}\nfn f(s: S) { s.ref_; s.r#ref; }\n"},
    )
    before = paths["m"].read_text()
    assert port._unify_keyword_fields({}) == ([], ["ref"])
    assert paths["m"].read_text() == before
