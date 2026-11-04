"""Durable VM snapshots, distinct from deterministic record/replay.

Launches still use QemuVM admission, transport, process ownership and cleanup.
Bundles own their media; attempts only write private copies. No restore path
consults today's boot images or silently boots from scratch after a load error.
"""

from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import tempfile
import time
from pathlib import Path
from types import SimpleNamespace

from tools import test_infra_compat as native

from ..qmp_client import QmpError
from ..vm import QemuVM, get_arch_config
from . import archcfg, config, provenance
from .artifacts import Run, Stage

ROOT = Path.home() / "vibeos-checkpoints"
TAG = "checkpoint"
FAIL_PATTERN = r"(?i)BUGCHECK|KeBugCheckEx|panicked at|RUST_PANIC"


class CheckpointError(RuntimeError):
    pass


def _native(op, **kwargs):
    try:
        return native.call("checkpoint_" + op, kwargs)
    except native.NativeError as exc:
        raise CheckpointError(str(exc)) from exc


def _vm_native(vm, op, **kwargs):
    try:
        return vm._call("checkpoint_" + op, **kwargs)
    except (native.NativeError, RuntimeError, OSError) as exc:
        raise CheckpointError(str(exc)) from exc


def digest(path: Path) -> str:
    return _native("digest", path=str(path))


def host_path(raw: str) -> Path:
    return Path(_native("host_path", raw=raw))


def guest_template(argv: list[str]) -> tuple[list[str], list[dict]]:
    template, media = _native("guest_template", argv=argv)
    return template, media


def pause(vm: QemuVM) -> None:
    _vm_native(vm, "pause")


def snapshot(vm: QemuVM, operation: str) -> None:
    _vm_native(vm, "snapshot", operation=operation)


def registers(vm: QemuVM) -> str:
    return _vm_native(vm, "registers")


def watch(vm: QemuVM, until: str, fail: str, timeout: float, log: Path) -> str:
    return _vm_native(vm, "watch", until=until, fail=fail, timeout=timeout, log=str(log))


def emulator_identity(executable: str) -> dict:
    return _native("emulator_identity", executable=executable)


class CaptureVM(QemuVM):
    """Constructor DTO: launch policy and ownership stay in the native VM."""

    def __init__(self, *args, **kwargs):
        super().__init__(*args, capture_restrict_network=True, **kwargs)


def export(
    vm: QemuVM,
    destination: Path,
    *,
    parent: str | None = None,
    extras: dict[str, Path] | None = None,
) -> dict:
    return _vm_native(
        vm,
        "export",
        destination=str(destination),
        parent=parent,
        extras={name: str(path) for name, path in (extras or {}).items()},
    )


def load(bundle: Path) -> dict:
    return _native("load", bundle=str(bundle))


class RestoredVM(QemuVM):
    """Pinned guest DTO; native owner stages, validates and launches it stopped."""

    def __init__(self, bundle: Path, manifest: dict, work: Path, windows: dict, gdb: bool):
        self.bundle = bundle
        self.manifest = manifest
        super().__init__(
            SimpleNamespace(**manifest["config"]),
            work,
            backend=manifest["backend"],
            windows=windows,
            gdb=gdb,
            serial_log=work / "serial.log",
            prepared_config=manifest["config"],
            checkpoint={"bundle": str(bundle), "manifest": manifest},
        )


def restore(vm: RestoredVM) -> None:
    _vm_native(vm, "restore", manifest=vm.manifest)


def command(args, repo: Path) -> int:
    run = Run("checkpoint-" + args.checkpoint_verb, getattr(args, "arch", "saved"))
    root = Path(args.root or ROOT).expanduser().resolve()
    cfg = config.load(repo)
    windows = dict(cfg.get("qemu.windows"))
    vm = None
    started = time.monotonic()
    try:
        if args.checkpoint_verb in ("capture", "run"):
            if not args.until:
                raise CheckpointError("--until must not be empty")
            try:
                re.compile(args.fail_on)
            except re.error as exc:
                raise CheckpointError(f"Invalid --fail-on regex: {exc}") from exc
        staging_root = Path(windows["staging_dir"]).resolve()
        if root == staging_root or staging_root in root.parents:
            raise CheckpointError("Checkpoint store must be outside Windows temporary staging")
        if any(
            os.environ.get(key)
            for key in (
                "VIBEOS_QEMU_RR",
                "VIBEOS_QEMU_GIC",
                "VIBEOS_QEMU_SMMU",
                "VIBEOS_QEMU_USB_DISK",
                "VIBEOS_QEMU_NVME",
                "VIBEOS_QEMU_NVME_BRIDGE",
                "VIBEOS_QEMU_XHCI",
                "VIBEOS_QEMU_XHCI_MSI",
                "VIBEOS_QEMU_CPU",
                "VIBEOS_QEMU_HEADLESS",
                "VIBEOS_QEMU_BOOT_NVME",
                "VIBEOS_QEMU_EXTRA_ARGS",
            )
        ):
            raise CheckpointError(
                "Unset RR, GIC, SMMU, USB_DISK, NVME, XHCI and EXTRA_ARGS for checkpoint commands"
            )
        if args.checkpoint_verb == "capture":
            if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*", args.name):
                raise CheckpointError("Name must contain only letters, digits, _, . and -")
            bundle = root / args.name
            if bundle.exists():
                raise CheckpointError(f"Checkpoint already exists: {bundle}")
            stale = provenance.check_staleness(repo, archcfg.base_arch(args.arch))
            if stale.stale:
                raise CheckpointError(
                    "Stale media; run vibeos-cli build: " + "; ".join(stale.reasons)
                )
            backend = cfg.get("qemu.backend")
            if (backend == "windows-host" and windows["accel"] != "tcg") or (
                backend == "native" and archcfg.base_arch(args.arch) != "arm64"
            ):
                raise CheckpointError("Capture requires Windows TCG or native ARM64 TCG")
            os.environ["VIBEOS_QEMU_SMP"] = str(args.smp)
            vm = CaptureVM(
                get_arch_config(archcfg.vm_arch(args.arch)),
                run.dir / "capture",
                backend=backend,
                windows=windows,
                serial_log=run.dir / "serial.log",
            )
            vm.start()
            outcome = watch(vm, args.until, args.fail_on, args.timeout, run.dir / "window.log")
            if outcome != "reached-marker":
                raise CheckpointError(f"Capture marker not reached: {outcome}")
            metadata = {
                "commit": provenance.git_head(repo),
                "capture_marker": args.until,
                "smp": args.smp,
                "source_run": str(run.dir),
            }
            metadata_path = run.dir / "capture.json"
            metadata_path.write_text(json.dumps(metadata, indent=2) + "\n")
            extras = {"capture.json": metadata_path, "capture-serial.log": run.dir / "window.log"}
            arch = archcfg.base_arch(args.arch)
            for source in (repo / "qemu").glob(f"symbols-{arch}*.tsv"):
                extras[source.name] = source
            for rel in (
                "ntoskrnl/ntoskrnl.exe",
                "ntoskrnl/ntkrnlmp/ntkrnlmp.exe",
                "hal/hal/hal.dll",
                "hal/halmp/halmp.dll",
            ):
                source = repo / f"build-{arch}" / rel
                if source.is_file():
                    extras[source.name] = source
            export(vm, bundle, extras=extras)
            run.extra["checkpoint"] = str(bundle)
        else:
            if args.checkpoint_verb == "run" and args.gdb and args.repeat != 1:
                raise CheckpointError(
                    "Use one attempt per --gdb run; disconnect before starting another run. "
                    "Loading with an attached debugger crashed Windows ARM64 QEMU "
                    "during verification."
                )
            raw = Path(args.bundle).expanduser()
            bundle = (raw if raw.is_dir() else root / raw).resolve()
            if bundle == staging_root or staging_root in bundle.parents:
                raise CheckpointError("Checkpoint bundle is inside temporary Windows staging")
            manifest = load(bundle)
            run.extra["checkpoint"] = str(bundle)
            if args.checkpoint_verb == "inspect":
                run.extra["manifest"] = manifest
                run.add(
                    Stage(
                        "inspect",
                        detail=f"{manifest['config']['arch']}, "
                        f"{manifest['vcpus']} vCPUs; media and QEMU identity verified",
                    )
                )
                return finish(run, repo)
            windows["accel"] = "tcg"
            attempts = bundle / "attempts"
            attempts.mkdir(exist_ok=True)
            session = Path(tempfile.mkdtemp(prefix="run-", dir=attempts))
            run.extra["attempts"] = str(session)
            vm = RestoredVM(
                bundle,
                manifest,
                session / "vm",
                windows,
                args.gdb or args.checkpoint_verb == "verify",
            )
            vm.start()
            if args.checkpoint_verb == "verify":
                from .checkpoint_verify import verify

                proof = verify(vm, args.pause_seconds)
                (session / "proof.json").write_text(json.dumps(proof, indent=2) + "\n")
                run.extra["proof"] = proof
                run.add(Stage("verify", detail="RAM, disk, all vCPUs and guest timer rolled back"))
                return finish(run, repo)
            for number in range(args.repeat):
                attempt = session / f"{number + 1:04d}"
                attempt.mkdir()
                before = time.monotonic()
                restore(vm)
                restored = time.monotonic()
                if args.gdb:
                    print(
                        f"[checkpoint] restored stopped; GDB {vm.gdb_host}:{vm.gdb_port}; "
                        "attach and continue (timeout includes debugger time)",
                        flush=True,
                    )
                else:
                    vm.qmp.execute("cont")
                try:
                    outcome = watch(
                        vm, args.until, args.fail_on, args.timeout, attempt / "serial.log"
                    )
                except (ConnectionError, OSError) as exc:
                    outcome = f"transport-failure: {exc}"
                result = {
                    "outcome": outcome,
                    "restore_seconds": restored - before,
                    "run_seconds": time.monotonic() - restored,
                    "seed": str(bundle),
                }
                (attempt / "result.json").write_text(json.dumps(result, indent=2) + "\n")
                if outcome != "reached-marker":
                    (attempt / "cpus.txt").write_text(vm.capture_cpu_state())
                    try:
                        extras = {
                            entry["name"]: bundle / entry["name"]
                            for entry in manifest.get("extras", [])
                        }
                        export(vm, attempt / "failure", parent=str(bundle), extras=extras)
                    except (CheckpointError, QmpError, OSError, subprocess.SubprocessError) as exc:
                        raise CheckpointError(
                            f"{outcome}; logs: {attempt}; failure snapshot unavailable: {exc}"
                        ) from exc
                    raise CheckpointError(f"{outcome}; failure checkpoint: {attempt / 'failure'}")
            run.extra["completed_attempts"] = args.repeat
        run.add(Stage("checkpoint", seconds=time.monotonic() - started, detail="complete"))
    except (
        CheckpointError,
        QmpError,
        OSError,
        ValueError,
        RuntimeError,
        subprocess.SubprocessError,
    ) as exc:
        run.add(Stage("checkpoint", status="fail", detail=str(exc)))
    finally:
        if vm is not None:
            vm.stop()
    return finish(run, repo)


def finish(run: Run, repo: Path) -> int:
    result = run.finalize(repo)
    if run.extra.get("attempts"):
        destination = Path(run.extra["attempts"])
        shutil.copyfile(run.dir / "summary.json", destination / "summary.json")
    return result


def positive(value: str) -> int:
    result = int(value)
    if result < 1:
        raise ValueError("must be positive")
    return result


def add_parser(sub) -> None:
    parser = sub.add_parser("checkpoint", help="durable VM capture and checkpoint retries")
    verbs = parser.add_subparsers(dest="checkpoint_verb", required=True)
    for verb in ("capture", "run", "inspect", "verify"):
        p = verbs.add_parser(verb)
        p.add_argument("--root", help=f"durable checkpoint root (default {ROOT})")
        if verb == "capture":
            p.add_argument("--arch", required=True, choices=archcfg.ACCEPTED_ARCHES)
            p.add_argument("--smp", type=positive, default=1)
            p.add_argument("--name", required=True)
        else:
            p.add_argument("bundle", help="checkpoint name or directory")
        if verb in ("capture", "run"):
            p.add_argument("--until", required=True, help="literal serial marker ending the window")
            p.add_argument("--fail-on", default=FAIL_PATTERN, help="failure regex (checked first)")
            p.add_argument("--timeout", type=positive, default=180)
        if verb == "run":
            p.add_argument("--repeat", type=positive, default=1)
            p.add_argument("--gdb", action="store_true", help="restore stopped for debugger attach")
        if verb == "verify":
            p.add_argument("--pause-seconds", type=positive, default=2)
            p.set_defaults(gdb=True)
        p.set_defaults(fn=command)
