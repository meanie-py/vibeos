"""Every name in kmtest-tests-<arch>.txt must be a test kmtest actually has on that arch.

The lists are hand-maintained, and were seeded by copying amd64's. A name the arch's
testlist.c compiles out makes kmtest fail the lookup (support.c: error 0x7b) and the
sweep records a NO-END row that reads like a crash -- RtlCaptureContextKM/UM sat on the
i386 and arm64 lists, and HalPortIo on arm64's, that way until 2026-10-09.
"""
from __future__ import annotations

import re
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[3]
TESTLISTS = (
    ROOT / "modules/rostests/kmtests/kmtest_drv/testlist.c",
    ROOT / "modules/rostests/kmtests/kmtest/testlist.c",
)
ARCH_MACROS = {"i386": {"_M_IX86"}, "amd64": {"_M_AMD64"}, "arm64": {"_M_ARM64"}}
_ENTRY = re.compile(r'^\s*\{\s*"([^"]+)"\s*,')


def _cond_value(cond: str, defined: set[str]) -> bool:
    """Evaluate the only conditional shapes testlist.c uses; refuse anything else."""
    cond = cond.strip()
    if m := re.fullmatch(r"#ifdef\s+(\w+)", cond):
        return m.group(1) in defined
    if m := re.fullmatch(r"#ifndef\s+(\w+)", cond):
        return m.group(1) not in defined
    if m := re.fullmatch(r"#if\s+(.+)", cond):
        terms = [t.strip() for t in m.group(1).split("||")]
        if all(re.fullmatch(r"defined\s*\(\s*\w+\s*\)", t) for t in terms):
            return any(re.search(r"\((\w+)\)", t.replace(" ", "")).group(1) in defined for t in terms)
    raise ValueError(f"testlist.c conditional this check cannot evaluate: {cond!r}")


def built_tests(arch: str) -> set[str]:
    defined = ARCH_MACROS[arch]
    names: set[str] = set()
    for path in TESTLISTS:
        stack: list[bool] = []
        for line in path.read_text().splitlines():
            s = line.strip()
            if s.startswith(("#if", "#ifdef", "#ifndef")):
                stack.append(_cond_value(s, defined))
            elif s.startswith("#else"):
                stack[-1] = not stack[-1]
            elif s.startswith("#endif"):
                stack.pop()
            elif all(stack) and (m := _ENTRY.match(line)):
                names.add(m.group(1))
    return names


def listed(arch: str) -> list[str]:
    path = ROOT / f"tools/payloads/kmtest/kmtest-tests-{arch}.txt"
    return [ln.strip() for ln in path.read_text().splitlines() if ln.strip() and not ln.startswith("#")]


@pytest.mark.parametrize("arch", sorted(ARCH_MACROS))
def test_every_listed_test_is_built_for_the_arch(arch: str) -> None:
    missing = sorted(set(listed(arch)) - built_tests(arch))
    assert not missing, f"{arch} list names tests kmtest does not build there: {missing}"


def test_parser_sees_the_arch_gated_entries() -> None:
    # Guards the parser itself: if it stopped honouring #ifdef, the check above would
    # pass vacuously on every arch.
    assert "RtlCaptureContextKM" in built_tests("amd64")
    assert "RtlCaptureContextKM" not in built_tests("arm64")
    assert "HalPortIo" in built_tests("i386")
    assert "HalPortIo" not in built_tests("arm64")
