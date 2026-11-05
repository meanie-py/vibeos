"""CLI and compatibility bindings for the native Raspberry Pi 5 controller."""

from __future__ import annotations

import os
from pathlib import Path

from tools import qemu_infra_compat as native
from tools.qemu_infra_runner import native_run_call, request_options, run_controller

from . import pi4, pi4lease
from .artifacts import Stage

BOARD = "pi5"
SSH_WAIT = 150
DESKTOP_RGB = (50, 121, 187)
DESKTOP_SHARE = 0.2
STORTEST_LABEL = "VIBEUSB"
STORTEST_MARKER = b"vibeos usb e2e marker\r\n"
STORTEST_WROTE = b"vibeos wrote this through fastfat+usbstor\r\n"
STORTEST_WAIT = 240


class _DropMarker:
    def search(self, text):
        return True if _call("drop_marker", text=text) else None


DROP_MEDIUM_AT = _DropMarker()
HARD_ERROR_INLINE = "completed inline: the requester has APCs disabled"


def _call(op, repo=None, **kwargs):
    return native.call(
        "bench_" + op, {"env": dict(os.environ), **kwargs}, **({"root": repo} if repo else {})
    )


def bench_dir():
    return Path(os.environ.get("PI5_BENCH", "/mnt/d/home/vibeos-rpi5-setup"))


def _script(repo, name):
    return ["bash", str(repo / "tools" / "pi5" / name)]


def ssh_up(repo):
    return _call("ssh_up", repo)


def plug_configured(repo):
    return _call("plug_configured", repo)


def payload_manifest():
    return _call("payload_manifest", board="pi5", bench=str(bench_dir()))


def desktop_share(path):
    return _call("desktop_share", path=str(path))


def stortest_image(path):
    _call("stortest_image", path=str(path))


def stortest_verdict(text):
    verdict, lines = _call("stortest_verdict", text=text)
    return verdict, lines


def stortest_readback(image):
    result = _call("stortest_readback", path=str(image))
    return bytes(result) if result is not None else None


def doctor_stages(repo):
    return [Stage(**s) for s in _call("doctor_stages", repo, board="pi5")]


def _run_readonly(args, repo, op):
    return native_run_call(
        None, repo, "bench_" + op, {**request_options(args, repo), "board": "pi5"}
    )


def cmd_status(args, repo):
    return _run_readonly(args, repo, "status")


def cmd_lease(args, repo):
    return _run_readonly(args, repo, "lease")


def cmd_doctor(args, repo):
    return run_controller(
        "bench_doctor", args, repo, command="pi5-doctor", arch="arm64", extra={"board": "pi5"}
    )


def cmd_boot(args, repo):
    return run_controller(
        "bench_boot", args, repo, command="pi5-boot", arch="arm64", extra={"board": "pi5"}
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
        result = native.call("bench_stage_pi5", kwargs, repo)
    except native.NativeError as error:
        print(str(error), file=sys.stderr)
        return 75 if error.kind == "LEASE_REFUSED" else 1
    if result is not None:
        print(json.dumps(result, indent=2))
    return 0


def cmd_serial(args, repo):
    try:
        return native_run_call(
            None,
            repo,
            "bench_service_serial_" + args.service,
            {"env": dict(os.environ), "board": "pi5"},
        )
    except native.NativeError as error:
        import sys

        print(str(error), file=sys.stderr)
        return 75 if error.kind == "BENCH_BUSY" else 1


def add_parser(sub) -> None:
    p = sub.add_parser("pi5", help="Raspberry Pi 5 bench: lease, doctor, boot this tree")
    p.set_defaults(fn=command)
    verbs = p.add_subparsers(dest="pi5_cmd", required=True)
    stage = verbs.add_parser("stage", help="stage this tree's fresh arm64 build")
    stage.add_argument("--allow-stale", action="store_true")
    stage.add_argument("--no-reboot", action="store_true")
    stage.add_argument("--freeldr-ini", action="store_true")
    serial = verbs.add_parser("serial", help="manage the leased native Pico listener")
    serial.add_argument("service", choices=["up", "down", "status"])
    verbs.add_parser("status", help="lease, power, Pico, serial, Linux stager, staged payload")
    lease = verbs.add_parser("lease", help="hold the board across several commands")
    lease.add_argument("action", choices=["take", "drop", "show"])
    lease.add_argument("--minutes", type=float, default=pi4lease.DEFAULT_MINUTES)
    lease.add_argument("--why", help="shown to other lanes while you hold it")
    lease.add_argument("--force", action="store_true", help="drop: release another lane's lease")
    verbs.add_parser("doctor", help="preflight the bench, with the fix for each problem")
    boot = verbs.add_parser(
        "boot", help="stage this tree's arm64 build over SSH, tryboot it; verdict"
    )
    boot.add_argument(
        "--timeout",
        type=float,
        default=pi4.DEFAULT_DESKTOP_TIMEOUT,
        help="seconds from the tryboot reboot to the desktop",
    )
    boot.add_argument("--allow-stale", action="store_true")
    boot.add_argument(
        "--options",
        nargs="+",
        default=[],
        metavar="OPT",
        help="loader options for this boot only, e.g. /PROCTRACE (pi5-bench.md)",
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
    boot.add_argument(
        "--stortest",
        action="store_true",
        help="serve a VIBEUSB image from the Pico, boot with /STORTEST, grade stortest.exe "
        "on serial and the written file on the image",
    )
    boot.add_argument(
        "--drop-medium",
        action="store_true",
        help="with --stortest: stop serving the image when stortest starts, so its mount "
        "reads a missing medium; passes when IoRaiseHardError completes inline and "
        "stortest still finishes",
    )


def command(args, repo):
    return {
        "serial": cmd_serial,
        "stage": cmd_stage,
        "status": cmd_status,
        "lease": cmd_lease,
        "doctor": cmd_doctor,
        "boot": cmd_boot,
    }[args.pi5_cmd](args, repo)


class ImageServerWatch:
    """Own the caller's child process and expose its state to the native waiter."""

    def __init__(self, server, log):
        import threading
        import time

        self.log, self.started = log, time.monotonic()
        self.lost_after = self.returncode = None

        def wait():
            rc = server.wait()
            self.returncode, self.lost_after = rc, time.monotonic() - self.started

        threading.Thread(target=wait, daemon=True).start()

    def lost(self):
        return self.lost_after is not None

    def stage(self):
        try:
            tail = self.log.read_text(errors="replace").strip().splitlines()[-2:]
        except OSError:
            tail = []
        return Stage(
            "serve-image",
            status="fail",
            detail=(
                f"image server exited {self.returncode} {self.lost_after:.0f}s "
                "after the reboot: the Pico dropped the medium"
            ),
            salient=[*tail, f"see {self.log}"],
        )


class MediumDrop:
    def __init__(self, serial, start, stop, poll=0.5):
        import threading

        self.dropped = False
        self._closed = threading.Event()

        def watch():
            while not self._closed.wait(poll):
                if _call("drop_marker", text=pi4._slice(serial, start)):
                    stop()
                    self.dropped = True
                    return

        self._thread = threading.Thread(target=watch, daemon=True)
        self._thread.start()

    def close(self):
        self._closed.set()
        self._thread.join(timeout=5)


def _await_stortest(run, serial, start, medium=None, drop=None):
    def medium_state(_):
        if medium is None or not medium.lost():
            return {"lost": False}
        stage = medium.stage()
        return {
            "lost": True,
            "returncode": medium.returncode,
            "lost_after": medium.lost_after,
            "salient": stage.salient,
        }

    callbacks = {
        "bench_medium_state": medium_state,
        "bench_drop_state": lambda _: {"dropped": drop.dropped if drop else False},
    }
    with native.DedicatedSession() as session:
        result = session.call(
            "bench_await_stortest",
            {
                "env": dict(os.environ),
                "serial": str(serial),
                "start": start,
                "has_medium": medium is not None,
                "has_drop": drop is not None,
            },
            callbacks=callbacks,
        )
    pi4._save_serial(run, result["text"])
    run.extra["stortest"] = result["lines"]
    grade = result["grade"]
    if grade["classification"]:
        run.classification = grade["classification"]
    run.add(
        Stage(
            grade["stage"], status=grade["status"], detail=grade["detail"], salient=grade["salient"]
        )
    )


def _grade_dropped_medium(run, text, verdict, lines, dropped):
    del verdict, lines
    grade = _call("grade_drop", text=text, dropped=dropped, server_lost=False)
    if grade["classification"]:
        run.classification = grade["classification"]
    run.add(
        Stage(
            grade["stage"], status=grade["status"], detail=grade["detail"], salient=grade["salient"]
        )
    )
