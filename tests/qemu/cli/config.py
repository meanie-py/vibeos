"""Layered CLI configuration, resolved by the shared native core."""

from __future__ import annotations

import os
from collections.abc import Mapping
from pathlib import Path
from typing import Any

from tools import build_infra_compat as native

LOCAL_FILENAME = ".vibeos-cli.local.json"


def _call(action, **kwargs):
    return native.call("cli_artifacts_config", {"action": action, **kwargs})


DEFAULTS: dict[str, Any] = _call("defaults")


def _deep_merge(base, over):
    return _call("merge", base=base, over=over)


def _set_dotted(data, dotted, value):
    result = _call("set", data=data, key=dotted, value=value)
    data.clear()
    data.update(result)


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
    # This compatibility helper exposes only the local layer; loading/merging is native.
    return native.call("cli_artifacts_config", {"action": "local", "repo": repo})


def _env_overrides(environ: Mapping[str, str]) -> dict[str, Any]:
    return _call("env", env=dict(environ))


def load(repo: Path, environ: Mapping[str, str] | None = None) -> Config:
    return Config(
        _call(
            "load",
            repo=repo,
            env=dict(os.environ if environ is None else environ),
            defaults=DEFAULTS,
        )
    )
