"""Shipping immutable consumer dependency edges through Cargo, CMake and Ninja.

Only the helper's archive production is fake (a small host C archive). The
consumer module, Python watch selection and depfile retargeting are shipping
code. Negative controls deliberately suppress source edges and prove staleness.
"""

import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

import pytest

from tools import artifact_inputs

REPO = Path(artifact_inputs.__file__).resolve().parents[1]


@pytest.mark.parametrize("watch_mode", ["full", "roots", "suppressed"])
def test_shipping_store_consumer_nested_edits_and_membership_reach_ninja(tmp_path, watch_mode):
    for command in ("cargo", "rustc", "cmake", "ninja", "cc", "ar"):
        assert shutil.which(command), f"required integration-test tool missing: {command}"
    root = tmp_path / "source with spaces"
    root.mkdir()
    immutable = tmp_path / "immutable archives"
    immutable.mkdir()
    compiled = False
    initial_stamp = time.time_ns() - 3_600_000_000_000

    def write(relative, text):
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        stamp = initial_stamp
        if compiled:
            stamp = max(time.time_ns(), *(p.stat().st_mtime_ns for p in root.rglob("*")))
            stamp += 1_000_000
        os.utime(path, ns=(stamp, stamp))

    workspace = "sdk/lib/rust"
    write(
        f"{workspace}/Cargo.toml",
        '[workspace]\nmembers=["outer","beep-rust","shared","unrelated"]\nresolver="2"\n',
    )
    write("rust-toolchain.toml", '[toolchain]\nchannel="1.92.0"\n')
    for name in ("outer", "beep-rust", "shared", "unrelated"):
        library = '\n[lib]\ncrate-type=["staticlib"]\n' if name == "beep-rust" else ""
        write(
            f"{workspace}/{name}/Cargo.toml",
            f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2021"\n{library}',
        )
    write(f"{workspace}/shared/src/lib.rs", "pub mod nested;\n")
    write(f"{workspace}/shared/src/nested/mod.rs", "pub const VALUE: u32 = 1;\n")
    write(f"{workspace}/beep-rust/src/lib.rs", "pub mod deep;\n")
    write(f"{workspace}/beep-rust/src/deep/mod.rs", "pub const VALUE: u32 = 3;\n")
    write(f"{workspace}/beep-rust/assets/data.txt", "2")
    write(f"{workspace}/unrelated/src/lib.rs", "pub fn unrelated() {}\n")
    write(
        f"{workspace}/outer/src/main.rs",
        'extern "C" { fn value() -> u32; } fn main(){println!("{}",unsafe{value()});}\n',
    )
    write(
        f"{workspace}/outer/build.rs",
        f"""
#[path={json.dumps(str(REPO / "sdk/lib/rust/reactos-link/src/artifact_store.rs"))}]
mod artifact_store;
use std::path::PathBuf;
fn main() {{
    let root = PathBuf::from({json.dumps(str(root))});
    let archive = artifact_store::get(&root, "outer", "beep-rust", "amd64",
        "x86_64-pc-windows-gnu", &[], true, false, false, "", false, &root).unwrap();
    println!("cargo:rustc-link-arg={{}}", archive.display());
    println!("cargo:rerun-if-changed=build.rs");
}}
""",
    )
    write("tools/__init__.py", "")
    for name in (
        "artifact_config",
        "artifact_inputs",
        "artifact_executor",
        "artifact_worker",
        "artifact_store",
    ):
        write(f"tools/{name}.py", "# fixture implementation watch\n")
    # The shipping Rust module launches this fake helper through its actual
    # command/protocol. It computes fresh watches every time, including new
    # nested source membership, using shipping runtime._watches.
    write(
        "tools/artifact_runtime.py",
        f"""
import hashlib
import importlib.util
import json
import os
import re
import subprocess
from pathlib import Path
root=Path({str(root)!r})
immutable=Path({str(immutable)!r})
mode={watch_mode!r}
spec=importlib.util.spec_from_file_location(
    "shipping_runtime", {str(REPO / "tools/artifact_runtime.py")!r})
runtime=importlib.util.module_from_spec(spec)
spec.loader.exec_module(runtime)
packages={{}}
reads={{"sdk/lib/rust/Cargo.toml":{{}}, "sdk/lib/rust/Cargo.lock":{{}}}}
total=0
for name in ("beep-rust", "shared"):
    relative="sdk/lib/rust/"+name
    package=root/relative
    directories=[""]
    for path in sorted(package.rglob("*")):
        if path.is_dir():
            directories.append(path.relative_to(package).as_posix())
        else:
            reads[path.relative_to(root).as_posix()]={{}}
            if path.suffix==".rs":
                total+=sum(map(int,re.findall(r"VALUE: u32 = (\\d+)",path.read_text())))
            elif path.name=="data.txt":
                total+=int(path.read_text())
    packages[relative]={{"inventory":{{"directories":directories}}}}
watches=runtime._watches(root,{{"provenance":{{"reads":reads}},"descriptor":{{"packages":packages}}}})
if mode=="roots":
    watches=[str(root/name) for name in packages]
elif mode=="suppressed":
    watches=[str(root/"sdk/lib/rust/outer/build.rs")]
payload="unsigned value(void){{return "+str(total)+";}}\\n"
identity=hashlib.sha256(payload.encode()).hexdigest()
archive=immutable/(identity+".a")
if not archive.exists():
    scratch=immutable/(identity+"-compile")
    scratch.mkdir()
    (scratch/"body.c").write_text(payload)
    subprocess.run(["cc","-c",str(scratch/"body.c"),"-o",str(scratch/"body.o")],check=True)
    subprocess.run(["ar","rcs",str(archive),str(scratch/"body.o")],check=True)
    archive.chmod(0o444)
with (root/"helper-invocations.jsonl").open("a") as stream:
    stream.write(json.dumps({{"value":total,"archive":str(archive),"watches":watches}})+"\\n")
print("archive\\t"+str(archive))
for path in watches:print("watch\\t"+path)
""",
    )
    write(
        "CMakeLists.txt",
        f'''
cmake_minimum_required(VERSION 3.20)
project(artifact_consumer_deps NONE)
cmake_policy(SET CMP0116 NEW)
set(binary "${{CMAKE_BINARY_DIR}}/cargo/debug/outer")
add_custom_command(OUTPUT "${{CMAKE_BINARY_DIR}}/app"
COMMAND cargo build --offline -p outer
  --manifest-path "${{CMAKE_SOURCE_DIR}}/sdk/lib/rust/Cargo.toml"
  --target-dir "${{CMAKE_BINARY_DIR}}/cargo"
COMMAND "${{CMAKE_COMMAND}}" -E copy "${{binary}}" "${{CMAKE_BINARY_DIR}}/app"
COMMAND "{sys.executable}" "{REPO}/tools/retarget_cargo_depfile.py"
  --source "${{binary}}.d" --cargo-output "${{binary}}"
  --target "${{CMAKE_BINARY_DIR}}/app" --output "${{CMAKE_BINARY_DIR}}/app.d"
DEPFILE "${{CMAKE_BINARY_DIR}}/app.d"
DEPENDS "${{CMAKE_SOURCE_DIR}}/sdk/lib/rust/outer/build.rs"
WORKING_DIRECTORY "${{CMAKE_SOURCE_DIR}}" VERBATIM)
add_custom_target(app_target ALL DEPENDS "${{CMAKE_BINARY_DIR}}/app")
''',
    )
    environment = dict(
        os.environ,
        RUSTUP_TOOLCHAIN="1.92.0",
        CARGO_BUILD_JOBS="1",
        VIBEOS_ARTIFACT_STORE_MODE="ON",
        VIBEOS_ARTIFACT_STORE_ROOT=str(immutable),
        VIBEOS_ARTIFACT_SUPERVISOR=str(tmp_path / "fake-supervisor.json"),
        VIBEOS_ARTIFACT_PYTHON=sys.executable,
        VIBEOS_ARTIFACT_TOOLCHAIN_ID="fixture",
        VIBEOS_ARTIFACT_VERIFY_NONCE="",
    )
    for key in (
        "CARGO_BUILD_TARGET",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_BUILD_RUSTFLAGS",
        "VIBEOS_EMIT_LLVM_IR",
        "RUSTC_WRAPPER",
        "CARGO_BUILD_RUSTC_WRAPPER",
        "CARGO_TARGET_DIR",
    ):
        environment.pop(key, None)

    def run(*command):
        completed = subprocess.run(
            command, cwd=root, env=environment, capture_output=True, text=True, timeout=90
        )
        assert completed.returncode == 0, completed.stdout + completed.stderr
        return completed.stdout

    def invocations():
        return [
            json.loads(line)
            for line in (root / "helper-invocations.jsonl").read_text().splitlines()
        ]

    def build(value):
        run("cmake", "--build", "build")
        assert run(str(root / "build/app")).strip() == str(value)

    run("cmake", "-S", ".", "-B", "build", "-G", "Ninja")
    build(6)
    compiled = True
    initial = invocations()
    assert len(initial) == 1
    first_archive = Path(initial[0]["archive"])
    assert first_archive.is_file() and first_archive.stat().st_mode & 0o222 == 0
    assert "body-storage" not in str(first_archive)
    write(f"{workspace}/unrelated/src/lib.rs", "pub fn unrelated() -> u32 {999}\n")
    build(6)
    assert invocations() == initial
    write(f"{workspace}/shared/src/nested/mod.rs", "pub const VALUE: u32 = 10;\n")
    if watch_mode != "full":
        build(6)
        assert invocations() == initial  # Proven missing edge, despite newer nested source.
        return
    build(15)
    assert len(invocations()) == 2
    write(f"{workspace}/beep-rust/assets/data.txt", "20")
    build(33)
    assert len(invocations()) == 3
    write(f"{workspace}/beep-rust/src/deep/new_value.rs", "pub const VALUE: u32 = 100;\n")
    build(133)
    assert len(invocations()) == 4
    assert first_archive.is_file()  # New content selected; old immutable bytes preserved.
    manifest = root / f"{workspace}/Cargo.toml"
    write(f"{workspace}/Cargo.toml", manifest.read_text() + "\n# unrelated formatting\n")
    build(133)
    assert len(invocations()) == 5
    before = len(invocations())
    build(133)
    assert len(invocations()) == before
