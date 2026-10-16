"""Read-only allocated-block/content inventory of explicit local Rust output roots.

Paths label roles, never content identity. SHA-256 reads every archive/rlib/rmeta
once per inode, checks metadata around the read, and revalidates aliases at the
end. Live trees are not an atomic snapshot: detected races exclude whole inodes
from content claims and make coverage incomplete. No symlinks are followed,
including root ancestors and directory replacements during traversal.
"""

from __future__ import annotations

import argparse
import collections
import hashlib
import json
import os
import re
import stat
import sys
import time
from collections.abc import Sequence
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ARTIFACT_SUFFIXES = {".a", ".lib", ".rlib", ".rmeta"}
BODY = re.compile(r"^body-storage-v1-(amd64|arm64|i386)-([0-9a-f]{16})-(\d+)-rustlib$")
DIRECT = re.compile(r"^rust-shared-(\d+)-target$")
ARCH = re.compile(r"(?:^|-)(amd64|arm64|i386)(?:-|$)")
_DIRECTORY = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC


def _utc() -> str:
    return datetime.now(timezone.utc).isoformat()


def _identity(st: os.stat_result) -> tuple[int, ...]:
    # ctime catches an identical-mtime rewrite; mode prevents accepting a FIFO
    # swapped in for a regular file. Allocation changes invalidate too.
    return (
        st.st_dev,
        st.st_ino,
        st.st_mode,
        st.st_size,
        st.st_mtime_ns,
        st.st_ctime_ns,
        st.st_blocks,
        st.st_nlink,
    )


def _open_root(path: Path) -> int:
    """Open every component without following any ancestor symlink."""
    fd = os.open("/", _DIRECTORY)
    try:
        for component in path.parts[1:]:
            next_fd = os.open(component, _DIRECTORY, dir_fd=fd)
            os.close(fd)
            fd = next_fd
        return fd
    except BaseException:
        os.close(fd)
        raise


def _open_parent(root_fd: int, parts: tuple[str, ...]) -> int:
    fd = os.dup(root_fd)
    try:
        for component in parts[:-1]:
            next_fd = os.open(component, _DIRECTORY, dir_fd=fd)
            os.close(fd)
            fd = next_fd
        return fd
    except BaseException:
        os.close(fd)
        raise


def classify(relative: Path, kind: str = "workspace") -> dict[str, Any]:
    """Classify layout only; labels do not establish reachability or identity."""
    parts = relative.parts
    suffix = relative.suffix.lower()
    label: dict[str, Any] = {"role": "other", "layout": "other"}
    if not parts:
        return label
    first = parts[0]
    arch = ARCH.search(first)
    if arch:
        label["architecture"] = arch[1]
    if "body-storage" in parts:
        label.update(
            role="consumer_snapshot" if suffix in ARTIFACT_SUFFIXES else "consumer_snapshot_other",
            layout="consumer_snapshot",
        )
        return label
    body = BODY.fullmatch(first) if kind == "workspace" else None
    if body:
        label.update(
            layout="body_shard",
            architecture=body[1],
            configuration_label=body[2],
            shard=int(body[3]),
            storage_root=first,
        )
        if suffix in {".rlib", ".rmeta"} and "deps" in parts[1:]:
            label["role"] = "body_dependencies"
        elif suffix in {".a", ".lib"} and parts[-2] in {"release", "debug", "deps"}:
            label["role"] = "body_primary_archive"
        else:
            label["role"] = "body_other"
    elif kind == "workspace" and first.endswith("-cargo-link"):
        label.update(layout="outer_coordinator", storage_root=first)
        if len(parts) > 2 and parts[1] in {"release", "debug"}:
            label["role"] = "outer_host_dependencies" if parts[2] == "deps" else "outer_host_other"
        else:
            label["role"] = "outer_target_other"
    elif kind == "workspace" and first.endswith("-rustlib"):
        label.update(layout="legacy_private_body", role="legacy_private_body", storage_root=first)
        if first.startswith("storage-pilot-"):
            label.update(layout="older_storage_pilot", role="older_storage_pilot")
    elif kind == "build":
        shared = next((p for p in parts if DIRECT.fullmatch(p)), None)
        private = next(
            (
                p
                for p in parts
                if p.endswith("-target") and (p.startswith("rust-") or "rust-target" in p)
            ),
            None,
        )
        if shared or private:
            label.update(
                layout="direct_cmake_shared" if shared else "direct_cmake_private",
                storage_root=shared or private,
            )
            if suffix in {".rlib", ".rmeta"} and "deps" in parts:
                label["role"] = "direct_cmake_dependencies"
            elif suffix in {".a", ".lib"} and parts[-2] in {"release", "debug", "deps"}:
                label["role"] = "direct_cmake_primary_archive"
            else:
                label["role"] = "direct_cmake_other"
        else:
            label.update(layout="build_other", role="build_other")
    return label


def _hash_file(parent_fd: int, name: str, expected: tuple[int, ...]) -> tuple[str, int]:
    flags = os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
    fd = os.open(name, flags, dir_fd=parent_fd)
    try:
        before = os.fstat(fd)
        if _identity(before) != expected or not stat.S_ISREG(before.st_mode):
            raise ValueError("changed before hashing")
        digest = hashlib.sha256()
        read_bytes = 0
        while True:
            chunk = os.read(fd, 1024 * 1024)
            if not chunk:
                break
            digest.update(chunk)
            read_bytes += len(chunk)
        after = os.fstat(fd)
        at_path = os.stat(name, dir_fd=parent_fd, follow_symlinks=False)
        if (
            _identity(after) != expected
            or _identity(at_path) != expected
            or read_bytes != before.st_size
        ):
            raise ValueError("changed during hashing or path was replaced")
        return digest.hexdigest(), read_bytes
    finally:
        os.close(fd)


def _counter_add(counter: dict[str, int], record: dict[str, Any]) -> None:
    counter["unique_inodes"] += 1
    counter[f"{record['kind']}_inodes"] += 1
    counter["allocated_bytes"] += record["allocated_bytes"]
    if record["kind"] == "regular":
        counter["logical_bytes"] += record["logical_bytes"]
    counter["path_count"] += len({p["path"] for p in record["paths"]})


def measure(
    root: Path,
    *,
    hash_content: bool = True,
    build_roots: Sequence[Path] = (),
    detailed: bool = False,
    root_kind: str = "workspace",
) -> dict[str, Any]:
    """Survey an explicit workspace target and optional build dirs, globally deduping inodes.

    Role totals partition allocated bytes. An inode with aliases in different
    roles belongs to ``ambiguous``; ``role_references`` deliberately overlaps.
    ``duplicates`` remains empty when hashing is disabled (legacy API).
    """
    started = _utc()
    monotonic = time.monotonic()
    if root_kind not in {"workspace", "build"}:
        raise ValueError(f"invalid root kind: {root_kind}")
    roots = [(Path(os.path.abspath(root)), root_kind)]
    roots.extend((Path(os.path.abspath(p)), "build") for p in build_roots)
    records: dict[tuple[int, int], dict[str, Any]] = {}
    seen_paths: set[tuple[int, str]] = set()
    errors: list[dict[str, str]] = []
    root_handles: dict[int, int] = {}
    root_info = [{"path": str(p), "kind": k} for p, k in roots]
    hash_bytes = 0
    hash_seconds = 0.0
    skipped_symlinks = 0

    def error(path: str, stage: str, message: str) -> None:
        errors.append({"path": path, "stage": stage, "message": message})

    def add(
        root_index: int, parts: tuple[str, ...], st: os.stat_result, parent_fd: int | None = None
    ) -> None:
        nonlocal hash_bytes, hash_seconds, skipped_symlinks
        absolute = str(roots[root_index][0].joinpath(*parts))
        if (root_index, absolute) in seen_paths:
            return
        seen_paths.add((root_index, absolute))
        key = (st.st_dev, st.st_ino)
        if stat.S_ISREG(st.st_mode):
            kind = "regular"
        elif stat.S_ISDIR(st.st_mode):
            kind = "directory"
        elif stat.S_ISLNK(st.st_mode):
            kind = "symlink"
            skipped_symlinks += 1
        else:
            kind = "special"
        path_record = {
            "path": absolute,
            "root_index": root_index,
            "relative": "/".join(parts),
            **classify(Path(*parts), roots[root_index][1]),
        }
        record = records.get(key)
        if record is None:
            record = {
                "device": st.st_dev,
                "inode": st.st_ino,
                "kind": kind,
                "allocated_bytes": st.st_blocks * 512,
                "logical_bytes": st.st_size,
                "mtime_ns": st.st_mtime_ns,
                "ctime_ns": st.st_ctime_ns,
                "link_count": st.st_nlink,
                "paths": [],
                "_identity": _identity(st),
                "hash_status": "not_candidate",
                "sha256": None,
            }
            records[key] = record
        elif record["_identity"] != _identity(st):
            record["hash_status"] = "raced"
            record["sha256"] = None
            error(absolute, "alias_stat", "inode changed between alias observations")
        record["paths"].append(path_record)
        candidate = kind == "regular" and Path(parts[-1]).suffix.lower() in ARTIFACT_SUFFIXES
        if candidate:
            record["candidate"] = True
        if not candidate or record["hash_status"] != "not_candidate":
            return
        if not hash_content:
            record["hash_status"] = "disabled"
            return
        assert parent_fd is not None
        begin = time.monotonic()
        try:
            digest, count = _hash_file(parent_fd, parts[-1], record["_identity"])
            record["sha256"] = digest
            record["hash_status"] = "verified"
            hash_bytes += count
        except (OSError, ValueError) as exc:
            record["hash_status"] = "raced" if isinstance(exc, ValueError) else "unreadable"
            error(absolute, "hash", str(exc))
        hash_seconds += time.monotonic() - begin

    def walk(root_index: int, directory_fd: int, parts: tuple[str, ...]) -> None:
        before = os.fstat(directory_fd)
        try:
            with os.scandir(directory_fd) as entries:
                for entry in entries:
                    child = (*parts, entry.name)
                    absolute = str(roots[root_index][0].joinpath(*child))
                    try:
                        st = entry.stat(follow_symlinks=False)
                        add(root_index, child, st, directory_fd)
                        if stat.S_ISDIR(st.st_mode):
                            child_fd = os.open(entry.name, _DIRECTORY, dir_fd=directory_fd)
                            try:
                                if _identity(os.fstat(child_fd)) != _identity(st):
                                    error(absolute, "walk", "directory replaced before open")
                                else:
                                    walk(root_index, child_fd, child)
                            finally:
                                os.close(child_fd)
                    except OSError as exc:
                        error(absolute, "walk", str(exc))
        except OSError as exc:
            error(str(roots[root_index][0].joinpath(*parts)), "walk", str(exc))
        after = os.fstat(directory_fd)
        if _identity(before) != _identity(after):
            error(
                str(roots[root_index][0].joinpath(*parts)),
                "walk",
                "directory changed during enumeration; membership coverage incomplete",
            )

    try:
        for index, (path, _) in enumerate(roots):
            try:
                fd = _open_root(path)
                root_handles[index] = fd
                add(index, (), os.fstat(fd))
                walk(index, fd, ())
            except OSError as exc:
                error(str(path), "root", str(exc))
        invalid_roots: set[int] = set()
        for index, original_fd in root_handles.items():
            try:
                fresh_fd = _open_root(roots[index][0])
                try:
                    original = os.fstat(original_fd)
                    fresh = os.fstat(fresh_fd)
                    if (original.st_dev, original.st_ino) != (fresh.st_dev, fresh.st_ino):
                        raise ValueError("root or ancestor replaced during inventory")
                finally:
                    os.close(fresh_fd)
            except (OSError, ValueError) as exc:
                invalid_roots.add(index)
                error(str(roots[index][0]), "root_revalidate", str(exc))
        # Recheck every candidate alias, including differently named hardlinks.
        # Opening parents relative to held root descriptors cannot follow a
        # malicious/replaced directory symlink outside the requested scope.
        for record in records.values():
            if record["hash_status"] != "verified":
                continue
            if any(p["root_index"] in invalid_roots for p in record["paths"]):
                record["hash_status"] = "raced"
                record["sha256"] = None
                continue
            for alias in record["paths"]:
                parts = tuple(Path(alias["relative"]).parts)
                try:
                    fd = _open_parent(root_handles[alias["root_index"]], parts)
                    try:
                        current = os.stat(parts[-1], dir_fd=fd, follow_symlinks=False)
                    finally:
                        os.close(fd)
                    if _identity(current) != record["_identity"]:
                        raise ValueError("candidate changed after hashing")
                except (OSError, ValueError) as exc:
                    record["hash_status"] = "raced"
                    record["sha256"] = None
                    error(alias["path"], "revalidate", str(exc))
    finally:
        for fd in root_handles.values():
            os.close(fd)

    return _report(
        records,
        root_info,
        errors,
        hash_content,
        detailed,
        started,
        time.monotonic() - monotonic,
        hash_bytes,
        hash_seconds,
        skipped_symlinks,
    )


def _report(
    records: dict[tuple[int, int], dict[str, Any]],
    roots: list[dict[str, str]],
    errors: list[dict[str, str]],
    hash_content: bool,
    detailed: bool,
    started: str,
    seconds: float,
    hash_bytes: int,
    hash_seconds: float,
    skipped_symlinks: int,
) -> dict[str, Any]:
    roles: dict[str, Any] = collections.defaultdict(collections.Counter)
    layouts: dict[str, Any] = collections.defaultdict(collections.Counter)
    role_references: dict[str, Any] = collections.defaultdict(collections.Counter)
    extensions: dict[str, int] = collections.Counter()
    total: dict[str, int] = collections.Counter(
        unique_inodes=0, allocated_bytes=0, logical_bytes=0, path_count=0
    )
    statuses: dict[str, int] = collections.Counter()
    groups: dict[str, list[dict[str, Any]]] = collections.defaultdict(list)
    configurations: dict[str, dict[str, Any]] = {}
    for record in records.values():
        record["roles"] = sorted({p["role"] for p in record["paths"]})
        record["layouts"] = sorted({p["layout"] for p in record["paths"]})
        _counter_add(total, record)
        exclusive = record["roles"][0] if len(record["roles"]) == 1 else "ambiguous"
        _counter_add(roles[exclusive], record)
        layout = record["layouts"][0] if len(record["layouts"]) == 1 else "ambiguous"
        _counter_add(layouts[layout], record)
        for role in record["roles"]:
            _counter_add(role_references[role], record)
        ext = sorted({Path(p["path"]).suffix for p in record["paths"]})
        extensions[ext[0] if len(ext) == 1 else "ambiguous"] += record["allocated_bytes"]
        if record.get("candidate"):
            statuses[record["hash_status"]] += 1
        if record["hash_status"] == "verified":
            groups[record["sha256"]].append(record)
        for path in record["paths"]:
            if path["layout"] == "body_shard":
                key = f"{path['root_index']}:{path['architecture']}:{path['configuration_label']}"
                config = configurations.setdefault(
                    key,
                    {
                        "root_index": path["root_index"],
                        "architecture": path["architecture"],
                        "directory_label": path["configuration_label"],
                        "observed_shards": set(),
                    },
                )
                config["observed_shards"].add(path["shard"])
    for config in configurations.values():
        config["observed_shards"] = sorted(config["observed_shards"])

    duplicates: list[dict[str, Any]] = []
    for digest, members in groups.items():
        if len(members) < 2:
            continue
        allocation = sum(r["allocated_bytes"] for r in members)
        group_roles = sorted({role for r in members for role in r["roles"]})
        duplicates.append(
            {
                "sha256": digest,
                "logical_bytes_per_inode": members[0]["logical_bytes"],
                "unique_inodes": len(members),
                "allocated_bytes": allocation,
                "duplicate_allocated_bytes": allocation
                - max(r["allocated_bytes"] for r in members),
                "roles": group_roles,
                "cross_role": len(group_roles) > 1,
                "snapshot_allocated_bytes": sum(
                    r["allocated_bytes"] for r in members if r["roles"] == ["consumer_snapshot"]
                ),
                "members": [
                    {"device": r["device"], "inode": r["inode"], "roles": r["roles"]}
                    for r in members
                ],
            }
        )
    duplicates.sort(key=lambda g: g["duplicate_allocated_bytes"], reverse=True)
    verified = [r for r in records.values() if r["hash_status"] == "verified"]
    content = {
        "verified_inodes": len(verified),
        "distinct_contents": len(groups),
        "verified_allocated_bytes": sum(r["allocated_bytes"] for r in verified),
        "verified_logical_bytes": sum(r["logical_bytes"] for r in verified),
        "duplicate_groups": len(duplicates),
        "duplicate_inodes": sum(g["unique_inodes"] - 1 for g in duplicates),
        "duplicate_allocated_bytes": sum(g["duplicate_allocated_bytes"] for g in duplicates),
        "cross_role_duplicate_groups": sum(g["cross_role"] for g in duplicates),
        "cross_role_duplicate_allocated_bytes": sum(
            g["duplicate_allocated_bytes"] for g in duplicates if g["cross_role"]
        ),
        "unique_content_allocated_bytes": sum(
            max(r["allocated_bytes"] for r in members) for members in groups.values()
        ),
        "hashed_bytes_read": hash_bytes,
        "hash_seconds": hash_seconds,
    }
    coverage = {
        "complete": not errors,
        "hash_complete": hash_content and not errors,
        "candidate_status_counts": dict(statuses),
        "errors": errors,
        "symlink_inodes_skipped": sum(r["kind"] == "symlink" for r in records.values()),
        "symlink_paths_skipped": skipped_symlinks,
        "ambiguous_role_inodes": roles.get("ambiguous", {}).get("unique_inodes", 0),
        "regular_inodes_with_unobserved_hardlinks": sum(
            r["kind"] == "regular" and r["link_count"] > len({p["path"] for p in r["paths"]})
            for r in records.values()
        ),
    }
    report: dict[str, Any] = {
        "schema_version": 2,
        "root": roots[0]["path"],
        "roots": roots,
        "started_at": started,
        "finished_at": _utc(),
        "elapsed_seconds": seconds,
        "content_hashed": hash_content,
        **dict(total),
        "roles": dict(roles),
        "role_references": dict(role_references),
        "layouts": dict(layouts),
        "allocated_by_extension": dict(extensions),
        "configurations": list(configurations.values()),
        "content": content,
        "duplicates": content if hash_content else {},
        "coverage": coverage,
        "method": {
            "allocated": (
                "st_blocks * 512, every observed inode counted once globally; "
                "includes directories/symlinks"
            ),
            "logical": "regular file st_size, once per inode",
            "identity": (
                "SHA-256 actual bytes across all .a/.lib/.rlib/.rmeta, "
                "regardless of filename or extension"
            ),
            "races": (
                "device/inode/mode/size/mtime_ns/ctime_ns/blocks/nlink checked before+after read "
                "and all aliases at end; directories checked around enumeration; roots reopened"
            ),
            "duplicates": (
                "sum allocated blocks minus largest member per content group; "
                "a conservative redundant-copy allocation, NOT reclaimable space"
            ),
            "roles": (
                "exclusive partition; multi-role inodes go to ambiguous; "
                "role_references overlaps intentionally"
            ),
            "limits": [
                "Live survey is not an atomic filesystem snapshot; "
                "files can change after their final check.",
                "Block accounting cannot detect reflink/dedup/compression sharing "
                "or filesystem metadata beyond observed inodes.",
                "Observed configuration/shard directory labels are not semantic "
                "action keys or effective shard settings.",
                "Private layout presence does not establish staleness, "
                "reachability, or permission to prune.",
                "Body shard intermediates can remain needed by other misses; "
                "net savings require action eligibility, store/snapshot/scratch/GC accounting.",
            ],
        },
    }
    if detailed:
        report["inventory"] = [
            {k: v for k, v in r.items() if not k.startswith("_")} for r in records.values()
        ]
        report["content_groups"] = duplicates
    return report


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    scope = parser.add_mutually_exclusive_group(required=True)
    scope.add_argument("--arch", choices=("i386", "amd64", "arm64"))
    scope.add_argument("--workspace", action="store_true")
    scope.add_argument("--root", type=Path, help="explicit workspace target root")
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--build-root", action="append", type=Path, default=[])
    parser.add_argument("--no-hash", action="store_true")
    parser.add_argument("--json", type=Path, help="write report; must be outside surveyed roots")
    parser.add_argument(
        "--detailed", action="store_true", help="include inode/path/content groups in JSON"
    )
    args = parser.parse_args()
    if args.detailed and not args.json:
        parser.error("--detailed requires --json; stdout is a bounded summary")
    root = args.root or args.repo / "sdk/lib/rust/target"
    build_roots = list(args.build_root)
    if args.arch:
        root = args.repo / f"build-{args.arch}"
    for surveyed in [root, *build_roots]:
        if args.json:
            output = args.json.resolve()
            source = surveyed.resolve()
            if output == source or source in output.parents:
                parser.error("--json must be outside every surveyed root")
    report = measure(
        root,
        hash_content=not args.no_hash,
        build_roots=build_roots,
        detailed=args.detailed,
        root_kind="build" if args.arch else "workspace",
    )
    if args.json:
        with args.json.open("x", encoding="utf-8") as stream:
            json.dump(report, stream, indent=2)
            stream.write("\n")
    summary = {k: v for k, v in report.items() if k not in {"inventory", "content_groups"}}
    # Error detail is durable in JSON; stdout remains bounded even on live builds.
    summary["coverage"] = {
        **report["coverage"],
        "errors": report["coverage"]["errors"][:10],
        "error_count": len(report["coverage"]["errors"]),
    }
    print(json.dumps(summary, indent=2))
    return 0 if report["coverage"]["complete"] else 2


if __name__ == "__main__":
    sys.exit(main())
