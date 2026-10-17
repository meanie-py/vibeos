"""Host-run the Pi Wi-Fi driver's tests (bwfm-rust, on sdhci-rust).

SDPCM framing, BCDC commands and data headers, firmware events, the SDIO commands and
the chip's backplane window are plain code over bytes and registers. On the board a
mistake in any of them looks the same: the chip answers nothing, or answers the wrong
request. So they are checked here, the SDIO and chip layers against a simulated SDHCI
with a CYW43455 on its bus (bwfm-rust/src/sim.rs), before there is a bus to try them on.
The kernel side (driver.rs) is compiled out under test.
"""

import os
import shutil
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
SDHCI = "sdk/lib/rust/sdhci-rust/src/lib.rs"
WPA = "sdk/lib/rust/wpa-rust/src/lib.rs"
BWFM = "sdk/lib/rust/bwfm-rust/src/lib.rs"


def _rustc(args: list[str]) -> subprocess.CompletedProcess[str]:
    rustc = shutil.which("rustc")
    assert rustc, "rustc is required for the Wi-Fi driver tests"
    env = os.environ.copy()
    env.pop("RUSTC_WRAPPER", None)
    env.pop("RUSTC_WORKSPACE_WRAPPER", None)
    return subprocess.run(
        [rustc, "--edition=2021", *args],
        capture_output=True,
        text=True,
        env=env,
        timeout=180,
        check=False,
    )


def test_bwfm(tmp_path: Path) -> None:
    externs = []
    for crate, module in (("sdhci_rust", SDHCI), ("wpa_rust", WPA)):
        assert (REPO / module).is_file(), f"missing {module}"
        rlib = tmp_path / f"lib{crate}.rlib"
        built = _rustc(
            ["--crate-type=rlib", f"--crate-name={crate}", str(REPO / module), "-o", str(rlib)]
        )
        assert built.returncode == 0, built.stdout + built.stderr
        externs += ["--extern", f"{crate}={rlib}"]
    binary = tmp_path / "tests"
    built = _rustc(["--test", str(REPO / BWFM), *externs, "-o", str(binary)])
    assert built.returncode == 0, built.stdout + built.stderr
    ran = subprocess.run([str(binary)], capture_output=True, text=True, timeout=60, check=False)
    assert ran.returncode == 0, ran.stdout + ran.stderr
    # Each layer must actually have run, not just compiled.
    for layer in ("proto::", "sdio::", "chip::", "nvram::", "bus::", "scan::", "join::", "net::"):
        assert f"test {layer}" in ran.stdout, ran.stdout
