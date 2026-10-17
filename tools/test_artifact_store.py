"""Real-process tests of immutable publication, supervisor lifetime and GC."""

import concurrent.futures
import json
import multiprocessing as mp
import os
import subprocess
import threading
import time
from pathlib import Path

import pytest

from tools import artifact_store as implementation
from tools.artifact_store import (
    ActionConflict,
    BlobCollision,
    BuildResult,
    CorruptStore,
    Output,
    SessionError,
    Store,
    StoreError,
    action_key,
    canonical,
)

DESCRIPTOR = {
    "schema": "fixture-action-v1",
    "request": {"body": "fixture"},
    "context": {"target": "host"},
}


def build_bytes(payload=b"archive bytes", *, readonly=False):
    def builder(scratch):
        target = scratch / "source" / "body.a"
        target.parent.mkdir()
        target.write_bytes(payload)
        if readonly:
            target.chmod(0o444)
            target.parent.chmod(0o555)
        return BuildResult({"body.a": Output(target)}, provenance={"fixture": True})

    return builder


def process_build(root, descriptor, counter, start, queue, payload=b"archive bytes"):
    try:
        store = Store(Path(root))
        with store.session() as session:
            start.wait()

            def builder(scratch):
                with open(counter, "ab", buffering=0) as output:
                    output.write(b"built\n")
                    os.fsync(output.fileno())
                time.sleep(0.1)
                target = scratch / "body.a"
                target.write_bytes(payload)
                return BuildResult({"body.a": target})

            handle = store.lookup_or_build(descriptor, builder, session=session)
            session.bind("lane", str(os.getpid()), handle.action_key)
            queue.put(
                {
                    "hit": handle.hit,
                    "bytes": handle.blob_path.read_bytes(),
                    "key": handle.action_key,
                }
            )
    except BaseException as error:
        queue.put({"error": repr(error)})
        raise


def blocked_producer(root, ready, release):
    store = Store(Path(root))
    with store.session() as session:

        def builder(scratch):
            ready.put({"token": session.token, "key": action_key(DESCRIPTOR)})
            release.wait()
            target = scratch / "body.a"
            target.write_bytes(b"archive bytes")
            return BuildResult({"body.a": target})

        store.lookup_or_build(DESCRIPTOR, builder, session=session)


def attached_reader(root, token, key, ready, release):
    store = Store(Path(root))
    with store.attach_session(token) as session:
        handle = session.pin(key)
        session.bind("child-lane", "consumer", key)
        ready.put(handle.blob_path.read_bytes())
        release.wait()


def run_gc(root, queue):
    try:
        queue.put(Store(Path(root)).gc(apply=True, grace_seconds=0))
    except BaseException as error:
        queue.put({"error": repr(error)})
        raise


@pytest.fixture
def store(tmp_path):
    return Store(tmp_path / "store")


def test_publish_hit_immutable_bytes_and_no_hardlinks(store):
    original = None

    def builder(scratch):
        nonlocal original
        original = scratch / "body.a"
        original.write_bytes(b"archive bytes")
        return BuildResult({"body.a": original})

    with store.session() as session:
        first = store.lookup_or_build(DESCRIPTOR, builder, session=session)
        second = store.lookup_or_build(
            DESCRIPTOR, lambda _: pytest.fail("hit compiled"), session=session
        )
        assert not first.hit and second.hit
        assert first.manifest == second.manifest
        assert first.blob_path.read_bytes() == b"archive bytes"
        assert first.blob_path.stat().st_nlink == 1
        assert not first.blob_path.stat().st_mode & 0o222
        assert original is not None
        assert not original.exists()
        assert not store.gc(apply=True, grace_seconds=0)["actions"]
    report = store.gc(apply=True, grace_seconds=0)
    assert len(report["actions"]) == len(report["blobs"]) == 1
    assert not first.blob_path.exists()


def test_four_same_key_processes_have_one_producer(tmp_path):
    context = mp.get_context("spawn")
    start, queue = context.Event(), context.Queue()
    counter = tmp_path / "counter"
    processes = [
        context.Process(
            target=process_build,
            args=(str(tmp_path / "store"), DESCRIPTOR, str(counter), start, queue),
        )
        for _ in range(4)
    ]
    for process in processes:
        process.start()
    start.set()
    results = [queue.get(timeout=20) for _ in processes]
    for process in processes:
        process.join(timeout=20)
        assert process.exitcode == 0
    assert counter.read_bytes() == b"built\n"
    assert sum(not result["hit"] for result in results) == 1
    assert {result["bytes"] for result in results} == {b"archive bytes"}
    assert len(list((tmp_path / "store/v1/blobs/sha256").glob("*/*"))) == 1


def test_distinct_actions_publish_same_blob_without_overwrite(tmp_path):
    context = mp.get_context("spawn")
    start, queue = context.Event(), context.Queue()
    counter = tmp_path / "counter"
    processes = [
        context.Process(
            target=process_build,
            args=(
                str(tmp_path / "store"),
                {**DESCRIPTOR, "variant": index},
                str(counter),
                start,
                queue,
            ),
        )
        for index in range(4)
    ]
    for process in processes:
        process.start()
    start.set()
    results = [queue.get(timeout=20) for _ in processes]
    for process in processes:
        process.join(timeout=20)
        assert process.exitcode == 0
    assert counter.read_bytes().count(b"built\n") == 4
    assert all(not result["hit"] for result in results)
    assert len(list((tmp_path / "store/v1/actions").glob("*.json"))) == 4
    assert len(list((tmp_path / "store/v1/blobs/sha256").glob("*/*"))) == 1


def test_killed_producer_publishes_nothing_and_gc_does_not_wait(tmp_path):
    context = mp.get_context("spawn")
    ready, release, gc_result = context.Queue(), context.Event(), context.Queue()
    process = context.Process(
        target=blocked_producer, args=(str(tmp_path / "store"), ready, release)
    )
    process.start()
    identity = ready.get(timeout=20)
    store = Store(tmp_path / "store")
    collector = context.Process(target=run_gc, args=(str(store.root), gc_result))
    collector.start()
    report = gc_result.get(timeout=10)  # would time out if GC waited for action lock
    collector.join(timeout=10)
    assert collector.exitcode == 0 and not report["staging"]
    assert not list((store.base / "actions").iterdir())
    process.kill()  # exact owned fixture process, never broad process matching
    process.join(timeout=10)
    assert process.exitcode != 0
    with pytest.raises(SessionError):
        store.attach_session(identity["token"])
    dead = store.gc(apply=True, grace_seconds=0)
    assert len(dead["staging"]) == len(dead["stale_leases"]) == 1
    with store.session() as session:
        handle = store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)
        assert handle.blob_path.read_bytes() == b"archive bytes"


def test_supervisor_child_attach_and_durable_ref_after_session_exit(store):
    context = mp.get_context("spawn")
    ready, release = context.Queue(), context.Event()
    with store.session() as supervisor:
        handle = store.lookup_or_build(DESCRIPTOR, build_bytes(), session=supervisor)
        process = context.Process(
            target=attached_reader,
            args=(str(store.root), supervisor.token, handle.action_key, ready, release),
        )
        process.start()
        assert ready.get(timeout=20) == b"archive bytes"
        release.set()
        process.join(timeout=20)
        assert process.exitcode == 0
        assert store.attach_session(supervisor.token)
    with pytest.raises(SessionError):
        store.attach_session(supervisor.token)
    assert not store.gc(apply=True, grace_seconds=0)["actions"]
    assert handle.blob_path.read_bytes() == b"archive bytes"
    with store.session() as new_supervisor:
        assert (
            store.restore_refs("child-lane", session=new_supervisor)[0].action_key
            == handle.action_key
        )
    with store.consumer_guard("child-lane", "consumer") as guard:
        store.release_ref("child-lane", "consumer", guard=guard)
    assert len(store.gc(apply=True, grace_seconds=0)["actions"]) == 1


def test_gc_vs_live_pin_and_explicit_ref_release(store):
    context = mp.get_context("spawn")
    with store.session() as session:
        handle = store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)
        queue = context.Queue()
        collectors = [
            context.Process(target=run_gc, args=(str(store.root), queue)) for _ in range(4)
        ]
        for process in collectors:
            process.start()
        for _ in collectors:
            assert not queue.get(timeout=20)["actions"]
            assert handle.blob_path.read_bytes() == b"archive bytes"
        for process in collectors:
            process.join(timeout=20)
            assert process.exitcode == 0
        with pytest.raises(StoreError, match="held consumer guard"):
            store.release_ref("lane", "consumer", guard=store.consumer_guard("lane", "consumer"))


def test_verify_rebuild_conflict_is_persistent_and_preserves_both(store):
    with store.session() as session:
        handle = store.lookup_or_build(DESCRIPTOR, build_bytes(b"first"), session=session)
        same = store.lookup_or_build(
            DESCRIPTOR, build_bytes(b"first"), session=session, verify=True
        )
        assert same.verified_rebuild and not same.hit
        with pytest.raises(ActionConflict):
            store.lookup_or_build(DESCRIPTOR, build_bytes(b"second"), session=session, verify=True)
        assert handle.blob_path.read_bytes() == b"first"
        evidence = json.loads(store._conflict_path(handle.action_key).read_bytes())
        retained = store._conflict_path(handle.action_key).parent / evidence["candidate_directory"]
        assert (retained / "output-0").read_bytes() == b"second"
        with pytest.raises(ActionConflict):
            store.lookup_or_build(
                DESCRIPTOR, lambda _: pytest.fail("conflict rebuilt"), session=session
            )


@pytest.mark.parametrize("corruption", ["missing", "truncated", "schema", "symlink"])
def test_corrupt_entry_never_silently_rebuilds(store, tmp_path, corruption):
    with store.session() as session:
        handle = store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)
        if corruption == "missing":
            handle.blob_path.unlink()
        elif corruption == "truncated":
            handle.blob_path.chmod(0o644)
            handle.blob_path.write_bytes(b"bad")
        elif corruption == "schema":
            manifest = dict(handle.manifest, schema="wrong")
            path = store._action_path(handle.action_key)
            path.chmod(0o644)
            path.write_bytes(canonical(manifest))
        else:
            handle.blob_path.unlink()
            external = tmp_path / "external"
            external.write_bytes(b"archive bytes")
            handle.blob_path.symlink_to(external)
        with pytest.raises(CorruptStore):
            store.lookup_or_build(
                DESCRIPTOR, lambda _: pytest.fail("corruption rebuilt"), session=session
            )


def test_digest_collision_compares_actual_bytes(store, monkeypatch):
    identity = "a" * 64
    monkeypatch.setattr(implementation, "_digest_file", lambda path: (identity, 4))
    with store.session() as session:
        first = store.lookup_or_build(DESCRIPTOR, build_bytes(b"aaaa"), session=session)
        with pytest.raises(BlobCollision):
            store.lookup_or_build(
                {**DESCRIPTOR, "variant": 2}, build_bytes(b"bbbb"), session=session
            )
        assert first.blob_path.read_bytes() == b"aaaa"
        assert not store._action_path(action_key({**DESCRIPTOR, "variant": 2})).exists()


def test_invalid_output_and_symlink_source_fail_before_publication(store, tmp_path):
    with store.session() as session:
        with pytest.raises(StoreError):
            store.lookup_or_build(
                DESCRIPTOR,
                lambda scratch: BuildResult({"../escape.a": scratch / "x"}),
                session=session,
            )

        def symlink(scratch):
            external = tmp_path / "external"
            external.write_bytes(b"data")
            target = scratch / "body.a"
            target.symlink_to(external)
            return BuildResult({"body.a": target})

        with pytest.raises(StoreError):
            store.lookup_or_build(DESCRIPTOR, symlink, session=session)
        assert not list((store.base / "actions").iterdir())


def test_readonly_worker_tree_cleanup_does_not_follow_symlinks(store, tmp_path):
    external = tmp_path / "external"
    external.mkdir(mode=0o555)

    def builder(scratch):
        (scratch / "outside-link").symlink_to(external, target_is_directory=True)
        return build_bytes(readonly=True)(scratch)

    with store.session() as session:
        handle = store.lookup_or_build(DESCRIPTOR, builder, session=session)
        assert handle.blob_path.exists()
    assert not list((store.base / "staging").iterdir())
    assert external.stat().st_mode & 0o777 == 0o555
    external.chmod(0o755)


def test_capability_owner_start_identity_and_lock_required(store):
    with store.session() as session:
        with pytest.raises(SessionError):
            store.attach_session(session.identifier + ":wrong")
        path = store._lease_path(session.identifier)
        record = json.loads(path.read_bytes())
        record["owner"]["start_identity"]["start_ticks"] = "not-current"
        implementation._replace_json(path, record)
        with pytest.raises(SessionError, match="dead/reused"):
            store.attach_session(session.token)
        # Even inconsistent PID evidence cannot overrule a held lease lock.
        report = store.gc(apply=True, grace_seconds=0)
        assert not report["stale_leases"] and report["lease_anomalies"]


def test_gc_dry_run_and_dependency_action_reachability(store):
    with store.session() as session:
        dependency = store.lookup_or_build(
            {"dependency": True}, build_bytes(b"dependency"), session=session
        )

        def parent(scratch):
            built = build_bytes(b"parent")(scratch)
            built.dependency_actions = (dependency.action_key,)
            return built

        handle = store.lookup_or_build(DESCRIPTOR, parent, session=session)
        session.bind("lane", "parent", handle.action_key)
    assert not store.gc(apply=True, grace_seconds=0)["actions"]
    with store.consumer_guard("lane", "parent") as guard:
        store.release_ref("lane", "parent", guard=guard)
    dry = store.gc(grace_seconds=0)
    assert len(dry["actions"]) == len(dry["blobs"]) == 2
    assert handle.blob_path.exists() and dependency.blob_path.exists()
    store.gc(apply=True, grace_seconds=0)
    assert not handle.blob_path.exists()
    assert (store.base / "locks" / (handle.action_key + ".lock")).exists()


def test_verify_once_per_supervisor_and_cold_verification_status(store):
    calls = []

    def builder(scratch):
        calls.append(True)
        return build_bytes()(scratch)

    with store.session() as supervisor:
        cold = store.lookup_or_build(
            DESCRIPTOR, builder, session=supervisor, verify=True, verify_once_per_session=True
        )
        assert not cold.verified_rebuild and not cold.session_verified
        with store.attach_session(supervisor.token) as child:
            reused = store.lookup_or_build(
                DESCRIPTOR, builder, session=child, verify=True, verify_once_per_session=True
            )
            assert reused.hit and not reused.session_verified
            verified = store.lookup_or_build(DESCRIPTOR, builder, session=child, verify=True)
            assert verified.verified_rebuild and verified.session_verified
            again = store.lookup_or_build(
                DESCRIPTOR, builder, session=child, verify=True, verify_once_per_session=True
            )
            assert again.hit and again.session_verified and not again.verified_rebuild
        assert len(calls) == 2
    with store.session() as fresh_supervisor:
        store.lookup_or_build(
            DESCRIPTOR, builder, session=fresh_supervisor, verify=True, verify_once_per_session=True
        )
    assert len(calls) == 3


def test_concurrent_cold_verification_compares_once_per_supervisor(store):
    calls = []
    ready = threading.Barrier(4)

    def builder(scratch):
        calls.append(scratch)
        time.sleep(0.05)
        return build_bytes()(scratch)

    with store.session() as supervisor:

        def consume(_):
            with store.attach_session(supervisor.token) as child:
                ready.wait(timeout=10)
                result = store.lookup_or_build(
                    DESCRIPTOR,
                    builder,
                    session=child,
                    verify=True,
                    verify_once_per_session=True,
                    require_verified=True,
                )
                if not result.session_verified:
                    result = store.lookup_or_build(
                        DESCRIPTOR,
                        builder,
                        session=child,
                        verify=True,
                        verify_once_per_session=True,
                        require_verified=True,
                    )
                return result

        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as workers:
            results = list(workers.map(consume, range(4)))
        assert len(calls) == 2  # Baseline plus one comparison, even under contention.
        assert all(result.session_verified for result in results)
        assert len({result.blob_path for result in results}) == 1


def test_restore_refs_verifies_shared_action_once(store, monkeypatch):
    with store.session() as session:
        handle = store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)
        for consumer in ("a", "b", "c"):
            session.bind("lane", consumer, handle.action_key)
    original_digest = implementation._digest_file
    reads = []

    def counted(path):
        reads.append(path)
        return original_digest(path)

    monkeypatch.setattr(implementation, "_digest_file", counted)
    with store.session() as supervisor:
        restored = store.restore_refs("lane", session=supervisor)
        assert len(restored) == 1
    assert reads == [handle.blob_path]


def test_explicit_missing_blob_repair_preserves_manifest(store):
    with store.session() as session:
        handle = store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)
        original = store._action_path(handle.action_key).read_bytes()
        handle.blob_path.unlink()
        with pytest.raises(CorruptStore):
            store.lookup(handle.action_key, session=session)
        repaired = store.repair_missing(handle.action_key, build_bytes(), session=session)
        assert repaired.blob_path.read_bytes() == b"archive bytes"
        assert store._action_path(handle.action_key).read_bytes() == original
        with pytest.raises(StoreError, match="at least one absent"):
            store.repair_missing(handle.action_key, build_bytes(), session=session)


def test_explicit_repair_rejects_wronghash_present_and_conflicting_rebuild(store):
    with store.session() as session:
        handle = store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)
        handle.blob_path.chmod(0o644)
        handle.blob_path.write_bytes(b"wrong")
        with pytest.raises(CorruptStore):
            store.repair_missing(
                handle.action_key, lambda _: pytest.fail("wronghash repaired"), session=session
            )
        handle.blob_path.unlink()
        with pytest.raises(ActionConflict):
            store.repair_missing(handle.action_key, build_bytes(b"not original"), session=session)
        assert not handle.blob_path.exists()
        assert store._conflict_path(handle.action_key).exists()


def test_failed_scratch_has_bounded_gc_retention_with_live_owner(store):
    with store.session() as session:
        with pytest.raises(StoreError):
            store.lookup_or_build(DESCRIPTOR, lambda _: BuildResult({}), session=session)
        report = store.gc(apply=True, grace_seconds=0)
        assert len(report["staging"]) == 1


def killed_during_publication(root, ready, release):
    original = implementation._rename_noreplace

    def interrupted(source, destination):
        published = original(source, destination)
        if published and "/blobs/sha256/" in str(destination):
            ready.put(str(destination))
            release.wait()
        return published

    implementation._rename_noreplace = interrupted
    store = Store(Path(root))
    with store.session() as session:
        store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)


def test_kill_after_blob_rename_never_exposes_partial_action(tmp_path):
    context = mp.get_context("spawn")
    ready, release = context.Queue(), context.Event()
    process = context.Process(
        target=killed_during_publication, args=(str(tmp_path / "store"), ready, release)
    )
    process.start()
    blob = Path(ready.get(timeout=20))
    assert blob.read_bytes() == b"archive bytes"
    process.kill()
    process.join(timeout=20)
    store = Store(tmp_path / "store")
    assert not list((store.base / "actions").iterdir())
    report = store.gc(apply=True, grace_seconds=0)
    assert len(report["blobs"]) == len(report["staging"]) == 1
    assert not blob.exists()
    with store.session() as session:
        assert store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session).blob_path.exists()


def test_disk_full_manifest_publication_exposes_no_action(store, monkeypatch):
    original = implementation._write_new

    def no_manifest(path, payload):
        if path.name == "action.json":
            raise OSError(28, "No space left on device")
        original(path, payload)

    monkeypatch.setattr(implementation, "_write_new", no_manifest)
    with store.session() as session:
        with pytest.raises(OSError, match="No space"):
            store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)
        assert not store.manifest_path(action_key(DESCRIPTOR)).exists()
        assert len(store.gc(apply=True, grace_seconds=0)["blobs"]) == 1


def test_status_counts_filesystem_bytes_and_stable_lock_objects(store):
    with store.session() as session:
        handle = store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)
        session.bind("lane", "consumer", handle.action_key)
        report = store.status()
        assert not report["verified"]
        assert report["roles"]["blobs"]["files"] == 1
        assert report["roles"]["blobs"]["logical_bytes"] == len(b"archive bytes")
        assert report["roles"]["actions"]["files"] == 1
        assert report["roles"]["refs"]["files"] == 1
        assert report["allocated_bytes"] >= handle.blob_path.stat().st_blocks * 512
        measured = subprocess.check_output(
            ["du", "-s", "--block-size=1", str(store.root)], text=True
        )
        assert report["allocated_bytes"] == int(measured.split()[0])


def test_gc_requires_both_released_lock_and_nonmatching_owner_before_reaping(store):
    with store.session() as session:
        handle = store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)
        path = store._lease_path(session.identifier)
        original = json.loads(path.read_bytes())
    # Simulate a lost lease lock from the still-live supervisor: never TTL-reap.
    implementation._replace_json(path, original)
    assert not store.gc(apply=True, grace_seconds=0)["stale_leases"]
    assert handle.blob_path.exists()
    with pytest.raises(SessionError, match="OS lease lock"):
        store.attach_session(session.token)
    # Same PID with a different start identity is a stale/reused owner.
    original["owner"]["start_identity"]["start_ticks"] = "old-process-start"
    implementation._replace_json(path, original)
    report = store.gc(apply=True, grace_seconds=0)
    assert len(report["stale_leases"]) == len(report["actions"]) == 1
    assert not handle.blob_path.exists()


def test_public_metadata_inspection_and_receipt_listing_do_not_certify_blobs(store):
    with store.session() as session:
        handle = store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)
        session.bind("lane", "cargo:consumer", handle.action_key)
        handle.blob_path.unlink()
        manifest = store.inspect_manifest(handle.action_key, session=session)
        assert manifest == handle.manifest
        refs = store.list_refs("lane")
        assert len(refs) == 1 and refs[0]["consumer"] == "cargo:consumer"
        assert not store.list_refs("other")
        with pytest.raises(CorruptStore):
            store.restore_refs("lane", session=session)
        path = store.manifest_path(handle.action_key)
        path.chmod(0o644)
        path.write_bytes(canonical({**manifest, "descriptor": {"tampered": True}}))
        with pytest.raises(CorruptStore):
            store.inspect_manifest(handle.action_key)


def test_corrupt_blob_quarantine_copies_evidence_without_moving_live_path(store):
    with store.session() as session:
        handle = store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)
        inode = handle.blob_path.stat().st_ino
        handle.blob_path.chmod(0o644)
        handle.blob_path.write_bytes(b"corrupt but preserve the live path")
        with pytest.raises(CorruptStore):
            session.pin(handle.action_key)
        assert handle.blob_path.stat().st_ino == inode
        assert handle.blob_path.read_bytes() == b"corrupt but preserve the live path"
        folders = list((store.base / "quarantine" / handle.action_key).glob("corruption-*"))
        assert len(folders) == 1
        retained = folders[0] / (handle.manifest["outputs"][0]["sha256"] + ".corrupt")
        assert retained.read_bytes() == handle.blob_path.read_bytes()
        assert retained.stat().st_ino != inode and retained.stat().st_nlink == 1
        assert (folders[0] / "manifest.raw").read_bytes() == store.manifest_path(
            handle.action_key
        ).read_bytes()
        diagnostic = json.loads((folders[0] / "diagnostic.json").read_bytes())
        assert (
            diagnostic["outputs"][0]["observed_sha256"] != handle.manifest["outputs"][0]["sha256"]
        )


def test_gc_drops_a_dead_ref_and_then_its_action(store):
    """A ref outlives its consumer unless the caller can say the consumer is gone."""
    with store.session() as session:
        handle = store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)
        session.bind("/gone/lane", "cargo:x|/tmp/vibeos/out|x", handle.action_key)
    # Every ref is a root without a predicate, and with one that says "alive".
    assert not store.gc(apply=True, grace_seconds=0)["actions"]
    assert not store.gc(apply=True, grace_seconds=0, ref_alive=lambda ref: True)["actions"]
    report = store.gc(apply=True, grace_seconds=0, ref_alive=lambda ref: False)
    assert len(report["dead_refs"]) == 1
    assert report["actions"] and report["blobs"]
    assert not list((store.base / "refs").glob("*.json"))
    assert not handle.blob_path.exists()


def test_gc_keeps_a_young_dead_ref(store):
    with store.session() as session:
        handle = store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)
        session.bind("/gone/lane", "consumer", handle.action_key)
    report = store.gc(apply=True, grace_seconds=3600, ref_alive=lambda ref: False)
    assert not report["dead_refs"] and not report["actions"]


def test_gc_hashes_only_the_blobs_it_deletes(store, monkeypatch):
    """Kept blobs are verified where they are consumed; hashing all of them here held the
    GC lock for minutes."""
    with store.session() as session:
        store.lookup_or_build(DESCRIPTOR, build_bytes(), session=session)
        hashed = []
        original = implementation._digest_file
        monkeypatch.setattr(
            implementation, "_digest_file", lambda path: hashed.append(path) or original(path)
        )
        assert not store.gc(apply=True, grace_seconds=0)["blobs"]
    assert hashed == []


def test_lane_ref_alive_maps_the_canonical_out_dir_back_to_the_lane(tmp_path):
    from tools.artifact_runtime import lane_ref_alive

    lane = tmp_path / "lane"
    out = lane / "sdk/lib/rust/target/cell/build/x-cargo-abc/out"
    out.mkdir(parents=True)
    consumer = "cargo:x-cargo|/tmp/vibeos/sdk/lib/rust/target/cell/build/x-cargo-abc/out|x|[]"
    ref = {"lane": str(lane), "consumer": consumer}
    assert lane_ref_alive(ref, canonical_root="/tmp/vibeos")
    out.rmdir()
    assert not lane_ref_alive(ref, canonical_root="/tmp/vibeos")
    assert not lane_ref_alive({"lane": str(tmp_path / "removed"), "consumer": consumer})
    # A shape it cannot read is kept, never guessed dead.
    assert lane_ref_alive({"lane": str(lane), "consumer": "standalone:x"})


def test_scheduled_gc_runs_once_per_interval_and_honours_the_kill_switch(store):
    from tools.artifact_runtime import GC_INTERVAL_SECONDS, scheduled_gc

    root = store.root
    assert scheduled_gc(root, now=1_000_000.0) is not None
    stamp = root / "last-gc"
    os.utime(stamp, (1_000_000.0, 1_000_000.0))
    assert scheduled_gc(root, now=1_000_000.0 + 60) is None
    assert scheduled_gc(root, now=1_000_000.0 + GC_INTERVAL_SECONDS + 1) is not None
    (root / "gc-disabled").touch()
    assert scheduled_gc(root, force=True) is None
