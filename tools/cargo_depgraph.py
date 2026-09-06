"""Read-only Cargo-resolved local dependency closure for build-edge planning.

Uses the locked, offline, all-feature graph without a target filter, deliberately
including target-specific and build dependencies. Explicitly include body crates
built by nested Cargo invocations: Cargo cannot infer those from a build script.
This inventories package roots; it does NOT claim to enumerate arbitrary build
script inputs or replace compiler depfiles.
"""

import argparse
import json
import subprocess
from pathlib import Path


def local_closure(metadata, roots):
    packages = {package["id"]: package for package in metadata["packages"]}
    resolve = metadata.get("resolve")
    if not resolve:
        raise ValueError("Cargo metadata has no resolved graph; --no-deps is insufficient")
    nodes = {node["id"]: node for node in resolve["nodes"]}
    pending = []
    for root in roots:
        matches = [p["id"] for p in packages.values() if p["name"] == root and p["source"] is None]
        if len(matches) != 1:
            raise ValueError(f"expected one local package named {root!r}, found {len(matches)}")
        pending.extend(matches)
    if not pending:
        raise ValueError("at least one root package is required")
    seen = set()
    while pending:
        package_id = pending.pop()
        if package_id in seen:
            continue
        if package_id not in nodes or package_id not in packages:
            raise ValueError(f"incomplete Cargo graph: {package_id}")
        seen.add(package_id)
        pending.extend(nodes[package_id]["dependencies"])
    return sorted(
        (
            {"name": packages[key]["name"], "manifest": packages[key]["manifest_path"]}
            for key in seen
            if packages[key]["source"] is None
        ),
        key=lambda item: (item["name"], item["manifest"]),
    )


def inspect(manifest, roots):
    proc = subprocess.run(
        [
            "cargo",
            "metadata",
            "--offline",
            "--locked",
            "--all-features",
            "--format-version",
            "1",
            "--manifest-path",
            str(manifest),
        ],
        cwd=manifest.parent,
        capture_output=True,
        text=True,
        timeout=120,
    )
    if proc.returncode:
        raise ValueError(f"locked offline Cargo metadata failed:\n{proc.stderr.strip()}")
    metadata = json.loads(proc.stdout)
    return {
        "roots": sorted(set(roots)),
        "workspace": metadata["workspace_root"],
        "scope": "all-features/all-targets; package closure only",
        "local_packages": local_closure(metadata, roots),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--packages", nargs="+", required=True)
    args = parser.parse_args()
    try:
        report = inspect(args.manifest.resolve(), args.packages)
    except (OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
        parser.exit(2, f"cargo-depgraph: {error}\n")
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
