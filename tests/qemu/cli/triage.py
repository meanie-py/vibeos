"""Failure triage (plan §14, §20.1). Cheap tier today: classify from serial +
debugcon, surface RUST_PANIC file:line and user-mode panics, match a known-flake
catalog, symbolicate via the offline index. Full gdb attach is P2 (smoke's
QemuVM doesn't expose a gdbstub yet); this never blocks and degrades gracefully.
"""

from __future__ import annotations

import re
import subprocess
from pathlib import Path

_BUGCHECK = re.compile(r"BugCheck\s+(0x[0-9A-Fa-f]+)|\*\*\* STOP:?\s*(0x[0-9A-Fa-f]+)")
_RUST_PANIC = re.compile(r"RUST PANIC HIT.*?PANIC LOC:\s*(\S+)", re.S)
_RUST_PANIC2 = re.compile(r"panicked at\s+'([^']*)',\s*(\S+)")
_PC = re.compile(r"\bPC[:=]\s*(0x[0-9A-Fa-f]+)|\b(?:RIP|EIP)[:=]\s*(0x[0-9A-Fa-f]+)")

# Documented intermittent flakes single-run gates have masked (see
# docs/engineering/deterministic-builds-plan.md + tasks #42/#43). Matching one
# re-labels a failure KNOWN_FLAKE so a *documented* flake is instantly told apart
# from a fresh regression (an unmatched/unknown red still classifies and blocks).
# Each entry: `when` ("bugcheck"|"hang", default "bugcheck"), optional `code`,
# `any` (all substrings present), `none` (all substrings ABSENT). Signatures are
# specific; they RETIRE when the lane owner lands the root fix (verify-gone).
KNOWN_FLAKES = [
    {
        # cross-worktree _tWinMain 0xF4 race (smss-csrss/ntdll/win32k all hit it).
        "name": "tWinMain_0xF4_race",
        "when": "bugcheck",
        "any": ["[EXPLORER] _tWinMain"],
        "code": "0xF4",
        "note": "known win32k/smss suspended-thread race; re-run to confirm",
    },
    {
        # i386: winlogon.rs:728 spawns nettest.exe UNCONDITIONALLY every boot;
        # it races explorer's csrss/win32k handoff so explorer never reaches
        # _tWinMain (~1/3 i386 boots). Pre-existing (netstack-landed). Root fix:
        # gate the nettest spawn to the net-test config (netstack owns).
        "name": "i386_nettest_explorer_race",
        "when": "hang",
        "any": ["[WINLOGON] nettest.exe created"],
        "none": ["[EXPLORER] _tWinMain entered"],
        "note": "i386 flake: winlogon nettest spawn races explorer handoff; netstack fix pending",
    },
    {
        # arm64: PspExitThread -> LpcExitThread System-thread-exit/timer race,
        # pre-existing intermittent on mainline (arm64-love's clock fix makes it
        # deterministic, doesn't cause it). Root fix owned by arm64-love.
        "name": "arm64_lpcexitthread_race",
        "when": "hang",
        "any": ["LpcExitThread"],
        "none": ["[EXPLORER] _tWinMain entered"],
        "note": "arm64 flake: PspExitThread->LpcExitThread exit race; arm64-love fix pending",
    },
]


def _match_known_flake(serial: str, code: str | None, when: str) -> dict | None:
    for fl in KNOWN_FLAKES:
        if fl.get("when", "bugcheck") != when:
            continue
        # entry keyed on a bugcheck code → require a matching code
        if fl.get("code") and (not code or code.lower() != fl["code"].lower()):
            continue
        if not all(s in serial for s in fl.get("any", [])):
            continue
        if any(s in serial for s in fl.get("none", [])):
            continue
        return fl
    return None


def classify(serial: str, *, reached: bool, fail_marker: str | None, debugcon: str | None) -> dict:
    """Return {classification, salient[], known_flake, fields...}."""
    salient: list[str] = []
    fields: dict = {}

    mp = _RUST_PANIC.search(serial) or _RUST_PANIC2.search(serial)
    if mp:
        loc = mp.groups()[-1]
        fields["panic_loc"] = loc
        salient.append(f"RUST PANIC at {loc}")
        return {"classification": "RUST_PANIC", "salient": salient, "known_flake": None, **fields}

    mb = _BUGCHECK.search(serial)
    code = None
    if mb:
        code = mb.group(1) or mb.group(2)
        fields["bugcheck"] = code
        salient.append(f"BugCheck {code}")
        line = next((ln.strip() for ln in serial.splitlines() if "BugCheck" in ln), "")
        if line:
            salient.append(line[:140])
        flake = _match_known_flake(serial, code, "bugcheck")
        return {
            "classification": "KNOWN_FLAKE" if flake else "BUGCHECK",
            "salient": salient + ([flake["note"]] if flake else []),
            "known_flake": flake["name"] if flake else None,
            **fields,
        }

    if fail_marker:
        salient.append(f"hit fail marker: {fail_marker}")
        return {"classification": "FAIL_MARKER", "salient": salient, "known_flake": None, **fields}

    if not reached:
        # hang/timeout — surface the last debugcon stage marker for pre-serial deaths
        if debugcon:
            tail = [ln for ln in debugcon.splitlines() if ln.strip()][-3:]
            salient += [f"debugcon: {ln}" for ln in tail]
        last = [ln for ln in serial.splitlines() if ln.strip()][-2:]
        salient += [f"serial tail: {ln[:120]}" for ln in last]
        # A hang whose serial matches a documented flake signature is labeled
        # KNOWN_FLAKE (told apart from a fresh regression); an unmatched hang
        # stays HANG_TIMEOUT and blocks as before.
        flake = _match_known_flake(serial, None, "hang")
        return {
            "classification": "KNOWN_FLAKE" if flake else "HANG_TIMEOUT",
            "salient": salient + ([flake["note"]] if flake else []),
            "known_flake": flake["name"] if flake else None,
            **fields,
        }

    return {"classification": "UNKNOWN", "salient": salient, "known_flake": None}


def symbolicate_pc(repo: Path, arch: str, serial: str) -> str | None:
    """Best-effort: resolve a faulting PC via the offline index. Skips silently
    if the index/tool is missing (graceful degradation)."""
    m = _PC.search(serial)
    if not m:
        return None
    pc = next(g for g in m.groups() if g)
    tool = repo / "qemu" / "addr2sym.sh"
    if not tool.is_file():
        return None
    try:
        cp = subprocess.run(
            ["bash", str(tool), arch, pc], cwd=repo, capture_output=True, text=True, timeout=30
        )
        out = cp.stdout.strip()
        return f"{pc} -> {out}" if out else None
    except (OSError, subprocess.SubprocessError):
        return None
