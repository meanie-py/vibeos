"""Compatibility bindings for native diagnostic preparation and sidecars."""

import sys

from tools import qemu_infra_compat as native


def _call(op, request):
    return native.call("diagnostics_" + op, request)


def prepare(request):
    from tests.qemu.cli.recordreplay import RecordReplayError

    try:
        result = _call("prepare", request)
    except native.NativeError as e:
        raise RecordReplayError(str(e)) from e
    for message in result.pop("messages", []):
        print(message, file=sys.stderr)
    return result


def sidecar(request):
    return _call("sidecar", request)
