"""A file compiled by several CMake targets answers with the RIGHT target's command.

Regression: cscript compiles wscript's host.c/main.c with `-DCSCRIPT_BUILD`, and every
compile-db reader took the first entry naming the file. wscript-rust was translated from
cscript's preprocessed text: it shipped non-interactive (`wshInteractive = VARIANT_FALSE`)
and printed "Usage: CScript ...".
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from rustconv import macro_authority as ma


def _entry(repo: Path, src: Path, target_dir: str, target: str, flags: str) -> dict:
    out = f"{target_dir}/CMakeFiles/{target}.dir/{src.name}.obj"
    return {
        "file": str(src),
        "command": f"clang {flags} -o {out} -c {src}",
        "output": out,
    }


@pytest.fixture
def tree(tmp_path, monkeypatch):
    monkeypatch.delenv(ma._TARGET_ENV, raising=False)
    repo = tmp_path / "repo"
    ws = repo / "app" / "wscript"
    cs = repo / "app" / "cscript"
    for d in (ws, cs):
        d.mkdir(parents=True)
        (d / "CMakeLists.txt").write_text(f"add_executable({d.name} x.c)\n")
    for name in ("host.c", "rust_varargs_shim.c"):
        (ws / name).write_text("")
    return repo, ws


def _write_db(repo: Path, entries: list[dict]) -> None:
    build = repo / "build-amd64"
    build.mkdir(exist_ok=True)
    build.joinpath("compile_commands.json").write_text(json.dumps(entries))


def test_named_target_wins(tree, monkeypatch):
    repo, ws = tree
    host = ws / "host.c"
    _write_db(
        repo,
        [
            _entry(repo, host, "app/cscript", "cscript", "-DCSCRIPT_BUILD"),
            _entry(repo, host, "app/wscript", "wscript", "-DUNICODE"),
        ],
    )
    monkeypatch.setenv(ma._TARGET_ENV, "cscript")
    assert "-DCSCRIPT_BUILD" in ma._compile_entry(repo, "amd64", host)
    monkeypatch.setenv(ma._TARGET_ENV, "wscript")
    assert "-DCSCRIPT_BUILD" not in ma._compile_entry(repo, "amd64", host)


def test_owner_beats_db_order_without_a_target(tree):
    repo, ws = tree
    host = ws / "host.c"
    _write_db(
        repo,
        [
            _entry(repo, host, "app/cscript", "cscript", "-DCSCRIPT_BUILD"),
            _entry(repo, host, "app/wscript", "wscript", "-DUNICODE"),
        ],
    )
    assert "-DCSCRIPT_BUILD" not in ma._compile_entry(repo, "amd64", host)


def test_flipped_owner_borrows_its_own_sibling_not_the_foreign_compile(tree, monkeypatch):
    # After the flip only cscript still compiles host.c; wscript's flags survive on its shim.
    repo, ws = tree
    host, shim = ws / "host.c", ws / "rust_varargs_shim.c"
    _write_db(
        repo,
        [
            _entry(repo, host, "app/cscript", "cscript", "-DCSCRIPT_BUILD"),
            _entry(repo, shim, "app/wscript", "wscript_shims", "-DUNICODE"),
        ],
    )
    monkeypatch.setenv(ma._TARGET_ENV, "wscript")
    assert ma._compile_entry(repo, "amd64", host) is None
    argv = ma.compile_argv(repo, "amd64", host)
    assert "-DCSCRIPT_BUILD" not in argv and "-DUNICODE" in argv and str(host) in argv
    monkeypatch.setenv(ma._TARGET_ENV, "cscript")
    assert "-DCSCRIPT_BUILD" in ma.compile_argv(repo, "amd64", host)


def test_disagreeing_owners_are_refused(tree):
    repo, ws = tree
    host = ws / "host.c"
    _write_db(
        repo,
        [
            _entry(repo, host, "app/wscript", "wscript", "-DV=1"),
            _entry(repo, host, "app/wscript", "wscript2", "-DV=2"),
        ],
    )
    with pytest.raises(ma.MacroAuthorityError, match="different flags"):
        ma._compile_entry(repo, "amd64", host)


def test_ancestor_target_owns_a_subdirectory_source(tmp_path, monkeypatch):
    # shell32's CMakeLists lists dialogs/*.cpp: the target lives one directory up.
    monkeypatch.delenv(ma._TARGET_ENV, raising=False)
    repo = tmp_path / "repo"
    sub = repo / "shell32" / "dialogs"
    sub.mkdir(parents=True)
    src = sub / "a.c"
    src.write_text("")
    _write_db(repo, [_entry(repo, src, "shell32", "shell32", "-DSHELL")])
    assert "-DSHELL" in ma._compile_entry(repo, "amd64", src)
