"""Compatibility facade for the native chrome provider (vibeos-build payload_providers.rs)."""

import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
if str(REPO) not in sys.path:
    sys.path.insert(0, str(REPO))
from tools.build_infra_compat import call  # noqa: E402


def stage_entries(arch):
    try:
        return call("payload_providers_stage_entries", {"spec": "chrome", "arch": arch}, REPO)
    except ValueError as e:
        raise SystemExit(str(e)) from e
