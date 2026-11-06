"""Compatibility API for the native deterministic QEMU trace mechanisms."""

from __future__ import annotations

import os
import subprocess
import time
from pathlib import Path
from typing import NamedTuple

from tools import qemu_infra_compat as native

DEFAULT_ICOUNT_SHIFT = 3
FATAL_MARKER = "[fatal]"
_SUPPORTED_REPLAY_VERSION = 0xE0200C
_V12_ASYNC_IDS = frozenset(range(3, 10))
_V12_SHUTDOWN_IDS = frozenset(range(10, 22))
ENV_VAR = "VIBEOS_QEMU_RR"
SHIFT_ENV_VAR = "VIBEOS_QEMU_RR_SHIFT"
BREAK_ENV_VAR = "VIBEOS_QEMU_RR_BREAK"


class RecordReplayError(RuntimeError):
    """A requested configuration or trace cannot be replayed faithfully."""


def _call(op, **kwargs):
    try:
        return native.call("record_replay_" + op, kwargs)
    except native.NativeError as e:
        raise RecordReplayError(str(e)) from e


def parse_drive(spec):
    return dict(_call("parse_drive", spec=spec))


def format_drive(d):
    return _call("format_drive", drive=list(d.items()))


def _sha256(path):
    return _call("sha256", path=path)


def stage_image(src, dst, src_format):
    _call("stage_image", src=src, dst=dst, format=src_format)


def transform(argv, *, mode, trace_dir, icount_shift=DEFAULT_ICOUNT_SHIFT, stage=True):
    argv, media = _call(
        "transform", argv=argv, mode=mode, trace_dir=trace_dir, shift=icount_shift, stage=stage
    )
    return argv, dict(media)


def is_x86_argv(argv):
    return _call("is_x86", argv=argv)


def _version():
    result = subprocess.run(["qemu-system-aarch64", "--version"], capture_output=True, text=True)
    if result.returncode:
        raise RecordReplayError("cannot read QEMU version: " + result.stderr.strip())
    lines = result.stdout.splitlines()
    return lines[0] if lines else "unknown"


def write_manifest(trace_dir, *, arch, argv, media, **extra):
    return Path(
        _call(
            "write_manifest",
            trace_dir=trace_dir,
            arch=arch,
            argv=argv,
            media=media,
            extra=extra,
            created=time.strftime("%Y-%m-%dT%H:%M:%S"),
            qemu_version=_version(),
        )
    )


def trace_read_position(trace_dir):
    return _call("trace_position", trace_dir=trace_dir)


def note_run(trace_dir, *, mode, serial_log):
    return Path(_call("note_run", trace_dir=trace_dir, mode=mode, serial_log=serial_log))


def last_run(trace_dir):
    return _call("last_run", trace_dir=trace_dir)


def snapshot_pristine(trace_dir):
    _call("snapshot_pristine", trace_dir=trace_dir)


def prepare_replay(trace_dir):
    _call("prepare_replay", trace_dir=trace_dir)


def load_manifest(trace_dir):
    return _call("load_manifest", trace_dir=trace_dir)


def check_trace(trace_dir):
    return _call("check_trace", trace_dir=trace_dir, version=_version())


def _host_shape(argv):
    return _call("host_shape", argv=argv)


def assert_same_shape(trace_dir, argv):
    _call("assert_same_shape", trace_dir=trace_dir, argv=argv)


def event_name(eid):
    return _call("event_name", eid=eid)


class ReplayEvent(NamedTuple):
    offset: int
    eid: int
    icount: int

    @property
    def name(self):
        return event_name(self.eid)


def walk_events(trace_dir):
    cursor = None
    while True:
        page = _call("events_page", trace_dir=trace_dir, cursor=cursor, limit=512)
        for e in page["events"]:
            yield ReplayEvent(**e)
        cursor = page["next"]
        if cursor is None:
            return


def instruction_boundaries(trace_dir):
    return _call("boundaries", trace_dir=trace_dir)


def events_around_offset(trace_dir, offset, before=5, after=5):
    a, b = _call("around", trace_dir=trace_dir, offset=offset, before=before, after=after)
    return [ReplayEvent(**e) for e in a], [ReplayEvent(**e) for e in b]


def trace_is_provably_complete(trace_dir):
    return _call("complete", trace_dir=trace_dir)


def snap_break_icount(trace_dir, want):
    return _call("snap_break", trace_dir=trace_dir, want=want)


def env_shift():
    return _call("env_shift", raw=os.environ.get(SHIFT_ENV_VAR, ""))


def env_break():
    return _call("env_break", raw=os.environ.get(BREAK_ENV_VAR, ""))


def env_request():
    result = _call("env_request", raw=os.environ.get(ENV_VAR, ""))
    return None if result is None else (result[0], Path(result[1]))
