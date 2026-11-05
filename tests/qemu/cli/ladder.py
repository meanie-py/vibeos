"""Native ladder pure-helper facade; root owns the live serial watcher."""

from __future__ import annotations

import os
import time
from pathlib import Path

from tools import qemu_infra_compat as native

ALSO_ACCEPTED: dict[str, frozenset[int]] = {}

LADDER: dict[str, int] = {
    "hello32": 0x600D,
    "gui32": 0x600D,
    "win32": 0x600D,
    "file32": 0x600D,
    "async32": 0x600D,
    "marshal32": 0x600D,
    "thread32": 0x600D,
    "fault32": 0x600D,
    "selfmod32": 0x600D,
    "reg32": 0x600D,
    # Promoted from DIAGNOSTIC 2026-09-19, the day it first reported 0x600D.
    # It was built to MEASURE guest CreateProcess while that gap was open; a
    # diagnostic guest that starts passing has finished its job, and leaving it
    # ungraded would mean a regression in the thing it proved could go unnoticed.
    "spawn32": 0x600D,
    # User APCs into 32-bit code (QueueUserAPC, ReadFileEx, waitable timers) and
    # per-thread cleanup -- Apc64.c at 32 bits.
    "apc32": 0x600D,
    # The first x86-64 guest (BTCpu64 M1, docs/engineering/wow64-x64-guest-design.md
    # Fork B). Parity in the sense this table means, one width up: the SAME binary
    # runs under FEX in 64-bit mode on arm64 and natively on amd64. Not a WoW64
    # guest, so it stages to system32 rather than syswow64 -- every check here is
    # keyed by name, and media_has_guests needs only SOME guest found, so the
    # different directory costs nothing.
    "hello64": 0x600D,
}

OPTIONAL: dict[str, int] = {
    "sqlite32": 0x600D,
    "lua32": 0x600D,
}

DIAGNOSTIC: dict[str, int] = {}

STAGED_LADDER: dict[str, int] = {
    g: 0x600D
    for g in (
        "gui64",
        "win64",
        "file64",
        "async64",
        "marshal64",
        "fault64",
        "thread64",
        "selfmod64",
        "reg64",
        "spawn64",
        "apc64",
    )
}

X64_STAGED_DIR = "system32"

LADDER_TIMEOUT_SECONDS = 420.0

OPTIONAL_EXTRA_SECONDS = 360.0

WINETEST_EXTRA_SECONDS = 200.0

DRAIN_FLOOR_ENV = "VIBEOS_LADDER_DRAIN_FLOOR"

STAGED_DIR = "syswow64"

RAMDISK_REL = "qemu/ramdisk.img"

GUEST_SOURCE: dict[str, str] = {
    "hello32": "tools/fex/hello-compute/Hello32.c",
    "gui32": "tools/fex/hello-compute/Gui32.c",
    "win32": "tools/fex/hello-compute/Win32App.c",
    "file32": "tools/fex/hello-compute/File32.c",
    "async32": "tools/fex/hello-compute/Async32.c",
    "marshal32": "tools/fex/hello-compute/Marshal32.c",
    "thread32": "tools/fex/hello-compute/Thread32.c",
    "fault32": "tools/fex/hello-compute/Fault32.c",
    "selfmod32": "tools/fex/hello-compute/Selfmod32.c",
    "reg32": "tools/fex/hello-compute/Reg32.c",
    "spawn32": "tools/fex/hello-compute/Spawn32.c",
    "apc32": "tools/fex/hello-compute/Apc64.c",
    "hello64": "tools/fex/hello-compute/Hello64.c",
    "gui64": "tools/fex/hello-compute/Gui32.c",
    "win64": "tools/fex/hello-compute/Win32App.c",
    "file64": "tools/fex/hello-compute/File32.c",
    "async64": "tools/fex/hello-compute/Async32.c",
    "marshal64": "tools/fex/hello-compute/Marshal32.c",
    "thread64": "tools/fex/hello-compute/Thread32.c",
    "fault64": "tools/fex/hello-compute/Fault32.c",
    "selfmod64": "tools/fex/hello-compute/Selfmod32.c",
    "reg64": "tools/fex/hello-compute/Reg32.c",
    "spawn64": "tools/fex/hello-compute/Spawn32.c",
    "apc64": "tools/fex/hello-compute/Apc64.c",
}

GUEST_HELPERS: dict[str, list[str]] = {
    "spawn64": ["tools/fex/hello-compute/XInject.c"],
}

GUEST_BUILDER = "tools/fex/build-hello-compute.sh"

OPTIONAL_SOURCE: dict[str, tuple[str, list[str]]] = {
    "lua32": (
        "tools/stress/lua/lua32.exe",
        ["tools/stress/lua/Lua32.c", "tools/stress/lua/build.sh"],
    ),
    "sqlite32": (
        "tools/stress/sqlite/sqlite32.exe",
        ["tools/stress/sqlite/Sqlite32.c", "tools/stress/sqlite/build.sh"],
    ),
}

GUEST_PREREQ = "build-i386/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a"

GUEST_PREREQ_X64 = "build-amd64/dll/win32/kernel32/CMakeFiles/libkernel32.dir/libkernel32.a"

GUEST_TOOLCHAIN = ".build-tools/llvm-mingw/bin/i686-w64-mingw32-clang"

CPU_LAYERS = {"wow64cpu.dll", "libwow64fex.dll"}

PROGRAM_DONE = "[WINLOGON] graded program complete"


def _config():
    return {
        "ladder": list(LADDER.items()),
        "optional": list(OPTIONAL.items()),
        "diagnostic": list(DIAGNOSTIC.items()),
        "staged_ladder": list(STAGED_LADDER.items()),
        "also_accepted": {k: list(v) for k, v in ALSO_ACCEPTED.items()},
        "guest_source": GUEST_SOURCE,
        "guest_helpers": GUEST_HELPERS,
        "optional_source": list(OPTIONAL_SOURCE.items()),
        "guest_builder": GUEST_BUILDER,
        "guest_prereq": GUEST_PREREQ,
        "guest_prereq_x64": GUEST_PREREQ_X64,
        "guest_toolchain": GUEST_TOOLCHAIN,
        "ramdisk_rel": RAMDISK_REL,
    }


def _call(op, kwargs, repo=None):
    kwargs["config"] = _config()
    if "names" in kwargs:
        kwargs["names"] = list(kwargs["names"])
    if "exe" in kwargs:
        kwargs["exe"] = str(kwargs["exe"])
    if "arch" in kwargs:
        from . import archcfg

        try:
            kwargs["media_rel"] = archcfg.arch_build(kwargs["arch"]).boot_media_rel
        except ValueError:
            pass
    return native.call("ladder_" + op, kwargs, **({"root": repo} if repo is not None else {}))


def verdict_ok(guest: str, got: int, want: int) -> bool:
    return _call("verdict_ok", {"guest": guest, "got": got, "want": want})


def staged_guests() -> list[str]:
    return _call("staged_guests", {})


def drain_floor() -> float:
    return _call("drain_floor", {"raw": os.environ.get(DRAIN_FLOOR_ENV)})


def stale_guests(repo: Path) -> list[str]:
    return _call("stale_guests", {}, repo)


def x64_refresh_deferred(repo: Path, todo: list[str]) -> list[str]:
    return _call("x64_refresh_deferred", {"todo": todo}, repo)


def guests_to_refresh(repo: Path) -> list[str]:
    return _call("guests_to_refresh", {}, repo)


def refresh_blocker(repo: Path) -> str | None:
    return _call("refresh_blocker", {}, repo)


def _newer_than(repo: Path, guest: str, exe: Path, sources: list[str]) -> list[str]:
    return _call("newer_than", {"guest": guest, "exe": exe, "sources": sources}, repo)


def _guest_names() -> set[str]:
    value = _call("guest_names", {})
    return set(value)


def _staged_per_manifest(repo: Path, arch: str, subdir: str, names: set[str]) -> set[str] | None:
    value = _call("staged_per_manifest", {"arch": arch, "subdir": subdir, "names": names}, repo)
    return set(value) if value is not None else None


def uses_ramdisk(arch: str) -> bool:
    return _call("uses_ramdisk", {"arch": arch})


def _staged_per_ramdisk(repo: Path, arch: str, subdir: str, names: set[str]) -> set[str] | None:
    value = _call("staged_per_ramdisk", {"arch": arch, "subdir": subdir, "names": names}, repo)
    return set(value) if value is not None else None


def _read_media(repo: Path, arch: str, subdir: str, names: set[str]) -> set[str] | None:
    value = _call("read_media", {"arch": arch, "subdir": subdir, "names": names}, repo)
    return set(value) if value is not None else None


def supports_wow64(repo: Path, arch: str) -> bool | None:
    return _call("supports_wow64", {"arch": arch}, repo)


def media_has_guests(repo: Path, arch: str) -> bool:
    return _call("media_has_guests", {"arch": arch}, repo)


def media_missing_runtime(repo: Path, arch: str) -> list[str]:
    return _call("media_missing_runtime", {"arch": arch}, repo)


def media_optional_staged(repo: Path, arch: str) -> list[str]:
    return _call("media_optional_staged", {"arch": arch}, repo)


def gate_decision(repo: Path, arch: str) -> tuple[list[str], bool, str]:
    return tuple(_call("gate_decision", {"arch": arch}, repo))


def markers() -> list[str]:
    return _call("markers", {})


def _optional_in_flight(buf: str) -> list[str]:
    return _call("optional_in_flight", {"buf": buf})


def verdicts(text: str) -> dict[str, int]:
    return _call("verdicts", {"text": text})


def winetests(text: str) -> dict[str, int]:
    return _call("winetests", {"text": text})


def winetests_crashed(text: str) -> dict[str, str]:
    return _call("winetests_crashed", {"text": text})


def winetests_timed_out(text: str) -> list[str]:
    return _call("winetests_timed_out", {"text": text})


def winetests_lost(text: str) -> list[str]:
    return _call("winetests_lost", {"text": text})


def sweeps_unfinished(text: str) -> list[str]:
    return _call("sweeps_unfinished", {"text": text})


def check(text: str, optional_staged: list[str] | None = None) -> dict | None:
    return _call("check", {"text": text, "optional_staged": optional_staged})


def summary(text: str) -> str:
    return _call("summary", {"text": text})


def _diagnostic_tail(seen: dict[str, int]) -> str:
    return _call("diagnostic_tail", {"seen": seen})


def watch(
    vm,
    seconds: float = LADDER_TIMEOUT_SECONDS,
    seen: str = "",
    optional_staged: list[str] | None = None,
    program_done_expected: bool = False,
) -> str:
    with native.DedicatedSession() as session:
        return session.call(
            "ladder_watch",
            {
                "config": _config(),
                "seconds": seconds,
                "seen": seen,
                "optional_staged": optional_staged,
                "program_done_expected": program_done_expected,
                "drain_floor_raw": os.environ.get(DRAIN_FLOOR_ENV),
                "optional_extra_seconds": OPTIONAL_EXTRA_SECONDS,
                "winetest_extra_seconds": WINETEST_EXTRA_SECONDS,
            },
            callbacks={
                "ladder_clock": lambda _a: time.monotonic(),
                "ladder_read_serial": lambda a: vm.serial.read_available(timeout=a["timeout"]),
            },
        )
