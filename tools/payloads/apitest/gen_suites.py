#!/usr/bin/env python3
"""Compatibility entry point and discovery bridge for the native suite generator."""

import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / "tools"))
from codegen_compat import main  # noqa: E402


def discover(build_dir: Path) -> list[str]:
    result = subprocess.run(
        [
            str(REPO / "tools/codegen"),
            "apitest-suites",
            "--build-dir",
            str(build_dir),
            "--discover-json",
        ],
        capture_output=True,
        text=True,
        check=True,
        timeout=180,
    )
    return json.loads(result.stdout)


if __name__ == "__main__":
    main("apitest-suites")
