"""Native compile recipes and PE layouts retain frozen Python bytes."""

import json
import struct
import subprocess
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]


def run(*args):
    return subprocess.run(
        [str(REPO / "tools/codegen"), *map(str, args)],
        capture_output=True,
        text=True,
        timeout=180,
        check=False,
    )


def pe(path, size, names):
    raw = bytearray(0x178 + 40 * len(names))
    raw[:2] = b"MZ"
    struct.pack_into("<I", raw, 0x3C, 0x80)
    raw[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<H", raw, 0x86, len(names))
    struct.pack_into("<H", raw, 0x94, 0xE0)
    struct.pack_into("<H", raw, 0x98, 0x10B)
    struct.pack_into("<I", raw, 0xD0, size)
    for i, name in enumerate(names):
        raw[0x178 + 40 * i : 0x178 + 40 * i + len(name)] = name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(raw)


def layout(root, variant):
    sections = {
        "rossym": [b".text", b".rossym"],
        "msvc": [b".text"],
        "dwarf": [b"/4", b"/8", b"/12", b"/16"],
        "wide": [b".text"],
    }[variant]
    for name, size in [
        ("ntdll.dll", 0x181000),
        ("user32.dll", 0x30000),
        ("kernel32.dll", 0x5000000 if variant == "msvc" else 0x80000),
        ("abc.acm", 0x34000),
        ("winspool.drv", 0x40000),
        ("zz.drv", 0x12000),
        ("other.dll", 0x20000),
        ("api-ms-win-test.dll", 0x10000),
        ("kbdus.dll", 0x10000),
    ]:
        pe(root / name, size, sections if name == "ntdll.dll" else [b".text"])


@pytest.mark.parametrize("variant", ["rossym", "msvc", "dwarf", "wide"])
def test_baseaddress_matches_python_layout_and_missing_priority_diagnostics(tmp_path, variant):
    source = tmp_path / "inputs"
    layout(source, variant)
    oracle = json.loads((REPO / "tools/vibeos-codegen/tests/build/baseaddress.json").read_text())[
        variant
    ]
    args = ["baseaddress", source, *(["-64"] if variant == "wide" else [])]
    result = run(*args, "--stdout")
    assert result.returncode == 0, result.stderr
    assert result.stdout == oracle["content"].replace("{INPUT}", str(source))
    assert result.stderr == oracle["warnings"]
    output = tmp_path / "outputs"
    result = run(*args, "--output-dir", output)
    assert result.returncode == 0, result.stderr
    dest = output / oracle["target"]
    assert dest.read_text() == oracle["content"].replace("{INPUT}", str(source))
    assert run(*args, "--output-dir", output, "--check").returncode == 0
    dest.write_text("stale")
    assert run(*args, "--output-dir", output, "--check").returncode == 1
    assert dest.read_text() == "stale"


def test_baseaddress_bad_input_preserves_destination(tmp_path):
    source = tmp_path / "inputs"
    layout(source, "msvc")
    output = tmp_path / "keep.cmake"
    output.write_text("keep")
    (source / "other.dll").write_bytes(b"MZ")
    result = run("baseaddress", source, "--output", output)
    assert result.returncode != 0 and "truncated PE" in result.stderr
    assert output.read_text() == "keep"
    (source / "other.dll").unlink()
    (source / "user32.dll").unlink()
    result = run("baseaddress", source, "--output", output)
    assert result.returncode != 0 and "reserved module" in result.stderr
    assert output.read_text() == "keep"


def database(root):
    db = root / "build-i386/compile_commands.json"
    db.parent.mkdir(parents=True)
    db.write_text(
        json.dumps(
            [
                {
                    "file": "/tmp/vibeos/sdk/include/asm/genincdata.c",
                    "command": (
                        "clang -DTEST=1 -I/tmp/vibeos/sdk/include "
                        "-I/tmp/vibeos/build-i386/sdk/include "
                        "-o sdk/CMakeFiles/genincdata.dir/genincdata.c.obj "
                        "-c /tmp/vibeos/sdk/include/asm/genincdata.c"
                    ),
                }
            ]
        )
    )
    return db


def test_cflags_json_and_native_output_modes(tmp_path):
    database(tmp_path)
    args = ["--root", tmp_path, "cflags", "--arch", "i386"]
    expected = "-DTEST=1\n-I{REPO}/sdk/include\n-c\n"
    result = run(*args, "--stdout")
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout) == expected
    output = tmp_path / "flags"
    assert run(*args, "--output", output).returncode == 0
    assert output.read_text() == expected
    assert run(*args, "--output", output, "--check").returncode == 0
    output.write_text("stale")
    assert run(*args, "--output", output, "--check").returncode == 1
    assert output.read_text() == "stale"


def test_cflags_missing_input_is_null_only_for_json_inspection(tmp_path):
    args = ["--root", tmp_path, "cflags", "--arch", "i386"]
    result = run(*args, "--stdout")
    assert result.returncode == 0 and json.loads(result.stdout) is None
    assert run(*args).returncode != 0
    assert not list(tmp_path.iterdir())


def test_cflags_rejects_foreign_database_and_conflicting_recipes(tmp_path):
    db = database(tmp_path)
    foreign = tmp_path.parent / (tmp_path.name + "-foreign.json")
    db.replace(foreign)
    db.symlink_to(foreign)
    result = run("--root", tmp_path, "cflags", "--arch", "i386", "--stdout")
    assert result.returncode != 0 and "outside checkout" in result.stderr
    db.unlink()
    rows = json.loads(foreign.read_text())
    rows.append(
        {
            **rows[0],
            "file": "other.c",
            "command": rows[0]["command"].replace("-DTEST=1", "-DTEST=2"),
        }
    )
    db.write_text(json.dumps(rows))
    result = run("--root", tmp_path, "cflags", "--arch", "i386", "--stdout")
    assert result.returncode != 0 and "single-flag-set invariant" in result.stderr


def test_legacy_entry_points_delegate_to_native_generator(tmp_path):
    database(tmp_path)
    result = subprocess.run(
        [sys.executable, str(REPO / "tools/gen_cflags.py"), "--arch", "i386", "--stdout"],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        timeout=180,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    native = run("--root", tmp_path, "cflags", "--arch", "i386", "--stdout")
    assert json.loads(result.stdout) == json.loads(native.stdout)
    source = tmp_path / "pe"
    layout(source, "msvc")
    output = tmp_path / "base.cmake"
    result = subprocess.run(
        [
            sys.executable,
            str(REPO / "sdk/tools/gen_baseaddress.py"),
            str(source),
            "--output",
            str(output),
        ],
        capture_output=True,
        text=True,
        timeout=180,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    assert output.read_text() == run("baseaddress", source, "--stdout").stdout


def test_cflags_all_inputs_render_before_any_write(tmp_path):
    database(tmp_path)
    output = tmp_path / "sdk/lib/rust/reactos-link/cflags/genincdata.i386.flags"
    output.parent.mkdir(parents=True)
    output.write_text("keep")
    result = run("--root", tmp_path, "cflags", "--arch", "i386", "--arch", "amd64")
    assert result.returncode != 0 and "no compile database" in result.stderr
    assert output.read_text() == "keep"
