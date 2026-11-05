"""Compatibility facade for the native vendordrv provider."""

import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
if str(REPO) not in sys.path:
    sys.path.insert(0, str(REPO))
from tools.build_infra_compat import call  # noqa: E402


def stage_entries(arch):
    try:
        return call("payload_providers_stage_entries", {"spec": "vendordrv", "arch": arch}, REPO)
    except ValueError as e:
        raise SystemExit(str(e)) from e


PREFERRED_DATE = "12/31/2099"


def _preferred_inf(src):
    try:
        return Path(call("payload_providers_preferred_inf", {"src": src}, REPO))
    except ValueError as e:
        raise SystemExit(str(e)) from e
