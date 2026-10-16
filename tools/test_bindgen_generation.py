"""Shipping generators + real bindgen/postprocessing, without installing Rust."""

import concurrent.futures
import os
import shutil
import subprocess
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
CRATE = Path("sdk/lib/rust/ntkrnl-sys")


def fixture_repo(root):
    root.mkdir()
    for name in (
        CRATE / "generate_bindings.sh",
        CRATE / "generate_hal_bindings.sh",
        CRATE / "postprocess_bindgen.py",
        Path("tools/bindgen-stage.sh"),
        Path("tools/rustconv/normalize_island_abi.py"),
        Path("tools/rustconv/widen_island_constants.py"),
    ):
        dest = root / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(REPO / name, dest)
    for name in (
        "ntoskrnl/mm/ARM3/miarm.h",
        "sdk/include/ntddkbd.h",
        "sdk/include/ntddmou.h",
        "sdk/include/reactos/x86x64/Cpuid.h",
    ):
        dest = root / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text("/* fixture */\n")
    (root / "ntoskrnl/ntoskrnl.h").write_text('#include "nested.h"\n')
    (root / "ntoskrnl/ntoskrnl.spec").write_text("@ extern FixtureGlobal\n")
    (root / CRATE / "wrapper_hal.h").write_text('#include "nested.h"\n')
    (root / ".build-tools/llvm-mingw/lib/clang/19/include").mkdir(parents=True)
    for arch in ("arm64", "amd64", "i386"):
        for header in ("ddk/wdm.h", "psdk/ntdef.h"):
            path = root / f"build-{arch}/sdk/include" / header
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("/* fixture generated header */\n")
    live = root / CRATE / "src"
    live.mkdir()
    (live / "bindgen_i386.rs").write_text('pub type Live = unsafe extern "stdcall" fn();\n')
    (live / "bindgen_amd64.rs").write_text('pub type Live = unsafe extern "C" fn();\n')
    write_header(root, 1)
    return root


def write_header(root, width):
    (root / "sdk/include/nested.h").write_text(
        "typedef void (__attribute__((stdcall)) *FixtureCallback)(unsigned int);\n"
        f"typedef struct {{ unsigned char data[{width}]; unsigned int flags:3; "
        "FixtureCallback callback; } FixturePayload;\n"
        "extern unsigned int FixtureGlobal;\n"
    )


def generate(root, kind, output=None, env=None):
    script = "generate_bindings.sh" if kind == "kernel" else "generate_hal_bindings.sh"
    args = ["bash", str(root / CRATE / script)]
    if output is not None:
        args += ["--output-dir", str(output)]
    return subprocess.run(
        args, cwd=root.parent, env=env, capture_output=True, text=True, timeout=90
    )


def snapshot(path):
    return {p.name: p.read_bytes() for p in path.glob("*.rs")}


def compiled_payload_size(stage):
    source = stage / "client.rs"
    source.write_text(
        '#[path = "src/bindgen_common.rs"] mod bindgen_common;\n'
        '#[path = "src/bindgen_amd64.rs"] mod bindings;\n'
        'fn main() { println!("{}", std::mem::size_of::<bindings::FixturePayload>()); }\n'
    )
    binary = stage / "client"
    subprocess.run(
        ["rustc", "--edition=2021", str(source), "-o", str(binary)],
        capture_output=True,
        text=True,
        check=True,
        timeout=90,
    )
    return int(subprocess.check_output([str(binary)], text=True))


@pytest.mark.parametrize("kind", ["kernel", "hal"])
def test_real_generation_isolated_concurrent_and_header_sensitive(tmp_path, kind):
    root = fixture_repo(tmp_path / "repo with spaces")
    live = snapshot(root / CRATE / "src")
    outputs = [tmp_path / "run one", tmp_path / "run two"]
    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        results = list(pool.map(lambda output: generate(root, kind, output), outputs))
    for result in results:
        assert result.returncode == 0, result.stdout + result.stderr
    assert snapshot(outputs[0] / "src") == snapshot(outputs[1] / "src")
    stem = "bindgen" if kind == "kernel" else "hal_bindgen"
    arches = ("arm64", "amd64", "i386") if kind == "kernel" else ("amd64", "i386")
    for output in outputs:
        assert (output / "COMPLETE").is_file()
        assert (output / "src/bindgen_common.rs").is_file()
        for arch in arches:
            final = (output / f"src/{stem}_{arch}.rs").read_text()
            assert 'extern "system"' in final
            assert "data: [" in final and "1usize]" in final
            assert "__BindgenBitfieldUnit" in final
            assert "pub struct __BindgenBitfieldUnit" not in final
            assert (
                "pub struct __BindgenBitfieldUnit" in (output / "src/bindgen_common.rs").read_text()
            )
            if kind == "hal":
                assert 'kind = "raw-dylib"' in final
            deps = (output / f"raw/{stem}_{arch}.d").read_text()
            assert "nested.h" in deps
            assert str(outputs[1 - outputs.index(output)]).replace(" ", "\\ ") not in deps
    write_header(root, 8)
    changed = tmp_path / "changed"
    result = generate(root, kind, changed)
    assert result.returncode == 0, result.stdout + result.stderr
    for arch in arches:
        assert "8usize]" in (changed / f"src/{stem}_{arch}.rs").read_text()
    if kind == "kernel":
        # Execute a host consumer of the real postprocessed amd64 fixture island.
        # HAL raw-dylib imports require a Windows target and aren't host-run here.
        assert compiled_payload_size(changed) > compiled_payload_size(outputs[0])
    assert snapshot(root / CRATE / "src") == live


@pytest.mark.parametrize("failure", ["bindgen", "normalizer", "missing-spec"])
def test_failure_never_completes_or_changes_live_bindings(tmp_path, failure):
    root = fixture_repo(tmp_path / "repo")
    live = snapshot(root / CRATE / "src")
    env = dict(os.environ)
    if failure == "bindgen":
        binaries = tmp_path / "bin"
        binaries.mkdir()
        fake = binaries / "bindgen"
        fake.write_text("#!/bin/sh\nexit 13\n")
        fake.chmod(0o755)
        env["PATH"] = f"{binaries}:{env['PATH']}"
    elif failure == "normalizer":
        (root / "tools/rustconv/normalize_island_abi.py").write_text("raise SystemExit(13)\n")
    else:
        (root / "ntoskrnl/ntoskrnl.spec").unlink()
    output = tmp_path / "failed"
    result = generate(root, "hal" if failure == "missing-spec" else "kernel", output, env)
    assert result.returncode == (1 if failure == "missing-spec" else 13), (
        result.stdout + result.stderr
    )
    if failure == "missing-spec":
        assert "required kernel data-export spec not found" in result.stderr
    assert not (output / "COMPLETE").exists()
    assert snapshot(root / CRATE / "src") == live


@pytest.mark.parametrize(
    "bad", ["existing-output", "foreign-build", "missing-header", "foreign-output", "linked-header"]
)
def test_refuses_unsafe_inputs_and_outputs(tmp_path, bad):
    root = fixture_repo(tmp_path / "repo")
    output = tmp_path / "stage"
    if bad == "existing-output":
        output.mkdir()
        (output / "sentinel").write_text("keep me")
    elif bad == "foreign-build":
        build = root / "build-arm64"
        build.rename(tmp_path / "foreign")
        build.symlink_to(tmp_path / "foreign", target_is_directory=True)
    elif bad == "missing-header":
        (root / "build-amd64/sdk/include/ddk/wdm.h").unlink()
    elif bad == "linked-header":
        header = root / "build-amd64/sdk/include/ddk/wdm.h"
        header.rename(tmp_path / "foreign.h")
        header.symlink_to(tmp_path / "foreign.h")
    else:
        foreign = tmp_path / "foreign"
        foreign.mkdir()
        subprocess.run(["git", "init", "-q", str(foreign)], check=True)
        output = foreign / "stage"
    result = generate(root, "kernel", output)
    assert result.returncode != 0
    assert not (output / "COMPLETE").exists()
    assert not (output / "raw").exists()
    if bad == "existing-output":
        assert (output / "sentinel").read_text() == "keep me"


def test_default_runs_are_private_and_foreign_staging_root_is_rejected(tmp_path):
    root = fixture_repo(tmp_path / "repo")
    for _ in range(2):
        result = generate(root, "kernel")
        assert result.returncode == 0, result.stdout + result.stderr
    runs = root / ".build-tools/bindgen-runs"
    assert len(list(runs.glob("kernel.*/COMPLETE"))) == 2
    runs.rename(tmp_path / "retained-runs")
    foreign = tmp_path / "foreign"
    foreign.mkdir()
    runs.symlink_to(foreign, target_is_directory=True)
    result = generate(root, "kernel")
    assert result.returncode != 0
    assert not list(foreign.iterdir())
