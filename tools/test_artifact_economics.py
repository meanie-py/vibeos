import json
import struct

import pytest

from tools import rust_storage_report as inventory
from tools.artifact_economics import analyze, map_bodies, marker_fields, size_model


def fields(*values):
    return b"".join(struct.pack("<Q", len(v)) + v for v in values)


def fixture(tmp_path, *, force=False, static=True, filename="libspecial_body.a"):
    repo = tmp_path / "repo"
    source = repo / "sdk/lib/rust/body-rust/Cargo.toml"
    source.parent.mkdir(parents=True)
    source.write_text(
        '[package]\nname="body-rust"\nversion="0.1.0"\n'
        '[lib]\nname="special_body"\ncrate-type='
        + json.dumps(["staticlib" if static else "rlib"])
        + "\n"
    )
    root = tmp_path / "target"
    out = root / "outer-cargo-amd64-cargo-link/triple/release/build/outer/out/body-storage/id"
    out.mkdir(parents=True)
    (out / filename).write_bytes(b"archive")
    request = f'body-rust|["feature"]|false|true|{str(force).lower()}|config'.encode()
    marker = out / ".body-storage.request-v1"
    marker.write_bytes(fields(b"amd64", b"body-rust", b"config", request, filename.encode()))
    return repo, root, marker


def test_manifest_staticlib_without_force_and_custom_lib_name(tmp_path):
    repo, root, _ = fixture(tmp_path)
    report = map_bodies(inventory.measure(root, detailed=True), repo)
    assert report["unknown"] == []
    body = report["bodies"]["body-rust"]
    assert body["library_name"] == "special_body"
    (request,) = body["configurations"][0]["requests"]
    assert request["staticlib"]
    assert not request["force_staticlib"]
    assert request["features"] == ["feature"]
    assert not request["default_features"]


def test_forced_staticlib_on_rlib_manifest(tmp_path):
    repo, root, _ = fixture(tmp_path, static=False, force=True)
    report = map_bodies(inventory.measure(root, detailed=True), repo)
    assert report["unknown"] == []
    (request,) = report["bodies"]["body-rust"]["configurations"][0]["requests"]
    assert request["staticlib"] and request["force_staticlib"]


def test_wrong_filename_is_unknown_never_package_guess(tmp_path):
    repo, root, _ = fixture(tmp_path, filename="libbody_rust.a")
    report = map_bodies(inventory.measure(root, detailed=True), repo)
    assert report["attributed_bodies"] == 0
    assert "manifest lib name" in report["unknown"][0]["reason"]


def test_changed_request_marker_is_unknown(tmp_path):
    repo, root, marker = fixture(tmp_path)
    saved = inventory.measure(root, detailed=True)
    marker.write_bytes(marker.read_bytes() + b"changed")
    report = map_bodies(saved, repo)
    assert report["attributed_bodies"] == 0
    assert "changed since inventory" in report["unknown"][0]["reason"]


@pytest.mark.parametrize("payload", [b"x", fields(b"one"), struct.pack("<Q", 999) + b"short"])
def test_bad_request_format_rejected(payload):
    with pytest.raises(ValueError):
        marker_fields(payload)


def test_archive_only_net_keeps_primaries_and_all_intermediates(tmp_path):
    repo, root, marker = fixture(tmp_path)
    first = marker.parent
    second = root / "second-cargo-amd64-cargo-link/triple/release/build/second/out/body-storage/id"
    second.mkdir(parents=True)
    (second / ".body-storage.request-v1").write_bytes(marker.read_bytes())
    (second / "libspecial_body.a").write_bytes(b"archive")
    primary = root / "body-storage-v1-amd64-0123456789abcdef-0-rustlib/triple/release"
    primary.mkdir(parents=True)
    (primary / "libspecial_body.a").write_bytes(b"archive")
    (primary / "deps").mkdir()
    (primary / "deps/libdependency.rlib").write_bytes(b"large retained dependency" * 4096)
    result = analyze(inventory.measure(root, detailed=True), repo)
    model = result["all_bodies_archive_only_model"]
    allocation = (first / "libspecial_body.a").stat().st_blocks * 512
    assert model["snapshot_inodes"] == 2
    assert model["unique_content_groups"] == 1
    assert model["snapshot_allocated_bytes"] == 2 * allocation
    assert model["new_store_blobs_allocated_bytes"] == allocation
    assert model["direct_archive_only_net_before_overheads_bytes"] == allocation
    assert model["copying_archive_only_net_before_overheads_bytes"] == -allocation
    assert model["shared_intermediate_reclaim_credit_bytes"] == 0
    assert model["primary_archive_reclaim_credit_bytes"] == 0
    assert result["source_eligible_archive_only_model"]["snapshot_inodes"] == 0
    assert result["all_bodies_by_architecture"]["amd64"] == model
    assert result["complete"]


def test_sparse_content_retention_conservative_and_ambiguous_excluded():
    common = {"hash_status": "verified", "roles": ["consumer_snapshot"], "sha256": "same"}
    rows = [
        common | {"allocated_bytes": 8192},
        common | {"allocated_bytes": 4096},
        common | {"allocated_bytes": 65536, "roles": ["consumer_snapshot", "body_primary_archive"]},
    ]
    result = size_model(rows)
    assert result["new_store_blobs_allocated_bytes"] == 8192
    assert result["direct_archive_only_net_before_overheads_bytes"] == 4096
    assert result["unknown_or_ambiguous_snapshot_inodes"] == 1


def test_missing_hashes_are_explicitly_incomplete(tmp_path):
    repo, root, _ = fixture(tmp_path)
    report = analyze(inventory.measure(root, detailed=True, hash_content=False), repo)
    assert not report["complete"]
    assert report["all_bodies_archive_only_model"]["snapshot_inodes"] == 1
    assert report["all_bodies_archive_only_model"]["verified_exclusive_snapshot_inodes"] == 0


def test_optional_source_discovery_uses_actual_request_and_no_context(tmp_path, monkeypatch):
    import sys
    import types

    repo, root, _ = fixture(tmp_path, force=True, static=False)
    (repo / "tools").mkdir()
    (repo / "tools/artifact_inputs.py").write_text("# fixture source descriptor\n")
    calls = []
    fake = types.ModuleType("tools.artifact_inputs")

    def discover(path, body, context, **kwargs):
        calls.append((path, body, context, kwargs))
        return {
            "eligible": True,
            "action_key": None,
            "source_projection_key": "fixture",
            "reasons": [],
            "provenance": {"fixture": True},
        }

    fake.discover = discover
    monkeypatch.setitem(sys.modules, "tools.artifact_inputs", fake)
    report = analyze(
        inventory.measure(root, detailed=True),
        repo,
        eligibility_top=1,
        cargo="fixture-cargo",
        scratch=tmp_path / "scratch",
    )
    path, body, context, options = calls[0]
    assert path == repo and body == "body-rust" and context == {}
    assert options == {
        "features": ("feature",),
        "default_features": False,
        "force_staticlib": True,
        "cargo": "fixture-cargo",
    }
    assert report["source_eligible_archive_only_model"]["snapshot_inodes"] == 1
    assert next(iter(report["source_discovery"].values()))["action_key"] is None
