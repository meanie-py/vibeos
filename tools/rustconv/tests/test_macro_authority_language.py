"""A borrowed sibling compile command must match the SOURCE LANGUAGE.

Regression: translating a C++ TU in a module whose other TUs are C borrowed a `.c`
sibling's command verbatim, `-std=gnu99` included. clang refuses that on a C++ input
("invalid argument '-std=gnu99' not allowed with 'C++'"), so the preprocess failed on
every arch and the run aborted before it could seal calls.
"""

from __future__ import annotations

import json
from pathlib import Path

from rustconv.macro_authority import (
    _drop_language_flags,
    _is_cxx,
    _sibling_target_entry,
)


def test_is_cxx_covers_the_usual_spellings():
    assert _is_cxx(Path("a.cpp")) and _is_cxx(Path("a.cc")) and _is_cxx(Path("a.cxx"))
    assert not _is_cxx(Path("a.c"))


def test_drop_language_flags_keeps_the_include_closure():
    argv = [
        "clang", "-std=gnu99", "-I/inc/one", "-DFOO=1", "-x", "c",
        "--target=x86_64-w64-mingw32", "-I/inc/two", "-c", "a.c",
    ]  # fmt: skip
    out = _drop_language_flags(argv)
    assert "-std=gnu99" not in out
    assert "-x" not in out and "c" not in out  # the `-x c` pair goes together
    # the module's environment — the whole reason to borrow — survives
    assert "-I/inc/one" in out and "-I/inc/two" in out and "-DFOO=1" in out
    assert "--target=x86_64-w64-mingw32" in out


def _db(repo: Path, arch: str, entries: list[tuple[Path, str]]) -> None:
    """Write a compile_commands.json of (source, flags) pairs."""
    build = repo / f"build-{arch}"
    build.mkdir(parents=True, exist_ok=True)
    build.joinpath("compile_commands.json").write_text(
        json.dumps([{"file": str(f), "command": f"clang {flags} -c {f}"} for f, flags in entries])
    )


def test_same_language_sibling_is_preferred(tmp_path):
    repo = tmp_path / "repo"
    mod = repo / "dll" / "win32" / "shell32"
    mod.mkdir(parents=True)
    (mod / "Target.cpp").write_text("")
    _db(
        repo,
        "amd64",
        [
            (mod / "Other.c", "-std=gnu99 -I/c-only"),
            (mod / "Sibling.cpp", "-std=gnu++17 -I/cxx"),
        ],
    )

    argv = _sibling_target_entry(repo, "amd64", mod / "Target.cpp")
    assert argv is not None
    assert "-std=gnu++17" in argv, "must borrow the C++ sibling, not the C one"
    assert "-std=gnu99" not in argv
    assert str(mod / "Target.cpp") in argv


def test_cross_language_fallback_drops_the_c_dialect(tmp_path):
    """Only C siblings exist: still usable, but the C dialect flag must not survive."""
    repo = tmp_path / "repo"
    mod = repo / "dll" / "win32" / "shell32"
    mod.mkdir(parents=True)
    (mod / "Target.cpp").write_text("")
    _db(repo, "amd64", [(mod / "Other.c", "-std=gnu99 -I/shared -DX=1")])

    argv = _sibling_target_entry(repo, "amd64", mod / "Target.cpp")
    assert argv is not None
    assert "-std=gnu99" not in argv, "would hard-error on a C++ input"
    assert "-I/shared" in argv and "-DX=1" in argv
    assert str(mod / "Target.cpp") in argv


def test_c_source_still_prefers_a_c_sibling(tmp_path):
    """The preference is symmetric — a C target must not pick up -std=gnu++17."""
    repo = tmp_path / "repo"
    mod = repo / "drivers" / "net" / "thing"
    mod.mkdir(parents=True)
    (mod / "Target.c").write_text("")
    _db(
        repo,
        "amd64",
        [
            (mod / "Helper.cpp", "-std=gnu++17 -I/cxx"),
            (mod / "Other.c", "-std=gnu99 -I/c"),
        ],
    )

    argv = _sibling_target_entry(repo, "amd64", mod / "Target.c")
    assert argv is not None
    assert "-std=gnu99" in argv and "-std=gnu++17" not in argv


def test_identifier_aliases_fails_loud_without_any_command(tmp_path):
    # It used to answer `{}` when the TU had no compile command, which after a cohab flip
    # silently dropped every A/W alias the re-resolve was meant to keep.
    import pytest

    from rustconv.macro_authority import MacroAuthorityError, identifier_aliases

    src = tmp_path / "dll" / "x" / "a.c"
    src.parent.mkdir(parents=True)
    src.write_text("int x;\n")
    with pytest.raises(MacroAuthorityError):
        identifier_aliases(tmp_path, src)


def test_a_cpp_tu_refuses_a_flipped_modules_c_stub(tmp_path):
    """cleanmgr/mspaint: once every .cpp is flipped, the C cohab stub is the only command
    left, and it lacks the C++-only include directories (ATL's). Borrowing it failed later on
    `atlbase.h`; it now says what to do."""
    import pytest

    from rustconv.macro_authority import MacroAuthorityError

    repo = tmp_path / "repo"
    mod = repo / "base" / "applications" / "cleanmgr"
    mod.mkdir(parents=True)
    (mod / "cleanmgr.cpp").write_text("")
    _db(repo, "amd64", [(mod / "rust_cohab_stub.c", "-std=gnu99 -I/c-only")])
    with pytest.raises(MacroAuthorityError, match="rust_cohab_stub.cpp"):
        _sibling_target_entry(repo, "amd64", mod / "cleanmgr.cpp")
    # A .cpp stub carries them, and is borrowed as before.
    _db(repo, "amd64", [(mod / "rust_cohab_stub.cpp", "-std=gnu++17 -I/atl")])
    argv = _sibling_target_entry(repo, "amd64", mod / "cleanmgr.cpp")
    assert argv is not None and "-I/atl" in argv
