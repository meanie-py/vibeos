"""Exercise the shipping Rust consumer's protocol and fail-closed dispatch."""

import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def test_rust_consumer_without_a_supervisor_builds_privately_and_forwards_request(tmp_path):
    module = ROOT / "sdk/lib/rust/reactos-link/src/artifact_store.rs"
    harness = tmp_path / "consumer.rs"
    harness.write_text(
        f'#[path="{module}"] mod artifact_store;\n'
        "fn main() { let root = std::env::args().nth(1).unwrap();\n"
        'let path = artifact_store::get(std::path::Path::new(&root), "cell",\n'
        '"crt-startup-rust", "amd64", "x86_64-pc-windows-gnu", &["crtexe"],\n'
        'false, false, false, "", false, std::path::Path::new(&root)); '
        'println!("result={path:?}"); }\n'
    )
    binary = tmp_path / "consumer"
    subprocess.run(
        ["rustc", "--edition=2021", str(harness), "-o", str(binary)],
        check=True,
        capture_output=True,
        timeout=60,
    )
    env = dict(os.environ, VIBEOS_ARTIFACT_STORE_MODE="OFF", OUT_DIR="/example/out")
    command = [str(binary), str(tmp_path)]
    result = subprocess.run(command, env=env, capture_output=True, text=True, check=True)
    assert "result=None" in result.stdout
    env["VIBEOS_ARTIFACT_STORE_MODE"] = "ON"
    env.pop("VIBEOS_ARTIFACT_SUPERVISOR", None)
    result = subprocess.run(command, env=env, capture_output=True, text=True)
    # On by default, so a raw ninja has no supervisor: it builds the body privately.
    assert result.returncode == 0 and "result=None" in result.stdout
    assert "no vibeos-cli build supervisor" in result.stdout
    package = tmp_path / "tools"
    package.mkdir()
    (package / "__init__.py").touch()
    archive = tmp_path / "immutable.a"
    archive.write_bytes(b"fixture")
    helper = package / "artifact_runtime.py"
    helper.write_text(
        "import sys\n"
        "assert '--no-default-features' in sys.argv\n"
        "assert sys.argv[sys.argv.index('--features')+1] == 'crtexe'\n"
        "assert '--flags=' in sys.argv\n"
        f"print('archive\\t{archive}')\n"
        f"print('watch\\t{tmp_path}')\n"
    )
    env.update(VIBEOS_ARTIFACT_SUPERVISOR="/supervisor.json", VIBEOS_ARTIFACT_PYTHON=sys.executable)
    result = subprocess.run(command, env=env, capture_output=True, text=True, check=True)
    assert f"cargo:rerun-if-changed={tmp_path}" in result.stdout
    assert str(archive) in result.stdout
    helper.write_text("raise RuntimeError('corrupt blob')\n")
    result = subprocess.run(command, env=env, capture_output=True, text=True)
    assert result.returncode != 0 and "corrupt blob" in result.stderr


def test_rust_response_validation(tmp_path):
    binary = tmp_path / "consumer-tests"
    subprocess.run(
        [
            "rustc",
            "--edition=2021",
            "--test",
            str(ROOT / "sdk/lib/rust/reactos-link/src/artifact_store.rs"),
            "-o",
            str(binary),
        ],
        check=True,
        capture_output=True,
        timeout=60,
    )
    subprocess.run([str(binary)], check=True, capture_output=True, timeout=60)
