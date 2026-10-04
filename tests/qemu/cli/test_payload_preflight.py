"""smoke --payload refuses media whose build did not stage that payload."""

import shutil
from pathlib import Path

from . import smoke

REPO = Path(__file__).resolve().parents[3]


def _repo_with_stamp(tmp_path: Path, stamp: str | None) -> Path:
    payloads = tmp_path / "tools" / "payloads"
    payloads.mkdir(parents=True)
    shutil.copy(REPO / "tools" / "payloads" / "compile.py", payloads / "compile.py")
    build = tmp_path / "build-amd64"
    build.mkdir()
    if stamp is not None:
        (build / "payload-selection.json").write_text(stamp)
    return tmp_path


def test_reads_what_the_build_stamped(tmp_path):
    repo = _repo_with_stamp(tmp_path, '{"payloads": ["kmtest"]}\n')
    assert smoke._staged_payloads(repo, "amd64") == ["kmtest"]


def test_a_build_with_no_stamp_staged_nothing(tmp_path):
    # An absent stamp must read as "nothing staged", which the preflight then refuses,
    # never as "anything goes".
    repo = _repo_with_stamp(tmp_path, None)
    assert smoke._staged_payloads(repo, "amd64") == []
