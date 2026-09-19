"""vibeos-cli layered configuration: committed defaults + gitignored local override + env.

Three layers, lowest-to-highest precedence:

  1. `DEFAULTS` (this file) — the committed, source-of-truth defaults. Robust: the tool
     works correctly even if no config file exists.
  2. `<repo>/.vibeos-cli.local.json` — a **gitignored** per-machine override. This is where
     a user with a special setup (e.g. the Windows-host QEMU backend) flips settings without
     touching tracked files. Deep-merged over DEFAULTS. See `.vibeos-cli.local.example.json`.
  3. environment variables (`_ENV_MAP`) — highest precedence, for ad-hoc one-off overrides,
     and to stay compatible with the existing `VIBEOS_*` knobs.

Access is dotted: `cfg.get("qemu.backend")`. JSON (stdlib) not TOML, since the venv may be
py3.10 (no `tomllib`) and the override is hand-edited rarely.
"""

from __future__ import annotations

import json
import os
from collections.abc import Mapping
from pathlib import Path
from typing import Any

LOCAL_FILENAME = ".vibeos-cli.local.json"

# Layer 1: committed defaults. The whole tool must behave identically to today when this is
# the only layer present (qemu.backend = "native" → existing Unix-socket/KVM path).
DEFAULTS: dict[str, Any] = {
    "qemu": {
        # "native": current behavior — local qemu-system-* + Unix-socket QMP/serial (KVM on
        #   x86, TCG on arm64). Works on real Linux and (nested) in WSL2.
        # "windows-host": launch the Windows-host QEMU.exe via WSL interop, media staged to a
        #   Windows-visible dir, QMP+serial over TCP (WSL listens, QEMU connects to 127.0.0.1).
        #   Escapes WSL2 nested-virt contention. See
        #   docs/engineering/windows-host-qemu-whpx-spike.md.
        #   DEFAULT (all agents on this Windows host) — validated boot ~1m35s vs
        #   contended native ~6min. A non-Windows-host env (real Linux / CI) overrides back via
        #   VIBEOS_QEMU_BACKEND=native or a .vibeos-cli.local.json with backend="native".
        "backend": "windows-host",
        "windows": {
            # Directory holding the Windows QEMU executables (qemu-system-*.exe), as a
            # WSL path under /mnt/<drive>.
            "qemu_dir": "/mnt/c/Program Files/qemu",
            # Windows-visible scratch dir (WSL path) where media (ISO/OVMF/vars) is staged so
            # the Windows QEMU can read it natively. Cleaned per run.
            "staging_dir": "/mnt/c/temp/vibeos-qemu",
            # x86 accelerator: "tcg" (pure emulation, the working default) or "whpx" (Windows
            # Hypervisor Platform hardware virt, only in qemu-system-x86_64.exe). Resolved
            # PER-ARCH at launch (backend.resolve_accel: arm64 always TCG; x86+whpx degrades
            # to TCG when the host has no hypervisor). Both x86 arches BOOT TO DESKTOP under
            # whpx as of 2026-07-03 (the historical blockers — the i386 freeldr "stack-switch
            # hang" and the amd64 uniata BUGCHECK 0x9 — are fixed: a Rust varargs bug and
            # kernel-irqchip=on respectively; see
            # docs/engineering/windows-host-qemu-whpx-spike.md). whpx stays OPT-IN
            # (VIBEOS_QEMU_WINDOWS_ACCEL=whpx or set here) while it earns gate-grade
            # reliability: its firmware CD-PIO phase is slow (vm.py scales the smoke
            # timeouts) and repeat-stability hasn't been proven yet.
            "accel": "tcg",
            # Address the Windows QEMU uses to reach our WSL-side TCP listeners. 127.0.0.1
            # works via WSL2 localhost forwarding (the reliable Windows->WSL direction).
            "host_addr": "127.0.0.1",
        },
    },
}

# Layer 3: explicit env overrides → dotted config key. Kept small + explicit so precedence is
# predictable (no magic VIBEOS_CFG_* reflection).
_ENV_MAP: dict[str, str] = {
    "VIBEOS_QEMU_BACKEND": "qemu.backend",
    "VIBEOS_QEMU_WINDOWS_DIR": "qemu.windows.qemu_dir",
    "VIBEOS_QEMU_WINDOWS_STAGING": "qemu.windows.staging_dir",
    "VIBEOS_QEMU_WINDOWS_ACCEL": "qemu.windows.accel",
    "VIBEOS_QEMU_WINDOWS_HOST_ADDR": "qemu.windows.host_addr",
}


def _deep_merge(base: dict[str, Any], over: dict[str, Any]) -> dict[str, Any]:
    """Recursively merge `over` into a copy of `base` (dicts merge; scalars/lists replace)."""
    out = dict(base)
    for k, v in over.items():
        if isinstance(v, dict) and isinstance(out.get(k), dict):
            out[k] = _deep_merge(out[k], v)
        else:
            out[k] = v
    return out


def _set_dotted(d: dict[str, Any], dotted: str, value: Any) -> None:
    keys = dotted.split(".")
    cur = d
    for k in keys[:-1]:
        nxt = cur.get(k)
        if not isinstance(nxt, dict):
            nxt = {}
            cur[k] = nxt
        cur = nxt
    cur[keys[-1]] = value


class Config:
    """A merged config tree with dotted access. Construct via `load()`."""

    def __init__(self, data: dict[str, Any]) -> None:
        self._data = data

    def get(self, dotted: str, default: Any = None) -> Any:
        cur: Any = self._data
        for k in dotted.split("."):
            if not isinstance(cur, dict) or k not in cur:
                return default
            cur = cur[k]
        return cur

    @property
    def data(self) -> dict[str, Any]:
        return self._data


def _local_overrides(repo: Path) -> dict[str, Any]:
    p = repo / LOCAL_FILENAME
    if not p.is_file():
        return {}
    try:
        loaded = json.loads(p.read_text())
    except (OSError, ValueError):
        return {}
    return loaded if isinstance(loaded, dict) else {}


def _env_overrides(environ: Mapping[str, str]) -> dict[str, Any]:
    over: dict[str, Any] = {}
    for env_key, dotted in _ENV_MAP.items():
        if env_key in environ and environ[env_key] != "":
            _set_dotted(over, dotted, environ[env_key])
    return over


def load(repo: Path, environ: Mapping[str, str] | None = None) -> Config:
    """Build the merged config: DEFAULTS < <repo>/.vibeos-cli.local.json < env."""
    environ = os.environ if environ is None else environ
    merged = _deep_merge(DEFAULTS, _local_overrides(repo))
    merged = _deep_merge(merged, _env_overrides(environ))
    return Config(merged)
