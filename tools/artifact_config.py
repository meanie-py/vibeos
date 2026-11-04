"""Compatibility API for native Rust artifact mechanisms; no Python algorithm fallback."""

from __future__ import annotations

import os
from pathlib import Path

if __package__:
    from .build_infra_compat import NativeError
    from .build_infra_compat import invoke as _invoke
else:
    from build_infra_compat import NativeError
    from build_infra_compat import invoke as _invoke

try:
    import tomllib as tomllib  # type: ignore[reportMissingImports]
except ImportError:
    import tomli as tomllib  # type: ignore[reportMissingImports]  # noqa: F401


class ConfigError(NativeError):
    pass


TARGET = "x86_64-pc-windows-gnu"
CANONICAL_ROOT = Path("/tmp/vibeos")
ARCHES = ("i386", "amd64", "arm64")


def cargo_scope(repo: Path) -> list[Path]:
    arguments = locals()
    return [
        Path(p) for p in _invoke("artifact_config_cargo_scope", arguments, error_type=ConfigError)
    ]


def mapped_cwd(repo: Path, cargo_cwd: Path | None, *, kernel: bool) -> Path:
    arguments = locals()
    arguments["canonical"] = CANONICAL_ROOT
    return Path(_invoke("artifact_config_mapped_cwd", arguments, error_type=ConfigError))


def _environment(environment: dict, broker: Path | None) -> None:
    arguments = locals()
    return _invoke("artifact_config__environment", arguments, error_type=ConfigError)


def _config_bytes(path: Path) -> bytes:
    arguments = locals()
    return bytes(_invoke("artifact_config__config_bytes", arguments, error_type=ConfigError))


def _allowed(
    config: dict, path: Path, requested: str | None = None, rustflags_set: bool = False
) -> None:
    arguments = locals()
    return _invoke("artifact_config__allowed", arguments, error_type=ConfigError)


def _audit(
    repo: Path,
    cwds: list[Path],
    *,
    environment: dict | None,
    broker: Path | None,
    requested: str | None = None,
    rustflags_set: bool = False,
) -> dict:
    arguments = locals()
    arguments["canonical"] = CANONICAL_ROOT
    arguments["environment"] = dict(os.environ if environment is None else environment)
    return _invoke("artifact_config__audit", arguments, error_type=ConfigError)


def audit_scope(repo: Path, *, environment: dict | None = None, broker: Path | None = None) -> dict:
    arguments = locals()
    arguments["canonical"] = CANONICAL_ROOT
    arguments["environment"] = dict(os.environ if environment is None else environment)
    return _invoke("artifact_config_audit_scope", arguments, error_type=ConfigError)


def audit_request(
    repo: Path,
    *,
    cargo_cwd: Path | None = None,
    kernel: bool = False,
    environment: dict | None = None,
    broker: Path | None = None,
    target: str = TARGET,
    rustflags_set: bool = False,
) -> dict:
    arguments = locals()
    arguments["canonical"] = CANONICAL_ROOT
    arguments["environment"] = dict(os.environ if environment is None else environment)
    return _invoke("artifact_config_audit_request", arguments, error_type=ConfigError)
