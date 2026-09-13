from pathlib import Path

from fn_coverage import audit


def _crate(tmp_path: Path):
    source = tmp_path / "source"
    source.mkdir()
    (source / "lost.c").write_text("void Lost(void) {}\n")
    (source / "CMakeLists.txt").write_text("# lost.c replaced by Rust\n")
    crate = tmp_path / "pilot-rust"
    (crate / "src").mkdir(parents=True)
    (crate / "src/lib.rs").write_text("")
    return source, crate


def test_missing_whole_translation_file_fails(tmp_path):
    source, crate = _crate(tmp_path)
    assert audit(source, crate)[0] == 1


def test_foreign_declaration_cannot_cover_a_missing_definition(tmp_path):
    source, crate = _crate(tmp_path)
    (crate / "src/lost.rs").write_text('extern "C" { fn Lost(); }')
    assert audit(source, crate)[0] == 1


def test_real_definition_covers_source(tmp_path):
    source, crate = _crate(tmp_path)
    (crate / "src/lost.rs").write_text('pub unsafe extern "system" fn Lost() {}')
    assert audit(source, crate)[0] == 0
