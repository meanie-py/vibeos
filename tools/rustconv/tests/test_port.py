"""rustconv port: the hand-offs it automates are idempotent and land where they should."""

import pytest

from rustconv import port

SCRIPT = """case "$DRIVER" in
  alpha)
    USERMODE=1
    DRIVER_DIR="${SRC}/dll/win32/alpha"
    ;;
  *) echo "unknown driver: $DRIVER" >&2; exit 2 ;;
esac
"""

ARM = """  beta)
    USERMODE=1
    DRIVER_DIR="${SRC}/base/setup/lib"
    ;;
"""


def test_profile_arm_is_inserted_before_the_default_and_replaced_in_place():
    once = port.upsert_profile_arm(SCRIPT, ARM, "beta")
    assert once.index("  beta)") < once.index("  *) echo")
    assert once.count("  alpha)") == 1
    # idempotent: the same arm again changes nothing
    assert port.upsert_profile_arm(once, ARM, "beta") == once
    # a re-derived arm REPLACES the old one rather than adding a second
    changed = port.upsert_profile_arm(once, ARM.replace("base/setup/lib", "base/setup/x"), "beta")
    assert changed.count("  beta)") == 1 and "base/setup/x" in changed and "setup/lib" not in changed


def test_script_without_a_default_arm_is_refused():
    with pytest.raises(port.PortError):
        port.upsert_profile_arm("case x in\nesac\n", ARM, "beta")


def test_run_cost_reads_the_last_running_total():
    log = "  [1/3] A... OK  [running $0.02]\n  [2/3] B... OK  [running $0.06]\n  [3/3] C... CACHED ($0)\n"
    assert port._run_cost(log) == pytest.approx(0.06)
    assert port._run_cost("nothing paid") == 0.0


def test_workspace_member_is_added_once(tmp_path, monkeypatch):
    manifest = tmp_path / "sdk" / "lib" / "rust" / "Cargo.toml"
    manifest.parent.mkdir(parents=True)
    manifest.write_text('[workspace]\nmembers = [\n    "a-rust",\n]\n')
    monkeypatch.setattr(port, "REPO", tmp_path)
    assert port.add_workspace_member("b-rust")
    assert not port.add_workspace_member("b-rust")
    text = manifest.read_text()
    assert text.count('"b-rust"') == 1 and text.index('"a-rust"') < text.index('"b-rust"')
