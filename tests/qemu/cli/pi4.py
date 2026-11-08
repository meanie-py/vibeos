"""CLI and compatibility bindings for the native Raspberry Pi 4 controller."""

from __future__ import annotations

import os
from pathlib import Path

from tools import qemu_infra_compat as native
from tools.qemu_infra_runner import native_run_call, request_options, run_controller

from . import ladder as ladder
from . import pi4lease
from .artifacts import Stage

MARKER = "shell ready"
DARK = 12
PI_WATTS_MAX = 20.0
DEFAULT_DESKTOP_TIMEOUT = 300


def _call(op, repo=None, **kwargs):
    return native.call(
        "bench_" + op, {"env": dict(os.environ), **kwargs}, **({"root": repo} if repo else {})
    )


def bench_dir():
    return Path(os.environ.get("PI4_BENCH", "/mnt/d/home/vibeos-rpi4-setup"))


def _script(repo, name):
    return ["bash", str(repo / "tools" / "pi4" / name)]


def parse_usb(text):
    return _call("parse_usb", text=text)


def parse_power(text):
    return _call("parse_power", text=text)


def listener_state(bench):
    return _call("listener_state", bench=str(bench))


def _pid_runs(pid, needle):
    return _call("pid_runs", pid=pid, needle=needle)


def serial_file(state):
    result = _call("serial_file", state=state)
    return Path(result) if result is not None else None


def payload_manifest(bench):
    return _call("payload_manifest", board="pi4", bench=str(bench))


def wait_for_desktop(serial, start, timeout):
    return tuple(_call("wait_desktop", serial=str(serial), start=start, timeout=timeout))


def _slice(serial, start):
    return _call("serial_slice", serial=str(serial), start=start)


def _brightness(path):
    return _call("brightness", path=str(path))


def doctor_stages(repo, run_dir=None):
    values = _call("doctor_stages", repo, board="pi4", run_dir=str(run_dir) if run_dir else None)
    return [Stage(**value) for value in values]


def _run_readonly(args, repo, op):
    return native_run_call(
        None, repo, "bench_" + op, {**request_options(args, repo), "board": "pi4"}
    )


def cmd_status(args, repo):
    return _run_readonly(args, repo, "status")


def cmd_lease(args, repo):
    return _run_readonly(args, repo, "lease")


def cmd_doctor(args, repo):
    return run_controller(
        "bench_doctor", args, repo, command="pi4-doctor", arch="arm64", extra={"board": "pi4"}
    )


def cmd_boot(args, repo):
    return run_controller(
        "bench_boot", args, repo, command="pi4-boot", arch="arm64", extra={"board": "pi4"}
    )


def cmd_stage(args, repo):
    import dataclasses
    import json
    import sys

    from . import archcfg

    kwargs: dict = {"env": dict(os.environ)}
    if getattr(args, "freeldr_ini", False):
        try:
            result = native.call("bench_stage_pi5_ini", kwargs, repo)
        except native.NativeError as error:
            print(str(error), file=sys.stderr)
            return 2
        sys.stdout.write(result)
        return 0
    kwargs.update(
        cfg=dataclasses.asdict(archcfg.arch_build("arm64")),
        no_reboot=getattr(args, "no_reboot", False),
        allow_stale=getattr(args, "allow_stale", False),
    )
    try:
        result = native.call("bench_stage_pi4", kwargs, repo)
    except native.NativeError as error:
        print(str(error), file=sys.stderr)
        return 75 if error.kind == "LEASE_REFUSED" else 1
    if result is not None:
        print(json.dumps(result, indent=2))
    return 0


def cmd_services(args, repo):
    try:
        return native_run_call(
            None,
            repo,
            "bench_service_" + args.service.replace("-", "_"),
            {"env": dict(os.environ), "pid": args.pid, "started": args.started},
        )
    except native.NativeError as error:
        import sys

        print(str(error), file=sys.stderr)
        return 75 if error.kind == "BENCH_BUSY" else 1


def add_parser(sub) -> None:
    p = sub.add_parser("pi4", help="Raspberry Pi 4B bench: lease, doctor, boot this tree")
    p.set_defaults(fn=command)
    verbs = p.add_subparsers(dest="pi4_cmd", required=True)
    stage = verbs.add_parser("stage", help="stage this tree's fresh arm64 build")
    stage.add_argument("--allow-stale", action="store_true")
    stage.add_argument("--no-reboot", action="store_true")
    stage.add_argument("--freeldr-ini", action="store_true")
    services = verbs.add_parser("services", help="manage leased native bench host services")
    services.add_argument(
        "service", choices=["status", "up", "http", "down", "serial-restart", "serial-down", "recover"]
    )
    services.add_argument("--pid", type=int, help="recover: exact process id from failed launch")
    services.add_argument("--started", type=int, help="recover: exact CIM UTC start ticks")
    verbs.add_parser("status", help="lease, power, Pico, serial listener, staged payload")
    lease = verbs.add_parser("lease", help="hold the board across several commands")
    lease.add_argument("action", choices=["take", "drop", "show"])
    lease.add_argument("--minutes", type=float, default=pi4lease.DEFAULT_MINUTES)
    lease.add_argument("--why", help="shown to other lanes while you hold it")
    lease.add_argument("--force", action="store_true", help="drop: release another lane's lease")
    verbs.add_parser("doctor", help="preflight every bench service, with the fix for each")
    boot = verbs.add_parser("boot", help="stage + netboot this tree's arm64 build; desktop verdict")
    boot.add_argument("--kernel", choices=["mp", "up"], default="mp")
    boot.add_argument(
        "--timeout",
        type=float,
        default=DEFAULT_DESKTOP_TIMEOUT,
        help="seconds from loader start to `shell ready`",
    )
    boot.add_argument(
        "--ladder",
        action="store_true",
        help="after the desktop, drain and grade the WoW64 ladder as `smoke --ladder` does "
        "(needs VIBEOS_FEX=1 media); records per-guest timings in summary.json",
    )
    boot.add_argument(
        "--until",
        metavar="TEXT",
        help="after the desktop, keep reading serial until a line contains TEXT (a driver "
        "that works on past the desktop); HANG_TIMEOUT if it never comes",
    )
    boot.add_argument(
        "--until-timeout",
        type=float,
        default=180.0,
        metavar="S",
        help="seconds --until waits after the desktop (default 180)",
    )
    boot.add_argument("--allow-stale", action="store_true")
    boot.add_argument("--skip-doctor", action="store_true")


def command(args, repo):
    return {
        "services": cmd_services,
        "stage": cmd_stage,
        "status": cmd_status,
        "lease": cmd_lease,
        "doctor": cmd_doctor,
        "boot": cmd_boot,
    }[args.pi4_cmd](args, repo)


class SerialTail:
    def __init__(self, path, offset, t0):
        self.path, self.pos, self.t0 = path, offset, t0
        self.times, self._tail = {}, ""

    @property
    def serial(self):
        return self

    def read_available(self, timeout=0.5):
        result = _call(
            "tail_read",
            serial=str(self.path),
            pos=self.pos,
            t0=self.t0,
            tail=self._tail,
            times=self.times,
            timeout=timeout,
        )
        self.pos, self.times, self._tail = result["pos"], result["times"], result["tail"]
        return result["text"]

    def durations(self):
        return _call("tail_durations", times=self.times)


def await_until(run, serial, start, needle, timeout):
    result = _call("until", serial=str(serial), start=start, needle=needle, timeout=timeout)
    _save_serial(run, result["text"])
    if result["classification"]:
        run.classification = result["classification"]
    run.add(Stage(**result["stage"]))


def _save_serial(run, text):
    path = run.dir / "serial.log"
    path.write_text(text)
    run.extra["serial_log"] = str(path)
