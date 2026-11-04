"""Compatibility entry points for native backend policy and media publication."""

from __future__ import annotations

import os
from contextlib import contextmanager

from tools.test_infra_compat import DedicatedSession, call


def _call(name, **kwargs):
    return call(f"backend_{name}", {"env": dict(os.environ), **kwargs})


def win_path(linux_path):
    return _call("win_path", path=str(linux_path))


def stage_file(src, staging):
    return _call("stage_file", src=str(src), staging=str(staging))


def stage_readonly_file(src, staging):
    return _call("stage_readonly_file", src=str(src), staging=str(staging))


def windows_qemu_bin(native_bin, qemu_dir, accel):
    return _call("windows_qemu_bin", native_bin=native_bin, qemu_dir=str(qemu_dir), accel=accel)


def windows_qemu_version(exe):
    return _call("windows_qemu_version", exe=exe)


def known_bad_windows_qemu(version):
    return _call("known_bad_windows_qemu", version=version)


def known_bad_allowed():
    return os.environ.get("VIBEOS_QEMU_ALLOW_KNOWN_BAD", "") == "1"


def known_bad_windows_qemu_fix(exe):
    return (
        f"Install a released QEMU build and set qemu.windows.qemu_dir (current {exe}); "
        "VIBEOS_QEMU_ALLOW_KNOWN_BAD=1 explicitly accepts this known defect."
    )


def rewrite_accel(args, accel, arch="amd64"):
    return _call("rewrite_accel", args=args, accel=accel, arch=arch)


def resolve_accel(arch, configured, whpx_ok=None):
    return _call("resolve_accel", arch=arch, configured=configured, whpx_ok=whpx_ok)


def whpx_available():
    return _call("whpx_available")


def stage_media_paths(args, staging):
    return _call("stage_media_paths", args=args, staging=str(staging))


def gateway_ip():
    return _call("gateway_ip")


def windows_free_tcp_port():
    return _call("windows_free_tcp_port")


def rewrite_netdev_for_windows(args, chardev_prefix="gf"):
    args, ids = _call("rewrite_netdev", args=args, prefix=chardev_prefix)
    return args, ids


def kill_by_uid(uid):
    return _call("kill_by_uid", uid=uid)


def windows_qemus():
    return _call("windows_qemus")


@contextmanager
def media_cache_lock(name="cache.lock"):
    with DedicatedSession() as session:
        result = session.call(
            "vm_media_cache_lock", {"name": name, "root": os.environ.get("VIBEOS_HOSTLOCK_DIR")}
        )
        try:
            yield
        finally:
            session.call("vm_media_cache_unlock", {"id": result["id"]})


def _prune_media_cache(cache, *, keep):
    _call("prune_media_cache", cache=str(cache), keep=str(keep))
