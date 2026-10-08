"""Host-run the arm64 firmware-table parsers' own unit tests.

Both are pure functions over a table's bytes, compiled for the target and never run
there: the loader's MADT/DBG2/SPCR/MCFG reader (which decides the GIC, the console and
the ECAM a board boots with) and the HAL's IORT reader (which decides the DeviceID an
ITS sees for each PCI device, and whether its DMA crosses an SMMU). A wrong answer from
either looks like a dead device on hardware nobody here has, so their known-answer
tests run on every check-tooling instead of whenever someone remembers.
"""

import os
import shutil
import subprocess
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
MODULES = [
    "sdk/lib/rust/freeldr-rust/src/arm64_serial_acpi.rs",
    "sdk/lib/rust/hal-rust/src/arm64/iort.rs",
]


@pytest.mark.parametrize("module", MODULES)
def test_firmware_table_parser(module: str, tmp_path: Path) -> None:
    source = REPO / module
    assert source.is_file(), f"missing {module}"
    rustc = shutil.which("rustc")
    assert rustc, "rustc is required for the firmware-table parser tests"
    binary = tmp_path / "tests"
    env = os.environ.copy()
    env.pop("RUSTC_WRAPPER", None)
    env.pop("RUSTC_WORKSPACE_WRAPPER", None)
    built = subprocess.run(
        [rustc, "--test", "--edition=2021", str(source), "-o", str(binary)],
        capture_output=True,
        text=True,
        env=env,
        timeout=180,
        check=False,
    )
    assert built.returncode == 0, built.stdout + built.stderr
    ran = subprocess.run([str(binary)], capture_output=True, text=True, timeout=60, check=False)
    assert ran.returncode == 0, ran.stdout + ran.stderr
    # A module whose tests were all cfg'd away would pass having checked nothing.
    assert "running 0 tests" not in ran.stdout, ran.stdout
