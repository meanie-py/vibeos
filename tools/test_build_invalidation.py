"""Real CMake -> Ninja -> Cargo invalidation controls, in isolated host fixtures.

These are not kernel builds. Exercise the shipping source-closure helper and
compare executed warm/cold outputs, not merely compiler logs or timestamps.
"""

import os
import subprocess
import sys
import time
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]


class BuildFixture:
    def __init__(self, root, *, track_assets=True, emitted_deps=False, depfile_policy="OLD"):
        self.root = root
        self.build = root / "build"
        self.configured = False
        self.initial_mtime_ns = time.time_ns() - 3_600_000_000_000
        self.env = {
            **os.environ,
            "RUSTC_WRAPPER": "",
            "CARGO_BUILD_RUSTC_WRAPPER": "",
            "CARGO_INCREMENTAL": "0",
        }
        # Host fixtures must not inherit kernel flags or a caller's target.
        for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_TARGET"):
            self.env.pop(key, None)
        self.write(
            "Cargo.toml",
            '[workspace]\nmembers = ["leaf", "peer", "other", "shared"]\nresolver = "2"\n',
        )
        self.write(
            "shared/Cargo.toml", '[package]\nname = "shared"\nversion = "0.1.0"\nedition = "2021"\n'
        )
        self.write("shared/src/lib.rs", "pub fn value() -> u32 { 1 }\n")
        for name in ("leaf", "peer", "other"):
            deps = '\n[dependencies]\nshared = { path = "../shared" }\n' if name != "other" else ""
            self.write(
                f"{name}/Cargo.toml",
                f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2021"\n{deps}',
            )
            self.write(f"{name}/src/main.rs", self.main_source(name, 1))
        self.write("leaf/payload.txt", "payload-one")
        self.write("leaf/schema.txt", "schema-one")
        self.write(
            "leaf/build.rs",
            """fn main() {
    println!("cargo:rerun-if-changed=schema.txt");
    println!("cargo:rustc-env=SCHEMA={}", std::fs::read_to_string("schema.txt").unwrap());
}
""",
        )
        self.write("record.cmake", 'file(APPEND "${LOG}" "${NAME}\\n")\n')
        helper = REPO / "sdk/cmake/rust-crate-deps.cmake"
        extra = (
            "${CMAKE_SOURCE_DIR}/leaf/payload.txt ${CMAKE_SOURCE_DIR}/leaf/schema.txt"
            if track_assets
            else ""
        )
        depfile_commands = ""
        depfile_option = ""
        policy = ""
        if emitted_deps:
            target = "${name}"
            if depfile_policy == "NEW":
                target = "${CMAKE_BINARY_DIR}/${name}"
                policy = "cmake_policy(SET CMP0116 NEW)"
            depfile_commands = f'''
        COMMAND "{sys.executable}" "{REPO}/tools/retarget_cargo_depfile.py"
            --source "${{CMAKE_BINARY_DIR}}/cargo/debug/${{name}}.d"
            --cargo-output "${{CMAKE_BINARY_DIR}}/cargo/debug/${{name}}"
            --target "{target}"
            --output "${{CMAKE_BINARY_DIR}}/${{name}}.d"
'''
            depfile_option = 'DEPFILE "${CMAKE_BINARY_DIR}/${name}.d"'
        self.write(
            "CMakeLists.txt",
            f'''cmake_minimum_required(VERSION 3.16)
project(invalidation NONE)
{policy}
include("{helper}")
foreach(name leaf peer other)
    rust_crate_sources(inputs "${{CMAKE_SOURCE_DIR}}" "${{name}}")
    set(assets)
    if(name STREQUAL "leaf")
        set(assets {extra})
    endif()
    add_custom_command(OUTPUT "${{CMAKE_BINARY_DIR}}/${{name}}"
        COMMAND cargo build --offline --manifest-path "${{CMAKE_SOURCE_DIR}}/Cargo.toml"
            --target-dir "${{CMAKE_BINARY_DIR}}/cargo" -p "${{name}}"
        COMMAND ${{CMAKE_COMMAND}} -E copy "${{CMAKE_BINARY_DIR}}/cargo/debug/${{name}}"
            "${{CMAKE_BINARY_DIR}}/${{name}}"
        COMMAND ${{CMAKE_COMMAND}} -E touch "${{CMAKE_BINARY_DIR}}/${{name}}"
        {depfile_commands}
        COMMAND ${{CMAKE_COMMAND}} -DLOG=${{CMAKE_BINARY_DIR}}/invocations
            -DNAME=${{name}} -P "${{CMAKE_SOURCE_DIR}}/record.cmake"
        DEPENDS ${{inputs}} ${{assets}} "${{CMAKE_SOURCE_DIR}}/Cargo.toml"
        {depfile_option}
        VERBATIM)
    add_custom_target(${{name}}_target ALL DEPENDS "${{CMAKE_BINARY_DIR}}/${{name}}")
endforeach()
''',
        )
        self.run("cmake", "-S", str(root), "-B", str(self.build), "-G", "Ninja")
        # Configure itself can straddle a host-clock adjustment: CMakeSystem.cmake
        # then appears newer than each newly regenerated build.ninja, causing
        # Ninja's 100-regeneration failure before any dependency edge is tested.
        # Establish the same initial epoch for generated configure state as for
        # fixture sources. Leave regeneration enabled and all dependency edges
        # intact; later mutations still use the strict ordering in write().
        for path in self.build.rglob("*"):
            if path.is_file():
                os.utime(path, ns=(self.initial_mtime_ns, self.initial_mtime_ns))
        self.configured = True

    def write(self, path, text):
        dest = self.root / path
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(text)
        # Test timestamp ordering, not the host clock. A WSL run produced copied
        # outputs OLDER than initial source files, causing a spurious no-op
        # failure. Initial inputs are safely in the past;
        # mutations are explicitly newer than every prior fixture artifact. This
        # does not add missing Ninja dependency edges (the negative controls must
        # still fail to rebuild). No sleeps or retries hide invalidation failures.
        mtime_ns = self.initial_mtime_ns
        if self.configured:
            mtime_ns = (
                max(
                    time.time_ns(),
                    max(p.stat().st_mtime_ns for p in self.root.rglob("*") if p.is_file()),
                )
                + 1_000_000
            )
        os.utime(dest, ns=(mtime_ns, mtime_ns))

    @staticmethod
    def main_source(name, value):
        shared = "shared::value()" if name != "other" else "0"
        extra = ', include_str!("../payload.txt"), env!("SCHEMA")' if name == "leaf" else ""
        pattern = "{}:{}:{}:{}" if name == "leaf" else "{}:{}"
        return f'fn main() {{ println!("{pattern}", {value}, {shared}{extra}); }}\n'

    def run(self, *cmd):
        cp = subprocess.run(
            cmd, cwd=self.root, env=self.env, capture_output=True, text=True, timeout=90
        )
        assert cp.returncode == 0, cp.stdout + cp.stderr
        self.last_stderr = cp.stderr
        return cp.stdout

    def outputs(self):
        return {name: self.run(str(self.build / name)) for name in ("leaf", "peer", "other")}

    def compile(self):
        self.run("ninja", "-C", str(self.build), "-j1", "-d", "explain")
        self.build_explanation = self.last_stderr
        return self.outputs()

    def invocations(self):
        return (self.build / "invocations").read_text().splitlines()

    def cold(self):
        self.run("cargo", "clean", "--target-dir", str(self.build / "cargo"))
        self.run("ninja", "-C", str(self.build), "-t", "clean")
        return self.compile()


def test_initial_configure_metadata_cannot_be_in_the_future(tmp_path, monkeypatch):
    original_run = BuildFixture.run
    injected = []

    def configure_with_clock_step(self, *cmd):
        result = original_run(self, *cmd)
        if cmd[0] == "cmake":
            system = next(self.build.glob("CMakeFiles/*/CMakeSystem.cmake"))
            future = time.time_ns() + 86_400_000_000_000
            os.utime(system, ns=(future, future))
            injected.append(system)
        return result

    monkeypatch.setattr(BuildFixture, "run", configure_with_clock_step)
    fixture = BuildFixture(tmp_path)
    assert injected
    assert injected[0].stat().st_mtime_ns == fixture.initial_mtime_ns
    initial = fixture.compile()
    before = fixture.invocations()
    assert fixture.compile() == initial
    assert fixture.invocations() == before, fixture.build_explanation


@pytest.mark.parametrize(
    "change,affected",
    [
        ("leaf", {"leaf"}),
        ("shared", {"leaf", "peer"}),
        ("other", {"other"}),
        ("asset", {"leaf"}),
        ("schema", {"leaf"}),
    ],
)
def test_warm_outputs_equal_cold_and_only_consumers_run(tmp_path, change, affected):
    fixture = BuildFixture(tmp_path)
    initial = fixture.compile()
    before = fixture.invocations()
    assert fixture.compile() == initial
    assert fixture.invocations() == before, (
        "unchanged build must not invoke Cargo; mtimes_ns="
        + repr(
            {
                str(path.relative_to(tmp_path)): path.stat().st_mtime_ns
                for path in [
                    tmp_path / "Cargo.toml",
                    tmp_path / "leaf/build.rs",
                    tmp_path / "peer/src/main.rs",
                    tmp_path / "other/src/main.rs",
                    fixture.build / "leaf",
                    fixture.build / "peer",
                    fixture.build / "other",
                ]
            }
        )
    )
    if change == "shared":
        fixture.write("shared/src/lib.rs", "pub fn value() -> u32 { 2 }\n")
    elif change == "asset":
        fixture.write("leaf/payload.txt", "payload-two")
    elif change == "schema":
        fixture.write("leaf/schema.txt", "schema-two")
    else:
        fixture.write(f"{change}/src/main.rs", fixture.main_source(change, 2))
    warm = fixture.compile()
    assert set(fixture.invocations()[len(before) :]) == affected
    assert {name for name in initial if initial[name] != warm[name]} == affected
    assert warm == fixture.cold()


@pytest.mark.parametrize("asset", ["payload.txt", "schema.txt"])
def test_oracle_detects_missing_non_rust_dependency(tmp_path, asset):
    """Negative control: source closure alone currently misses these real inputs.

    Keep production's conservative edges until the replacement passes this
    oracle without explicit asset dependencies. This test must expose staleness.
    """
    fixture = BuildFixture(tmp_path, track_assets=False)
    initial = fixture.compile()
    fixture.write(f"leaf/{asset}", "changed")
    warm = fixture.compile()
    assert warm == initial, "control must demonstrate the incomplete edge"
    assert warm != fixture.cold(), "oracle must catch the stale final executable"


def test_added_and_removed_shared_module_reaches_final_executables(tmp_path):
    fixture = BuildFixture(tmp_path)
    initial = fixture.compile()
    fixture.write("shared/src/added.rs", "pub fn value() -> u32 { 3 }\n")
    fixture.write("shared/src/lib.rs", "mod added;\npub use added::value;\n")
    before = fixture.invocations()
    warm = fixture.compile()
    assert set(fixture.invocations()[len(before) :]) == {"leaf", "peer"}
    assert warm != initial
    assert warm == fixture.cold()

    fixture.write("shared/src/lib.rs", "pub fn value() -> u32 { 1 }\n")
    (tmp_path / "shared/src/added.rs").unlink()
    before = fixture.invocations()
    warm = fixture.compile()
    assert set(fixture.invocations()[len(before) :]) == {"leaf", "peer"}
    assert warm == initial == fixture.cold()


def test_build_script_edit_reaches_final_executable(tmp_path):
    fixture = BuildFixture(tmp_path)
    initial = fixture.compile()
    before = fixture.invocations()
    fixture.write("leaf/build.rs", 'fn main() { println!("cargo:rustc-env=SCHEMA=build-two"); }\n')
    warm = fixture.compile()
    assert fixture.invocations()[len(before) :] == ["leaf"]
    assert warm["leaf"] != initial["leaf"]
    assert warm == fixture.cold()


@pytest.mark.parametrize("crate,affected", [("leaf", {"leaf"}), ("shared", {"leaf", "peer"})])
@pytest.mark.parametrize("emitted_deps", [False, True])
def test_manifest_edit_reaches_only_consumers(tmp_path, crate, affected, emitted_deps):
    fixture = BuildFixture(tmp_path, emitted_deps=emitted_deps)
    version = 'env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap()'
    if crate == "shared":
        fixture.write("shared/src/lib.rs", f"pub fn value() -> u32 {{ {version} }}\n")
    else:
        fixture.write("leaf/src/main.rs", fixture.main_source("leaf", version))
    initial = fixture.compile()
    before = fixture.invocations()
    manifest = f"{crate}/Cargo.toml"
    fixture.write(manifest, (tmp_path / manifest).read_text().replace('"0.1.0"', '"0.2.0"'))
    warm = fixture.compile()
    assert set(fixture.invocations()[len(before) :]) == affected
    assert {name for name in initial if initial[name] != warm[name]} == affected
    assert warm == fixture.cold()


@pytest.mark.parametrize("asset", ["payload.txt", "schema.txt"])
@pytest.mark.parametrize("policy", ["OLD", "NEW"])
def test_cargo_emitted_dependencies_close_non_rust_input_gap(tmp_path, asset, policy):
    fixture = BuildFixture(
        tmp_path / "with spaces", track_assets=False, emitted_deps=True, depfile_policy=policy
    )
    initial = fixture.compile()
    before = fixture.invocations()
    assert fixture.compile() == initial
    assert fixture.invocations() == before, fixture.build_explanation
    fixture.write(f"leaf/{asset}", "changed")
    warm = fixture.compile()
    assert fixture.invocations()[len(before) :] == ["leaf"]
    assert warm != initial
    assert warm == fixture.cold()
