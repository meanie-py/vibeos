"""Exercise the shipping persistence algorithm against deterministic hive failures."""

import json
import os
import shutil
import subprocess
from pathlib import Path


def test_registry_security_persistence(tmp_path: Path) -> None:
    repo = Path(__file__).resolve().parents[1]
    fixture = repo / "tools/fixtures/registry_security_persistence.rs"
    shipping = repo / "sdk/lib/rust/config-rust/src/cmsecurity_persist.rs"
    assert shipping.is_file(), f"Missing shipping helper: {shipping}"
    rustc = shutil.which("rustc")
    assert rustc, "rustc is required for registry persistence controls"
    source = tmp_path / "persistence.rs"
    source.write_text(fixture.read_text().replace("__SHIPPING_MODULE__", json.dumps(str(shipping))))
    binary = tmp_path / "persistence-tests"
    env = os.environ.copy()
    env.pop("RUSTC_WRAPPER", None)
    env.pop("RUSTC_WORKSPACE_WRAPPER", None)
    compile_result = subprocess.run(
        [rustc, "--test", "--edition=2021", str(source), "-o", str(binary)],
        capture_output=True,
        text=True,
        env=env,
        timeout=120,
        check=False,
    )
    assert compile_result.returncode == 0, compile_result.stdout + compile_result.stderr
    result = subprocess.run([str(binary)], capture_output=True, text=True, timeout=30, check=False)
    assert result.returncode == 0, result.stdout + result.stderr
