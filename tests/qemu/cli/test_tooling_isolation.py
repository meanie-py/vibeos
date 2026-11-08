"""Concurrent native tooling suites own separate pytest temp roots."""

import json
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

from tests.qemu.cli import tooling


def test_parallel_pytest_temp_roots_are_disjoint(tmp_path):
    script = (
        "import json,sys,time; from pathlib import Path; "
        "p=Path(sys.argv[sys.argv.index('--basetemp')+1]); "
        "assert p.parent.is_dir(); time.sleep(.05); print(json.dumps(str(p)))"
    )
    with ThreadPoolExecutor(max_workers=2) as pool:
        jobs = [
            pool.submit(tooling._run, f"pytest:{i}", [sys.executable, "-c", script], tmp_path)
            for i in range(2)
        ]
        results = [job.result() for job in jobs]
    assert all(r[1] for r in results)
    roots = [Path(json.loads(r[2])) for r in results]
    assert len(set(roots)) == 2
    assert all(not str(p).startswith("/tmp/vibeos") and not p.parent.exists() for p in roots)


def test_non_pytest_command_is_unchanged(tmp_path):
    command = [sys.executable, "-c", "import sys; print(repr(sys.argv[1:]))", "unchanged"]
    assert tooling._run("rust:vibeos-build", command, tmp_path) == (
        "rust:vibeos-build",
        True,
        "['unchanged']",
    )
