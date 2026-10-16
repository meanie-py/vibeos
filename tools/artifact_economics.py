"""Read-only snapshot attribution and conservative archive-copy storage scenarios.

Input is a detailed rust_storage_report inventory plus its checkout. Request
markers and Cargo manifests identify bodies; SHA256 establishes byte equality.
Shared/private intermediates and primary archives always receive zero credit.
Optional current-source discovery is eligibility evidence, never a cache hit.
"""

from __future__ import annotations

import argparse
import collections
import hashlib
import json
import os
import re
import stat
import struct
import sys
import tempfile
from pathlib import Path

try:
    import tomllib  # type: ignore[reportMissingImports]
except ModuleNotFoundError:
    import tomli as tomllib

from tools.rust_storage_report import _identity, _open_root


def stable_read(path, expected=None):
    directory = _open_root(path.parent)
    try:
        fd = os.open(
            path.name, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK, dir_fd=directory
        )
        try:
            before = os.fstat(fd)
            if not stat.S_ISREG(before.st_mode):
                raise ValueError("not a regular file")
            if expected and (
                before.st_dev,
                before.st_ino,
                before.st_size,
                before.st_mtime_ns,
                before.st_ctime_ns,
                before.st_blocks * 512,
                before.st_nlink,
            ) != (
                expected["device"],
                expected["inode"],
                expected["logical_bytes"],
                expected["mtime_ns"],
                expected["ctime_ns"],
                expected["allocated_bytes"],
                expected["link_count"],
            ):
                raise ValueError("changed since inventory")
            if before.st_size > 1024 * 1024:
                raise ValueError("unexpectedly large attribution input")
            payload = bytearray()
            while True:
                chunk = os.read(fd, 65536)
                if not chunk:
                    break
                payload.extend(chunk)
            after = os.fstat(fd)
            at_path = os.stat(path.name, dir_fd=directory, follow_symlinks=False)
            if (
                _identity(before) != _identity(after)
                or _identity(before) != _identity(at_path)
                or len(payload) != before.st_size
            ):
                raise ValueError("changed while reading")
            return bytes(payload)
        finally:
            os.close(fd)
    finally:
        os.close(directory)


def marker_fields(payload):
    fields = []
    offset = 0
    while offset < len(payload):
        if offset + 8 > len(payload):
            raise ValueError("truncated request field length")
        (length,) = struct.unpack_from("<Q", payload, offset)
        offset += 8
        if length > len(payload) - offset:
            raise ValueError("truncated request field")
        fields.append(payload[offset : offset + length])
        offset += length
    if len(fields) != 5:
        raise ValueError(f"expected 5 request marker fields, found {len(fields)}")
    return fields


def map_bodies(report, repo):
    by_path = {p["path"]: r for r in report["inventory"] for p in r["paths"]}
    manifest_cache = {}
    bodies = collections.defaultdict(list)
    unknown = []
    snapshot_inodes = 0
    for record in report["inventory"]:
        if "consumer_snapshot" not in record["roles"] or record["kind"] != "regular":
            continue
        snapshot_inodes += 1
        attributions = []
        for alias in record["paths"]:
            if alias["role"] != "consumer_snapshot":
                continue
            archive = Path(alias["path"])
            marker = archive.parent / ".body-storage.request-v1"
            try:
                expected = by_path.get(str(marker))
                if expected is None:
                    raise ValueError("request marker not inventoried")
                payload = stable_read(marker, expected)
                arch_raw, body_raw, config, request_raw, filename_raw = marker_fields(payload)
                arch, body = arch_raw.decode(), body_raw.decode()
                filename, request = filename_raw.decode(), request_raw.decode()
                if not re.fullmatch(r"[A-Za-z0-9_-]+", body):
                    raise ValueError("invalid body package component")
                if arch not in {"amd64", "arm64", "i386"}:
                    raise ValueError("unknown marker architecture")
                if filename != archive.name:
                    raise ValueError("marker archive name mismatch")
                if body not in manifest_cache:
                    manifest_path = repo / "sdk/lib/rust" / body / "Cargo.toml"
                    manifest_payload = stable_read(manifest_path)
                    manifest = tomllib.loads(manifest_payload.decode())
                    package = manifest["package"]["name"]
                    if package != body:
                        raise ValueError("marker package does not match manifest package")
                    library = manifest.get("lib", {}).get("name", package.replace("-", "_"))
                    manifest_cache[body] = {
                        "manifest_path": str(manifest_path),
                        "manifest_sha256": hashlib.sha256(manifest_payload).hexdigest(),
                        "package": package,
                        "library_name": library,
                        "manifest_crate_types": manifest.get("lib", {}).get("crate-type", ["rlib"]),
                    }
                library = manifest_cache[body]["library_name"]
                request_parts = request.split("|", 5)
                if len(request_parts) != 6 or request_parts[0] != body:
                    raise ValueError("request invocation does not identify marker package")
                _, features_raw, default_raw, kernel_raw, static_raw, _ = request_parts
                if request_parts[5].encode() != config:
                    raise ValueError("request configuration does not match namespace configuration")
                if any(
                    value not in {"true", "false"}
                    for value in [default_raw, kernel_raw, static_raw]
                ):
                    raise ValueError("request boolean not understood")
                features = json.loads(features_raw)
                if not isinstance(features, list) or not all(isinstance(f, str) for f in features):
                    raise ValueError("request features not string list")
                staticlib = (
                    static_raw == "true"
                    or "staticlib" in manifest_cache[body]["manifest_crate_types"]
                )
                expected_filename = f"lib{library}.{'a' if staticlib else 'rlib'}"
                if filename != expected_filename:
                    raise ValueError("archive name does not match Cargo manifest lib name")
                consumer = Path(alias["relative"]).parts[0]
                attributions.append(
                    {
                        "path": str(archive),
                        "body": body,
                        "architecture": arch,
                        "consumer_directory": consumer,
                        "features": features,
                        "default_features": default_raw == "true",
                        "kernel": kernel_raw == "true",
                        "force_staticlib": static_raw == "true",
                        "staticlib": staticlib,
                        "archive_name": filename,
                        "configuration_sha256": hashlib.sha256(config).hexdigest(),
                        "request_sha256": hashlib.sha256(request_raw).hexdigest(),
                        "request_marker_sha256": hashlib.sha256(payload).hexdigest(),
                    }
                )
            except (OSError, ValueError, KeyError, UnicodeError) as exc:
                unknown.append(
                    {
                        "device": record["device"],
                        "inode": record["inode"],
                        "allocated_bytes": record["allocated_bytes"],
                        "path": str(archive),
                        "reason": str(exc),
                    }
                )
        identities = {(a["body"], a["architecture"], a["request_sha256"]) for a in attributions}
        if len(identities) != 1:
            if identities:
                unknown.append(
                    {
                        "device": record["device"],
                        "inode": record["inode"],
                        "allocated_bytes": record["allocated_bytes"],
                        "reason": "same inode has conflicting snapshot request attributions",
                    }
                )
            continue
        body = attributions[0]["body"]
        bodies[body].append(
            {
                "device": record["device"],
                "inode": record["inode"],
                "allocated_bytes": record["allocated_bytes"],
                "logical_bytes": record["logical_bytes"],
                "sha256": record["sha256"],
                "hash_status": record["hash_status"],
                "roles": record["roles"],
                "attributions": attributions,
            }
        )
    output_bodies = {}
    for body, rows in bodies.items():
        group = collections.defaultdict(list)
        for row in rows:
            alias = row["attributions"][0]
            group[(alias["architecture"], alias["configuration_sha256"])].append(row)
        configs = []
        for (arch, config), members in group.items():
            hashes = collections.defaultdict(list)
            for row in members:
                if row["hash_status"] == "verified":
                    hashes[row["sha256"]].append(row)
            configs.append(
                {
                    "architecture": arch,
                    "configuration_sha256": config,
                    "snapshot_inodes": len(members),
                    "snapshot_allocated_bytes": sum(r["allocated_bytes"] for r in members),
                    "snapshot_logical_bytes": sum(r["logical_bytes"] for r in members),
                    "verified_content_groups": len(hashes),
                    "content_unique_allocated_bytes": sum(
                        max(r["allocated_bytes"] for r in rs) for rs in hashes.values()
                    ),
                    "consumers": sorted(
                        {a["consumer_directory"] for r in members for a in r["attributions"]}
                    ),
                    "requests": [
                        {
                            "request_sha256": a["request_sha256"],
                            "features": a["features"],
                            "default_features": a["default_features"],
                            "kernel": a["kernel"],
                            "staticlib": a["staticlib"],
                            "force_staticlib": a["force_staticlib"],
                        }
                        for a in {
                            a["request_sha256"]: a for r in members for a in r["attributions"]
                        }.values()
                    ],
                }
            )
        output_bodies[body] = {
            **manifest_cache[body],
            "snapshot_inodes": len(rows),
            "snapshot_allocated_bytes": sum(r["allocated_bytes"] for r in rows),
            "configurations": configs,
            "snapshots": rows,
        }
    return {
        "schema_version": 1,
        "inventory_started_at": report["started_at"],
        "inventory_finished_at": report["finished_at"],
        "snapshot_inodes": snapshot_inodes,
        "attributed_bodies": len(output_bodies),
        "attributed_snapshot_inodes": sum(x["snapshot_inodes"] for x in output_bodies.values()),
        "attributed_snapshot_allocated_bytes": sum(
            x["snapshot_allocated_bytes"] for x in output_bodies.values()
        ),
        "unknown": unknown,
        "bodies": dict(
            sorted(
                output_bodies.items(),
                key=lambda pair: pair[1]["snapshot_allocated_bytes"],
                reverse=True,
            )
        ),
        "method": [
            "Read .body-storage.request-v1 length-delimited fields; verify unchanged "
            "against saved inventory and before/after no-symlink reads.",
            "Resolve marker package to current Cargo.toml package name and lib name; "
            "verify archive filename; keep manifest SHA256.",
            "Count each observed snapshot inode once; "
            "hashes/configuration/request fields preserved separately.",
            "Configuration/request SHA256 identifies stored descriptor bytes, "
            "not semantic action keys or eligibility.",
            "Historical/stale configurations are retained; no currentness, "
            "cross-lane reuse, or reclaimability is inferred.",
        ],
    }


def size_model(rows):
    verified = [
        r for r in rows if r["hash_status"] == "verified" and r["roles"] == ["consumer_snapshot"]
    ]
    contents = collections.defaultdict(list)
    for row in verified:
        contents[row["sha256"]].append(row)
    total = sum(r["allocated_bytes"] for r in verified)
    store = sum(max(r["allocated_bytes"] for r in group) for group in contents.values())
    return {
        "snapshot_inodes": len(rows),
        "verified_exclusive_snapshot_inodes": len(verified),
        "unknown_or_ambiguous_snapshot_inodes": len(rows) - len(verified),
        "snapshot_allocated_bytes": total,
        "unique_content_groups": len(contents),
        "new_store_blobs_allocated_bytes": store,
        "direct_archive_only_net_before_overheads_bytes": total - store,
        "copying_archive_only_net_before_overheads_bytes": -store,
        "additional_publication_peak_before_worker_source_overheads_bytes": store,
        "shared_intermediate_reclaim_credit_bytes": 0,
        "primary_archive_reclaim_credit_bytes": 0,
    }


def size_by_architecture(rows):
    architectures = sorted({r["attributions"][0]["architecture"] for r in rows})
    return {
        arch: size_model([r for r in rows if r["attributions"][0]["architecture"] == arch])
        for arch in architectures
    }


def summarize(mapping, repo, *, eligibility_top=0, cargo="cargo", scratch=None):
    if eligibility_top < 0:
        raise ValueError("eligibility_top must be nonnegative")
    discover = None
    if eligibility_top:
        if scratch is None:
            raise ValueError("explicit scratch directory required for source discovery")
        from tools.artifact_inputs import discover

        scratch = scratch.resolve()
        scratch.mkdir(parents=True, exist_ok=True)
    top = list(mapping["bodies"])[:eligibility_top]
    discovery = {}
    bodies = {}
    all_rows = []
    eligible_top_rows = []
    for name, body in mapping["bodies"].items():
        rows = body["snapshots"]
        all_rows.extend(rows)
        requests = collections.defaultdict(list)
        for row in rows:
            alias = row["attributions"][0]
            requests[
                (alias["architecture"], alias["configuration_sha256"], alias["request_sha256"])
            ].append(row)
        variants = []
        for (arch, config, request), members in requests.items():
            alias = members[0]["attributions"][0]
            kwargs = {
                "features": tuple(alias["features"]),
                "default_features": alias["default_features"],
                "force_staticlib": alias["force_staticlib"],
            }
            source_id = json.dumps({"body": name, **kwargs}, sort_keys=True)
            if name in top and source_id not in discovery:
                assert discover is not None
                original_temp = tempfile.tempdir
                tempfile.tempdir = str(scratch)
                try:
                    result = discover(repo, name, context={}, cargo=cargo, **kwargs)
                finally:
                    tempfile.tempdir = original_temp
                discovery[source_id] = {
                    k: result.get(k)
                    for k in [
                        "eligible",
                        "reasons",
                        "source_projection_key",
                        "action_key",
                        "missing_context",
                        "files",
                        "bytes",
                        "package_count",
                        "provenance",
                    ]
                }
            projection = discovery.get(source_id)
            if projection and projection["eligible"]:
                eligible_top_rows.extend(members)
            variants.append(
                {
                    "architecture": arch,
                    "configuration_sha256": config,
                    "request_sha256": request,
                    "features": alias["features"],
                    "default_features": alias["default_features"],
                    "kernel": alias["kernel"],
                    "force_staticlib": alias["force_staticlib"],
                    "staticlib": alias["staticlib"],
                    "consumers": sorted(
                        {a["consumer_directory"] for r in members for a in r["attributions"]}
                    ),
                    "source_projection_id": source_id if projection else None,
                    "source_projection_eligible": projection["eligible"] if projection else None,
                    **size_model(members),
                }
            )
        bodies[name] = {
            k: body[k]
            for k in [
                "manifest_path",
                "manifest_sha256",
                "package",
                "library_name",
                "manifest_crate_types",
            ]
        } | {"archive_only_model": size_model(rows), "requests": variants}
    source_script = Path(__file__).with_name("artifact_inputs.py")
    return {
        "schema_version": 1,
        "inventory_started_at": mapping["inventory_started_at"],
        "inventory_finished_at": mapping["inventory_finished_at"],
        "attributed_bodies": mapping["attributed_bodies"],
        "unknown": mapping["unknown"],
        "all_bodies_archive_only_model": size_model(all_rows),
        "all_bodies_by_architecture": size_by_architecture(all_rows),
        "source_eligible_archive_only_model": size_model(eligible_top_rows),
        "source_eligible_by_architecture": size_by_architecture(eligible_top_rows),
        "eligibility_bodies": top,
        "source_discovery": discovery,
        "discovery_tool_sha256": (
            hashlib.sha256(source_script.read_bytes()).hexdigest() if eligibility_top else None
        ),
        "discovery_cargo": cargo,
        "bodies": bodies,
        "method": [
            "Body/feature/default/force-staticlib attribution from verified markers/manifests.",
            "SHA256 actual inventory bytes, unique inodes; retain largest allocation per content.",
            "Direct removes verified consumer snapshots and adds blobs; all other outputs stay.",
            "Copying keeps snapshots and adds blobs; still owes source/scratch/ref/GC overhead.",
            "Optional discover uses current source and actual recorded request features; "
            "missing context means action_key is null.",
            "Source support does not prove execution eligibility, historical provenance, "
            "cross-lane reuse, or reproducibility.",
            "Keep historical/configuration variants; no currentness or pruning assumptions.",
        ],
    }


def analyze(inventory, repo, *, eligibility_top=0, cargo="cargo", scratch=None):
    """Attribute saved bytes; fail unknown attribution explicitly in coverage."""
    mapping = map_bodies(inventory, Path(repo))
    report = summarize(
        mapping,
        Path(repo),
        eligibility_top=eligibility_top,
        cargo=cargo,
        scratch=Path(scratch) if scratch is not None else None,
    )
    report["inventory_coverage"] = inventory["coverage"]
    report["attribution_coverage"] = {
        k: mapping[k]
        for k in [
            "snapshot_inodes",
            "attributed_bodies",
            "attributed_snapshot_inodes",
            "attributed_snapshot_allocated_bytes",
            "unknown",
        ]
    }
    report["complete"] = (
        inventory["coverage"]["complete"]
        and inventory["coverage"]["hash_complete"]
        and not mapping["unknown"]
        and mapping["snapshot_inodes"] == mapping["attributed_snapshot_inodes"]
    )
    report["attribution_method"] = mapping["method"]
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("inventory", type=Path)
    parser.add_argument("repo", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument(
        "--eligibility-top",
        type=int,
        default=0,
        help="discover top N bodies by snapshot bytes; source support only",
    )
    parser.add_argument("--cargo", default="cargo", help="audited Cargo executable")
    parser.add_argument(
        "--scratch",
        type=Path,
        help="private temporary resolution directory (required for discovery)",
    )
    args = parser.parse_args()
    if args.eligibility_top and not args.scratch:
        parser.error("--eligibility-top requires --scratch")
    payload = args.inventory.read_bytes()
    inventory = json.loads(payload)
    for source in inventory["roots"]:
        root = Path(source["path"]).resolve()
        output = args.output.resolve()
        if root == output or root in output.parents:
            parser.error("output must be outside every surveyed root")
        if args.scratch:
            scratch = args.scratch.resolve()
            if root == scratch or root in scratch.parents:
                parser.error("scratch must be outside every surveyed root")
    report = analyze(
        inventory,
        args.repo,
        eligibility_top=args.eligibility_top,
        cargo=args.cargo,
        scratch=args.scratch,
    )
    report["inventory_file"] = str(args.inventory.absolute())
    report["inventory_sha256"] = hashlib.sha256(payload).hexdigest()
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(report, stream, indent=2)
        stream.write("\n")
    print(
        json.dumps(
            {
                "complete": report["complete"],
                "attributed_bodies": report["attributed_bodies"],
                "unknown_count": len(report["unknown"]),
                "all_bodies": report["all_bodies_archive_only_model"],
                "source_eligible": report["source_eligible_archive_only_model"],
                "source_variants": {
                    k: {"eligible": v["eligible"], "reasons": v["reasons"]}
                    for k, v in report["source_discovery"].items()
                },
            },
            indent=2,
        )
    )
    return 0 if report["complete"] else 2


if __name__ == "__main__":
    sys.exit(main())
