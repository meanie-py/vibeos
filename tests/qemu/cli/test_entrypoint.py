"""Nested CLI invocations keep the same import inputs across commands/worktrees."""

import json
import os
import shutil
import subprocess
from pathlib import Path


def test_nested_cli_replaces_only_its_owned_import_prefix(tmp_path):
    repo = Path(__file__).resolve().parents[3]
    probe = tmp_path / "python-probe"
    probe.write_text(
        "#!/usr/bin/env python3\nimport json, os\n"
        "print(json.dumps({k: os.environ[k] for k in "
        "('PYTHONPATH', 'VIBEOS_CLI_PYTHONPATH_ROOT')}))\n"
    )
    probe.chmod(0o755)
    env = {**os.environ, "VIBEOS_PYTHON": str(probe), "PYTHONPATH": "/user/extra"}
    env.pop("VIBEOS_CLI_PYTHONPATH_ROOT", None)

    def invoke(root, incoming):
        proc = subprocess.run(
            ["bash", str(root / "vibeos-cli"), "check-tooling"],
            env=incoming,
            capture_output=True,
            text=True,
            check=True,
        )
        return json.loads(proc.stdout)

    first = invoke(repo, env)
    assert first["PYTHONPATH"] == f"{repo}:/user/extra"
    assert invoke(repo, {**env, **first}) == first
    other = tmp_path / "integration"
    other.mkdir()
    shutil.copy2(repo / "vibeos-cli", other / "vibeos-cli")
    switched = invoke(other, {**env, **first})
    assert switched["PYTHONPATH"] == f"{other}:/user/extra"
    assert switched["VIBEOS_CLI_PYTHONPATH_ROOT"] == str(other)
