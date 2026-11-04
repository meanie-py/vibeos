"""Compatibility API for native Rust artifact mechanisms; no Python algorithm fallback."""

from __future__ import annotations

from pathlib import Path
from typing import Any

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


class Ineligible(NativeError):
    pass


SCHEMA = "vibeos-body-input-projection-v1-spike"


def canonical(value: Any) -> bytes:
    arguments = locals()
    return _invoke("artifact_inputs_canonical", arguments, error_type=Ineligible).encode("ascii")


def digest(value: Any) -> str:
    arguments = locals()
    return _invoke("artifact_inputs_digest", arguments, error_type=Ineligible)


def _toml(value: Any) -> str:
    arguments = locals()
    return _invoke("artifact_inputs__toml", arguments, error_type=Ineligible)


def _without_comments(text: str) -> str:
    arguments = locals()
    return _invoke("artifact_inputs__without_comments", arguments, error_type=Ineligible)


def explain(before: dict, after: dict) -> list[dict]:
    arguments = locals()
    return _invoke("artifact_inputs_explain", arguments, error_type=Ineligible)


def unchanged(discovery: dict) -> bool:
    arguments = locals()
    return _invoke("artifact_inputs_unchanged", arguments, error_type=Ineligible)


def discover(
    repo_root: Path,
    body: str,
    context: dict | None = None,
    *,
    features: tuple[str, ...] = (),
    default_features: bool = True,
    profile: str = "release",
    force_staticlib: bool = False,
    cargo: str = "cargo",
    workspace: Path | None = None,
) -> dict:
    arguments = locals()
    return _invoke("artifact_inputs_discover", arguments, error_type=Ineligible)
