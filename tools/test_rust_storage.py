"""Exercise shared Cargo feature isolation and CMake's artifact ownership guard."""

import os
import subprocess
from pathlib import Path

from migrate_rust_storage import migrate

ROOT = Path(__file__).resolve().parents[1]


def test_migration_preserves_outer_and_multi_package_builds():
    text = """set(OUT "${CMAKE_CURRENT_BINARY_DIR}/rust-target")
add_custom_command(
 COMMAND ${CMAKE_COMMAND} -E env CARGO_TARGET_DIR=${OUT}
 cargo build -p widget-rust --release
 VERBATIM)
add_custom_target(widget)
"""
    changed, crates = migrate(text)
    assert crates == ["widget-rust"]
    assert (
        'rust_storage_target(OUT "${CMAKE_CURRENT_BINARY_DIR}/rust-target" widget-rust)' in changed
    )
    assert migrate(changed) == (changed, [])
    for extra in (
        text.replace("widget-rust", "widget-cargo"),
        text.replace("--release", "-p second --release"),
    ):
        assert migrate(extra) == (extra, [])


def test_shared_directory_keeps_feature_variants_and_invalidates_sources(tmp_path):
    # The selected packages deliberately need opposite features of one dependency.
    # Run shipping directory selection and real Cargo, then execute both results.
    (tmp_path / "Cargo.toml").write_text(
        '[workspace]\nmembers=["dep", "left", "right"]\nresolver="2"\n'
    )
    for name in ("dep", "left", "right"):
        crate = tmp_path / name
        (crate / "src").mkdir(parents=True)
        manifest = f'[package]\nname="{name}"\nversion="0.1.0"\nedition="2021"\n'
        if name == "dep":
            manifest += "[features]\nflip=[]\n"
            (crate / "src/lib.rs").write_text(
                'pub fn value() -> u8 { if cfg!(feature="flip") { 7 } else { 3 } }\n'
            )
        else:
            manifest += (
                '[dependencies]\ndep={path="../dep"'
                + (',features=["flip"]' if name == "left" else "")
                + "}\n"
            )
            (crate / "src/main.rs").write_text('fn main() { println!("{}", dep::value()); }\n')
        (crate / "Cargo.toml").write_text(manifest)
    (tmp_path / "CMakeLists.txt").write_text(
        "cmake_minimum_required(VERSION 3.17)\nproject(storage NONE)\n"
        'set(VIBEOS_RUST_STORAGE_SHARDS 1 CACHE STRING "")\n'
        f'include("{ROOT}/sdk/cmake/rust-storage.cmake")\n'
        "foreach(name left right)\n"
        'rust_storage_target(output "${CMAKE_BINARY_DIR}/${name}-target" ${name})\n'
        'file(WRITE "${CMAKE_BINARY_DIR}/${name}.path" "${output}")\nendforeach()\n'
    )
    subprocess.run(
        ["cmake", "-S", str(tmp_path), "-B", str(tmp_path / "build")],
        check=True,
        capture_output=True,
    )
    target = (tmp_path / "build/left.path").read_text()
    assert target == (tmp_path / "build/right.path").read_text()
    assert Path(target).is_relative_to(tmp_path / "build")
    env = dict(os.environ, RUSTC_WRAPPER="")
    for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_TARGET"):
        env.pop(key, None)

    def run(name):
        subprocess.run(
            ["cargo", "build", "--offline", "-p", name, "--target-dir", target],
            cwd=tmp_path,
            env=env,
            check=True,
            capture_output=True,
            timeout=60,
        )
        return subprocess.check_output([str(Path(target) / "debug" / name)], text=True).strip()

    assert run("left") == "7"
    assert run("right") == "3"
    assert run("left") == "7"
    (tmp_path / "dep/src/lib.rs").write_text(
        'pub fn value() -> u8 { if cfg!(feature="flip") { 9 } else { 5 } }\n'
    )
    assert run("right") == "5"
    assert run("left") == "9"
    with (tmp_path / "CMakeLists.txt").open("a") as stream:
        stream.write('rust_storage_target(duplicate "${CMAKE_BINARY_DIR}/another-target" left)\n')
    cp = subprocess.run(
        ["cmake", "-S", str(tmp_path), "-B", str(tmp_path / "build")],
        capture_output=True,
        text=True,
    )
    assert cp.returncode != 0
    assert "multiple owners" in cp.stderr


def test_report_counts_inodes_once_and_does_not_follow_symlinks(tmp_path):
    from rust_storage_report import measure

    (tmp_path / "one.rmeta").write_bytes(b"metadata")
    os.link(tmp_path / "one.rmeta", tmp_path / "hardlink.rmeta")
    (tmp_path / "two.rmeta").write_bytes(b"metadata")
    foreign = tmp_path.parent / "foreign.rmeta"
    foreign.write_bytes(b"foreign")
    (tmp_path / "symlink.rmeta").symlink_to(foreign)
    result = measure(tmp_path)
    assert result["duplicates"][".rmeta"] == {
        "total_bytes": 16,
        "duplicate_bytes": 8,
        "files": 2,
        "distinct": 1,
    }
