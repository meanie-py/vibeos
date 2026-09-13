"""Exercise persisted facts with actual subprocess extraction and competing writers."""

from __future__ import annotations

import multiprocessing
import os
import subprocess
import sys
from pathlib import Path

import pytest

from tools.binary_facts import BinaryFacts, BinaryFactsError


@pytest.fixture
def inputs(tmp_path: Path) -> tuple[Path, Path, Path]:
    binary = tmp_path / "kernel.exe"
    binary.write_text("first")
    scanner = tmp_path / "scanner.py"
    scanner.write_text("# scanner v1")
    tool = tmp_path / "fake-objdump"
    tool.write_text(
        f"#!{sys.executable}\n"
        "from pathlib import Path\n"
        "import sys\n"
        "with Path(__file__).with_suffix('.calls').open('a') as f: f.write('call\\n')\n"
        "print(Path(sys.argv[1]).read_text())\n"
    )
    tool.chmod(0o755)
    return binary, scanner, tool


def context(inputs: tuple[Path, Path, Path]) -> BinaryFacts:
    binary, scanner, tool = inputs
    return BinaryFacts(
        binary.parent,
        namespace="test",
        implementation=[scanner],
        tools=[tool],
        options={"arch": "x"},
    )


def extract(inputs: tuple[Path, Path, Path]) -> list[dict[str, str]]:
    binary, _, tool = inputs
    output = subprocess.run([str(tool), str(binary)], check=True, capture_output=True, text=True)
    return [{"function": output.stdout.strip()}]


def calls(inputs: tuple[Path, Path, Path]) -> int:
    return len(inputs[2].with_suffix(".calls").read_text().splitlines())


def test_content_and_extractor_identity(inputs: tuple[Path, Path, Path]) -> None:
    binary, scanner, tool = inputs
    cache = context(inputs)
    first = cache.get(binary, lambda: extract(inputs))
    assert context(inputs).get(binary, lambda: extract(inputs)) == first
    assert calls(inputs) == 1
    old = binary.stat()
    binary.write_text("other")
    os.utime(binary, ns=(old.st_atime_ns, old.st_mtime_ns))
    assert cache.get(binary, lambda: extract(inputs)) == [{"function": "other"}]
    assert calls(inputs) == 2
    scanner.write_text("# scanner v2")
    with pytest.raises(BinaryFactsError, match="implementation/tool changed"):
        cache.get(binary, lambda: extract(inputs))
    context(inputs).get(binary, lambda: extract(inputs))
    tool.write_text(tool.read_text() + "# changed tool\n")
    context(inputs).get(binary, lambda: extract(inputs))
    assert calls(inputs) == 4


def test_corrupt_record_fails_loudly(inputs: tuple[Path, Path, Path]) -> None:
    cache = context(inputs)
    cache.get(inputs[0], lambda: extract(inputs))
    record = next(cache.root.glob("*.json"))
    record.write_text('{"key":"wrong", "facts": [], "sha256":"wrong"}')
    with pytest.raises(BinaryFactsError, match="corrupt binary facts"):
        cache.get(inputs[0], lambda: extract(inputs))
    assert calls(inputs) == 1


def test_failure_and_input_change_never_publish(inputs: tuple[Path, Path, Path]) -> None:
    binary, _, tool = inputs
    cache = context(inputs)

    def fail() -> None:
        subprocess.run(
            [str(tool), str(binary.with_name("missing"))], check=True, capture_output=True
        )

    with pytest.raises(subprocess.CalledProcessError):
        cache.get(binary, fail)
    assert not list(cache.root.glob("*.json"))

    def mutate() -> list[dict[str, str]]:
        result = extract(inputs)
        old = binary.stat()
        binary.write_text("other")
        os.utime(binary, ns=(old.st_atime_ns, old.st_mtime_ns))
        return result

    with pytest.raises(BinaryFactsError, match="input changed"):
        cache.get(binary, mutate)
    assert not list(cache.root.glob("*.json"))
    assert cache.get(binary, lambda: extract(inputs)) == [{"function": "other"}]


def _competing_reader(inputs: tuple[Path, Path, Path]) -> None:
    assert context(inputs).get(inputs[0], lambda: extract(inputs)) == [{"function": "first"}]


def test_processes_share_one_extraction(inputs: tuple[Path, Path, Path]) -> None:
    workers = [
        multiprocessing.get_context("fork").Process(target=_competing_reader, args=(inputs,))
        for _ in range(8)
    ]
    for worker in workers:
        worker.start()
    for worker in workers:
        worker.join(20)
        assert worker.exitcode == 0
    assert calls(inputs) == 1
    assert len(list(context(inputs).root.glob("*.json"))) == 1


def test_disabled_and_policy_applied_fresh(
    inputs: tuple[Path, Path, Path], monkeypatch: pytest.MonkeyPatch
) -> None:
    cache = context(inputs)
    facts = cache.get(inputs[0], lambda: extract(inputs))
    baseline = {"first"}
    assert not [item for item in facts if item["function"] not in baseline]
    baseline.clear()
    facts = cache.get(inputs[0], lambda: extract(inputs))
    assert [item for item in facts if item["function"] not in baseline]
    assert calls(inputs) == 1
    monkeypatch.setenv("VIBEOS_BINARY_CACHE", "0")
    context(inputs).get(inputs[0], lambda: extract(inputs))
    context(inputs).get(inputs[0], lambda: extract(inputs))
    assert calls(inputs) == 3


def test_symlink_change_cannot_publish(inputs: tuple[Path, Path, Path]) -> None:
    binary, _, _ = inputs
    link = binary.with_name("linked.exe")
    link.symlink_to(binary)
    other = binary.with_name("other.exe")
    other.write_text("other")
    cache = context(inputs)

    def swap() -> list[str]:
        link.unlink()
        link.symlink_to(other)
        return [link.read_text()]

    with pytest.raises(BinaryFactsError, match="input symlink changed"):
        cache.get(link, swap)
    assert not list(cache.root.glob("*.json"))


def test_architecture_and_locale_separate_facts(
    inputs: tuple[Path, Path, Path], monkeypatch: pytest.MonkeyPatch
) -> None:
    binary, scanner, tool = inputs
    context(inputs).get(binary, lambda: extract(inputs))
    BinaryFacts(
        binary.parent,
        namespace="test",
        implementation=[scanner],
        tools=[tool],
        options={"arch": "y"},
    ).get(binary, lambda: extract(inputs))
    monkeypatch.setenv("LC_ALL", "C")
    context(inputs).get(binary, lambda: extract(inputs))
    assert calls(inputs) == 3
