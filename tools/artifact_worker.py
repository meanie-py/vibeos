"""Verified source capture for the immutable-body execution spike.

Keys describe copied bytes exclusively, never an atomic original-checkout state.
There is no build launcher: file permissions do not isolate a compiler. Original
rediscovery detects changes but is not a proof against concurrent source writers.
"""

from __future__ import annotations

import hashlib
import os
import stat
import time
from pathlib import Path
from typing import Any

from tools.artifact_inputs import canonical, discover


class SnapshotError(RuntimeError):
    """Capture could not prove its declared prerequisites."""


def _within(path: Path, root: Path) -> bool:
    return path == root or root in path.parents


def _verified_copy(source: Path, output: Path, expected: dict) -> int:
    before = source.lstat()
    if not stat.S_ISREG(before.st_mode):
        raise SnapshotError(f"snapshot input is not a regular file: {source}")
    payload = source.read_bytes()
    after = source.lstat()
    identity = lambda item: (  # noqa: E731
        item.st_dev,
        item.st_ino,
        item.st_size,
        item.st_mtime_ns,
        item.st_ctime_ns,
    )
    if identity(before) != identity(after):
        raise SnapshotError(f"snapshot input changed during copy: {source}")
    if (
        len(payload) != expected["bytes"]
        or hashlib.sha256(payload).hexdigest() != expected["sha256"]
    ):
        raise SnapshotError(f"snapshot input differs from discovery: {source}")
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("xb") as destination:
        destination.write(payload)
    if hashlib.sha256(output.read_bytes()).hexdigest() != expected["sha256"]:
        raise SnapshotError(f"snapshot destination verification failed: {output}")
    return len(payload)


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
    """Copy verified discovered bytes, then derive identity on the captured tree.

    A source edit detected during capture fails loudly. The snapshot is a definite
    set of bytes, but may span edits just like a conventional compiler's reads;
    no atomic-checkout provenance is claimed. Later execution must mount this
    tree read-only, and lane acceptance must revalidate current inputs.
    """
    repo = Path(repo_root).resolve()
    destination = Path(destination).absolute()
    if _within(destination.resolve(), repo):
        raise SnapshotError("snapshot destination must be outside the live source repository")
    started = time.monotonic()
    arguments: dict[str, Any] = {
        "features": features,
        "default_features": default_features,
        "profile": profile,
        "force_staticlib": force_staticlib,
        "cargo": cargo,
        "workspace": workspace,
    }
    discovery = discover(repo, body, context, **arguments)
    if not discovery["eligible"]:
        raise SnapshotError("source discovery ineligible: " + "; ".join(discovery["reasons"]))
    if expected_projection_key and expected_projection_key != discovery["source_projection_key"]:
        raise SnapshotError("source projection changed before source capture")
    reads = dict(discovery["provenance"]["reads"])
    for relative, entry in (additional_inputs or {}).items():
        if relative in reads and reads[relative] != entry:
            raise SnapshotError(f"additional input conflicts with discovery: {relative}")
        reads[relative] = entry
    destination.mkdir(parents=True, exist_ok=False)
    copied = 0
    for relative, entry in sorted(reads.items()):
        path = Path(relative)
        if path.is_absolute() or ".." in path.parts:
            raise SnapshotError(f"invalid snapshot logical path: {relative}")
        source = repo / path
        if not _within(source.resolve(), repo):
            raise SnapshotError(f"snapshot source escapes repository: {relative}")
        copied += _verified_copy(source, destination / path, entry)
    # Preserve empty directories too: source-root membership is an input.
    for root, package in discovery["descriptor"]["packages"].items():
        for directory in package["inventory"]["directories"]:
            (destination / root / directory).mkdir(parents=True, exist_ok=True)
    snapshot_arguments = dict(arguments)
    if workspace is not None:
        snapshot_arguments["workspace"] = destination / Path(workspace).resolve().relative_to(repo)
    captured = discover(destination, body, context, **snapshot_arguments)
    if (
        not captured["eligible"]
        or captured["source_projection_key"] != discovery["source_projection_key"]
    ):
        raise SnapshotError("copied source projection does not match the discovered byte inventory")
    current = discover(repo, body, context, **arguments)
    if (
        not current["eligible"]
        or current["source_projection_key"] != captured["source_projection_key"]
        or current["provenance"]["reads"] != discovery["provenance"]["reads"]
    ):
        raise SnapshotError("source input inventory changed during source capture")
    report = {
        "schema": "vibeos-body-source-capture-v1-spike",
        "source_projection_key": captured["source_projection_key"],
        "action_key": captured["action_key"],
        "checkout_consistency": "captured bytes only; original checkout not claimed atomic",
        "copied_files": len(reads),
        "copied_bytes": copied,
        "reads": reads,
        "source_manifest_policy": "raw manifests preserved; semantic projection rederived",
        "execution_status": "source capture only; compiler isolation and ABI command unproven",
        "elapsed_seconds": time.monotonic() - started,
    }
    (destination / "artifact-source-capture.json").write_bytes(canonical(report))
    for current_root, directories, files in os.walk(destination, topdown=False):
        for name in files:
            os.chmod(Path(current_root) / name, 0o444)
        for name in directories:
            os.chmod(Path(current_root) / name, 0o555)
    os.chmod(destination, 0o555)
    return report


def executor_requirements() -> dict:
    """Unimplemented requirements, explicit so capture cannot masquerade as a worker."""
    return {
        "build_supported": False,
        "required_before_build": [
            "Key only captured immutable bytes; do not claim atomic original-checkout provenance",
            "Use caller's exact reactos-link invocation and staticlib mode; kernel "
            "body is kernel-rust",
            "Reduce workspace membership without changing projected normal "
            "dependency/features/profile semantics",
            "Provision or project unused dev dependency resolution; original "
            "lock/manifests may require registry caches",
            "Mount captured sources read-only at /tmp/vibeos inside private "
            "user/mount/network namespace",
            "Use a minimal filesystem root; unshare around the ordinary host root "
            "does not restrict reads",
            "Identify and mount actual compiler/Cargo/sysroot/target "
            "libraries/runtime loader read-only",
            "Private writable target, Cargo home, HOME and tmp; explicit "
            "environment; no host sccache daemon",
            "Acquire host-jobs admission before compilation; capture and compiler "
            "outputs included in disk admission",
            "Observe file reads and compiler commands; reject undeclared reads "
            "before output acceptance",
            "Revalidate current lane inputs against captured projection before "
            "accepting build result",
        ],
    }
