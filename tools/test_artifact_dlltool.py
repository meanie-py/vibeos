"""Actual raw-dylib import recipe under the worker's minimal filesystem root."""

import json
import os
import subprocess
from pathlib import Path

import pytest

from tools import artifact_executor
from tools.artifact_executor import PIN, WORKSPACE, launch, provision

REPO = artifact_executor.IMPLEMENTATION_REPO


@pytest.fixture(scope="module")
def provisioned_toolchain():
    root = Path.home() / ".rustup/toolchains" / PIN
    return root, provision(root)


@pytest.mark.parametrize("missing", [None, "llvm-runtime", "dlltool"])
def test_raw_dylib_requires_declared_llvm_dlltool_and_runtime(
    tmp_path, provisioned_toolchain, missing
):
    toolchain, recipe = provisioned_toolchain
    native_tools = dict(recipe["native_tools"])
    expected = {
        "/native-tools/bin/llvm-dlltool",
        "/native-tools/lib/libLLVM.so.19.1",
    }
    assert set(native_tools) == expected
    for canonical in expected:
        assert canonical in recipe["mounted_inputs"]
    if missing == "llvm-runtime":
        del native_tools["/native-tools/lib/libLLVM.so.19.1"]
    elif missing == "dlltool":
        del native_tools["/native-tools/bin/llvm-dlltool"]
    config = {
        "repo": str(REPO),
        "toolchain": str(toolchain),
        "runtime_libraries": recipe["runtime_libraries"],
        "native_tools": native_tools,
    }
    for name in ("root", "projection", "target", "cargo-home", "home", "scratch"):
        path = tmp_path / name
        path.mkdir()
        config[name] = str(path)
    workspace = Path(config["projection"]) / WORKSPACE
    workspace.mkdir(parents=True)
    (workspace / "Cargo.toml").write_text(
        '[package]\nname="raw-dylib-fixture"\nversion="0.1.0"\nedition="2021"\n'
        '[lib]\npath="input.rs"\ncrate-type=["rlib"]\n'
    )
    (workspace / "Cargo.lock").write_text(
        'version = 4\n[[package]]\nname="raw-dylib-fixture"\nversion="0.1.0"\n'
    )
    (workspace / "input.rs").write_text(
        '#![no_std]\n#[link(name="fixture.dll",kind="raw-dylib",modifiers="+verbatim")]\n'
        'extern "system" { fn fixture_native(value: u64) -> u64; }\n'
        '#[no_mangle]\npub unsafe extern "system" fn fixture_calls_native(value: u64)->u64 '
        "{ fixture_native(value) }\n"
    )
    config["argv"] = [
        "/toolchain/bin/cargo",
        "build",
        "--offline",
        "--locked",
        "--jobs",
        "1",
        "--target",
        "x86_64-pc-windows-gnu",
        "--manifest-path",
        "/tmp/vibeos/sdk/lib/rust/Cargo.toml",
    ]
    config["env"] = {
        "PATH": "/native-tools/bin:/toolchain/bin",
        "HOME": "/home",
        "CARGO_HOME": "/cargo-home",
        "CARGO_TARGET_DIR": "/target",
        "TMPDIR": "/scratch",
        "RUSTC": "/toolchain/bin/rustc",
        "CARGO_INCREMENTAL": "0",
        "LC_ALL": "C",
        "RUSTFLAGS": "-C dlltool=/native-tools/bin/llvm-dlltool",
    }
    config_path = tmp_path / "worker.json"
    config_path.write_text(json.dumps(config))
    environment = dict(os.environ)
    environment.pop("VIBEOS_HOST_JOB_HELD", None)
    broker = REPO / ".cache/host-jobs/host-jobs"
    assert broker.is_file(), f"host-jobs must be provisioned before this fixture: {broker}"
    command = [str(broker), "--kind", "compile", "--", *launch(config_path)]
    result = subprocess.run(command, env=environment, capture_output=True, text=True, check=False)
    (tmp_path / "compiler.log").write_text(result.stdout + result.stderr)
    output = tmp_path / "target/x86_64-pc-windows-gnu/debug/libraw_dylib_fixture.rlib"
    if missing is None:
        assert result.returncode == 0, result.stderr
        assert output.is_file() and output.read_bytes().startswith(b"!<arch>\n")
    else:
        assert result.returncode != 0, result.stderr
        assert not output.exists()
        assert "dlltool" in result.stderr


def test_raw_import_staticlibs_reproduce_and_link_function_and_data(
    tmp_path, provisioned_toolchain
):
    """Two real cold workers compare complete archives, without normalization."""
    toolchain, recipe = provisioned_toolchain
    archives = []
    for index in range(2):
        output = tmp_path / ("independent-worker-" + str(index))
        output.mkdir()
        config = {
            "repo": str(REPO),
            "toolchain": str(toolchain),
            "runtime_libraries": recipe["runtime_libraries"],
            "native_tools": recipe["native_tools"],
        }
        for name in ("root", "projection", "target", "cargo-home", "home", "scratch"):
            (output / name).mkdir()
            config[name] = str(output / name)
        workspace = output / "projection" / WORKSPACE
        workspace.mkdir(parents=True)
        (workspace / "Cargo.toml").write_text(
            '[package]\nname="raw-fixture"\nversion="0.1.0"\nedition="2021"\n'
            '[lib]\npath="input.rs"\ncrate-type=["staticlib"]\n'
            '[profile.release]\npanic="abort"\ndebug=1\ncodegen-units=1\n'
        )
        (workspace / "Cargo.lock").write_text(
            'version=4\n[[package]]\nname="raw-fixture"\nversion="0.1.0"\n'
        )
        (workspace / "input.rs").write_text(
            '#![no_std]\n#[link(name="fixture.dll",kind="raw-dylib",modifiers="+verbatim")]\n'
            'extern "system" { fn FixtureFunction(value:u64)->u64; static mut FixtureData:u64; }\n'
            '#[no_mangle]\npub unsafe extern "system" fn fixture_entry()->u64 {'
            "FixtureFunction(core::ptr::read_volatile(core::ptr::addr_of!(FixtureData))) }\n"
            "#[panic_handler]\nfn panic(_: &core::panic::PanicInfo)->! { loop {} }\n"
        )
        context = artifact_executor.request_context(toolchain, recipe, "raw-fixture")
        config["argv"] = context["effective_config"]["argv"]
        config["env"] = context["environment"]
        assert "-C dlltool=/native-tools/bin/llvm-dlltool" in config["env"]["RUSTFLAGS"]
        config_path = output / "worker.json"
        config_path.write_text(json.dumps(config))
        environment = dict(os.environ)
        environment.pop("VIBEOS_HOST_JOB_HELD", None)
        broker = REPO / ".cache/host-jobs/host-jobs"
        result = subprocess.run(
            [str(broker), "--kind", "compile", "--", *launch(config_path)],
            env=environment,
            capture_output=True,
            text=True,
            check=False,
        )
        (output / "compiler.log").write_text(result.stdout + result.stderr)
        assert result.returncode == 0, result.stderr
        archive = output / "target/x86_64-pc-windows-gnu/release/libraw_fixture.a"
        payload = archive.read_bytes()
        assert payload.startswith(b"!<arch>\n")
        archives.append(payload)
        llvm = REPO / ".build-tools/llvm-mingw/bin"
        pe = output / "fixture.exe"
        linked = subprocess.run(
            [
                str(broker),
                "--kind",
                "link",
                "--",
                str(llvm / "lld-link"),
                "/entry:fixture_entry",
                "/subsystem:console",
                "/nodefaultlib",
                "/machine:x64",
                str(archive),
                "/out:" + str(pe),
            ],
            env=environment,
            capture_output=True,
            text=True,
            check=False,
        )
        assert linked.returncode == 0, linked.stderr
        imports = subprocess.run(
            [str(llvm / "llvm-readobj"), "--coff-imports", str(pe)],
            capture_output=True,
            text=True,
            check=True,
        ).stdout
        assert "Name: fixture.dll" in imports
        assert "FixtureFunction" in imports and "FixtureData" in imports
    assert archives[0] == archives[1], "independent compiler import generation must reproduce"
