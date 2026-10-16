"""Immutable artifact discovery, supervised body builds and explicit store maintenance."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path


def add_parser(sub) -> None:
    parser = sub.add_parser("artifact-store", help="measure immutable artifact reuse candidates")
    commands = parser.add_subparsers(dest="artifact_command", required=True)
    status = commands.add_parser("status", help="account for store bytes and references")
    status.add_argument("--store", type=Path, required=True)
    status.set_defaults(fn=maintenance_command)
    gc = commands.add_parser("gc", help="collect unreferenced store objects; dry-run by default")
    gc.add_argument("--store", type=Path, required=True)
    gc.add_argument("--apply", action="store_true")
    gc.add_argument("--grace-seconds", type=float, default=3600)
    gc.add_argument(
        "--keep-dead-refs",
        action="store_true",
        help="treat every durable ref as live, even when its worktree or cargo out dir is gone",
    )
    gc.set_defaults(fn=maintenance_command)
    repair = commands.add_parser("repair", help="explicitly reproduce one missing indexed blob")
    repair.add_argument("--store", type=Path, required=True)
    repair.add_argument("--action", required=True)
    repair.add_argument(
        "--output", type=Path, required=True, help="new durable repair report directory"
    )
    repair.set_defaults(fn=repair_command)
    body_build = commands.add_parser(
        "build-body", help="build or reuse one supervised immutable amd64 archive"
    )
    body_build.add_argument("--store", type=Path, required=True)
    body_build.add_argument("--body", required=True)
    body_build.add_argument("--features", action="append", default=[])
    body_build.add_argument("--no-default-features", action="store_true")
    body_build.add_argument("--kernel", action="store_true")
    body_build.add_argument("--verify", action="store_true")
    body_build.add_argument("--clean", action="store_true")
    body_build.add_argument(
        "--output", type=Path, required=True, help="new durable report directory"
    )
    body_build.set_defaults(fn=body_command)
    inventory = commands.add_parser(
        "inventory", help="inventory physical outputs and content reuse"
    )
    inventory.add_argument("--root", type=Path, help="workspace Cargo target directory")
    inventory.add_argument("--build-root", type=Path, action="append", default=[])
    inventory.add_argument("--no-hash", action="store_true", help="allocated bytes only")
    inventory.add_argument("--output", type=Path, required=True, help="durable JSON report")
    inventory.set_defaults(fn=inventory_command)
    plan = commands.add_parser(
        "plan", help="compare eligible body input projections across worktrees"
    )
    plan.add_argument(
        "--revision",
        action="append",
        default=[],
        help="also project a historical tracked closure under the proposed context",
    )
    plan.add_argument("--body", action="append", required=True)
    plan.add_argument("--worktree", type=Path, action="append", default=[])
    plan.add_argument("--features", action="append", default=[])
    plan.add_argument("--no-default-features", action="store_true")
    plan.add_argument("--force-staticlib", action="store_true")
    plan.add_argument("--profile", default="release")
    plan.add_argument("--context", type=Path, help="explicit proposed execution context JSON")
    plan.add_argument("--output", type=Path, required=True, help="durable JSON report")
    plan.set_defaults(fn=plan_command)
    experiment = commands.add_parser(
        "experiment", help="run a prepared isolated worker under disk and CPU admission"
    )
    experiment.add_argument("--prepared", type=Path, required=True)
    experiment.set_defaults(fn=experiment_command)
    prepare = commands.add_parser("prepare", help="capture and isolate a measurement worker")
    prepare.add_argument(
        "--body",
        choices=(
            "beep-rust",
            "kernel-rust",
            "crt-startup-rust",
            "crt0-rust",
            "crt-dllmain-rust",
            "conutils-rust",
        ),
        required=True,
    )
    prepare.add_argument("--features", action="append", default=[])
    prepare.add_argument("--no-default-features", action="store_true")
    prepare.add_argument("--user-body", action="store_true")
    prepare.add_argument("--output", type=Path, required=True)
    prepare.set_defaults(fn=prepare_command)
    economics = commands.add_parser("economics", help="attribute snapshot bytes to body requests")
    economics.add_argument("--inventory", type=Path, required=True)
    economics.add_argument("--output", type=Path, required=True)
    economics.add_argument("--eligibility-top", type=int, default=0)
    economics.add_argument("--scratch", type=Path)
    economics.set_defaults(fn=economics_command)


def maintenance_command(args, repo: Path) -> int:
    from tools.artifact_runtime import lane_ref_alive
    from tools.artifact_store import Store

    try:
        if not (args.store / "v1").is_dir():
            raise ValueError(f"no initialized artifact store at {args.store}")
        store = Store(args.store)
        result = (
            store.status()
            if args.artifact_command == "status"
            else store.gc(
                apply=args.apply,
                grace_seconds=args.grace_seconds,
                ref_alive=None if args.keep_dead_refs else lane_ref_alive,
            )
        )
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0
    except (OSError, ValueError, RuntimeError) as exc:
        print(f"artifact maintenance failed: {exc}", file=sys.stderr)
        return 2


def body_command(args, repo: Path) -> int:
    from tools.artifact_executor import PIN
    from tools.artifact_runtime import supervise

    from . import diskspace, hostjobs

    try:
        if not args.store.is_absolute():
            raise ValueError("--store must be absolute")
        args.output.mkdir(parents=True, exist_ok=False)
        hostjobs.ensure(repo)
        with (
            diskspace.reserve(repo, "check-tooling"),
            supervise(
                repo,
                args.store,
                Path.home() / ".rustup/toolchains" / PIN,
                args.output / "supervisor",
                clean=args.clean,
                verify=args.verify,
            ) as supervisor,
        ):
            command = [
                sys.executable,
                "-m",
                "tools.artifact_runtime",
                "request",
                "--supervisor",
                str(supervisor),
                "--body",
                args.body,
                "--consumer",
                "standalone-body:"
                + args.body
                + ":"
                + ",".join(args.features)
                + (":no-default" if args.no_default_features else ":default")
                + (":kernel" if args.kernel else ":user"),
                "--features",
                ",".join(args.features),
                "--flags=" + ("-C force-frame-pointers=yes " if args.kernel else ""),
            ]
            if args.no_default_features:
                command.append("--no-default-features")
            if args.kernel:
                command.append("--kernel")
            result = subprocess.run(
                command,
                cwd=repo,
                env=hostjobs.environment(repo, dict(os.environ), canonical=False),
                capture_output=True,
                text=True,
            )
            (args.output / "request.stdout").write_text(result.stdout)
            (args.output / "request.stderr").write_text(result.stderr)
            if result.returncode:
                raise RuntimeError(f"body request failed: {result.stderr}")
            archives = [
                line.split("\t", 1)[1]
                for line in result.stdout.splitlines()
                if line.startswith("archive\t")
            ]
            if len(archives) != 1 or not Path(archives[0]).is_file():
                raise RuntimeError(
                    "successful request did not produce exactly one existing archive"
                )
            print(f"immutable archive: {archives[0]}")
        print(f"artifact evidence: {args.output}")
        return 0
    except (OSError, ValueError, RuntimeError) as exc:
        print(f"artifact body failed: {exc}", file=sys.stderr)
        return 2


def repair_command(args, repo: Path) -> int:
    from tools.artifact_executor import PIN
    from tools.artifact_runtime import supervise

    from . import diskspace, hostjobs

    try:
        if not args.store.is_absolute():
            raise ValueError("--store must be absolute")
        if not (args.store / "v1").is_dir():
            raise ValueError(f"no initialized artifact store at {args.store}")
        output = args.output.absolute()
        output.mkdir(parents=True, exist_ok=False)
        hostjobs.ensure(repo)
        with (
            diskspace.reserve(repo, "check-tooling"),
            supervise(
                repo,
                args.store,
                Path.home() / ".rustup/toolchains" / PIN,
                output / "supervisor",
                repair_action=args.action,
            ) as supervisor,
        ):
            command = [
                sys.executable,
                "-m",
                "tools.artifact_runtime",
                "repair",
                "--supervisor",
                str(supervisor),
                "--action",
                args.action,
            ]
            result = subprocess.run(
                command,
                cwd=repo,
                env=hostjobs.environment(repo, dict(os.environ), canonical=False),
                capture_output=True,
                text=True,
                check=False,
            )
            (output / "repair.stdout").write_text(result.stdout)
            (output / "repair.stderr").write_text(result.stderr)
            if result.returncode:
                raise RuntimeError(f"explicit repair failed: {result.stderr}")
            receipt = json.loads(result.stdout)
            if receipt.get("action_key") != args.action or receipt.get("repaired") is not True:
                raise RuntimeError("repair did not return the requested reproduced action")
        print(f"repaired immutable action: {args.action}")
        print(f"artifact evidence: {output}")
        return 0
    except (OSError, ValueError, RuntimeError) as exc:
        print(f"artifact repair failed: {exc}", file=sys.stderr)
        return 2


def economics_command(args, repo: Path) -> int:
    from tools.artifact_economics import analyze

    try:
        report = analyze(
            json.loads(args.inventory.read_text()),
            repo,
            eligibility_top=args.eligibility_top,
            scratch=args.scratch,
        )
        _write(args.output, report)
    except (OSError, ValueError, RuntimeError) as exc:
        print(f"artifact economics failed: {exc}")
        return 2
    print(f"artifact economics: {args.output}")
    return 0 if report.get("complete") else 2


def prepare_command(args, repo: Path) -> int:
    from tools.artifact_executor import PIN, prepare

    try:
        result = prepare(
            repo,
            args.body,
            Path.home() / ".rustup/toolchains" / PIN,
            args.output.absolute(),
            features=tuple(args.features),
            default_features=not args.no_default_features,
            kernel=not args.user_body,
        )
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
        print(f"artifact prepare failed: {exc}")
        return 2
    print(f"prepared {result['body']} in {result['prepare_seconds']:.3f}s: {args.output}")
    return 0


def experiment_command(args, repo: Path) -> int:
    from . import diskspace

    prepared = args.prepared.resolve()
    if not (prepared / "worker.json").is_file():
        print(f"artifact experiment: missing prepared worker: {prepared}")
        return 2
    broker = repo / ".cache/host-jobs/host-jobs"
    if not broker.is_file():
        print(f"artifact experiment: missing compiler admission broker: {broker}")
        return 2
    # This validates one isolated body, with no native link, media, or OS tree.
    # Use the existing compiler-test budget, not the full OS build estimate.
    with diskspace.reserve(repo, "check-tooling"):
        completed = subprocess.run(
            [
                str(broker),
                "--kind",
                "compile",
                "--",
                sys.executable,
                "-m",
                "tools.artifact_executor",
                "run",
                str(prepared),
            ],
            cwd=repo,
            check=False,
        )
    return completed.returncode


def _write(path: Path, report: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    # Reports are evidence, not store objects. Refuse accidental replacement.
    with path.open("x") as stream:
        json.dump(report, stream, indent=2)
        stream.write("\n")


def inventory_command(args, repo: Path) -> int:
    from tools.rust_storage_report import measure

    root = args.root if args.root is not None else repo / "sdk/lib/rust/target"
    try:
        report = measure(
            root, hash_content=not args.no_hash, build_roots=args.build_root, detailed=True
        )
        _write(args.output, report)
    except (OSError, ValueError) as exc:
        print(f"artifact inventory failed: {exc}")
        return 2
    print(f"artifact inventory: {report['allocated_bytes'] / 1024**3:.3f} GiB allocated")
    print(f"report: {args.output}")
    return 2 if report.get("errors") else 0


def _revision(repo: Path) -> str:
    cp = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "HEAD"],
        capture_output=True,
        text=True,
        check=True,
        timeout=30,
    )
    return cp.stdout.strip()


def plan_command(args, repo: Path) -> int:
    try:
        return _plan_command(args, repo)
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        print(f"artifact plan failed: {exc}")
        return 2


def _plan_command(args, repo: Path) -> int:
    from tools.artifact_inputs import discover

    start = time.monotonic()
    records: list[dict] = []
    context = json.loads(args.context.read_text()) if args.context else {}
    if not isinstance(context, dict):
        raise ValueError("execution context must be a JSON object")
    for worktree in args.worktree or [repo]:
        for body in args.body:
            result = discover(
                worktree,
                body,
                context,
                features=tuple(args.features),
                default_features=not args.no_default_features,
                profile=args.profile,
                force_staticlib=args.force_staticlib,
            )
            records.append(
                {
                    "worktree": str(worktree.resolve()),
                    "head": _revision(worktree),
                    "body": body,
                    "result": result,
                }
            )
    if args.revision:
        from tools.artifact_history import capture

        for revision in args.revision:
            for body in args.body:
                with tempfile.TemporaryDirectory(prefix="vibeos-artifact-history-") as scratch:
                    snapshot = Path(scratch) / "source"
                    commit = capture(repo, revision, body, snapshot)
                    result = discover(
                        snapshot,
                        body,
                        context,
                        features=tuple(args.features),
                        default_features=not args.no_default_features,
                        profile=args.profile,
                        force_staticlib=args.force_staticlib,
                    )
                    records.append(
                        {
                            "worktree": "historical tracked snapshot",
                            "head": commit,
                            "body": body,
                            "result": result,
                            "historical_limit": "Proposed current execution context, not proof of "
                            "historical toolchains/configuration or untracked generated inputs.",
                        }
                    )
    report = {
        "schema_version": 1,
        "kind": "input-projection-only",
        "note": "Key matches are reuse candidates, not observed artifact hits or verified outputs. "
        "An explicit context is a proposed recipe, "
        "not proof of the worktree's prior build settings.",
        "seconds": time.monotonic() - start,
        "records": records,
    }
    _write(args.output, report)
    for record in records:
        result = record["result"]
        key = result.get("action_key") or result.get("source_projection_key")
        print(
            f"{record['worktree']} {record['body']}: "
            f"{key if result.get('eligible') else result.get('reasons')}"
        )
    print(f"report: {args.output}")
    return 0
