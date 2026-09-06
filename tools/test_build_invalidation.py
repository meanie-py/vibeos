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
    def __init__(
        self, root, *, track_assets=True, emitted_deps=False, depfile_policy="OLD", track_lock=False
    ):
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
        lock_input = '"${CMAKE_SOURCE_DIR}/Cargo.lock"' if track_lock else ""
        if track_lock:
            self.run("cargo", "generate-lockfile", "--offline")
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
        DEPENDS ${{inputs}} ${{assets}} "${{CMAKE_SOURCE_DIR}}/Cargo.toml" {lock_input}
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
        self.clock_step_ns = 0
        self._mark_clock()

    # A backwards CLOCK_REALTIME step between writing an input and stamping the
    # output built from it makes the output look OLDER than its input, so Ninja
    # re-runs an edge that is genuinely up to date. That is a host fault, not a
    # missing dependency edge: a real invalidation bug fails on EVERY run, while
    # this fails only when the clock happens to step, and no dependency edge can
    # make the ordering observable again. Detect it instead of hiding it behind
    # a sleep or a retry -- CLOCK_MONOTONIC carries the same NTP slew as
    # CLOCK_REALTIME, so realtime advancing less than monotonic isolates a step
    # and nothing else -- though only a STEP: a pure frequency error moves both
    # clocks together and slips past. Seen on WSL2 when systemd-timesyncd and the
    # kernel slew fell into a feedback loop -- timesyncd steps the clock back, the
    # kernel slews harder, the slew saturates at tick 11000 (nominal 10000) where
    # it can never converge, so timesyncd steps again, ~2.4s every 32s. Fixed
    # host-side by disabling systemd-timesyncd; the guard stays, because it
    # defends against a step from any cause.
    # See docs/engineering/translation-debugging-field-manual.md.
    def _mark_clock(self):
        self._clock_mark = (time.time_ns(), time.monotonic_ns())

    def _check_clock(self):
        real, mono = self._clock_mark
        lost = (time.monotonic_ns() - mono) - (time.time_ns() - real)
        if lost > 50_000_000:
            self.clock_step_ns = max(self.clock_step_ns, lost)

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
        if self.configured:
            # The ordering this stamp establishes is only observable while the
            # clock does not step backwards before the outputs are stamped.
            self._mark_clock()

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
        self._check_clock()
        if self.clock_step_ns:
            pytest.skip(
                "host CLOCK_REALTIME stepped backwards "
                f"{self.clock_step_ns / 1e9:.3f}s during this build, so an output "
                "can be stamped older than the input it was built from and "
                "timestamp ordering is unobservable; fix the host clock "
                "(see docs/engineering/translation-debugging-field-manual.md)"
            )
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


@pytest.mark.parametrize("kind", ["c", "S"])
@pytest.mark.parametrize("change", ["source", "header"])
@pytest.mark.parametrize("mode", ["source-only", "source-depfile", "closure-depfile"])
def test_native_inputs_reach_linked_rust_executable(tmp_path, kind, change, mode):
    """Compile real C/preprocessed assembly, not a simulated native-input reader.

    The assembly exports data, avoiding architecture-specific instructions. The
    fixture intentionally owns its compiler calls; it does not certify cc-rs or
    reactos-link's production header discovery.
    """
    fixture = BuildFixture(tmp_path, track_assets=False, emitted_deps=mode != "source-only")
    source = (
        '#include "value.h"\nunsigned int native_value = VALUE + OFFSET;\n'
        if kind == "c"
        else '#include "value.h"\n.data\n.balign 4\n.globl native_value\n'
        'native_value:\n.long VALUE + OFFSET\n.section .note.GNU-stack,"",@progbits\n'
    )
    fixture.write("leaf/native/value.h", "#define VALUE 1\n")
    fixture.write(f"leaf/native/value.{kind}", source.replace("OFFSET", "0"))
    fixture.write(
        "leaf/build.rs",
        """use std::{env, path::PathBuf, process::Command};
fn main() {
    println!("cargo:rerun-if-changed=native/value.KIND");
    WATCH_HEADER
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let object = out.join("native.o");
    let archive = out.join("libnative.a");
    assert!(Command::new("cc").args(["-c", "native/value.KIND", "-o"])
        .arg(&object).status().unwrap().success());
    assert!(Command::new("ar").arg("crs").arg(&archive).arg(&object)
        .status().unwrap().success());
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=native");
}
""".replace("KIND", kind).replace(
            "WATCH_HEADER",
            'println!("cargo:rerun-if-changed=native/value.h");'
            if mode != "source-depfile"
            else "",
        ),
    )
    fixture.write(
        "leaf/src/main.rs",
        'extern "C" { static native_value: u32; }\n'
        'fn main() { println!("{}", unsafe { native_value }); }\n',
    )
    initial = fixture.compile()
    assert initial["leaf"] == "1\n"
    before = fixture.invocations()
    assert fixture.compile() == initial
    assert fixture.invocations() == before, fixture.build_explanation
    if change == "source":
        fixture.write(f"leaf/native/value.{kind}", source.replace("OFFSET", "1"))
    else:
        fixture.write("leaf/native/value.h", "#define VALUE 2\n")
    warm = fixture.compile()
    missing_edge = mode == "source-only" or (mode == "source-depfile" and change == "header")
    if missing_edge:
        assert fixture.invocations() == before
        assert warm == initial
        # Force only the outer layer: a missing Cargo header watch remains stale
        # even when Ninja has been made to invoke Cargo again.
        if mode == "source-depfile":
            (fixture.build / "leaf").unlink()
            assert fixture.compile() == initial
            assert fixture.invocations()[len(before) :] == ["leaf"]
    else:
        assert fixture.invocations()[len(before) :] == ["leaf"]
        assert warm["leaf"] == "2\n"
        assert warm["peer"] == initial["peer"]
        assert warm["other"] == initial["other"]
    cold = fixture.cold()
    assert cold["leaf"] == "2\n"
    assert (warm != cold) if missing_edge else (warm == cold)


@pytest.mark.parametrize("emitted_deps", [False, True])
def test_workspace_inherited_version_reaches_final_executables(tmp_path, emitted_deps):
    fixture = BuildFixture(tmp_path, emitted_deps=emitted_deps)
    workspace = (tmp_path / "Cargo.toml").read_text()
    fixture.write("Cargo.toml", workspace + '\n[workspace.package]\nversion = "0.1.0"\n')
    for name in ("leaf", "peer", "other"):
        manifest = f"{name}/Cargo.toml"
        fixture.write(
            manifest,
            (tmp_path / manifest)
            .read_text()
            .replace('version = "0.1.0"', "version.workspace = true"),
        )
        fixture.write(
            f"{name}/src/main.rs",
            fixture.main_source(name, 'env!("CARGO_PKG_VERSION_MINOR").parse::<u32>().unwrap()'),
        )
    initial = fixture.compile()
    before = fixture.invocations()
    fixture.write("Cargo.toml", workspace + '\n[workspace.package]\nversion = "0.2.0"\n')
    warm = fixture.compile()
    assert set(fixture.invocations()[len(before) :]) == {"leaf", "peer", "other"}
    assert all(warm[name] != initial[name] for name in initial)
    assert warm == fixture.cold()


@pytest.mark.parametrize("track_lock", [False, True])
def test_lockfile_only_git_pin_change_requires_explicit_edge(tmp_path, track_lock):
    """Two local Git revisions provide a real resolution change without network."""
    fixture = BuildFixture(tmp_path, emitted_deps=True, track_lock=track_lock)
    dependency = tmp_path / "git-dependency"
    fixture.write(
        "git-dependency/Cargo.toml",
        '[package]\nname = "pinned"\nversion = "0.1.0"\nedition = "2021"\n',
    )
    fixture.run("git", "init", str(dependency))
    revisions = []
    for value in (1, 2):
        fixture.write("git-dependency/src/lib.rs", f"pub fn value() -> u32 {{ {value} }}\n")
        fixture.run("git", "-C", str(dependency), "add", "Cargo.toml", "src/lib.rs")
        fixture.run(
            "git",
            "-C",
            str(dependency),
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-m",
            f"value {value}",
        )
        revisions.append(fixture.run("git", "-C", str(dependency), "rev-parse", "HEAD").strip())
    # Keep Cargo's Git cache fixture-local; production/shared caches are untouched.
    fixture.env["CARGO_HOME"] = str(tmp_path / "cargo-home")
    manifest = (tmp_path / "leaf/Cargo.toml").read_text()
    fixture.write("leaf/Cargo.toml", manifest + f'pinned = {{ git = "{dependency.as_uri()}" }}\n')
    fixture.write("leaf/src/main.rs", fixture.main_source("leaf", "pinned::value()"))
    # Fetch only file://, then every compile and pin update is offline.
    fixture.run("cargo", "generate-lockfile")
    fixture.run("cargo", "update", "--offline", "-p", "pinned", "--precise", revisions[0])
    initial = fixture.compile()
    before = fixture.invocations()
    assert fixture.compile() == initial
    assert fixture.invocations() == before
    fixture.run("cargo", "update", "--offline", "-p", "pinned", "--precise", revisions[1])
    fixture.write("Cargo.lock", (tmp_path / "Cargo.lock").read_text())
    warm = fixture.compile()
    if track_lock:
        assert set(fixture.invocations()[len(before) :]) == {"leaf", "peer", "other"}
        assert warm["leaf"] != initial["leaf"]
        assert warm["peer"] == initial["peer"]
        assert warm["other"] == initial["other"]
        assert warm == fixture.cold()
    else:
        assert fixture.invocations() == before
        assert warm == initial
        assert warm != fixture.cold(), "Cargo depfiles alone must expose the missing lockfile edge"


@pytest.mark.parametrize("mode", ["source-only", "directory-depfile", "files-depfile"])
def test_generated_rust_tracks_directory_input_add_edit_remove(tmp_path, mode):
    """Cargo directory watches are recursive; Ninja depfile directory edges are not."""
    emitted_deps = mode != "source-only"
    fixture = BuildFixture(tmp_path, track_assets=False, emitted_deps=emitted_deps)
    fixture.write("leaf/inputs/one.txt", "1")
    fixture.write(
        "leaf/build.rs",
        """fn main() {
    println!("cargo:rerun-if-changed=inputs");
    let value: u32 = std::fs::read_dir("inputs").unwrap().map(|entry| {
        let path = entry.unwrap().path();
        WATCH_FILE
        std::fs::read_to_string(path).unwrap().trim().parse::<u32>().unwrap()
    }).sum();
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    std::fs::write(out.join("generated.rs"), format!("const VALUE: u32 = {value};")).unwrap();
}
""".replace(
            "WATCH_FILE",
            'println!("cargo:rerun-if-changed={}", path.display());'
            if mode == "files-depfile"
            else "",
        ),
    )
    fixture.write(
        "leaf/src/main.rs",
        'include!(concat!(env!("OUT_DIR"), "/generated.rs"));\n'
        'fn main() { println!("{VALUE}"); }\n',
    )
    initial = fixture.compile()
    assert initial["leaf"] == "1\n"
    before = fixture.invocations()
    assert fixture.compile() == initial
    assert fixture.invocations() == before, fixture.build_explanation
    for operation, expected in (("add", "3\n"), ("edit", "5\n"), ("remove", "4\n")):
        before = fixture.invocations()
        if operation == "remove":
            (tmp_path / "leaf/inputs/one.txt").unlink()
        else:
            fixture.write("leaf/inputs/two.txt", "2" if operation == "add" else "4")
        warm = fixture.compile()
        if mode == "files-depfile" or (mode == "directory-depfile" and operation != "edit"):
            assert fixture.invocations()[len(before) :] == ["leaf"], operation
            assert warm["leaf"] == expected
            assert warm == fixture.cold()
        else:
            assert fixture.invocations() == before, "negative control must miss the input edge"
            cold = fixture.cold()
            assert warm != cold
            assert cold["leaf"] == expected
