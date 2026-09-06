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
        self.compiled = False
        self.initial_mtime_ns = time.time_ns() - 3_600_000_000_000
        self.setup_mtime_ns = self.initial_mtime_ns
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
        self.write(
            "complete.py",
            """import os
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
target = Path(sys.argv[1])
# Only an executed build edge calls this. Preserve the data and dependency
# graph, but give its output the timestamp ordering a monotonic clock would.
latest = max(p.stat().st_mtime_ns for p in root.rglob("*"))
os.utime(target, ns=(latest + 1_000_000, latest + 1_000_000))
""",
        )
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
        COMMAND "{sys.executable}" "${{CMAKE_SOURCE_DIR}}/complete.py"
            "${{CMAKE_BINARY_DIR}}/${{name}}"
        COMMAND ${{CMAKE_COMMAND}} -DLOG=${{CMAKE_BINARY_DIR}}/invocations
            -DNAME=${{name}} -P "${{CMAKE_SOURCE_DIR}}/record.cmake"
        DEPENDS ${{inputs}} ${{assets}} "${{CMAKE_SOURCE_DIR}}/Cargo.toml" {lock_input}
        {depfile_option}
        VERBATIM)
    add_custom_target(${{name}}_target ALL DEPENDS "${{CMAKE_BINARY_DIR}}/${{name}}")
endforeach()
''',
        )
        self.configure()

    def configure(self):
        self.run("cmake", "-S", str(self.root), "-B", str(self.build), "-G", "Ninja")
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
    # and nothing else. Seen on WSL2 with a miscalibrated TSC (`dmesg` reports
    # "Adjusting tsc more than 11%"): the kernel holds a +7.7% tick and
    # systemd-timesyncd, pinned at its 32s minimum poll, steps realtime back
    # ~2.4s every poll.
    def _mark_clock(self):
        self._clock_mark = (time.time_ns(), time.monotonic_ns())

    def _check_clock(self):
        real, mono = self._clock_mark
        lost = (time.monotonic_ns() - mono) - (time.time_ns() - real)
        if lost > 50_000_000:
            self.clock_step_ns = max(self.clock_step_ns, lost)

    def write(self, path, text):
        dest = self.root / path
        added = not dest.exists()
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(text)
        # Test timestamp ordering, not the host clock. A WSL run produced copied
        # outputs OLDER than initial source files, causing a spurious no-op
        # failure. Initial inputs are safely in the past;
        # mutations are explicitly newer than every prior fixture artifact. This
        # does not add missing Ninja dependency edges (the negative controls must
        # still fail to rebuild). No sleeps or retries hide invalidation failures.
        # Distinct setup writes still need distinct mtimes: Git's stat cache
        # otherwise misses same-size edits while constructing local revisions.
        # Use whole seconds: some Git builds compare only the seconds field.
        mtime_ns = self.next_mtime()
        os.utime(dest, ns=(mtime_ns, mtime_ns))
        if added:
            os.utime(dest.parent, ns=(mtime_ns, mtime_ns))
        if self.configured:
            self._mark_clock()

    def next_mtime(self):
        self.setup_mtime_ns += 1_000_000_000
        if not self.compiled:
            return self.setup_mtime_ns
        return (
            max(time.time_ns(), max(p.stat().st_mtime_ns for p in self.root.rglob("*"))) + 1_000_000
        )

    def remove(self, path):
        dest = self.root / path
        mtime_ns = self.next_mtime()
        dest.unlink()
        os.utime(dest.parent, ns=(mtime_ns, mtime_ns))
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
        if not self.compiled:
            # Tests customize sources/build.rs/CMake after construction. Those
            # are INITIAL inputs, not mutations of an already built artifact.
            # Keep their epoch in the past too, and explicitly reconfigure to
            # consume any CMake edits with that same initial timestamp.
            self.configure()
        self.run("ninja", "-C", str(self.build), "-j1", "-d", "explain")
        self.compiled = True
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


def test_initial_setup_edits_do_not_use_mutation_epoch(tmp_path, monkeypatch):
    fixture = BuildFixture(tmp_path)
    with monkeypatch.context() as patch:
        # A forward host-clock step during setup must not timestamp an initial
        # source newer than the compiler artifacts produced after it steps back.
        patch.setattr(time, "time_ns", lambda: fixture.initial_mtime_ns + 86_400_000_000_000)
        fixture.write("leaf/src/main.rs", fixture.main_source("leaf", 7))
    assert (
        fixture.initial_mtime_ns
        < (tmp_path / "leaf/src/main.rs").stat().st_mtime_ns
        < time.time_ns()
    )
    initial = fixture.compile()
    assert initial["leaf"].startswith("7:")
    before = fixture.invocations()
    assert fixture.compile() == initial
    assert fixture.invocations() == before
    fixture.write("leaf/src/main.rs", fixture.main_source("leaf", 8))
    assert (tmp_path / "leaf/src/main.rs").stat().st_mtime_ns > (
        fixture.build / "leaf"
    ).stat().st_mtime_ns
    warm = fixture.compile()
    assert fixture.invocations()[len(before) :] == ["leaf"]
    assert warm["leaf"].startswith("8:")
    assert warm == fixture.cold()


def test_executed_outputs_remain_newer_than_future_dated_inputs(tmp_path):
    fixture = BuildFixture(tmp_path)
    fixture.compile()
    fixture.write("leaf/src/main.rs", fixture.main_source("leaf", 9))
    source = tmp_path / "leaf/src/main.rs"
    future = time.time_ns() + 86_400_000_000_000
    os.utime(source, ns=(future, future))
    warm = fixture.compile()
    assert warm["leaf"].startswith("9:")
    assert (fixture.build / "leaf").stat().st_mtime_ns > future
    before = fixture.invocations()
    assert fixture.compile() == warm
    assert fixture.invocations() == before, fixture.build_explanation
    assert warm == fixture.cold()


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
    fixture.remove("shared/src/added.rs")
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


@pytest.mark.parametrize("policy", ["OLD", "NEW"])
@pytest.mark.parametrize("tracked", [False, True])
def test_native_hpp_runtime_include_discovery(tmp_path, policy, tracked):
    fixture = BuildFixture(tmp_path / "with spaces", emitted_deps=True, depfile_policy=policy)
    manifest = (fixture.root / "leaf/Cargo.toml").read_text()
    fixture.write(
        "leaf/Cargo.toml",
        manifest + "\n[build-dependencies]\n"
        f'reactos-link = {{ path = "{REPO}/sdk/lib/rust/reactos-link" }}\n',
    )
    # Cargo's exported .d escapes spaces but not #/$; exercise those separately
    # at the hpp protocol boundary below, not as a falsely green Ninja control.
    fragment = "nested value.h"
    fixture.write("leaf/templates/top.h", "$include(middle.h)\n")
    fixture.write("leaf/templates/middle.h", f"$include({fragment})\n")
    fixture.write(f"leaf/templates/{fragment}", "pub const VALUE: u32 = 1;\n")
    fixture.write(
        "leaf/build.rs",
        """use std::{path::{Path, PathBuf}, process::Command};
fn main() {
    println!("cargo:rerun-if-changed=templates/top.h");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let exe = out.join("hpp");
    assert!(Command::new("cc").arg("HPP_SOURCE").arg("-o").arg(&exe)
        .status().unwrap().success());
    let mut cmd = Command::new(exe);
    cmd.current_dir("templates").arg("top.h").arg(out.join("generated.rs"));
    let status = RUN;
    assert!(status.success());
}
""".replace("HPP_SOURCE", str(REPO / "sdk/tools/hpp/hpp.c")).replace(
            "RUN",
            'reactos_link::native_deps::run_hpp(&mut cmd, Path::new("templates/top.h"), '
            "|cmd| cmd.status()).unwrap()"
            if tracked
            else "cmd.status().unwrap()",
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
    assert fixture.invocations() == before
    fixture.write(f"leaf/templates/{fragment}", "pub const VALUE: u32 = 2;\n")
    warm = fixture.compile()
    assert fixture.invocations()[len(before) :] == (["leaf"] if tracked else [])
    assert warm["leaf"] == ("2\n" if tracked else "1\n")
    cold = fixture.cold()
    assert cold["leaf"] == "2\n"
    assert (warm == cold) if tracked else (warm != cold)
    if tracked:
        fixture.write("leaf/templates/replacement.h", "pub const VALUE: u32 = 3;\n")
        fixture.write("leaf/templates/middle.h", "$include(replacement.h)\n")
        assert fixture.compile()["leaf"] == "3\n"
        before = fixture.invocations()
        fixture.write("leaf/templates/replacement.h", "pub const VALUE: u32 = 4;\n")
        warm = fixture.compile()
        assert warm["leaf"] == "4\n"
        assert fixture.invocations()[len(before) :] == ["leaf"]
        before = fixture.invocations()
        assert fixture.compile() == warm
        assert fixture.invocations() == before
        assert warm == fixture.cold()


def test_native_hpp_real_templates_and_failure_reporting(tmp_path):
    exe = tmp_path / "hpp"
    subprocess.run(["cc", str(REPO / "sdk/tools/hpp/hpp.c"), "-o", str(exe)], check=True)
    templates = REPO / "sdk/include/xdk"
    depfile = tmp_path / "inputs.d"
    for name in ("winnt", "ntdef", "devioctl", "wdm", "ntddk", "ntifs"):
        plain, tracked = tmp_path / f"{name}.plain", tmp_path / f"{name}.tracked"
        subprocess.run([str(exe), f"{name}.template.h", str(plain)], cwd=templates, check=True)
        subprocess.run(
            [str(exe), f"{name}.template.h", str(tracked), "--depfile", str(depfile)],
            cwd=templates,
            check=True,
        )
        assert plain.read_bytes() == tracked.read_bytes()
        inputs = depfile.read_text().removeprefix("vibeos-native:").split()
        assert f"{name}.template.h" in inputs
        assert len(inputs) > 1
        assert all((templates / p).is_file() for p in inputs)
    source = tmp_path / "bad.h"
    special = tmp_path / "space #$fragment.h"
    special.write_text("fragment output\n")
    source.write_text(f"$include({special.name})\n")
    subprocess.run(
        [str(exe), str(source), str(tmp_path / "special.out"), "--depfile", str(depfile)],
        cwd=tmp_path,
        check=True,
    )
    assert " space\\ \\#$$fragment.h" in depfile.read_text()
    assert (tmp_path / "special.out").read_text() == "fragment output\n"
    for contents in ("$include(missing.h)\n", "$include(missing-close\n"):
        source.write_text(contents)
        depfile.write_text("vibeos-native: stale.h\n")
        result = subprocess.run(
            [str(exe), str(source), str(tmp_path / "bad.out"), "--depfile", str(depfile)],
            cwd=tmp_path,
            capture_output=True,
            text=True,
        )
        assert result.returncode != 0
        assert result.stderr
        assert not depfile.exists()
    source.write_text("valid output\n")
    result = subprocess.run(
        [str(exe), str(source), str(tmp_path / "bad.out"), "--depfile", str(tmp_path / "absent/d")],
        capture_output=True,
        text=True,
    )
    assert result.returncode != 0
    assert "Cannot open dependency file" in result.stderr


@pytest.mark.parametrize("failed_output", ["header", "depfile"])
def test_native_hpp_write_errors_fail_loudly(tmp_path, failed_output):
    # Linux's full device fails writes, including stdio's buffered fclose path.
    # Use an owned symlink so hpp's failed-depfile cleanup cannot remove it.
    assert Path("/dev/full").exists(), "host write-error control requires /dev/full"
    exe = tmp_path / "hpp"
    subprocess.run(["cc", str(REPO / "sdk/tools/hpp/hpp.c"), "-o", str(exe)], check=True)
    source, output, depfile = tmp_path / "input.h", tmp_path / "output.h", tmp_path / "input.d"
    source.write_text("valid output\n")
    (output if failed_output == "header" else depfile).symlink_to("/dev/full")
    result = subprocess.run(
        [str(exe), str(source), str(output), "--depfile", str(depfile)],
        capture_output=True,
        text=True,
    )
    assert result.returncode != 0
    assert "generation failed" in result.stderr
    assert not depfile.exists()
    assert Path("/dev/full").exists()


@pytest.mark.parametrize("cache", ["disk", "invocation"])
def test_native_generator_output_is_only_cached_within_one_invocation(tmp_path, cache):
    fixture = BuildFixture(tmp_path, emitted_deps=True)
    manifest = (tmp_path / "leaf/Cargo.toml").read_text()
    fixture.write(
        "leaf/Cargo.toml",
        manifest + "\n[build-dependencies]\n"
        f'reactos-link = {{ path = "{REPO}/sdk/lib/rust/reactos-link" }}\n',
    )
    fixture.write("leaf/native/value.h", "#define VALUE 1\n")
    fixture.write(
        "leaf/native/generator.c",
        '#include "value.h"\n#include <stdio.h>\n'
        'int main(void) { printf("pub const VALUE: u32 = %u;\\n", VALUE); }\n',
    )
    fixture.write(
        "leaf/build.rs",
        """use std::{path::{Path, PathBuf}, process::Command, io::Write};
fn main() {
    // Keep the negative control observable even though its stale-file shortcut
    // prevents the compiler from discovering any inputs on a later invocation.
    println!("cargo:rerun-if-changed=native/value.h");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let dst = out.join("generated.rs");
    let cache = reactos_link::native_deps::InvocationOutput::default();
    for _ in 0..2 {
        let generate = || {
            let exe = out.join("generator");
            let mut command = Command::new("cc");
            command.current_dir("native").args(["generator.c", "-o"]).arg(&exe);
            assert!(reactos_link::native_deps::run(&mut command,
                Path::new("native/generator.c"), |cmd| cmd.status()).unwrap().success());
            let result = Command::new(&exe).output().unwrap();
            assert!(result.status.success());
            std::fs::write(&dst, result.stdout).unwrap();
            std::fs::OpenOptions::new().create(true).append(true).open("generator-runs")
                .unwrap().write_all(b"run\\n").unwrap();
            dst.clone()
        };
        CACHE
    }
}
""".replace(
            "CACHE",
            "if !dst.exists() { generate(); }"
            if cache == "disk"
            else "cache.get_or_generate(generate);",
        ),
    )
    fixture.write(
        "leaf/src/main.rs",
        'include!(concat!(env!("OUT_DIR"), "/generated.rs"));\n'
        'fn main() { println!("{VALUE}"); }\n',
    )
    initial = fixture.compile()
    assert initial["leaf"] == "1\n"
    journal = tmp_path / "leaf/generator-runs"
    assert journal.read_text().splitlines() == ["run"]
    before = fixture.invocations()
    assert fixture.compile() == initial
    assert fixture.invocations() == before
    fixture.write("leaf/native/value.h", "#define VALUE 2\n")
    warm = fixture.compile()
    assert fixture.invocations()[len(before) :] == ["leaf"]
    if cache == "disk":
        assert warm == initial
        assert journal.read_text().splitlines() == ["run"]
    else:
        assert warm["leaf"] == "2\n"
        assert journal.read_text().splitlines() == ["run", "run"]
    cold = fixture.cold()
    assert cold["leaf"] == "2\n"
    assert (warm != cold) if cache == "disk" else (warm == cold)


@pytest.mark.parametrize("policy", ["OLD", "NEW"])
@pytest.mark.parametrize("asset", ["payload.txt", "schema.txt"])
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


@pytest.mark.parametrize("kind", ["c", "S"])
@pytest.mark.parametrize("replay", [False, True])
def test_production_native_discovery_tracks_transitive_system_headers(tmp_path, kind, replay):
    fixture = BuildFixture(tmp_path / "with spaces", emitted_deps=True)
    manifest = (fixture.root / "leaf/Cargo.toml").read_text()
    fixture.write(
        "leaf/Cargo.toml",
        manifest + '\n[build-dependencies]\ncc = "1"\n'
        f'reactos-link = {{ path = "{REPO}/sdk/lib/rust/reactos-link" }}\n',
    )
    fixture.write("leaf/native/outer.h", "#include <nested value.h>\n")
    fixture.write("leaf/system/nested value.h", "#define VALUE 1\n")
    source = (
        '#include "outer.h"\nunsigned int native_value = VALUE;\n'
        if kind == "c"
        else '#include "outer.h"\n.data\n.balign 4\n.globl native_value\n'
        'native_value:\n.long VALUE\n.section .note.GNU-stack,"",@progbits\n'
    )
    fixture.write(f"leaf/native/value.{kind}", source)
    fixture.write(
        "leaf/build.rs",
        """fn main() {
    let mut build = cc::Build::new();
    build.file("native/value.KIND").flag("-isystem").flag("system");
    let source = std::path::Path::new("native/value.KIND");
    for object in reactos_link::native_deps::compile(&build, source) {
        println!("cargo:rustc-link-arg={}", object.display());
    }
}
""".replace("KIND", kind),
    )
    fixture.write(
        "leaf/src/main.rs",
        'extern "C" { static native_value: u32; }\n'
        'fn main() { println!("{}", unsafe { native_value }); }\n',
    )
    if replay:
        fixture.write(
            "leaf/build.rs",
            """fn main() {
    let out = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let object = out.join("native.o");
    // Relative inputs are relative to native/, not the build script's cwd.
    let mut command = std::process::Command::new("cc");
    command.current_dir("native").args(["-c", "value.KIND", "-isystem", "../system", "-o"])
        .arg(&object);
    let status = reactos_link::native_deps::run(&mut command,
        std::path::Path::new("native/value.KIND"), |cmd| cmd.status()).unwrap();
    assert!(status.success());
    println!("cargo:rustc-link-arg={}", object.display());
}
""".replace("KIND", kind),
        )
    initial = fixture.compile()
    assert initial["leaf"] == "1\n"
    before = fixture.invocations()
    assert fixture.compile() == initial
    assert fixture.invocations() == before
    fixture.write("leaf/system/nested value.h", "#define VALUE 2\n")
    warm = fixture.compile()
    assert fixture.invocations()[len(before) :] == ["leaf"]
    assert warm["leaf"] == "2\n"
    assert warm["peer"] == initial["peer"]
    assert warm["other"] == initial["other"]
    assert warm == fixture.cold()

    if kind == "c" and not replay:
        fixture.write(
            "parser/Cargo.toml",
            '[workspace]\n[package]\nname = "native-depfile-tests"\nversion = "0.1.0"\n'
            'edition = "2021"\n[lib]\n'
            f'path = "{REPO}/sdk/lib/rust/reactos-link/src/native_deps.rs"\n'
            '[dependencies]\ncc = "1"\n',
        )
        fixture.run(
            "cargo",
            "test",
            "--offline",
            "--manifest-path",
            str(fixture.root / "parser/Cargo.toml"),
            "--target-dir",
            str(fixture.build / "cargo"),
            "--lib",
        )


@pytest.mark.parametrize("policy", ["OLD", "NEW"])
@pytest.mark.parametrize("track_headers", [False, True])
def test_real_bindgen_depfile_tracks_nested_header_and_layout(tmp_path, track_headers, policy):
    fixture = BuildFixture(tmp_path, emitted_deps=True, depfile_policy=policy)
    fixture.write("bindings/wrapper.h", '#include "nested.h"\n')
    fixture.write(
        "bindings/nested.h", "#define VALUE 1\nstruct Payload { unsigned char data[1]; };\n"
    )
    generated = fixture.build / "bindings.rs"
    cmake = (tmp_path / "CMakeLists.txt").read_text()
    depfile = 'DEPFILE "${CMAKE_BINARY_DIR}/bindings.d"' if track_headers else ""
    retarget = ""
    if policy == "OLD":
        retarget = f'''
    COMMAND "{sys.executable}" "{REPO}/tools/retarget_cargo_depfile.py"
        --source "${{CMAKE_BINARY_DIR}}/bindings.d"
        --cargo-output "${{CMAKE_BINARY_DIR}}/bindings.rs"
        --target bindings.rs --output "${{CMAKE_BINARY_DIR}}/bindings.d"
'''
    fixture.write(
        "CMakeLists.txt",
        cmake
        + f"""
add_custom_command(OUTPUT "${{CMAKE_BINARY_DIR}}/bindings.rs"
    COMMAND bindgen "${{CMAKE_SOURCE_DIR}}/bindings/wrapper.h"
        --output "${{CMAKE_BINARY_DIR}}/bindings.rs"
        --depfile "${{CMAKE_BINARY_DIR}}/bindings.d" --no-layout-tests
    {retarget}
    COMMAND "{sys.executable}" "${{CMAKE_SOURCE_DIR}}/complete.py"
        "${{CMAKE_BINARY_DIR}}/bindings.rs"
    DEPENDS "${{CMAKE_SOURCE_DIR}}/bindings/wrapper.h"
    {depfile}
    VERBATIM)
add_custom_command(OUTPUT "${{CMAKE_BINARY_DIR}}/leaf" APPEND
    DEPENDS "${{CMAKE_BINARY_DIR}}/bindings.rs")
""",
    )
    fixture.write("leaf/build.rs", "fn main() {}\n")
    fixture.write(
        "leaf/src/main.rs",
        f'include!("{generated}");\n'
        'fn main() { println!("{}:{}", VALUE, std::mem::size_of::<Payload>()); }\n',
    )
    initial = fixture.compile()
    assert initial["leaf"] == "1:1\n"
    before = fixture.invocations()
    assert fixture.compile() == initial
    assert fixture.invocations() == before
    fixture.write(
        "bindings/nested.h", "#define VALUE 2\nstruct Payload { unsigned char data[8]; };\n"
    )
    warm = fixture.compile()
    if track_headers:
        assert fixture.invocations()[len(before) :] == ["leaf"]
        assert warm["leaf"] == "2:8\n"
    else:
        assert fixture.invocations() == before
        assert warm == initial
    cold = fixture.cold()
    assert cold["leaf"] == "2:8\n"
    assert (warm == cold) if track_headers else (warm != cold)


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
            fixture.remove("leaf/inputs/one.txt")
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
