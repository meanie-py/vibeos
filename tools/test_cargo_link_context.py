"""Real Cargo build scripts must overlap through the shipping Ninja pool."""

import os
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]


def test_ninja_cargo_links_do_not_serialize_on_the_output_lock(tmp_path):
    (tmp_path / "Cargo.toml").write_text('[workspace]\nmembers = ["left", "right"]\n')
    for name, peer in (("left", "right"), ("right", "left")):
        crate = tmp_path / name
        (crate / "src").mkdir(parents=True)
        (crate / "Cargo.toml").write_text(
            f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2021"\n'
        )
        (crate / "src/lib.rs").write_text("pub fn value() -> u32 { 42 }\n")
        (crate / "build.rs").write_text(
            "use std::{env, fs, path::PathBuf, thread, time::{Duration, Instant}};\n"
            'fn main() { let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());\n'
            f'fs::write(root.join("../{name}.ready"), "ready").unwrap();\n'
            "let start = Instant::now();\n"
            f'while !root.join("../{peer}.ready").exists() {{\n'
            'assert!(start.elapsed().as_secs() < 15, "Cargo output lock serialized peers");\n'
            "thread::sleep(Duration::from_millis(10)); } }\n"
        )
    (tmp_path / "CMakeLists.txt").write_text(
        "cmake_minimum_required(VERSION 3.17)\nproject(cargo_parallel NONE)\n"
        f'include("{REPO}/sdk/cmake/cargo_link_context.cmake")\n'
        "foreach(name left right)\n"
        'reactos_cargo_link_context("${CMAKE_SOURCE_DIR}" "${name}" host output)\n'
        "add_custom_target(${name} ALL\n"
        'COMMAND cargo build --offline --manifest-path "${CMAKE_SOURCE_DIR}/Cargo.toml" '
        '--target-dir "${output}" -p ${name}\n'
        "JOB_POOL vibeos_cargo_links VERBATIM)\nendforeach()\n"
    )
    env = dict(os.environ, VIBEOS_CARGO_LINK_JOBS="2", CARGO_BUILD_JOBS="1", RUSTC_WRAPPER="")
    for key in ("CARGO_BUILD_TARGET", "CARGO_ENCODED_RUSTFLAGS", "RUSTFLAGS"):
        env.pop(key, None)
    for cmd in (
        ["cmake", "-S", ".", "-B", "build", "-G", "Ninja"],
        ["cmake", "--build", "build", "--parallel", "2"],
    ):
        cp = subprocess.run(cmd, cwd=tmp_path, env=env, text=True, capture_output=True, timeout=60)
        assert cp.returncode == 0, cp.stdout + cp.stderr
    assert (tmp_path / "left.ready").is_file()
    assert (tmp_path / "right.ready").is_file()
