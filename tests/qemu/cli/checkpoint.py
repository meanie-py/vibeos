"""Durable VM snapshots, distinct from deterministic record/replay.

Launches still use QemuVM admission, transport, process ownership and cleanup.
Bundles own their media; attempts only write private copies. No restore path
consults today's boot images or silently boots from scratch after a load error.
"""

from __future__ import annotations

import dataclasses
import hashlib
import json
import os
import re
import shutil
import subprocess
import tempfile
import time
from pathlib import Path

from .. import backend as be
from ..qmp_client import QmpError
from ..vm import ArchConfig, QemuVM, get_arch_config
from . import archcfg, config, provenance
from .artifacts import Run, Stage
from .recordreplay import format_drive, parse_drive

ROOT = Path.home() / "vibeos-checkpoints"
TAG = "checkpoint"
FAIL_PATTERN = r"(?i)BUGCHECK|KeBugCheckEx|panicked at|RUST_PANIC"


class CheckpointError(RuntimeError):
    pass


def digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for data in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(data)
    return h.hexdigest()


def host_path(raw: str) -> Path:
    match = re.fullmatch(r"([A-Za-z]):[/\\](.*)", raw)
    if match:
        return Path("/mnt") / match[1].lower() / match[2].replace("\\", "/")
    path = Path(raw)
    if not path.is_absolute():
        raise CheckpointError(f"Expected an absolute media path: {raw}")
    return path


def guest_template(argv: list[str]) -> tuple[list[str], list[dict]]:
    """Remove harness transports and inventory every supported external input.

    Deliberately closed option set: an unhandled file/backend cannot accidentally
    become an unpinned dependency. Expand this alongside its staging support.
    """
    out: list[str] = []
    media: list[dict] = []
    pairs = {
        "-M",
        "-machine",
        "-cpu",
        "-accel",
        "-m",
        "-smp",
        "-device",
        "-global",
        "-fw_cfg",
        "-display",
        "-vga",
        "-rtc",
        "-nic",
        "-net",
    }
    controls = {"-name", "-qmp", "-mon", "-serial", "-monitor", "-debugcon", "-gdb"}
    flags = {"-no-reboot", "-no-shutdown", "-nodefaults"}
    i = 1
    while i < len(argv):
        option = argv[i]
        i += 1
        if option == "-S":
            continue
        if option in flags:
            out.append(option)
            continue
        if i >= len(argv):
            raise CheckpointError(f"Missing value for {option}")
        value = argv[i]
        i += 1
        if option in controls:
            continue
        if option == "-chardev":
            if not re.search(r",id=(ser0|qmp0)(,|$)", value):
                raise CheckpointError(f"External chardev cannot be checkpointed: {value}")
            continue
        if option == "-drive":
            drive = parse_drive(value)
            if ",," in value or "file" not in drive:
                raise CheckpointError(f"Unsupported drive syntax: {value}")
            readonly = drive.get("readonly") == "on"
            if not readonly and drive.get("format") != "qcow2":
                raise CheckpointError("Writable checkpoint media must be qcow2")
            name = f"media-{len(media):02d}.{'raw' if readonly else 'qcow2'}"
            media.append(
                {
                    "name": name,
                    "source": str(host_path(drive["file"])),
                    "readonly": readonly,
                    "format": drive.get("format", "raw"),
                }
            )
            drive["file"] = "@" + name
            out.extend([option, format_drive(drive)])
        elif option in pairs:
            if option == "-nic" and value not in ("none", "user,restrict=on"):
                raise CheckpointError("Live networking is not checkpointable")
            if option == "-net" and value != "nic":
                raise CheckpointError("Only a disconnected default NIC is supported")
            if option == "-accel" and value.split(",")[0] != "tcg":
                raise CheckpointError("Checkpoint capture currently requires TCG")
            if re.search(r"(^|,)(file|mem-path|romfile|chardev|netdev|rng|fsdev)=", value):
                raise CheckpointError(f"Unpinned external device input: {option} {value}")
            if option == "-device" and any(x in value for x in ("usb", "vfio", "virtio-net")):
                raise CheckpointError(f"Unsupported checkpoint device: {value}")
            out.extend([option, value])
        else:
            raise CheckpointError(f"Unsupported checkpoint option: {option}")
    if not media or not any(not m["readonly"] for m in media):
        raise CheckpointError("No writable snapshot store")
    return out, media


def pause(vm: QemuVM) -> None:
    vm.qmp.execute("stop")
    state = vm.qmp.execute("query-status")["return"]
    if state["running"] or state["status"] not in ("paused", "prelaunch", "debug"):
        raise CheckpointError(f"QEMU did not stop cleanly: {state}")


def snapshot(vm: QemuVM, operation: str) -> None:
    pause(vm)
    reply = vm.qmp.hmp(f"{operation} {TAG}").strip()
    # HMP reports failures as successful QMP strings, with inconsistent casing.
    if reply:
        raise CheckpointError(f"{operation} failed: {reply}")
    pause(vm)


def registers(vm: QemuVM) -> str:
    value = vm.qmp.hmp("info registers -a").strip()
    if not value or not re.search(r"\b(PC|RIP|EIP)=", value):
        raise CheckpointError("Cannot obtain all-vCPU register witness")
    return value


def watch(vm: QemuVM, until: str, fail: str, timeout: float, log: Path) -> str:
    """Failure wins when both patterns occur in one serial read."""
    failure = re.compile(fail)
    deadline = time.monotonic() + timeout
    text = ""
    with log.open("w") as stream:
        while time.monotonic() < deadline:
            chunk = vm.serial_output(timeout=min(0.1, max(0.001, deadline - time.monotonic())))
            stream.write(chunk)
            stream.flush()
            text += chunk
            if failure.search(text):
                return "failure-pattern"
            if getattr(getattr(vm, "serial", None), "_sock_dead", False):
                raise ConnectionError("Serial socket closed during checkpoint window")
            if until in text:
                return "reached-marker"
    return "timeout"


def emulator_identity(executable: str) -> dict:
    resolved = Path(shutil.which(executable) or executable).resolve()
    result = subprocess.run(
        [str(resolved), "--version"], capture_output=True, text=True, check=True, timeout=30
    )
    dependencies = {}
    if resolved.suffix == ".exe":
        for path in sorted(resolved.parent.rglob("*")):
            if path.is_file() and path.suffix.lower() in (".dll", ".bin", ".rom", ".fd"):
                dependencies[str(path.relative_to(resolved.parent))] = digest(path)
    return {
        "path": str(resolved),
        "sha256": digest(resolved),
        "version": result.stdout,
        "dependencies": dependencies,
    }


class CaptureVM(QemuVM):
    def _build_windows_cmd(self) -> list[str]:
        # Keep default PCI placement and local DHCP; block outside connections.
        # Removing the NIC changes ARM64 firmware's persisted boot device path.
        argv = [*super()._build_windows_cmd(), "-nic", "user,restrict=on"]
        guest_template(argv)
        return argv

    def _build_native_cmd(self) -> list[str]:
        argv = [*super()._build_native_cmd(), "-nic", "user,restrict=on"]
        guest_template(argv)
        return argv


def export(
    vm: QemuVM,
    destination: Path,
    *,
    parent: str | None = None,
    extras: dict[str, Path] | None = None,
) -> dict:
    """Save stopped, close QEMU to flush disks, then copy before harness cleanup."""
    launch = json.loads((vm.live_log_path.parent / f"{vm.config.arch}_launch.json").read_text())
    template, media = guest_template(launch["argv"])
    if launch["accel"] != "tcg":
        raise CheckpointError("Checkpoint export requires TCG")
    snapshot(vm, "savevm")
    witness = registers(vm)
    cpus = vm.qmp.execute("query-cpus-fast")["return"]
    # Close before copying: never export a partially cached qcow2 allocation table.
    vm.qmp.execute("quit")
    if vm._proc is None:
        raise CheckpointError("Missing QEMU process")
    exit_code = vm._proc.wait(timeout=30)
    if exit_code != 0:
        raise CheckpointError(f"QEMU exited abnormally before checkpoint export: {exit_code}")
    if destination.exists():
        raise CheckpointError(f"Checkpoint already exists: {destination}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = Path(tempfile.mkdtemp(prefix=".partial-", dir=destination.parent))
    try:
        has_vm_state = False
        for item in media:
            source = Path(item.pop("source"))
            target = temporary / item["name"]
            with source.open("rb") as src, target.open("wb") as dst:
                shutil.copyfileobj(src, dst, length=16 * 1024)
            item["sha256"] = digest(target)
            info = subprocess.run(
                ["qemu-img", "info", "--output=json", "-f", item["format"], str(target)],
                capture_output=True,
                text=True,
                check=True,
            )
            image = json.loads(info.stdout)
            if image.get("dirty-flag"):
                raise CheckpointError(f"Image was not cleanly closed: {item['name']}")
            if image.get("backing-filename"):
                raise CheckpointError("Backing chains must be flattened before capture")
            if not item["readonly"] and not any(
                snap.get("name") == TAG for snap in image.get("snapshots", [])
            ):
                raise CheckpointError(f"Missing snapshot in {item['name']}")
            has_vm_state |= any(
                snap.get("name") == TAG and snap.get("vm-state-size", 0) > 0
                for snap in image.get("snapshots", [])
            )
        if not has_vm_state:
            raise CheckpointError("Snapshot contains no saved VM state")
        manifest = {
            "schema": 1,
            "config": dataclasses.asdict(vm.config),
            "backend": vm._backend,
            "accel": "tcg",
            "argv": template,
            "emulator": emulator_identity(launch["argv"][0]),
            "media": media,
            "registers": witness,
            "vcpus": len(cpus),
            "parent": parent,
            "created": time.time(),
            "semantics": "checkpoint-retry, not deterministic replay",
            "debugcon": "-debugcon" in launch["argv"],
        }
        manifest["extras"] = []
        for name, source in (extras or {}).items():
            target = temporary / name
            shutil.copyfile(source, target)
            manifest["extras"].append({"name": name, "sha256": digest(target)})
        (temporary / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
        temporary.rename(destination)
        return manifest
    except BaseException:
        shutil.rmtree(temporary)
        raise


def load(bundle: Path) -> dict:
    manifest = json.loads((bundle / "manifest.json").read_text())
    if manifest.get("schema") != 1 or manifest.get("accel") != "tcg":
        raise CheckpointError("Unsupported checkpoint schema/accelerator")
    if manifest["backend"] not in ("windows-host", "native"):
        raise CheckpointError("Unsupported checkpoint backend")
    for item in manifest["media"]:
        if not re.fullmatch(r"media-\d+\.(raw|qcow2)", item["name"]):
            raise CheckpointError("Invalid bundle media name")
        path = bundle / item["name"]
        if path.is_symlink() or digest(path) != item["sha256"]:
            raise CheckpointError(f"Checkpoint media missing or damaged: {path}")
    for item in manifest.get("extras", []):
        name = item["name"]
        if Path(name).name != name or name in (".", ".."):
            raise CheckpointError("Invalid checkpoint extra name")
        if digest(bundle / name) != item["sha256"]:
            raise CheckpointError(f"Checkpoint metadata/symbols damaged: {name}")
    if emulator_identity(manifest["emulator"]["path"]) != manifest["emulator"]:
        raise CheckpointError("QEMU build changed; use the checkpoint's original QEMU installation")
    return manifest


class RestoredVM(QemuVM):
    """Keep normal ownership/admission; substitute pinned guest inputs only."""

    def __init__(self, bundle: Path, manifest: dict, work: Path, windows: dict, gdb: bool):
        self.bundle = bundle
        self.manifest = manifest
        super().__init__(
            ArchConfig(**manifest["config"]),
            work,
            backend=manifest["backend"],
            windows=windows,
            gdb=gdb,
            serial_log=work / "serial.log",
        )

    def _boot_is_tcg(self) -> bool:
        return True

    def _guest_args(self, staging: Path, windows: bool) -> list[str]:
        paths = {}
        for item in self.manifest["media"]:
            src = self.bundle / item["name"]
            if windows:
                stage = be.stage_readonly_file if item["readonly"] else be.stage_file
                paths["@" + item["name"]] = stage(src, staging)
            else:
                target = staging / item["name"]
                shutil.copyfile(src, target)
                paths["@" + item["name"]] = str(target)
            if digest(staging / item["name"]) != item["sha256"]:
                raise CheckpointError(f"Checkpoint media changed during staging: {src}")
        args = list(self.manifest["argv"])
        for i, option in enumerate(args[:-1]):
            if option == "-drive":
                drive = parse_drive(args[i + 1])
                drive["file"] = paths[drive["file"]]
                args[i + 1] = format_drive(drive)
        return args

    def _debugcon_args(self, staging: Path, windows: bool) -> list[str]:
        if not self.manifest.get("debugcon"):
            return []
        self.debugcon_path = staging / "debugcon.bin"
        path = be.win_path(self.debugcon_path) if windows else str(self.debugcon_path)
        return ["-debugcon", f"file:{path}"]

    def _prepare_launch(self) -> None:
        windows = self._backend == "windows-host"
        staging = self._prepare_staging() if windows else self.work_dir
        self._prepared_guest_args = self._guest_args(staging, windows)
        self._prepared_debugcon_args = self._debugcon_args(staging, windows)

    def _build_native_cmd(self) -> list[str]:
        self.serial.listen(str(self.serial_path))
        self._set_live_tee()
        return [
            self.manifest["emulator"]["path"],
            "-S",
            "-qmp",
            f"unix:{self.qmp_path},server,nowait",
            "-chardev",
            f"socket,id=ser0,path={self.serial_path},server=off",
            "-serial",
            "chardev:ser0",
            "-monitor",
            "none",
            *self._gdb_args(win_stack=False),
            *self._prepared_debugcon_args,
            *self._prepared_guest_args,
        ]

    def _build_windows_cmd(self) -> list[str]:
        ser = self.serial.listen_tcp("0.0.0.0", 0)
        qmp = self.qmp.listen_tcp("0.0.0.0", 0)
        self._set_live_tee()
        self._win_uid = f"vibeos-{os.getpid()}-{ser}"
        host = self._win["host_addr"]
        return [
            self.manifest["emulator"]["path"],
            "-S",
            "-name",
            self._win_uid,
            "-chardev",
            f"socket,id=qmp0,host={host},port={qmp},server=off",
            "-mon",
            "chardev=qmp0,mode=control",
            "-chardev",
            f"socket,id=ser0,host={host},port={ser},server=off",
            "-serial",
            "chardev:ser0",
            "-monitor",
            "none",
            *self._gdb_args(win_stack=True),
            *self._prepared_debugcon_args,
            *self._prepared_guest_args,
        ]


def restore(vm: RestoredVM) -> None:
    snapshot(vm, "loadvm")
    if registers(vm) != vm.manifest["registers"]:
        raise CheckpointError("Restored vCPU registers differ from saved checkpoint")
    if len(vm.qmp.execute("query-cpus-fast")["return"]) != vm.manifest["vcpus"]:
        raise CheckpointError("Restored CPU topology differs")
    # Serial transport buffers are outside VM state. Drain to an idle interval
    # while all vCPUs are stopped so an old marker cannot satisfy the next trial.
    deadline = time.monotonic() + 5
    while vm.serial.read_available(timeout=0.2):
        if time.monotonic() >= deadline:
            raise CheckpointError("Serial transport did not quiesce while stopped")


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
                "VIBEOS_QEMU_USB_DISK",
                "VIBEOS_QEMU_NVME",
                "VIBEOS_QEMU_XHCI",
                "VIBEOS_QEMU_EXTRA_ARGS",
            )
        ):
            raise CheckpointError("Unset RR, USB_DISK, NVME, XHCI and EXTRA_ARGS for checkpoint commands")
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
