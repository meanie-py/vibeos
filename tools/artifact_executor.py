"""Compatibility API for the native isolated artifact executor."""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import TYPE_CHECKING, Any, NoReturn

if __package__ in {None, ""}:
    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from tools import build_infra_compat as bridge

AMD64 = "x86_64-pc-windows-gnu"
_metadata = None

if TYPE_CHECKING:
    PIN: str
    CANON: Path
    WORKSPACE: Path
    IMPLEMENTATION_REPO: Path
    PREPARED_SCHEMA: str
    TARGETS: dict[str, str]
    FLAGS: dict[tuple[str, bool], str]


def __getattr__(name: str) -> Any:
    global _metadata
    if name not in {
        "PIN",
        "CANON",
        "WORKSPACE",
        "IMPLEMENTATION_REPO",
        "PREPARED_SCHEMA",
        "TARGETS",
        "FLAGS",
    }:
        raise AttributeError(name)
    if _metadata is None:
        _metadata = bridge.call("artifact_executor_constants", {})
    value = _metadata[name]
    if name in {"CANON", "WORKSPACE", "IMPLEMENTATION_REPO"}:
        return Path(value)
    if name == "TARGETS":
        return dict(value)
    if name == "FLAGS":
        return {(arch, kernel): flags for arch, kernel, flags in value}
    return value


def _raise(error) -> NoReturn:
    if error.kind == "KeyboardInterrupt":
        raise KeyboardInterrupt(str(error)) from error
    if error.kind == "InterruptedError":
        raise InterruptedError(str(error)) from error
    raise RuntimeError(str(error)) from error


def _invoke(op, arguments) -> Any:
    try:
        if op in {"run", "prepare"}:
            with bridge.DedicatedSession("--api-jsonl") as connection:
                return connection.call("artifact_executor_" + op, arguments)
        return bridge.call("artifact_executor_" + op, arguments)
    except bridge.NativeError as error:
        _raise(error)


def file_digest(path: Path) -> dict:
    return _invoke("file_digest", locals())


def allocated(root: Path) -> dict:
    return _invoke("allocated", locals())


def value_digest(value: dict) -> str:
    return _invoke("value_digest", locals())


def implementation_identity() -> dict:
    return _invoke("implementation_identity", locals())


def tree_identity(root: Path) -> dict:
    return _invoke("tree_identity", locals())


def _provisioned_copy(tool: dict) -> dict:
    return _invoke("_provisioned_copy", locals())


def provision(toolchain: Path) -> dict:
    return _invoke("provision", locals())


def normalize(snapshot: Path, projection: dict, destination: Path) -> dict:
    return _invoke("normalize", locals())


def launch(config_path: Path, probe: bool = False) -> list[str]:
    # Imports execute before chroot. Script-style launch must work without an
    # ambient PYTHONPATH; the compiler still receives only config['env'].
    return _invoke("launch", locals())


def validate_graph(output: Path, descriptor: dict, config: dict) -> dict:
    return _invoke("validate_graph", locals())


def prepared_identity(config: dict, output: Path, context: dict) -> dict:
    return _invoke("prepared_identity", locals())


def validate_prepared(output: Path, report: dict) -> dict:
    return _invoke("validate_prepared", locals())


def request_context(
    toolchain: Path,
    provisioned: dict,
    body: str,
    *,
    features: tuple[str, ...] = (),
    default_features: bool = True,
    kernel: bool = True,
    target: str = AMD64,
    flags: str | None = None,
) -> dict:
    return _invoke("request_context", locals())


def prepare(
    repo: Path,
    body: str,
    toolchain: Path,
    output: Path,
    *,
    features: tuple[str, ...] = (),
    default_features: bool = True,
    kernel: bool = True,
    provisioned: dict | None = None,
    target: str = AMD64,
    flags: str | None = None,
) -> dict:
    return _invoke("prepare", locals())


def run(output: Path) -> dict:
    return _invoke("run", locals())


def inside(config: dict, *, probe=False):
    # A one-shot inherited memfd carries the DTO across exec; native closes it before cargo.
    binary = bridge.call("artifact_implementation_identity", {})["executable"]
    fd = os.memfd_create("artifact-worker-config", flags=0)
    try:
        payload = json.dumps(config).encode()
        with os.fdopen(os.dup(fd), "wb") as stream:
            stream.write(payload)
        os.lseek(fd, 0, os.SEEK_SET)
        os.set_inheritable(fd, True)
        # --root as the native namespace entry passes it: without one the binary
        # canonicalizes its compile-time default, which does not exist when it was
        # provisioned inside a canon-run mount ("build-infra: No such file or directory").
        os.execv(
            binary,
            [
                binary,
                "--root",
                "/",
                "artifact-executor",
                "inside-fd-probe" if probe else "inside-fd",
                str(fd),
            ],
        )
    finally:
        os.close(fd)


def main():
    tool = bridge.REPO / "tools/build-infra"
    os.execv(str(tool), [str(tool), "artifact-executor", *sys.argv[1:]])


if __name__ == "__main__":
    main()
