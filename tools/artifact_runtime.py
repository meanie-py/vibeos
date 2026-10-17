"""Body adapter: supervised leases, exact discovery and isolated misses, per arch.

The outer build owns disk admission and the supervisor through final media/link
completion. Requests wait for the store action lock before acquiring a compile
permit. Selected requests fail loudly; this module has no private-build fallback.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import stat
import subprocess
import sys
import tempfile
import time
import uuid
from contextlib import contextmanager
from functools import wraps
from pathlib import Path
from types import SimpleNamespace

SCHEMA = "vibeos-artifact-runtime-v1"
TARGET = "x86_64-pc-windows-gnu"
_DLLTOOL_FLAG = "-C dlltool=/native-tools/bin/llvm-dlltool"
KERNEL_FLAGS = "-C force-frame-pointers=yes"


class RuntimeError_(RuntimeError):
    """A selected request could not meet the pilot contract."""


def _canonical(value):
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True, allow_nan=False
    ).encode()


def _key(value):
    return hashlib.sha256(_canonical(value)).hexdigest()


def _apis():
    # Direct script invocation and python -m both work without ambient PYTHONPATH.
    if __package__ in {None, ""}:
        sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    from tools import artifact_executor, artifact_inputs, artifact_store

    return SimpleNamespace(
        executor=artifact_executor, inputs=artifact_inputs, storage=artifact_store
    )


def _config_guard():
    from tools import artifact_config

    return artifact_config


def _regular(path):
    try:
        return stat.S_ISREG(path.lstat().st_mode)
    except FileNotFoundError:
        return False


def _read_json(path):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    with os.fdopen(fd, "rb") as stream:
        before = os.fstat(stream.fileno())
        if not stat.S_ISREG(before.st_mode):
            raise RuntimeError_(f"not a regular JSON file: {path}")
        payload = stream.read()
        after = os.fstat(stream.fileno())
    stamp = lambda st: (st.st_dev, st.st_ino, st.st_size, st.st_mtime_ns, st.st_ctime_ns)  # noqa:E731
    if stamp(before) != stamp(after) or stamp(after) != stamp(path.lstat()):
        raise RuntimeError_(f"JSON changed while reading: {path}")
    result = json.loads(payload)
    if not isinstance(result, dict):
        raise RuntimeError_(f"JSON must be an object: {path}")
    return result


def _write_json(path, value):
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC, 0o600)
    with os.fdopen(fd, "wb") as stream:
        stream.write(_canonical(value))
        stream.flush()
        os.fsync(stream.fileno())


def _paths(repo, store_root):
    if store_root == repo or repo in store_root.parents or store_root in repo.parents:
        raise RuntimeError_("artifact store and source checkout must be disjoint directories")


def supervisor_config(path):
    """Read private context; callers must not print its session capability."""
    state = _read_json(Path(path))
    if state.get("schema") != SCHEMA:
        raise RuntimeError_("unsupported artifact supervisor schema")
    for name in ("repo", "store_root", "toolchain", "activity", "broker"):
        if not isinstance(state.get(name), str) or not Path(state[name]).is_absolute():
            raise RuntimeError_(f"supervisor {name} must be an absolute path")
    _paths(Path(state["repo"]), Path(state["store_root"]))
    if _key(state["provisioned"]) != state["toolchain_key"]:
        raise RuntimeError_("supervisor toolchain descriptor changed")
    if not isinstance(state.get("clean"), bool) or not isinstance(state.get("verify"), bool):
        raise RuntimeError_("supervisor clean/verify must be booleans")
    repair_action = state.get("repair_action")
    if repair_action is not None and (
        not isinstance(repair_action, str) or not re.fullmatch(r"[0-9a-f]{64}", repair_action)
    ):
        raise RuntimeError_("repair action must be a SHA-256 action identity")
    return state


def _receipt_stats(activity):
    attempts = [_read_json(path) for path in sorted(activity.glob("*.json"))]
    receipts = [r for r in attempts if r.get("status", "success") == "success"]
    failures = [r for r in attempts if r.get("status") == "failed"]
    ineligible = [r for r in attempts if r.get("status") == "ineligible"]
    peaks = [
        r["sampled_worker_target_peak_allocated_bytes"]
        for r in receipts
        if r.get("sampled_worker_target_peak_allocated_bytes") is not None
    ]
    return {
        "requested": len(attempts),
        "successful_requests": len(receipts),
        "failed_requests": len(failures),
        "failure_examples": [{"body": r["body"], "error": r["error"]} for r in failures[:10]],
        "ineligible_requests": len(ineligible),
        "ineligible_bodies": {r["body"]: r["reasons"][0] for r in ineligible},
        "hits": sum(bool(r["hit"]) for r in receipts),
        "misses": sum(not r["hit"] for r in receipts),
        "builder_calls": sum(r["builder_calls"] for r in receipts),
        "failed_request_builder_calls": "unknown" if failures else 0,
        "verification_compared_requests": sum(bool(r["verification_compared"]) for r in receipts),
        "actual_verified_rebuild_requests": sum(bool(r["verified_rebuild"]) for r in receipts),
        "unique_actions": sorted({r["action_key"] for r in receipts}),
        "repaired_actions": sorted({r["action_key"] for r in receipts if r.get("repaired")}),
        "sampled_worker_target_peak_allocated_bytes": max(peaks) if peaks else None,
        "worker_requests_without_peak_measurement": sum(
            r["builder_calls"] > 0 and r.get("sampled_worker_target_peak_allocated_bytes") is None
            for r in receipts
        ),
        "request_timings": {
            name: sum(r.get("timings", {}).get(name, 0) for r in receipts)
            for name in (
                "discovery_seconds",
                "lookup_wait_build_seconds",
                "revalidation_seconds",
                "total_seconds",
                "compile_seconds",
            )
        },
    }


def record_outer_result(supervisor, exit_code, *, _api=None):
    """Caller returning an rc instead of raising records its actual build result."""
    if not isinstance(exit_code, int) or isinstance(exit_code, bool):
        raise RuntimeError_("outer build exit_code must be an integer")
    state = supervisor_config(supervisor)
    api = _api or _apis()
    store = api.storage.Store(Path(state["store_root"]))
    with store.attach_session(state["session_token"]):
        _write_json(Path(state["outer_result"]), {"exit_code": exit_code})


def _publish_summary(context_dir, private, summary):
    # Each private run retains its immutable report. The public location is a
    # replaceable pointer/report for callers reusing the context directory.
    _write_json(private / "summary.json", summary)
    temporary = context_dir / (".summary-" + uuid.uuid4().hex + ".json")
    _write_json(temporary, summary)
    os.replace(temporary, context_dir / "summary.json")
    descriptor = os.open(context_dir, os.O_RDONLY | os.O_DIRECTORY | os.O_CLOEXEC)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


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
    """Own outer-build leases and record public, capability-free outcome evidence.

    `arch` is the architecture being built; a clean build retires only its refs.

    Caller already owns its disk reservation. No compile permit is acquired
    here. Final toolchain validation precedes a successful/verified outcome.
    """
    started = time.monotonic()
    context_dir = Path(context_dir).resolve()
    context_dir.mkdir(parents=True, exist_ok=True)
    private = Path(tempfile.mkdtemp(prefix="artifact-supervisor-", dir=context_dir))
    activity = private / "activity"
    activity.mkdir(mode=0o700)
    summary = {
        "schema": "vibeos-artifact-supervisor-summary-v1",
        "started_ns": time.time_ns(),
        "finished_ns": None,
        "status": "failed",
        "error": None,
        "clean": clean,
        "verify": verify,
        "repair_action": repair_action,
        "repair_success": False,
        "toolchain_key": None,
        "toolchain_postcheck": "not_run",
        "config_preflight": "not_run",
        "config_postcheck": "not_run",
        "verification_success": False,
        "requests": None,
        "timings": {},
    }
    failure = None
    try:
        api = _api or _apis()
        repo, store_root, toolchain = (Path(p).resolve() for p in (repo, store_root, toolchain))
        summary.update(repo=str(repo), store_root=str(store_root), toolchain=str(toolchain))
        _paths(repo, store_root)
        broker = repo / ".cache/host-jobs/host-jobs"
        if not _regular(broker) or not os.access(broker, os.X_OK):
            raise RuntimeError_(f"compiler admission broker missing or not executable: {broker}")
        if not isinstance(clean, bool) or not isinstance(verify, bool):
            raise RuntimeError_("clean and verify must be booleans")
        if repair_action is not None and (
            not isinstance(repair_action, str)
            or not re.fullmatch(r"[0-9a-f]{64}", repair_action)
            or clean
            or verify
        ):
            raise RuntimeError_(
                "repair-only supervision requires one action key without clean/verify"
            )
        config_started = time.monotonic()
        try:
            summary["cargo_config_preflight"] = _config_guard().audit_scope(repo, broker=broker)
            summary["config_preflight"] = "valid"
        except BaseException:
            summary["config_preflight"] = "failed"
            raise
        finally:
            summary["timings"]["config_preflight_seconds"] = time.monotonic() - config_started
        provision_started = time.monotonic()
        try:
            provisioned = api.executor.provision(toolchain)
        finally:
            summary["timings"]["provision_seconds"] = time.monotonic() - provision_started
        summary["toolchain_key"] = _key(provisioned)
        store = api.storage.Store(store_root)
        with store.session() as session:
            try:
                preflight_started = time.monotonic()
                try:
                    if repair_action is None:
                        restored = store.restore_refs(str(repo), session=session)
                    else:
                        # Explicit repair pins indexed metadata; this is not an
                        # archive handle and does not certify any existing blob.
                        store.inspect_manifest(repair_action, session=session)
                        restored = []
                finally:
                    summary["timings"]["preflight_seconds"] = time.monotonic() - preflight_started
                state = {
                    "schema": SCHEMA,
                    "repo": str(repo),
                    "store_root": str(store_root),
                    "toolchain": str(toolchain),
                    "provisioned": provisioned,
                    "toolchain_key": summary["toolchain_key"],
                    "session_token": session.token,
                    "activity": str(activity),
                    "broker": str(broker),
                    "summary": str(context_dir / "summary.json"),
                    "outer_result": str(private / "outer-result.json"),
                    "clean": clean,
                    "verify": verify,
                    "restored_actions": sorted({h.action_key for h in restored}),
                    "repair_action": repair_action,
                    "arch": arch,
                    "strict": os.environ.get("VIBEOS_ARTIFACT_STRICT") == "1",
                }
                summary["restored_actions"] = state["restored_actions"]
                summary["preflight_mode"] = (
                    "restore lane refs" if repair_action is None else "pin repair metadata only"
                )
                path = private / "supervisor.json"
                _write_json(path, state)
                outer_started = time.monotonic()
                try:
                    yield path
                finally:
                    summary["timings"]["outer_work_seconds"] = time.monotonic() - outer_started
                summary["requests"] = _receipt_stats(activity)
                if repair_action is not None:
                    if summary["requests"]["repaired_actions"] != [repair_action]:
                        raise RuntimeError_(
                            "repair requested but no matching successful repair recorded"
                        )
                    restored = store.restore_refs(str(repo), session=session)
                    summary["restored_actions"] = sorted({h.action_key for h in restored})
                outer = Path(state["outer_result"])
                summary["outer_exit_code"] = (
                    _read_json(outer)["exit_code"] if outer.exists() else None
                )
                if (
                    verify
                    and not summary["requests"]["verification_compared_requests"]
                    and summary["outer_exit_code"] in {None, 0}
                ):
                    raise RuntimeError_(
                        "artifact verification requested but no selected action compared"
                    )
            finally:
                config_started = time.monotonic()
                try:
                    summary["cargo_config_postcheck"] = _config_guard().audit_scope(
                        repo, broker=broker
                    )
                    summary["config_postcheck"] = "valid"
                except BaseException:
                    summary["config_postcheck"] = "failed"
                    raise
                finally:
                    summary["timings"]["config_postcheck_seconds"] = (
                        time.monotonic() - config_started
                    )
                postcheck_started = time.monotonic()
                try:
                    if api.executor.provision(toolchain) != provisioned:
                        summary["toolchain_postcheck"] = "changed"
                        raise RuntimeError_(
                            "supervisor toolchain/runtime changed during outer build"
                        )
                    summary["toolchain_postcheck"] = "identical"
                except BaseException:
                    if summary["toolchain_postcheck"] != "changed":
                        summary["toolchain_postcheck"] = "failed"
                    raise
                finally:
                    summary["timings"]["postcheck_seconds"] = time.monotonic() - postcheck_started
        if summary["outer_exit_code"] not in {None, 0} or summary["requests"]["failed_requests"]:
            summary["error"] = {
                "type": "OuterBuildFailure",
                "message": f"outer exit code {summary['outer_exit_code']}; "
                f"{summary['requests']['failed_requests']} failed selected requests",
            }
        else:
            summary["status"] = "success"
            summary["verification_success"] = bool(verify)
            summary["repair_success"] = repair_action is not None
    except BaseException as exc:
        failure = exc
        summary["error"] = {"type": type(exc).__name__, "message": str(exc)}
        raise
    finally:
        # Failed compiler requests have no successful receipt; retain the
        # original failure and state explicitly when receipt coverage fails.
        if summary["requests"] is None:
            try:
                summary["requests"] = _receipt_stats(activity)
            except (OSError, ValueError, KeyError) as exc:
                summary["receipt_error"] = {"type": type(exc).__name__, "message": str(exc)}
                if failure is None:
                    summary["status"] = "failed"
                    summary["verification_success"] = False
                    summary["error"] = summary["receipt_error"]
                    failure = exc
        summary["finished_ns"] = time.time_ns()
        summary["timings"]["total_seconds"] = time.monotonic() - started
        _publish_summary(context_dir, private, summary)
        if failure is not None and summary.get("receipt_error") == summary["error"]:
            raise failure


def _request_options(
    body,
    features,
    default_features,
    kernel,
    flags,
    architecture,
    target,
    build_std,
    emit_ir,
    force_staticlib,
):
    targets = _apis().executor.TARGETS
    if not isinstance(body, str) or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_-]*", body):
        raise RuntimeError_(f"body must name one workspace package: {body!r}")
    if targets.get(architecture) != target or build_std or emit_ir or force_staticlib:
        raise RuntimeError_(
            "store requires builtin-target manifest staticlibs without build-std/IR"
        )
    if not isinstance(kernel, bool) or not isinstance(default_features, bool):
        raise RuntimeError_("kernel and default_features must be booleans")
    if (
        not isinstance(flags, str)
        or flags.strip() != _apis().executor.FLAGS[(architecture, kernel)]
    ):
        raise RuntimeError_(f"unsupported RUSTFLAGS for {architecture}: {flags!r}")
    if not isinstance(features, (list, tuple)) or any(
        not isinstance(f, str) or not re.fullmatch(r"[A-Za-z0-9_-]+", f) for f in features
    ):
        raise RuntimeError_("features must be explicit nonempty package feature names")
    return {"features": tuple(sorted(set(features))), "default_features": default_features}


def _recipe(context):
    """The (target, flags) recipe an action context was made from."""
    flags = context["effective_flags"]
    if not flags.endswith(_DLLTOOL_FLAG):
        raise RuntimeError_("action context lacks the audited dlltool flag")
    return {"target": context["target"]["builtin"], "flags": flags[: -len(_DLLTOOL_FLAG)].strip()}


def _watches(repo, discovery):
    paths = set()
    for relative in discovery["provenance"]["reads"]:
        path = repo / relative
        if Path(relative).is_absolute() or ".." in Path(relative).parts:
            raise RuntimeError_("discovery watch path is not repository relative")
        # Ninja must also see file edits and directory membership changes;
        # relying only on Cargo's recursive directory scan can miss the edge
        # that would invoke Cargo in the first place.
        paths.add(path)
    for relative, package in discovery["descriptor"]["packages"].items():
        if Path(relative).is_absolute() or ".." in Path(relative).parts:
            raise RuntimeError_("discovery package root is not repository relative")
        paths.add(repo / relative)
        for directory in package["inventory"]["directories"]:
            if Path(directory).is_absolute() or ".." in Path(directory).parts:
                raise RuntimeError_("discovery membership directory escapes package")
            paths.add(repo / relative / directory)
    paths.add(repo / "rust-toolchain.toml")
    for name in (
        "artifact_runtime",
        "artifact_inputs",
        "artifact_executor",
        "artifact_worker",
        "artifact_store",
        "artifact_config",
    ):
        paths.add(repo / "tools" / (name + ".py"))
    for path in paths:
        if not path.exists() or path.is_symlink():
            raise RuntimeError_(f"required watch is absent or symlinked: {path}")
    return sorted({str(p) for p in paths})


def retire_build_refs(supervisor, *, _api=None):
    """Release Cargo refs AFTER the clean build's output removal under build lock.

    Only the supervised arch's: a clean build removes that arch's Cargo outputs, and a
    consumer's identity carries its per-arch target dir (`<cell>-<arch>-cargo-link`).
    The other arches' fresh build scripts still point at their blobs, so releasing
    their refs would let GC delete a file the next warm build links.

    The supervisor's preflight pins remain until outer-build exit. The caller
    must already have removed current Cargo outputs using its ordinary clean
    mechanism; this function never deletes outputs or standalone-body refs.
    """
    state = supervisor_config(supervisor)
    if not state["clean"]:
        raise RuntimeError_("Cargo ref retirement requires a clean supervisor after output removal")
    api = _api or _apis()
    store = api.storage.Store(Path(state["store_root"]))
    lane = state["repo"]
    retired = []
    arch = state.get("arch")
    # Per-cell dirs end `<cell>-<arch>-cargo-link`; shared shards are
    # `cell-link-v1-<arch>-<n>`.
    markers = (f"-{arch}-cargo-link/", f"/cell-link-v1-{arch}-") if arch else ("",)
    with store.attach_session(state["session_token"]):
        for reference in store.list_refs(lane):
            consumer = reference["consumer"]
            if consumer.startswith("cargo:") and any(m in consumer for m in markers):
                with store.consumer_guard(lane, reference["consumer"]) as guard:
                    store.release_ref(lane, reference["consumer"], guard=guard)
                retired.append(reference)
    return retired


GC_INTERVAL_SECONDS = 24 * 3600


def lane_ref_alive(reference: dict, *, canonical_root: str | None = None) -> bool:
    """Is the consumer a durable ref was bound for still there?

    A ref's lane is the worktree path, and a `cargo:` consumer names the build-script
    `out` directory under the canonical mount (`/tmp/vibeos`, which is the lane during
    its build). Map it back to the lane and look. A missing worktree, or an `out`
    directory cargo has since replaced, can never link the blob again. Unknown consumer
    shapes are kept: this only ever answers "dead" for something it can see is gone.
    """
    lane = Path(reference["lane"])
    if not lane.is_dir():
        return False
    consumer = reference["consumer"]
    if not consumer.startswith("cargo:"):
        return True
    fields = consumer[len("cargo:") :].split("|")
    root = (canonical_root or os.environ.get("VIBEOS_CANON") or "/tmp/vibeos").rstrip("/")
    if len(fields) < 2 or not fields[1].startswith(root + "/"):
        return True
    return (lane / fields[1][len(root) + 1 :]).is_dir()


def scheduled_gc(store_root: Path, *, now: float | None = None, force: bool = False):
    """Collect the shared store at most once per GC_INTERVAL_SECONDS. Returns the report
    or None when skipped.

    Run after a green build (build.py), so the fleet needs no cron and no owner: whichever
    lane finishes first after the interval does it, under the store's own GC lock and
    a non-blocking claim so two lanes never queue up behind each other. The kill switch
    is a `gc-disabled` file in the store root, outside every worktree, so it reaches
    every lane at once.
    """
    import fcntl

    if not (store_root / "v1").is_dir() or (store_root / "gc-disabled").exists():
        return None
    stamp = store_root / "last-gc"
    now = time.time() if now is None else now
    try:
        if not force and now - stamp.stat().st_mtime < GC_INTERVAL_SECONDS:
            return None
    except FileNotFoundError:
        pass
    with open(store_root / "gc-claim.lock", "w") as claim:
        try:
            fcntl.flock(claim, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            return None
        report = _apis().storage.Store(store_root).gc(apply=True, ref_alive=lane_ref_alive)
        stamp.write_text(
            json.dumps({"at": now, **{k: len(v) for k, v in report.items() if isinstance(v, list)}})
        )
        return report


def _execute(repo, broker, output, executor):
    if os.environ.get("VIBEOS_HOST_JOB_HELD") == "1":
        return executor.run(output)
    command = [
        str(broker),
        "--kind",
        "compile",
        "--",
        sys.executable,
        "-m",
        "tools.artifact_executor",
        "run",
        str(output),
    ]
    implementation_root = Path(executor.__file__).resolve().parents[1]
    completed = subprocess.run(
        command, cwd=implementation_root, capture_output=True, text=True, check=False
    )
    if completed.returncode:
        diagnostic = (completed.stderr + completed.stdout)[-2000:]
        raise RuntimeError_(
            f"isolated compiler failed ({completed.returncode}); "
            f"evidence {output / 'report.json'}: {diagnostic}"
        )
    return _read_json(output / "report.json")


def _staticlib(manifest):
    """A manifest staticlib: `cargo build --lib` writes the archive the store
    publishes. An extra `rlib` (cmlib-rust, rtl-rust: bodies that are also Rust
    dependencies) changes nothing about that archive."""
    kinds = manifest.get("lib", {}).get("crate-type")
    return isinstance(kinds, list) and "staticlib" in kinds and set(kinds) <= {"staticlib", "rlib"}


def _ineligible(state, body, reasons, started):
    """An unsupported graph builds through the existing private path: a dispatch
    decision, recorded with its reason, never a claimed hit. Strict mode (tests and
    eligibility work) refuses it instead, so nothing exercises the old path quietly."""
    if state.get("strict"):
        raise RuntimeError_(f"strict store: {body} is ineligible: " + "; ".join(reasons))
    result = {
        "schema": SCHEMA,
        "status": "ineligible",
        "body": body,
        "reasons": list(reasons),
        "timings": {"total_seconds": time.monotonic() - started},
    }
    _write_json(Path(state["activity"]) / (uuid.uuid4().hex + ".json"), result)
    return result


def _record_request_failures(function):
    @wraps(function)
    def observed(supervisor, body, consumer, **options):
        started = time.monotonic()
        try:
            return function(supervisor, body, consumer, **options)
        except Exception as original:
            # A dead/invalid supervisor cannot own new activity evidence. The
            # request still fails; never create a replacement lease or fallback.
            try:
                state = supervisor_config(supervisor)
                api = options.get("_api") or _apis()
                store = api.storage.Store(Path(state["store_root"]))
                with store.attach_session(state["session_token"]):
                    _write_json(
                        Path(state["activity"]) / (uuid.uuid4().hex + ".json"),
                        {
                            "schema": SCHEMA,
                            "status": "failed",
                            "body": body,
                            "error": {"type": type(original).__name__, "message": str(original)},
                            "timings": {"total_seconds": time.monotonic() - started},
                        },
                    )
            except Exception as evidence_error:
                raise RuntimeError_(
                    f"request failed ({original}); activity recording failed: {evidence_error}"
                ) from original
            raise

    return observed


def _body_builder(api, state, repo, toolchain, body, kernel, options, context, discovery, builds):
    """One isolated build/acceptance path for ordinary misses and exact repairs."""
    expected = discovery["action_key"]

    def builder(scratch):
        output = scratch / "execution"
        prepared = api.executor.prepare(
            repo,
            body,
            toolchain,
            output,
            kernel=kernel,
            provisioned=state["provisioned"],
            **options,
            **_recipe(context),
        )
        if prepared["action_key"] != expected or prepared["context"] != context:
            raise RuntimeError_("worker preparation differs from discovered action/context")
        report = _execute(repo, Path(state["broker"]), output, api.executor)
        archive = Path(report.get("archive", {}).get("path", ""))
        if (
            report.get("action_key") != expected
            or report.get("context") != context
            or report.get("compile_status") != "success"
            or report.get("lane_input_revalidation") != "identical"
            or report.get("toolchain_postcheck") != "identical"
            or not archive.is_absolute()
            or not _regular(archive)
            or scratch.resolve() not in archive.resolve().parents
        ):
            raise RuntimeError_(f"isolated worker did not prove its result; evidence {output}")
        builds.append(report)
        return api.storage.BuildResult(
            outputs={"body.a": api.storage.Output(archive, role="archive")},
            input_inventory=discovery["provenance"]["reads"],
            provenance={
                "runtime_schema": SCHEMA,
                "source_projection_key": discovery["source_projection_key"],
                "worker": {
                    key: report.get(key)
                    for key in (
                        "compile_status",
                        "compile_seconds",
                        "rustc_invocations",
                        "lane_input_revalidation",
                        "toolchain_postcheck",
                        "retained",
                        "sampled_target_peak_allocated_bytes",
                    )
                },
            },
        )

    return builder


@_record_request_failures
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
    """Return a leased immutable archive and current-lane dependency watches.

    A body whose graph the store does not model returns an `ineligible` result and
    builds privately (fatal under VIBEOS_ARTIFACT_STRICT=1). A dead lease, invalid
    store entry, compiler error, or changed input is fatal.
    """
    started = time.monotonic()
    options = _request_options(
        body,
        features,
        default_features,
        kernel,
        flags,
        architecture,
        target,
        build_std,
        emit_ir,
        force_staticlib,
    )
    if not isinstance(consumer, str) or not consumer or any(c in consumer for c in "\0\r\n\t"):
        raise RuntimeError_(
            "consumer must be a nonempty stable identity without control characters"
        )
    api = _api or _apis()
    state = supervisor_config(supervisor)
    if state.get("repair_action") is not None:
        raise RuntimeError_("repair-only supervisor cannot serve consumer requests")
    repo, toolchain = Path(state["repo"]), Path(state["toolchain"])
    store = api.storage.Store(Path(state["store_root"]))
    with store.attach_session(state["session_token"]) as session:
        config_before = _config_guard().audit_request(
            repo,
            cargo_cwd=cargo_cwd,
            kernel=kernel,
            broker=Path(state["broker"]),
            target=target,
            rustflags_set=bool(flags.strip()),
        )
        manifest_path = repo / "sdk/lib/rust" / body / "Cargo.toml"
        manifest = (
            api.inputs.tomllib.loads(manifest_path.read_text()) if manifest_path.is_file() else {}
        )
        if manifest.get("package", {}).get("name") != body or not _staticlib(manifest):
            return _ineligible(
                state, body, ["not a manifest staticlib at sdk/lib/rust/<body>"], started
            )
        context = api.executor.request_context(
            toolchain,
            state["provisioned"],
            body,
            kernel=kernel,
            target=target,
            flags=flags.strip(),
            **options,
        )
        discovery_started = time.monotonic()
        discovery = api.inputs.discover(
            repo, body, context, cargo=str(toolchain / "bin/cargo"), **options
        )
        if not discovery.get("eligible") or not discovery.get("action_key"):
            return _ineligible(state, body, discovery.get("reasons") or ["no action key"], started)
        discovery_seconds = time.monotonic() - discovery_started
        descriptor, expected = discovery["descriptor"], discovery["action_key"]
        if _key(descriptor) != expected or descriptor["context"] != context:
            raise RuntimeError_("discovery action key/context mismatch")
        watches = _watches(repo, discovery)
        builds = []

        builder = _body_builder(
            api, state, repo, toolchain, body, kernel, options, context, discovery, builds
        )

        force = state["clean"] or state["verify"]
        lookup_started = time.monotonic()
        handle = store.lookup_or_build(
            descriptor,
            builder,
            session=session,
            verify=force,
            verify_once_per_session=force,
            require_verified=state["verify"],
        )
        if state["verify"] and not handle.session_verified:
            # A cold miss establishes a baseline, not reproduction. Force one
            # independent comparison; the session then coalesces other readers.
            handle = store.lookup_or_build(
                descriptor,
                builder,
                session=session,
                verify=True,
                verify_once_per_session=True,
                require_verified=True,
            )
        if handle.action_key != expected or (state["verify"] and not handle.session_verified):
            raise RuntimeError_("store result/action verification mismatch")
        lookup_build_seconds = time.monotonic() - lookup_started
        revalidation_started = time.monotonic()
        current = (
            discovery
            if api.inputs.unchanged(discovery)
            else api.inputs.discover(
                repo, body, context, cargo=str(toolchain / "bin/cargo"), **options
            )
        )
        if not current.get("eligible") or current.get("action_key") != expected:
            raise RuntimeError_("lane inputs changed while waiting/building the artifact")
        revalidation_seconds = time.monotonic() - revalidation_started
        config_after = _config_guard().audit_request(
            repo,
            cargo_cwd=cargo_cwd,
            kernel=kernel,
            broker=Path(state["broker"]),
            target=target,
            rustflags_set=bool(flags.strip()),
        )
        watches = sorted(set(watches) | set(config_after["watches"]))
        reference = store.bind(str(repo), consumer, expected, session=session)
        result = {
            "schema": SCHEMA,
            "cargo_config": {"before": config_before, "after": config_after},
            "body": body,
            "action_key": expected,
            "archive": str(handle.blob_path),
            "watches": watches,
            "hit": handle.hit and not builds,
            "builder_calls": len(builds),
            "sampled_worker_target_peak_allocated_bytes": max(
                (
                    r["sampled_target_peak_allocated_bytes"]
                    for r in builds
                    if "sampled_target_peak_allocated_bytes" in r
                ),
                default=None,
            ),
            "verified_rebuild": handle.verified_rebuild,
            "verification_compared": handle.session_verified,
            "manifest": str(store.manifest_path(expected)),
            "reference": str(reference),
            "timings": {
                "discovery_seconds": discovery_seconds,
                "lookup_wait_build_seconds": lookup_build_seconds,
                "revalidation_seconds": revalidation_seconds,
                "total_seconds": time.monotonic() - started,
                "compile_seconds": sum(r.get("compile_seconds", 0) for r in builds),
            },
        }
        protocol_lines(result)  # Validate before making a successful receipt.
        _write_json(Path(state["activity"]) / (uuid.uuid4().hex + ".json"), result)
        return result


def _repair_request(api, state, manifest):
    """Reconstruct only an audited request whose current descriptor is exact."""
    descriptor = manifest["descriptor"]
    request = descriptor.get("request", {})
    package = request.get("package")
    if not isinstance(package, str):
        raise RuntimeError_("repair manifest has no selected package")
    body = Path(package).name
    if package != "sdk/lib/rust/" + body:
        raise RuntimeError_("repair package is not a workspace body")
    if request.get("profile") != "release" or request.get("invocation") != "cargo build --lib":
        raise RuntimeError_("repair requires a manifest staticlib release recipe")
    if manifest["dependency_actions"] or [
        (output["name"], output["role"]) for output in manifest["outputs"]
    ] != [("body.a", "archive")]:
        raise RuntimeError_("repair requires one standalone body archive")
    options = _request_options(
        body,
        request.get("features"),
        request.get("default_features"),
        False,
        "",
        "amd64",
        TARGET,
        False,
        False,
        False,
    )
    toolchain = Path(state["toolchain"])
    executor = api.executor
    contexts = [
        (
            kernel,
            executor.request_context(
                toolchain,
                state["provisioned"],
                body,
                kernel=kernel,
                target=executor.TARGETS[arch],
                flags=flags,
                **options,
            ),
        )
        for (arch, kernel), flags in executor.FLAGS.items()
    ]
    matches = [
        (kernel, context) for kernel, context in contexts if context == descriptor.get("context")
    ]
    if len(matches) != 1:
        raise RuntimeError_(
            "current toolchain/implementation cannot reproduce indexed repair recipe"
        )
    kernel, context = matches[0]
    repo = Path(state["repo"])
    package_manifest = api.inputs.tomllib.loads((repo / package / "Cargo.toml").read_text())
    if package_manifest.get("package", {}).get("name") != body or not _staticlib(package_manifest):
        raise RuntimeError_("repair requires the current package's manifest staticlib")
    discovery = api.inputs.discover(
        repo, body, context, cargo=str(toolchain / "bin/cargo"), **options
    )
    if (
        not discovery.get("eligible")
        or discovery.get("action_key") != manifest["action_key"]
        or discovery.get("descriptor") != descriptor
        or _key(descriptor) != manifest["action_key"]
    ):
        raise RuntimeError_(
            "current source/toolchain key does not exactly match indexed repair action"
        )
    return body, kernel, options, context, discovery


def repair(supervisor, action, *, _api=None):
    """Explicit missing-only reproduction; never creates or replaces a consumer ref."""
    started = time.monotonic()
    try:
        return _repair(supervisor, action, started, _api=_api)
    except Exception as original:
        try:
            state = supervisor_config(supervisor)
            api = _api or _apis()
            store = api.storage.Store(Path(state["store_root"]))
            with store.attach_session(state["session_token"]):
                _write_json(
                    Path(state["activity"]) / (uuid.uuid4().hex + ".json"),
                    {
                        "schema": SCHEMA,
                        "status": "failed",
                        "body": "repair",
                        "repair_action": action,
                        "error": {"type": type(original).__name__, "message": str(original)},
                        "timings": {"total_seconds": time.monotonic() - started},
                    },
                )
        except Exception as evidence_error:
            raise RuntimeError_(
                f"repair failed ({original}); activity recording failed: {evidence_error}"
            ) from original
        raise


def _repair(supervisor, action, started, *, _api=None):
    api = _api or _apis()
    state = supervisor_config(supervisor)
    if state.get("repair_action") != action:
        raise RuntimeError_("repair requires its exact repair-only supervisor")
    repo, toolchain = Path(state["repo"]), Path(state["toolchain"])
    store = api.storage.Store(Path(state["store_root"]))
    with store.attach_session(state["session_token"]) as session:
        manifest = store.inspect_manifest(action, session=session)
        discovery_started = time.monotonic()
        body, kernel, options, context, discovery = _repair_request(api, state, manifest)
        discovery_seconds = time.monotonic() - discovery_started
        builds = []
        builder = _body_builder(
            api, state, repo, toolchain, body, kernel, options, context, discovery, builds
        )
        repair_started = time.monotonic()
        handle = store.repair_missing(action, builder, session=session)
        repair_seconds = time.monotonic() - repair_started
        revalidation_started = time.monotonic()
        current = api.inputs.discover(
            repo, body, context, cargo=str(toolchain / "bin/cargo"), **options
        )
        if not current.get("eligible") or current.get("action_key") != action:
            raise RuntimeError_("lane inputs changed during explicit repair")
        restored = store.restore_refs(str(repo), session=session)
        result = {
            "schema": SCHEMA,
            "body": body,
            "action_key": action,
            "archive": str(handle.blob_path),
            "watches": [],
            "reference": None,
            "repaired": True,
            "hit": False,
            "builder_calls": len(builds),
            "verified_rebuild": True,
            "verification_compared": True,
            "manifest": str(store.manifest_path(action)),
            "restored_actions": sorted({h.action_key for h in restored}),
            "sampled_worker_target_peak_allocated_bytes": max(
                (
                    r["sampled_target_peak_allocated_bytes"]
                    for r in builds
                    if "sampled_target_peak_allocated_bytes" in r
                ),
                default=None,
            ),
            "timings": {
                "discovery_seconds": discovery_seconds,
                "lookup_wait_build_seconds": repair_seconds,
                "revalidation_seconds": time.monotonic() - revalidation_started,
                "compile_seconds": sum(r.get("compile_seconds", 0) for r in builds),
                "total_seconds": time.monotonic() - started,
            },
        }
        _write_json(Path(state["activity"]) / (uuid.uuid4().hex + ".json"), result)
        return result


def protocol_lines(result):
    """Small line protocol for build.rs; metadata stays in the private receipt."""
    if result.get("status") == "ineligible":
        reason = re.sub(r"[\0\r\n\t]", " ", "; ".join(result["reasons"]))
        return f"ineligible\t{reason}\n"
    values = [("archive", result["archive"]), *(("watch", p) for p in result["watches"])]
    for _, path in values:
        if not Path(path).is_absolute() or any(c in path for c in "\0\r\n\t"):
            raise RuntimeError_("protocol paths must be absolute and contain no tabs/newlines")
    return "".join(kind + "\t" + path + "\n" for kind, path in values)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    child = commands.add_parser("request")
    child.add_argument("--supervisor", type=Path, required=True)
    child.add_argument("--body", required=True)
    child.add_argument("--consumer", required=True)
    child.add_argument("--features", default="")
    child.add_argument("--no-default-features", action="store_true")
    child.add_argument("--kernel", action="store_true")
    child.add_argument("--flags", default="")
    child.add_argument("--cargo-cwd", type=Path)
    child.add_argument("--arch", default="amd64")
    child.add_argument("--target", default=TARGET)
    repair_parser = commands.add_parser(
        "repair", help="explicitly reproduce one missing indexed blob"
    )
    repair_parser.add_argument("--supervisor", type=Path, required=True)
    repair_parser.add_argument("--action", required=True)
    args = parser.parse_args()
    try:
        if args.command == "repair":
            result = repair(args.supervisor, args.action)
            print(json.dumps(result, sort_keys=True))
            return 0
        result = request(
            args.supervisor,
            args.body,
            args.consumer,
            features=tuple(args.features.split(",")) if args.features else (),
            default_features=not args.no_default_features,
            kernel=args.kernel,
            flags=args.flags,
            architecture=args.arch,
            target=args.target,
            **({"cargo_cwd": args.cargo_cwd} if args.cargo_cwd is not None else {}),
        )
        sys.stdout.write(protocol_lines(result))
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
        print(f"artifact request failed: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
