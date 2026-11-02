"""Compatibility API for native Rust artifact mechanisms; no Python algorithm fallback."""

from __future__ import annotations

from pathlib import Path

if __package__:
    from .build_infra_compat import NativeError
    from .build_infra_compat import invoke as _invoke
else:
    from build_infra_compat import NativeError
    from build_infra_compat import invoke as _invoke


class SnapshotError(NativeError, RuntimeError):
    pass


def _verified_copy(source: Path, output: Path, expected: dict) -> int:
    arguments = locals()
    return _invoke("artifact_worker__verified_copy", arguments, error_type=SnapshotError)


def capture(
    repo_root: Path,
    body: str,
    context: dict,
    destination: Path,
    *,
    expected_projection_key: str | None = None,
    features: tuple[str, ...] = (),
    default_features: bool = True,
    profile: str = "release",
    force_staticlib: bool = False,
    cargo: str = "cargo",
    workspace: Path | None = None,
    additional_inputs: dict[str, dict] | None = None,
) -> dict:
    arguments = locals()
    return _invoke("artifact_worker_capture", arguments, error_type=SnapshotError)


def executor_requirements() -> dict:
    arguments = locals()
    return _invoke("artifact_worker_executor_requirements", arguments, error_type=SnapshotError)
