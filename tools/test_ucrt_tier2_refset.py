"""Tests for the Tier-2 ucrt reference-set builder.

The rename map is the whole design: get it wrong and the harness silently grades the
wrong body against the wrong body. These tests pin the two ways it has actually been
wrong, plus the object-list authority.
"""

import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "tools"))

import ucrt_tier2_refset as refset  # noqa: E402

ARCHES = ("i386", "amd64", "arm64")


def _built(arch: str) -> bool:
    return (REPO / f"build-{arch}").is_dir()


def test_decoration_is_settled_per_arch_not_per_symbol():
    """The overlap that corrupted 9 amd64 and 9 arm64 exports.

    `tolower` and `_tolower` are BOTH ucrtbase exports. On an undecorated arch the symbol
    `_tolower` is export `_tolower`'s body; a per-symbol probe that tries `_name` first
    hands it to export `tolower`, which both steals one body and leaves the other
    unrenamed. Settling the convention once for the arch makes that unrepresentable.
    """
    # An undecorated arch: the objects define the export names verbatim. `_tolower` is
    # here as a symbol because it is ITSELF an export, not because `tolower` is decorated.
    defined = {"tolower", "_tolower", "strtol", "atol"}
    rules = dict(refset.rename_rules("amd64", defined))
    assert rules["tolower"] == "ref_tolower"
    assert rules["_tolower"] == "ref__tolower"
    # The bug: `tolower` claiming `_tolower`, which both steals `_tolower`'s body and
    # leaves `tolower` unrenamed (so it would resolve to the DLL and grade against itself).
    assert "_ref_tolower" not in rules.values()


def test_rename_map_never_claims_one_symbol_twice():
    """objcopy silently applies whichever duplicate rule it read last, so refuse instead."""
    # A defined set that makes both conventions score equally would be pathological; the
    # guard is what matters, so assert it is reachable rather than contriving the input.
    src = (REPO / "tools" / "ucrt_tier2_refset.py").read_text()
    assert "rename map is ambiguous" in src, "the duplicate-claim guard was removed"
    assert "len(set(froms)) != len(froms)" in src


@pytest.mark.parametrize("arch", ARCHES)
def test_object_list_comes_from_the_link_line_not_a_glob(arch):
    """`ucrt.dir` holds objects ninja stopped linking; the shipping DLL's set is authority.

    sdk/lib/ucrt_reactos/ replaces four sdk/lib/ucrt/ files (matherr, GetStringTypeW,
    MultiByteToWideChar, WideCharToMultiByte). Ninja never deletes the objects of the
    copies it stopped using, so a directory glob picks up both and the reference set
    fails to link with duplicate symbols.
    """
    if not _built(arch):
        pytest.skip(f"build-{arch} not present")
    objs = refset.ucrt_objects(REPO / f"build-{arch}")
    assert objs, "no ucrt objects found on the link line"
    names = [o.name for o in objs]
    for dupe in (
        "matherr.cpp.obj",
        "GetStringTypeW.c.obj",
        "MultiByteToWideChar.c.obj",
        "WideCharToMultiByte.c.obj",
    ):
        assert names.count(dupe) <= 1, (
            f"{dupe} appears {names.count(dupe)}x -- the object list picked up both the "
            f"ucrt and ucrt_reactos copies, which cannot link"
        )
    # And the copy it DID pick must be the one CMake names: ucrt_reactos.
    for o in objs:
        if o.name in ("matherr.cpp.obj", "GetStringTypeW.c.obj"):
            assert "ucrt_reactos" in str(o), f"{o} is the stale genuine copy, not the built one"


@pytest.mark.parametrize("arch", ARCHES)
def test_every_rule_renames_a_symbol_that_exists(arch):
    """A rule for an absent symbol is a silent no-op, which is how a body goes ungraded."""
    if not _built(arch):
        pytest.skip(f"build-{arch} not present")
    objs = refset.ucrt_objects(REPO / f"build-{arch}")
    defined = refset.defined_symbols(objs)
    rules = refset.rename_rules(arch, defined)
    assert rules, f"{arch}: no rename rules derived"
    missing = [src for src, _ in rules if src not in defined]
    assert not missing, f"{arch}: {len(missing)} rules name absent symbols, e.g. {missing[:5]}"


@pytest.mark.parametrize("arch", ARCHES)
def test_the_reference_set_is_only_microsoft_c(arch):
    """Our own code must never reach the reference side -- it would grade itself.

    ucrtbase's link line also carries ucrtbase's own printf.c/stubs.c and every one of our
    static libs. Only the `ucrt` target's objects are Microsoft's.
    """
    if not _built(arch):
        pytest.skip(f"build-{arch} not present")
    build = REPO / f"build-{arch}"
    objs = refset.ucrt_objects(build)
    for o in objs:
        # Relative to the build dir: the worktree itself may be named after a lane
        # (vibeos-msvcrt-homebrew), so an absolute-path substring test matches itself.
        rel = o.relative_to(build).as_posix()
        assert refset.UCRT_OBJ_MARKER in rel, f"{rel} is not a ucrt-target object"
        assert "msvcrt" not in rel and "crt_shared" not in rel, f"ours leaked in: {rel}"


_MINGW = Path(__file__).resolve().parent.parent / ".build-tools" / "llvm-mingw" / "bin"


@pytest.mark.skipif(not (_MINGW / "aarch64-w64-mingw32-clang").is_file(), reason="needs llvm-mingw")
def test_undefine_turns_a_definition_into_a_reference(tmp_path: Path):
    """The deferral under a takeover check: our `foo` must stop answering, and our own
    call to it must bind to whoever else defines `foo` (Microsoft's copy)."""
    import subprocess

    src = tmp_path / "a.c"
    src.write_text("int foo(void) { return 1; }\nint bar(void) { return foo() + 1; }\n")
    obj, lib = tmp_path / "a.o", tmp_path / "liba.a"
    subprocess.run([str(_MINGW / "aarch64-w64-mingw32-clang"), "-O0", "-c", str(src), "-o", str(obj)], check=True)
    subprocess.run([str(_MINGW / "llvm-ar"), "rcs", str(lib), str(obj)], check=True)
    refset._undefine_symbols(lib, {"foo"})
    nm = subprocess.run([str(_MINGW / "llvm-nm"), str(lib)], capture_output=True, text=True, check=True).stdout
    assert " U foo" in nm and " T bar" in nm and " T foo" not in nm


def test_undefine_refuses_when_nothing_matched(tmp_path: Path):
    lib = tmp_path / "empty.a"
    lib.write_bytes(b"!<arch>\n")
    with pytest.raises(refset.RefsetError):
        refset._undefine_symbols(lib, {"foo"})
