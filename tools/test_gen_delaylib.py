"""Native delay imports retain NONAME ordinals and all architecture thunks."""

import subprocess
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
FIXTURES = REPO / "tools/vibeos-codegen/tests/bindings"


def run(*args):
    return subprocess.run(
        [str(REPO / "tools/codegen"), "delaylib", *map(str, args)], capture_output=True, check=False
    )


@pytest.mark.parametrize("arch", ["i386", "amd64", "arm64"])
def test_assembly_matches_legacy_and_noname_never_binds_by_name(arch):
    result = run("--def", FIXTURES / "delay.def", "--arch", arch, "--stdout")
    assert result.returncode == 0, result.stderr.decode()
    assert result.stdout == (FIXTURES / f"delay-{arch}.s").read_bytes()
    assert b'.asciz "SHOnCWMCommandLine"' not in result.stdout
    assert b'.asciz "ByName"' in result.stdout
    assert (
        b"0x8000007f" in result.stdout
        if arch == "i386"
        else b"0x7f\n\t.long 0x80000000" in result.stdout
    )


def test_noname_without_ordinal_fails_loud(tmp_path):
    source = tmp_path / "x.def"
    source.write_text("LIBRARY a.dll\nEXPORTS\n Broken NONAME\n")
    result = run("--def", source, "--arch", "amd64", "--stdout")
    assert result.returncode != 0
    assert b"no @ordinal" in result.stderr


@pytest.mark.parametrize("arch,machine", [("i386", 0x14C), ("amd64", 0x8664), ("arm64", 0xAA64)])
def test_archive_contains_real_coff_and_is_preserved_on_failure(tmp_path, arch, machine):
    clang = REPO / ".build-tools/llvm-mingw/bin/clang"
    ar = REPO / ".build-tools/llvm-mingw/bin/llvm-ar"
    output = tmp_path / "libfixture.a"
    args = [
        "--def",
        FIXTURES / "delay.def",
        "--arch",
        arch,
        "--out",
        output,
        "--clang",
        clang,
        "--ar",
        ar,
    ]
    result = run(*args)
    assert result.returncode == 0, result.stderr.decode()
    archive = output.read_bytes()
    assert archive.startswith(b"!<arch>\n")
    members = subprocess.run(
        [str(ar), "t", str(output)], capture_output=True, check=True
    ).stdout.splitlines()
    assert members == [b"libfixture.delay.o"]
    obj = subprocess.run(
        [str(ar), "p", str(output), members[0].decode()], capture_output=True, check=True
    ).stdout
    assert int.from_bytes(obj[:2], "little") == machine
    for option in ("--clang", "--ar"):
        failed = run(*args, option, "/bin/false")
        assert failed.returncode != 0
        assert output.read_bytes() == archive
