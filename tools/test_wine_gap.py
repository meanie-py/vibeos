import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import wine_gap


def _mod(root: Path, kind: str, name: str, makefile: str, src_lines: int = 3):
    d = root / kind / name
    d.mkdir(parents=True)
    (d / "Makefile.in").write_text(makefile)
    (d / "main.c").write_text("x\n" * src_lines)
    (d / "tests").mkdir()
    (d / "tests" / "t.c").write_text("x\n" * 100)  # tests are never counted


def test_absent_modules_are_sized_classified_and_import_checked(tmp_path):
    wine, repo = tmp_path / "wine", tmp_path / "repo"
    _mod(wine, "dlls", "kernel32", "IMPORTS = ntdll\n")
    _mod(wine, "dlls", "taskschd", "IMPORTS = oleaut32 ole32\n", src_lines=5)
    _mod(wine, "dlls", "mfreadwrite", "IMPORTS = mfplat ole32\n")
    _mod(wine, "dlls", "winex11.drv", "UNIXLIB = winex11.so\nIMPORTS = user32\n")
    (wine / "programs").mkdir()
    (repo / "dll" / "kernel32").mkdir(parents=True)
    (repo / "dll" / "kernel32" / "kernel32.spec").write_text("")
    (repo / "dll" / "ole").mkdir(parents=True)
    (repo / "dll" / "ole" / "CMakeLists.txt").write_text(
        "add_library(ole32 MODULE x.c)\nadd_library(oleaut32 MODULE y.c)\n"
    )
    mods = {m.name: m for m in wine_gap.survey(wine, repo)}
    assert mods["kernel32"].present and not mods["taskschd"].present
    assert mods["taskschd"].lines == 5 and mods["taskschd"].missing_imports == []
    assert mods["mfreadwrite"].area.startswith("Media Foundation")
    assert mods["mfreadwrite"].missing_imports == ["mfplat"]
    x11 = mods["winex11.drv"]
    assert x11.unix and not x11.useful and x11.area == "Unix backend"
    text = wine_gap.report(list(mods.values()), list_all=True)
    assert "mfplat (1)" in text and "taskschd" in text


def test_makefile_imports_follow_continuations_and_skip_variables():
    mk = "IMPORTS = uuid ole32 \\\n  $(PNG_PE_LIBS) shlwapi\nDELAYIMPORTS = -lcomctl32\n"
    assert wine_gap.makefile_imports(mk) == ["comctl32", "ole32", "shlwapi", "uuid"]
