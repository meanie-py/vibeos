"""Host-run the Wi-Fi supplicant's own unit tests (wpa-rust).

The WDI layer's WPA2-Personal handshake is pure code over bytes: SHA-1, HMAC, the PSK and
PTK derivations, AES key unwrap, and the EAPOL-Key state machine. On the bench a mistake in
any of them looks the same — the access point silently drops message 2 or 4 — so the
standard known-answer vectors and a simulated authenticator run on every check-tooling.
"""

import os
import shutil
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
MODULE = "sdk/lib/rust/wpa-rust/src/lib.rs"


def test_wpa_supplicant(tmp_path: Path) -> None:
    source = REPO / MODULE
    assert source.is_file(), f"missing {MODULE}"
    rustc = shutil.which("rustc")
    assert rustc, "rustc is required for the supplicant tests"
    binary = tmp_path / "tests"
    env = os.environ.copy()
    env.pop("RUSTC_WRAPPER", None)
    env.pop("RUSTC_WORKSPACE_WRAPPER", None)
    # Optimized: the PSK vectors are 8192 HMACs each.
    built = subprocess.run(
        [rustc, "--test", "--edition=2021", "-O", str(source), "-o", str(binary)],
        capture_output=True,
        text=True,
        env=env,
        timeout=180,
        check=False,
    )
    assert built.returncode == 0, built.stdout + built.stderr
    ran = subprocess.run([str(binary)], capture_output=True, text=True, timeout=60, check=False)
    assert ran.returncode == 0, ran.stdout + ran.stderr
    assert "running 0 tests" not in ran.stdout, ran.stdout
