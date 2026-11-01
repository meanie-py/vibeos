"""Exercise the native ctype source/island derivation and independent COFF proof."""

import subprocess
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
MASKS = dict(_UPPER=1, _LOWER=2, _DIGIT=4, _SPACE=8, _PUNCT=16, _CONTROL=32, _BLANK=64, _HEX=128)


def run(root, *args):
    return subprocess.run(
        [str(REPO / "tools/codegen"), "--root", str(root), "ctype-tables", *map(str, args)],
        capture_output=True,
        text=True,
        check=False,
    )


def fixture(root):
    source = root / "dll/win32/msvcrt/ctype.c"
    source.parent.mkdir(parents=True)
    values = ["_S_|_C_", *["0"] * 256]
    source.write_text(
        "WORD MSVCRT__ctype [257] = {"
        + ",".join(values)
        + "};\n"
        + "WORD MSVCRT__wctype[257] = {"
        + ",".join(values)
        + "};\n"
    )
    island = root / "sdk/lib/rust/msvcrt-sys/src/bindgen_i386.rs"
    island.parent.mkdir(parents=True)
    island.write_text("".join(f"pub const {k}: u32 = {v};\n" for k, v in MASKS.items()))
    tools = root / ".build-tools"
    tools.mkdir()
    (tools / "llvm-mingw").symlink_to(REPO / ".build-tools/llvm-mingw", target_is_directory=True)
    return source, island


def compile_object(root, wrong=False):
    # Literal values are independently emitted by clang; the generator parses its
    # own macro-bearing C source and must refuse one disagreement in this object.
    values = [40, *[0] * 256]
    if wrong:
        values[65] = 129
    text = "typedef unsigned short WORD;\n" + "\n".join(
        f"WORD {name}[257] = {{" + ",".join(map(str, values)) + "};"
        for name in ("MSVCRT__ctype", "MSVCRT__wctype")
    )
    src = root / "object.c"
    src.write_text(text)
    obj = root / "build-i386/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir/ctype.c.obj"
    obj.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            str(REPO / ".build-tools/llvm-mingw/bin/clang"),
            "--target=i686-w64-windows-gnu",
            "-c",
            str(src),
            "-o",
            str(obj),
        ],
        capture_output=True,
        check=True,
    )
    assert obj.stat().st_size > 1028
    return obj


def test_real_committed_block_matches_source_island_and_local_object():
    result = run(REPO, "--check")
    assert result.returncode == 0, result.stderr


def test_source_only_proof_is_explicit_and_emit_preserves_surroundings(tmp_path):
    fixture(tmp_path)
    result = run(tmp_path)
    assert result.returncode == 0, result.stderr
    assert "NOT proven" in result.stderr
    assert "514/514 values proven" not in result.stderr
    target = tmp_path / "target.rs"
    target.write_text(
        "// prefix\n// <gen_ctype_tables BEGIN>\nstale\n// <gen_ctype_tables END>\n// tail\n"
    )
    assert run(tmp_path, "--emit", target).returncode == 0
    text = target.read_text()
    assert text.startswith("// prefix\n") and text.endswith("\n// tail\n")
    assert run(tmp_path, "--check", target).returncode == 0
    assert run(tmp_path, "--emit", "-").stdout == result.stdout


def test_compiled_object_proof_rejects_a_single_wrong_entry(tmp_path):
    fixture(tmp_path)
    compile_object(tmp_path)
    result = run(tmp_path)
    assert result.returncode == 0, result.stderr
    assert "514/514 values proven" in result.stderr
    compile_object(tmp_path, wrong=True)
    failed = run(tmp_path)
    assert failed.returncode != 0
    assert "do NOT ship" in failed.stderr
    assert failed.stdout == ""


@pytest.mark.parametrize("bad", ["unresolved", "0x1ffff", "/* unterminated"])
def test_bad_source_cannot_replace_output(tmp_path, bad):
    source, _ = fixture(tmp_path)
    source.write_text(source.read_text().replace("_S_|_C_", bad, 1))
    output = tmp_path / "keep.rs"
    output.write_text("keep")
    assert run(tmp_path, "--emit", output).returncode != 0
    assert output.read_text() == "keep"


def test_unrecognized_or_short_island_fails(tmp_path):
    _, island = fixture(tmp_path)
    island.write_text("// missing masks")
    result = run(tmp_path)
    assert result.returncode != 0 and "_UPPER" in result.stderr


def test_object_from_another_checkout_is_refused(tmp_path):
    root = tmp_path / "own"
    root.mkdir()
    fixture(root)
    obj = compile_object(root)
    other = tmp_path / "outside.obj"
    obj.replace(other)
    obj.symlink_to(other)
    result = run(root)
    assert result.returncode != 0
    assert "outside checkout" in result.stderr
