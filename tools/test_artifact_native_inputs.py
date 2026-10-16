"""Native-link eligibility and pinned-compiler staticlib boundary evidence.

The tiny compiler fixtures acquire the existing per-compiler broker. Tooling
already owns disk admission; these fixtures write under 20 MiB per test and do
not acquire a nested tooling reservation that could deadlock their parent.
"""

from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path

import pytest

from tools import test_artifact_inputs as inputs
from tools.artifact_inputs import discover

fixture = inputs.fixture
REPO = Path(inputs.__file__).resolve().parents[1]
TOOLCHAIN = Path.home() / ".rustup/toolchains/1.92.0-x86_64-unknown-linux-gnu"
ABSENT_LIBRARY = "vibeos_deliberately_absent_native_library_56aac883"
ABSENT_SYMBOL = "vibeos_deliberately_absent_native_function_56aac883"


@pytest.fixture(autouse=True)
def pinned_environment(monkeypatch):
    """Fail explicitly if the requested compiler is unavailable; never skip."""
    rustc = TOOLCHAIN / "bin/rustc"
    assert rustc.is_file(), f"required pinned compiler missing: {rustc}"
    identity = subprocess.run(
        [str(rustc), "-Vv"], capture_output=True, text=True, check=True
    ).stdout
    assert identity.startswith("rustc 1.92.0 "), identity
    monkeypatch.setenv("RUSTUP_TOOLCHAIN", "1.92.0")


@pytest.mark.parametrize(
    "attribute",
    [
        '#[link(name = "fixture_absent")]',
        '#[link(name = "fixture_absent", kind = "dylib")]',
        '#[link(kind = "dylib", name = "fixture_absent",)]',
        '#[link(name = "fixture_absent.dll", kind = "raw-dylib")]',
        '#[link(name = "fixture_absent.dll", kind = "raw-dylib", modifiers = "+verbatim")]',
    ],
)
def test_deferred_native_library_declarations_are_eligible(fixture, attribute):
    inputs.write(
        fixture,
        "body/src/lib.rs",
        attribute + '\nextern "C" { fn fixture_absent_function(); }\n',
    )
    baseline = inputs.project(fixture, cargo=str(TOOLCHAIN / "bin/cargo"))
    assert baseline["eligible"]
    assert baseline["descriptor"]["packages"]["body"]["inventory"]["files"]


@pytest.mark.parametrize(
    "attribute",
    [
        '#[link(name = "fixture_absent", kind = "static")]',
        '#[link(name = "fixture_absent", modifiers = "+verbatim")]',
        '#[link(name = "fixture_absent", kind = "dylib", modifiers = "+verbatim")]',
        '#[link(name = "fixture_absent", kind = "raw-dylib", modifiers = "+bundle")]',
        '#[link(name = "fixture_absent", unknown_key = "value")]',
        '#[link(name = "fixture_absent", kind = "framework")]',
        '#[link(name = "fixture_absent", kind = "link-arg")]',
        '#[link(kind = "dylib")]',
        '#[link(name = concat!("fixture", "_absent"))]',
        '#[link(name = "fixture_absent", kind = "dylib", kind = "raw-dylib")]',
        '#[cfg_attr(target_os = "windows", link(name = "fixture_absent", kind = "static"))]',
        '#[cfg_attr(any(), link(name = "fixture_absent", kind = "dylib"))]',
    ],
)
def test_unaudited_native_library_inputs_are_ineligible(fixture, attribute):
    inputs.write(
        fixture,
        "body/src/lib.rs",
        attribute + '\nextern "C" { fn fixture_absent_function(); }\n',
    )
    result = discover(
        fixture,
        "body",
        inputs.CONTEXT,
        workspace=fixture,
        cargo=str(TOOLCHAIN / "bin/cargo"),
    )
    assert not result["eligible"], result
    assert result["action_key"] is None
    assert any("native" in reason or "link" in reason for reason in result["reasons"]), result


@pytest.mark.parametrize("target", ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-gnu"])
def test_pinned_staticlib_defers_dylib_but_requires_static_archive(tmp_path, target):
    """Real staticlibs call the missing symbol; optimization cannot remove it."""
    broker = REPO / ".cache/host-jobs/host-jobs"
    assert broker.is_file(), f"host-jobs must be provisioned before compiler fixture: {broker}"
    environment = {
        "PATH": os.environ["PATH"],
        "HOME": str(tmp_path),
        "TMPDIR": str(tmp_path),
    }
    # Preserve fleet policy/ownership, but never the inherited compiler-cache
    # wrappers or VIBEOS_HOST_JOB_HELD: every rustc obtains its own admission.
    environment.update(
        (key, value)
        for key, value in os.environ.items()
        if key.startswith("VIBEOS_") and key != "VIBEOS_HOST_JOB_HELD"
    )
    cases = []
    for kind in ("default", "dylib", "static"):
        root = tmp_path / kind
        root.mkdir()
        attribute = f'#[link(name = "{ABSENT_LIBRARY}"'
        if kind != "default":
            attribute += f', kind = "{kind}"'
        attribute += ")]"
        source = root / "input.rs"
        source.write_text(
            "#![no_std]\n"
            + attribute
            + f'\nextern "C" {{ fn {ABSENT_SYMBOL}() -> i32; }}\n'
            + '#[no_mangle]\npub unsafe extern "C" fn fixture_calls_native() -> i32 { '
            + ABSENT_SYMBOL
            + "() }\n"
            + "#[panic_handler]\nfn panic(_: &core::panic::PanicInfo) -> ! { loop {} }\n"
        )
        archive = root / "libfixture.a"
        command = [
            str(broker),
            "--kind",
            "compile",
            "--",
            str(TOOLCHAIN / "bin/rustc"),
            str(source),
            "--edition=2021",
            "--crate-name",
            "fixture",
            "--crate-type",
            "staticlib",
            "--target",
            target,
            "-C",
            "panic=abort",
            "-C",
            "opt-level=1",
            "--print=native-static-libs",
            "-o",
            str(archive),
        ]
        result = subprocess.run(
            command, cwd=root, env=environment, capture_output=True, text=True, check=False
        )
        case = {
            "target": target,
            "kind": kind,
            "command": command,
            "exit_code": result.returncode,
            "stderr": result.stderr,
            "archive_exists": archive.is_file(),
        }
        cases.append(case)
        (tmp_path / "native-link-evidence.json").write_text(json.dumps(cases, indent=2))
        if kind == "static":
            assert result.returncode != 0, case
            assert "could not find native static library" in result.stderr, case
            assert not archive.exists(), case
        else:
            assert result.returncode == 0, case
            assert archive.is_file() and archive.read_bytes().startswith(b"!<arch>\n"), case
            assert "native-static-libs:" in result.stderr and ABSENT_LIBRARY in result.stderr
            symbols = subprocess.run(
                ["nm", "-u", str(archive)], capture_output=True, text=True, check=True
            ).stdout
            assert any("U " + ABSENT_SYMBOL in line for line in symbols.splitlines()), (
                "missing native call must remain unresolved in the archive"
            )
