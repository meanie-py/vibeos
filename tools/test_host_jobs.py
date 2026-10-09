"""Exercise shipping admission through real CMake and Cargo compiler invocations."""

import json
import os
import subprocess
from pathlib import Path

import pytest

from tests.qemu.cli import hostjobs

REPO = Path(__file__).resolve().parents[1]


@pytest.fixture(scope="module")
def runner():
    return hostjobs.ensure(REPO)


def run(cmd, cwd, env, timeout=120):
    result = subprocess.run(cmd, cwd=cwd, env=env, capture_output=True, text=True, timeout=timeout)
    assert result.returncode == 0, result.stdout + result.stderr
    return result.stdout


def test_runner_process_contracts(runner, tmp_path):
    # The runner's tests use real competing processes and crash/priority controls.
    run(
        [
            "cargo",
            "test",
            "--offline",
            "--manifest-path",
            str(REPO / "tools/host-jobs/Cargo.toml"),
            "--target-dir",
            str(tmp_path / "host-tests"),
            "--",
            "--test-threads=1",
        ],
        REPO,
        os.environ,
        # A cold compile into a fresh target dir plus a serial process suite: ~10 s on
        # this host at load average 40, but the gate runs it beside three builds and it
        # was killed at 120 s on 2026-10-09 while its tests were passing. A safety net,
        # not a deadline; the suite's own timing checks are calibrated inside it.
        timeout=600,
    )


def test_cmake_native_compile_and_link(runner, tmp_path):
    root = tmp_path / "project with spaces"
    root.mkdir()
    local = root / ".cache/host-jobs/host-jobs"
    local.parent.mkdir(parents=True)
    local.symlink_to(runner)  # immutable host tool, not another tree's build artifacts
    (root / "main.c").write_text('#include <stdio.h>\nint main(void) { puts("42"); }\n')
    (root / "CMakeLists.txt").write_text(
        "cmake_minimum_required(VERSION 3.17)\nproject(REACTOS C)\n"
        f'include("{REPO}/sdk/cmake/host-jobs.cmake")\n'
        "add_executable(answer main.c)\n"
    )
    env = dict(os.environ, VIBEOS_HOSTLOCK_DIR=str(tmp_path / "locks"), VIBEOS_HOST_JOB_SLOTS="2")
    run(["cmake", "-S", ".", "-B", "build", "-G", "Ninja"], root, env)
    run(["cmake", "--build", "build", "--verbose"], root, env)
    assert run([str(root / "build/answer")], root, env).strip() == "42"
    rules = (root / "build/CMakeFiles/rules.ninja").read_text()
    assert "--kind compile --" in rules and "--kind link --" in rules
    status = json.loads(run([str(runner), "--status"], root, env))
    assert status is not None


def test_cargo_native_dependency_discovery_survives_admission(runner, tmp_path):
    (tmp_path / "src").mkdir()
    (tmp_path / "Cargo.toml").write_text(
        '[package]\nname="admitted-native"\nversion="0.1.0"\nedition="2021"\n'
        '[build-dependencies]\ncc="1"\n'
    )
    (tmp_path / "value.h").write_text("#define VALUE 42\n")
    (tmp_path / "value.c").write_text('#include "value.h"\nint value(void) { return VALUE; }\n')
    (tmp_path / "build.rs").write_text(
        f'#[path="{REPO}/sdk/lib/rust/reactos-link/src/native_deps.rs"] mod native_deps;\n'
        'fn main() { let mut b = cc::Build::new(); b.file("value.c");\n'
        'for p in native_deps::compile(&b, std::path::Path::new("value.c")) {\n'
        'println!("cargo:rustc-link-arg={}", p.display()); } }\n'
    )
    (tmp_path / "src/main.rs").write_text(
        'extern "C" { fn value() -> i32; }\nfn main() { println!("{}", unsafe { value() }); }\n'
    )
    env = dict(
        os.environ,
        VIBEOS_HOSTLOCK_DIR=str(tmp_path / "locks"),
        VIBEOS_HOST_JOBS=str(runner),
        VIBEOS_HOST_JOB_SLOTS="2",
        RUSTC_WRAPPER=str(runner),
    )
    for key in (
        "CARGO_BUILD_TARGET",
        "CARGO_ENCODED_RUSTFLAGS",
        "RUSTFLAGS",
        "VIBEOS_INNER_RUSTC_WRAPPER",
    ):
        env.pop(key, None)
    for expected in (42, 43):
        (tmp_path / "value.h").write_text(f"#define VALUE {expected}\n")
        run(["cargo", "build", "--offline"], tmp_path, env)
        assert run([str(tmp_path / "target/debug/admitted-native")], tmp_path, env).strip() == str(
            expected
        )


def test_environment_preserves_cache_and_reports_explicit_policy(runner, tmp_path, monkeypatch):
    from tests.qemu.cli import buildpolicy, hostlock

    monkeypatch.setattr(hostlock, "HOSTLOCK_DIR", tmp_path)
    # Durable policy config now lives apart from the ephemeral lock tree; without this
    # the test reads the real host policy and asserts against this box's capacity.
    monkeypatch.setattr(hostlock, "POLICY_DIR", tmp_path)
    monkeypatch.setenv("VIBEOS_HOSTLOCK_DIR", str(tmp_path))
    (tmp_path / "build-policy.json").write_text(json.dumps(buildpolicy.TRIAL))
    env = hostjobs.environment(
        REPO, {"RUSTC_WRAPPER": "sccache"}, canonical=False, run_dir=tmp_path / "run"
    )
    assert env["CARGO_BUILD_JOBS"] == "1"
    assert env["RUSTC_WRAPPER"] == str(runner)
    assert env["VIBEOS_INNER_RUSTC_WRAPPER"] == "sccache"
    assert env["VIBEOS_JOB_RUN"] == str(tmp_path / "run")
    assert env["VIBEOS_JOB_OWNER_PID"] == str(os.getpid())
    assert int(env["VIBEOS_HOST_JOB_SLOTS"]) == 18
    assert int(env["VIBEOS_HOST_CPU_LEASES"]) == 6
    assert hostjobs.status(REPO)["cpu_slots"] == 18
