"""Compatibility DTOs; native vibeos-test owns verdict algorithms."""

from __future__ import annotations

import re
from pathlib import Path

from tools import qemu_infra_compat as native

KERNEL_ASSERT_RE = re.compile(r"\*\*\* ASSERT FAILED: .* at ntoskrnl-rust/")
_MAX_SANE_SYMBOL_OFFSET = 0x20000
KUSER_SHARED_DATA = {"arm64": 0xFFFFF78000000000, "amd64": 0xFFFFF78000000000, "i386": 0xFFDF0000}
KNOWN_FLAKES = native.call("verdict_known_flakes", {})


def _call(op, kwargs, repo=None):
    kwargs["known_flakes"] = KNOWN_FLAKES
    return native.call("verdict_" + op, kwargs, **({"root": repo} if repo is not None else {}))


def _flake_note(flake: dict | None) -> list[str]:
    return _call("flake_note", {"flake": flake})


def _match_known_flake(serial: str, code: str | None, when: str) -> dict | None:
    return _call("match_known_flake", {"serial": serial, "code": code, "when": when})


def wedge_evidence(serial: str) -> dict:
    return _call("wedge_evidence", {"serial": serial})


def kernel_asserts(serial: str) -> list[str]:
    return _call("kernel_asserts", {"serial": serial})


def post_boot_check(serial: str, debugcon_raw: str | None = None) -> dict | None:
    return _call("post_boot_check", {"serial": serial, "debugcon_raw": debugcon_raw})


def classify(
    serial: str,
    *,
    reached: bool,
    fail_marker: str | None,
    debugcon: str | None,
    debugcon_raw: str | None = None,
) -> dict:
    return _call(
        "classify",
        {
            "serial": serial,
            "reached": reached,
            "fail_marker": fail_marker,
            "debugcon": debugcon,
            "debugcon_raw": debugcon_raw,
        },
    )


def kernel_runtime_base(serial: str) -> str:
    return _call("kernel_runtime_base", {"serial": serial})


def kernel_relocation(arch: str, serial: str) -> str | None:
    return _call("kernel_relocation", {"arch": arch, "serial": serial})


def faulting_module_offset(serial: str) -> tuple[str, int] | None:
    value = _call("faulting_module_offset", {"serial": serial})
    return tuple(value) if value is not None else None


def faulting_pc(serial: str) -> str | None:
    return _call("faulting_pc", {"serial": serial})


def _resolve_addr(
    repo: Path, arch: str, pc: str, *, mp: bool = False, reloc: str | None = None
) -> str | None:
    return _call("resolve_addr", {"arch": arch, "pc": pc, "mp": mp, "reloc": reloc}, repo)


def symbolicate_pc(repo: Path, arch: str, serial: str) -> str | None:
    return _call("symbolicate_pc", {"arch": arch, "serial": serial}, repo)


def _resolve_in_module(repo: Path, arch: str, pc: str, module: str, rva: int) -> str:
    return _call("resolve_in_module", {"arch": arch, "pc": pc, "module": module, "rva": rva}, repo)


def cpu_pcs(state: str) -> list[tuple[int, str]]:
    return [tuple(row) for row in _call("cpu_pcs", {"state": state})]


def cpu_sps(state: str) -> list[tuple[int, str]]:
    return [tuple(row) for row in _call("cpu_sps", {"state": state})]


def cpu_fps(state: str) -> list[tuple[int, str]]:
    return [tuple(row) for row in _call("cpu_fps", {"state": state})]


def cpu_frames(state: str) -> list[tuple[int, int, str]]:
    return [tuple(row) for row in _call("cpu_frames", {"state": state})]


def cpu_kernel_at_user_pc(state: str) -> list[tuple[int, str]]:
    return [tuple(row) for row in _call("cpu_kernel_at_user_pc", {"state": state})]


def guest_interrupt_time(state: str) -> int | None:
    return _call("guest_interrupt_time", {"state": state})


def symbolicate_cpu_state(
    repo: Path, arch: str, state: str, *, mp: bool = False, serial: str = ""
) -> list[str]:
    return _call(
        "symbolicate_cpu_state", {"arch": arch, "state": state, "mp": mp, "serial": serial}, repo
    )
