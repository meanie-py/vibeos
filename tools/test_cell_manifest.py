import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))

import cell_manifest as m

B = "/tmp/vibeos/build-i386"
T = "/tmp/vibeos/sdk/lib/rust/target/cell-link-v1-i386-3"
COMMAND = " && ".join(
    [
        "cd /tmp/vibeos/sdk/lib/rust",
        '/usr/bin/cmake -E env "RUSTFLAGS=-C force-frame-pointers=yes" '
        f"CARGO_TARGET_DIR={T} CARGO_BUILD_JOBS=1 VIBEOS_ARTIFACT_STORE_MODE=ON "
        "cargo build --release --target i686-pc-windows-gnu -p beep-cargo --bin beep "
        "--manifest-path /tmp/vibeos/sdk/lib/rust/Cargo.toml",
        f"/usr/bin/cmake -E copy {T}/i686-pc-windows-gnu/release/beep.exe "
        f"{B}/drivers/beep/beep.sys",
        f"{B}/host-tools/bin/pefixup --kernelmodedriver {B}/drivers/beep/beep.sys",
        "/usr/bin/cmake -E env python3 /tmp/vibeos/sdk/tools/fix_lld_forwarding.py "
        f"{B}/drivers/beep/beep.sys",
        f"/usr/bin/cmake -E env python3 /tmp/vibeos/tools/retarget_cargo_depfile.py --source x "
        f"--output {B}/drivers/beep/beep.sys.d",
    ]
)


def test_a_cell_command_reads_back_as_data():
    c = m.parse_cell("drivers/beep/beep.sys ", COMMAND, "i386")
    assert c == {
        "crate": "beep-cargo",
        "target": "i686-pc-windows-gnu",
        "build_std": False,
        # The cell's env, not the host's (the artifact store's stays out).
        "env": {"RUSTFLAGS": "-C force-frame-pointers=yes"},
        "target_dir": "cell-link-v1-i386-3",
        "bins": ["beep"],
        "outputs": [
            {
                "bin": "beep",
                "out": "drivers/beep/beep.sys",
                "post": ["pefixup --kernelmodedriver", "fix_lld_forwarding"],
            }
        ],
    }


def test_an_unknown_post_link_step_is_refused():
    bad = COMMAND.replace("/usr/bin/cmake -E env python3 /tmp/vibeos/sdk/tools/fix_lld", "mystery")
    with pytest.raises(m.ManifestError, match="unmodelled post-link"):
        m.parse_cell("drivers/beep/beep.sys ", bad, "i386")


def test_an_output_the_command_does_not_copy_is_refused():
    with pytest.raises(m.ManifestError, match="declared outputs"):
        m.parse_cell("drivers/beep/beep.sys drivers/beep/other.sys ", COMMAND, "i386")
