"""Native binding generators: parser bridge, semantic checks, and build dependencies."""

import subprocess
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(REPO / "tools"))

COM = """pub struct IFooVtbl {
    pub RunW: ::core::option::Option<unsafe extern "C" fn(
        This: *mut IFoo, cb: Option<unsafe extern "C" fn(i32) -> HRESULT>) -> HRESULT>,
}"""


def run(*args, input=None):
    return subprocess.run(
        [str(REPO / "tools/codegen"), *map(str, args)],
        input=input,
        text=True,
        capture_output=True,
        check=False,
    )


def test_com_python_api_preserves_tuple_shapes_and_nested_callbacks():
    from gen_commacros import parse_vtbls

    assert parse_vtbls(COM) == {
        "IFoo": [("RunW", [("cb", 'Option<unsafe extern "C" fn(i32) -> HRESULT>')], "HRESULT")]
    }
    assert parse_vtbls("pub struct Nothing {}") == {}
    with pytest.raises(ValueError, match="native COM parser failed"):
        parse_vtbls("pub struct IFooVtbl {")
    result = run("com-macros", "--bindgen", "-", "--stdout", input=COM)
    assert result.returncode == 0, result.stderr
    assert "IFoo_Run(" in result.stdout and "IFoo_RunW(" in result.stdout
    assert run("com-macros", "--bindgen", "-", "--only", "INope", input=COM).returncode != 0


def test_enum_computed_values_check_and_no_vacuous_success(tmp_path):
    header = tmp_path / "enum.h"
    header.write_text("enum Example { First=1, Second, Mask=(1 << 4), Alias=Mask|Second };")
    args = ["enum-consts", header, "Example"]
    result = run(*args)
    assert result.returncode == 0, result.stderr
    assert "pub const Alias: u32 = 18;" in result.stdout
    existing = tmp_path / "consts.rs"
    existing.write_text("pub const Alias: u32 = 0x12;")
    assert run(*args, "--check", existing).returncode == 0
    for text in ("pub const Alias: u32 = 19;", "pub const Alias: u32 = mystery;", "// none"):
        existing.write_text(text)
        assert run(*args, "--check", existing).returncode != 0
    assert run(*args, "--only", "Missing").returncode != 0


def test_ioctl_sdk_resolution_handles_non_utf8_headers(tmp_path):
    headers = tmp_path / "sdk/include"
    headers.mkdir(parents=True)
    (headers / "base.h").write_bytes(
        b"//\xff\n#define DEVICE 0x22\n#define CTL(d,n) ((d << 16) | (n << 2))\n"
    )
    header = headers / "device.h"
    header.write_text("#define IOCTL_TEST CTL(DEVICE, 3)\n#define IOCTL_NEGATIVE -1\n")
    result = run("--root", tmp_path, "ioctl-consts", "sdk/include/device.h", "--prefix", "IOCTL_")
    assert result.returncode == 0, result.stderr
    assert "pub const IOCTL_TEST: u32 = 0x0022000c;" in result.stdout
    assert "pub const IOCTL_NEGATIVE: u32 = -0x0000001;" in result.stdout


def test_rpc_cmake_tracks_native_templates_and_reruns(tmp_path):
    # Use the real build function with a tiny isolated generator source tree.
    source = tmp_path / "source"
    (source / "tools/vibeos-codegen/src").mkdir(parents=True)
    stamp = source / "tools/vibeos-codegen/src/template.txt"
    stamp.write_text("first")
    for rel in (
        "rust-toolchain.toml",
        "tools/vibeos-codegen/Cargo.toml",
        "tools/vibeos-codegen/Cargo.lock",
    ):
        (source / rel).write_text("")
    launcher = source / "tools/codegen"
    launcher.write_text('#!/bin/sh\nexec "' + str(REPO / "tools/codegen") + '" "$@"\n')
    launcher.chmod(0o755)
    (tmp_path / "CMakeLists.txt").write_text(
        "cmake_minimum_required(VERSION 3.20)\nproject(binding NONE)\n"
        f'set(REACTOS_SOURCE_DIR "{source}")\ninclude("{REPO}/sdk/cmake/widl-support.cmake")\n'
        "add_rpc_guards(x prelude.h)\nadd_custom_target(generate ALL DEPENDS ${x_GUARD_SOURCE})\n"
    )
    build = tmp_path / "build"
    subprocess.run(
        ["cmake", "-G", "Ninja", "-S", str(tmp_path), "-B", str(build)],
        capture_output=True,
        check=True,
    )
    (build / "x_c.h").write_text("DWORD __cdecl GetVersion(void);")
    subprocess.run(["cmake", "--build", str(build)], capture_output=True, check=True)
    # Changed contents force generator work, even though unchanged output mtimes stay stable.
    stamp.write_text("second")
    rebuilt = subprocess.run(
        ["cmake", "--build", str(build)], capture_output=True, text=True, check=True
    )
    assert "Generating x_guard.c, x_guard.h" in rebuilt.stdout
    unchanged = subprocess.run(
        ["cmake", "--build", str(build)], capture_output=True, text=True, check=True
    )
    assert "Generating x_guard.c, x_guard.h" not in unchanged.stdout
    assert "GetVersion_guarded" in (build / "x_guard.h").read_text()


def test_crt_prototypes_local_object_symbols_and_failures(tmp_path):
    fixtures = REPO / "tools/vibeos-codegen/tests/bindings"
    source = tmp_path / "dll/win32/msvcrt/fixture.c"
    source.parent.mkdir(parents=True)
    source.write_bytes((fixtures / "fixture.c").read_bytes())
    obj = tmp_path / "build-amd64/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir/fixture.c.obj"
    obj.parent.mkdir(parents=True)
    obj.write_bytes(b"fixture object")
    nm = tmp_path / ".build-tools/llvm-mingw/bin/llvm-nm"
    nm.parent.mkdir(parents=True)
    symbols = fixtures / "fixture_symbols.txt"
    nm.write_text(f'#!/bin/sh\ncat "{symbols}"\n')
    nm.chmod(0o755)
    args = ["--root", tmp_path, "ref-protos", "fixture.c", "--output-dir", tmp_path]
    result = run(*args)
    assert result.returncode == 0, result.stderr
    for suffix in ("protos.h", "no_rename.txt"):
        assert (tmp_path / f"msvcrt_fixture_{suffix}").read_bytes() == (
            fixtures / f"fixture_{suffix}"
        ).read_bytes()
    assert run(*args, "--check").returncode == 0
    nm.write_text("#!/bin/sh\nexit 0\n")
    assert "no defined text symbols" in run(*args).stderr
    nm.write_text("#!/bin/sh\nexit 1\n")
    assert run(*args).returncode != 0
    obj.unlink()
    assert "not built" in run(*args).stderr
