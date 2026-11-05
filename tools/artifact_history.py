"""Compatibility API for native Rust artifact mechanisms; no Python algorithm fallback."""

from __future__ import annotations

from pathlib import Path

if __package__:
    from .build_infra_compat import invoke as _invoke
else:
    from build_infra_compat import invoke as _invoke

if __package__:
    from .artifact_inputs import Ineligible
else:
    from artifact_inputs import Ineligible


def capture(repo: Path, revision: str, body: str, destination: Path) -> str:
    arguments = locals()
    return _invoke("artifact_history_capture", arguments, error_type=Ineligible)
