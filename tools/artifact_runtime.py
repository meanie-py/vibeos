"""Compatibility front door for native artifact requests and outer-build supervision."""

from __future__ import annotations

import os
import sys
from contextlib import contextmanager
from pathlib import Path
from typing import Any, NoReturn

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools import build_infra_compat as bridge

SCHEMA = "vibeos-artifact-runtime-v1"
TARGET = "x86_64-pc-windows-gnu"
KERNEL_FLAGS = "-C force-frame-pointers=yes"
GC_INTERVAL_SECONDS = 86400


class RuntimeError_(RuntimeError):
    pass


def _raise(error) -> NoReturn:
    from tools import artifact_config, artifact_store

    if error.kind in artifact_store._ERRORS:
        artifact_store._raise(error)
    types = {
        "RuntimeError_": RuntimeError_,
        "RuntimeError": RuntimeError,
        "ConfigError": artifact_config.ConfigError,
        "ValueError": ValueError,
        "KeyError": KeyError,
        "OSError": OSError,
        "KeyboardInterrupt": KeyboardInterrupt,
        "InterruptedError": InterruptedError,
    }
    raise types.get(error.kind, RuntimeError_)(str(error)) from error


def _invoke(op, arguments) -> Any:
    try:
        return bridge.call("artifact_runtime_" + op, arguments)
    except bridge.NativeError as error:
        _raise(error)


def _environment(_arguments):
    return dict(os.environ)


def _call(connection, op, arguments) -> Any:
    try:
        return connection.call(
            op,
            {**arguments, "environment": dict(os.environ), "environment_callback": True},
            callbacks={"environment": _environment},
        )
    except bridge.NativeError as error:
        _raise(error)


def _native_host(_api):
    if _api is not None:
        raise RuntimeError_(
            "Artifact host implementations now use the native RuntimeHost interface"
        )


@contextmanager
def supervise(
    repo,
    store_root,
    toolchain,
    context_dir,
    clean=False,
    verify=False,
    *,
    repair_action=None,
    arch=None,
    _api=None,
):
    _native_host(_api)
    with bridge.DedicatedSession("artifact-supervisor-service") as connection:
        path = Path(
            _call(
                connection,
                "open",
                {
                    "repo": repo,
                    "store_root": store_root,
                    "toolchain": toolchain,
                    "context_dir": context_dir,
                    "options": {
                        "clean": clean,
                        "verify": verify,
                        "repair_action": repair_action,
                        "arch": arch,
                    },
                },
            )
        )
        try:
            yield path
        except BaseException as original:
            failure = {"type": type(original).__name__, "message": str(original)}
            try:
                _call(connection, "close", {"outer_failure": failure})
            except BaseException as final:
                if type(final) is not type(original) or str(final) != str(original):
                    raise final from original
            raise
        else:
            _call(connection, "close", {"outer_failure": None})


def supervisor_config(path):
    return _invoke("supervisor_config", locals())


def record_outer_result(supervisor, exit_code, *, _api=None):
    _native_host(_api)
    return _invoke("record_outer_result", {"supervisor": supervisor, "exit_code": exit_code})


def request(
    supervisor,
    body,
    consumer,
    *,
    features=(),
    default_features=True,
    kernel=False,
    flags="",
    architecture="amd64",
    target=TARGET,
    build_std=False,
    emit_ir=False,
    force_staticlib=False,
    cargo_cwd=None,
    _api=None,
):
    _native_host(_api)
    options = {
        "features": features,
        "default_features": default_features,
        "kernel": kernel,
        "flags": flags,
        "architecture": architecture,
        "target": target,
        "build_std": build_std,
        "emit_ir": emit_ir,
        "force_staticlib": force_staticlib,
        "cargo_cwd": cargo_cwd,
    }
    with bridge.DedicatedSession("artifact-supervisor-service") as connection:
        return _call(
            connection,
            "request",
            {"supervisor": supervisor, "body": body, "consumer": consumer, "options": options},
        )


def repair(supervisor, action, *, _api=None):
    _native_host(_api)
    with bridge.DedicatedSession("artifact-supervisor-service") as connection:
        return _call(connection, "repair", {"supervisor": supervisor, "action": action})


def retire_build_refs(supervisor, *, _api=None):
    _native_host(_api)
    return _invoke("retire_build_refs", {"supervisor": supervisor})


def lane_ref_alive(reference: dict, *, canonical_root=None):
    return _invoke("lane_ref_alive", locals())


def scheduled_gc(store_root: Path, *, now=None, force=False):
    return _invoke("scheduled_gc", locals())


def protocol_lines(result):
    return _invoke("protocol_lines", locals())


def _watches(repo, discovery):
    return _invoke("_watches", locals())


def _receipt_stats(activity):
    return _invoke("_receipt_stats", locals())


def _canonical(value):
    from tools.artifact_inputs import canonical

    return canonical(value)


def _key(value):
    from tools.artifact_inputs import digest

    return digest(value)


def main():
    tool = bridge.REPO / "tools/build-infra"
    os.execv(str(tool), [str(tool), "artifact-runtime", *sys.argv[1:]])


if __name__ == "__main__":
    main()
