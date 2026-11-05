"""Python API compatibility for the native QEMU session owner.

VM launch policy, admission, media preparation, protocol parsing and cleanup live in Rust.
"""

from __future__ import annotations

import os
import sys
import tempfile
from pathlib import Path
from typing import Any

from tools.qemu_infra_compat import REPO, DedicatedSession, NativeError, call

from .qmp_client import QmpClient
from .serial_client import SerialClient

QEMU_DIR = REPO / "qemu"
USB_STICK_LABEL = "VIBESTICK"
USB_STICK_FILE = "VIBEOS.TXT"
USB_STICK_TEXT = b"hello from the usb stick\r\n"
STORTEST_LABEL = "VIBEUSB"
STORTEST_MARKER = b"vibeos usb e2e marker\r\n"
HOSTILE_UART = 0x5000
HOSTILE_UART_IRQ = 5
HANG_RESAMPLE_SECONDS = 5.0


def _call(name, **kwargs):
    try:
        return call(f"vm_{name}", {"env": dict(os.environ), **kwargs})
    except NativeError as exc:
        if name in {
            "smp_selected_iso",
            "stortest_selected_iso",
            "testkernel_boot_img",
            "make_temp_disk",
            "make_usb_stick_image",
        }:
            raise RuntimeError(str(exc)) from exc
        raise


GUESTFWD_ECHO_PORTS = _call("echo_ports")


def _guestfwd_echo_opts(addr):
    return _call("guestfwd_echo_opts", addr=addr)


def _option(name):
    return _call("env_options")[name]


def smp_count():
    return _option("smp")


def gic_version():
    return _option("gic")


def smmu_requested():
    return _option("smmu")


def cpu_model():
    return _option("cpu")


def boot_nvme():
    return _option("boot_nvme")


def headless():
    return _option("headless")


def stortest_enabled():
    return _option("stortest")


def writable_root():
    return _option("writable_root")


def stress_icount_shift():
    return _option("icount_shift")


def stress_tcg_thread():
    return _option("tcg_thread")


def stress_timeout_scale():
    return _option("timeout_scale")


def xhci_interrupts():
    return _call("xhci_interrupts")


def nvme_bridge():
    return _call("nvme_bridge")


def tripwire_args():
    return _call("tripwire_args")


def hostile_hazards():
    return set(_call("hostile_hazards"))


def smp_selected_iso(iso, work_dir):
    return Path(_call("smp_selected_iso", iso=str(iso), work_dir=str(work_dir)))


def stortest_selected_iso(iso, work_dir):
    return Path(_call("stortest_selected_iso", iso=str(iso), work_dir=str(work_dir)))


def testkernel_boot_img(src, work_dir):
    return Path(_call("testkernel_boot_img", src=str(src), work_dir=str(work_dir)))


def apply_stress_knobs(cmd, arch):
    return _call("apply_stress_knobs", args=cmd, arch=arch)


def console_uart_args(arch, chardev):
    return _call("console_uart_args", arch=arch, chardev=chardev)


def _acpitable_relative(args):
    return _call("acpitable_relative", args=args)


def make_spcr(path, port, irq):
    _call("make_spcr", path=str(path), port=port, irq=irq)


def make_temp_disk(out):
    _call("make_temp_disk", path=str(out))


def boot_media_device_args(iso, *, ide_bus=None, default_transport="ide-cd"):
    return _call(
        "boot_media_device_args", iso=str(iso), ide_bus=ide_bus, default_transport=default_transport
    )


def make_usb_stick_image(path, fat=16, mib=64, label=USB_STICK_LABEL, files=None):
    if files is None:
        files = {USB_STICK_FILE: USB_STICK_TEXT}
    _call(
        "make_usb_stick_image",
        path=str(path),
        fat=fat,
        mib=mib,
        label=label,
        files=[[name, list(data)] for name, data in files.items()],
    )


class ArchConfig:
    arch: str
    qemu_bin: str
    memory: str
    machine_args: list[str]
    boot_marker: str
    boot_timeout: float
    qmp_timeout: float
    min_boot_bytes: int
    desktop_settle_seconds: float
    hang_silence_seconds: float
    required_milestones: list[str]
    _variant = "arm64"
    temp_disk = True
    ECHO_GUEST_PORT = 7777
    TDI_SRV_GUEST_PORT = 8888
    PAYLOAD_GUEST_PORT = 8080
    GUESTFWD_ADDR = "10.0.2.100"
    echo_hostfwd_port = 0
    tdi_srv_hostfwd_port = 0
    payload_hostfwd_port = 0

    def __init__(self, **overrides):
        if type(self) is ArchConfig and "arch" in overrides:
            self._variant = overrides["arch"]
        with tempfile.TemporaryDirectory(prefix="vibeos-config-") as work:
            self.__dict__.update(_call("arch_config", arch=self._variant, work_dir=work))
        self.__dict__.update(overrides)

    def hostile_machine_args(self):
        return _call("machine_args", arch=self._variant)

    def hostile_memory(self):
        return "6G" if "mem" in hostile_hazards() else self.memory

    def _request(self, work_dir):
        return {
            "arch": self._variant,
            "work_dir": str(work_dir),
            "env": dict(os.environ),
            "memory": self.memory,
            "temp_disk": self.temp_disk,
            "overrides": {
                key: getattr(self, key)
                for key in (
                    "boot_marker",
                    "boot_timeout",
                    "qmp_timeout",
                    "min_boot_bytes",
                    "desktop_settle_seconds",
                    "hang_silence_seconds",
                    "required_milestones",
                )
            },
        }

    def prepare_args(self, work_dir):
        data = _call("prepare_config", **self._request(work_dir))
        return data["arch_args"]

    def qemu_args(self, work_dir):
        return self.prepare_args(work_dir)

    def network_args(self):
        raise RuntimeError(
            "network ports are allocated by native VM start under its handshake lease"
        )


class Arm64Config(ArchConfig):
    _variant = "arm64"


class Arm64TestKernelConfig(ArchConfig):
    _variant = "arm64-test"


class Arm64NetConfig(ArchConfig):
    _variant = "arm64-net"


class I386Config(ArchConfig):
    _variant = "i386-uefi"


class I386UefiConfig(I386Config):
    pass


class I386UefiNetConfig(ArchConfig):
    _variant = "i386-uefi-net"


class Amd64Config(ArchConfig):
    _variant = "amd64-uefi"


class Amd64UefiConfig(Amd64Config):
    pass


class Amd64UefiNetConfig(ArchConfig):
    _variant = "amd64-uefi-net"


class Amd64BiosConfig(ArchConfig):
    _variant = "amd64-bios"


ARCH_CONFIGS = {
    cls._variant: cls
    for cls in (
        Arm64Config,
        Arm64TestKernelConfig,
        Arm64NetConfig,
        I386UefiConfig,
        I386UefiNetConfig,
        Amd64UefiConfig,
        Amd64UefiNetConfig,
        Amd64BiosConfig,
    )
}


def get_arch_config(arch):
    if arch not in ARCH_CONFIGS:
        raise ValueError(f"Unknown architecture: {arch!r}. Choose from: {list(ARCH_CONFIGS)}")
    return ARCH_CONFIGS[arch]()


class _OwnedProcess:
    def __init__(self, vm, pid, args):
        self.vm, self.pid, self.args = vm, pid, args

    def poll(self):
        return self.vm._call("poll_exit")

    def wait(self, timeout=None):
        return self.vm.wait_exit(86400 if timeout is None else timeout)


class QemuVM:
    gdb_host: str
    gdb_port: int
    net_host: str
    launch_timings: dict[str, float]

    def __init__(
        self,
        config,
        work_dir,
        *,
        backend="native",
        windows=None,
        gdb=False,
        gdb_wait=False,
        serial_log=None,
        checkpoint=None,
        capture_restrict_network=False,
        prepared_config=None,
        max_lifetime=None,
    ):
        self.config = config
        if prepared_config is None and not hasattr(config, "_up_boot_timeout"):
            config._up_boot_timeout = config.boot_timeout
            config._up_hang_silence_seconds = config.hang_silence_seconds
        self.work_dir = Path(work_dir)
        self._backend = backend
        self._win = windows or {}
        self._session = DedicatedSession()
        self._closed = False
        env = {**os.environ, "VIBEOS_TEST_PYTHON": sys.executable}
        if max_lifetime is not None:
            env["VIBEOS_QEMU_MAX_LIFETIME"] = str(max_lifetime)
        request = config._request(work_dir) if prepared_config is None else dict(prepared_config)
        request.update(
            work_dir=str(work_dir),
            backend=backend,
            windows=self._win,
            gdb=gdb,
            gdb_wait=gdb_wait,
            serial_log=str(serial_log) if serial_log is not None else None,
            env=env,
        )
        if prepared_config is None:
            request["overrides"]["boot_timeout"] = config._up_boot_timeout
            request["overrides"]["hang_silence_seconds"] = config._up_hang_silence_seconds
            request["timeout_is_up"] = True
        request["owner_pid"] = os.getpid()
        request["capture_restrict_network"] = capture_restrict_network
        if checkpoint is not None:
            request["checkpoint"] = checkpoint
        if prepared_config is not None:
            request["live_log"] = str(serial_log or Path(work_dir) / "serial.log")
        # Metadata-only construction; mutable media are prepared after native admission.
        result = self._session.call("vm_create", request)
        self._id = result["id"]
        self.serial = SerialClient(_session=self._session, _vm_id=self._id)
        self.qmp = QmpClient(_session=self._session, _vm_id=self._id)
        self._boot_output = ""
        self._proc = None
        self.hang_cpu_state = ""
        self.on_qmp_ready = None
        self._sync(result["handles"])

    def _call(self, operation, **kwargs):
        try:
            return self._session.call(f"vm_{operation}", {"id": self._id, **kwargs})
        except NativeError as exc:
            if "timed out" in str(exc).lower() or "timeout" in str(exc).lower():
                raise TimeoutError(str(exc)) from exc
            if "exited" in str(exc).lower() or "closed" in str(exc).lower():
                raise ConnectionError(str(exc)) from exc
            raise RuntimeError(str(exc)) from exc

    @property
    def scratch_path(self):
        # On the windows-host backend QEMU runs against the STAGED copies, so the
        # scratch disk and its sibling sticks (usbdisk.qcow2, ...) that the guest
        # writes live in the staging dir. The work_dir originals stay pristine:
        # reading them made stortest see no WRITETST.TXT on any arch.
        staging = getattr(self, "_win_staging", None)
        base = staging if staging is not None else self.work_dir
        return base / "scratch.qcow2"

    def _sync(self, handles):
        for key in ("gdb_host", "gdb_port", "net_host", "launch_timings"):
            setattr(self, key, handles.get(key))
        self.live_log_path = Path(handles["live_log"])
        self._win_uid = handles.get("win_uid", "")
        self._win_staging = Path(handles["staging"]) if handles.get("staging") else None
        self.config.__dict__.update(handles.get("config", {}))
        self.config.__dict__.update(handles.get("network", {}))
        if handles.get("qemu_pid"):
            self._proc = _OwnedProcess(self, handles["qemu_pid"], handles.get("argv", []))

    def start(self):
        if isinstance(self.config, ArchConfig) and (
            type(self.config).qemu_args is not ArchConfig.qemu_args
            or type(self.config).prepare_args is not ArchConfig.prepare_args
        ):
            self._call("admit")
            prepare = (
                self.config.prepare_args
                if type(self.config).prepare_args is not ArchConfig.prepare_args
                else self.config.qemu_args
            )
            try:
                args = prepare(self.work_dir)
                self._call("override_arch_args", args=args)
            except BaseException:
                self.stop()
                raise
        self._sync(self._call("start"))
        if self.on_qmp_ready is not None:
            self.on_qmp_ready(self.qmp)

    def wait_for_boot(self):
        try:
            self._boot_output = self._call("wait_for_boot")
            return self._boot_output
        except BaseException:
            self._boot_output = self.serial._buf
            raise

    def wait_exit(self, timeout=20.0):
        return self._call("wait_exit", timeout=timeout)

    def serial_output(self, timeout=1.0):
        return self._call("serial_output", timeout=timeout)

    def save_snapshot(self, name="ready"):
        self._call("save_snapshot", name=name)

    def restore_snapshot(self, name="ready"):
        self._call("restore_snapshot", name=name)

    def screendump(self, path):
        return Path(self._call("screendump", path=str(path)))

    def capture_cpu_state(self):
        return self._call("capture_cpu_state")

    def _nvme_enabled(self):
        return self._call("nvme_enabled")

    def read_debugcon(self):
        return bytes(self._call("read_debugcon"))

    def debugcon_report(self):
        raw = self.read_debugcon()
        return (
            call("verdict_debugcon_report", {"raw": list(raw), "arch": self.config.arch})
            if raw
            else None
        )

    def report_replay_break(self):
        return self._call("report_replay_break")

    def _qemu_stderr_tail(self):
        return self._call("stderr_tail")

    def save_hang_state(self, phase):
        self.hang_cpu_state = self._call("save_hang_state", phase=phase)
        return self.hang_cpu_state

    def detach_for_debug(self, owner_pid, run_id):
        result = self._call("detach", owner_pid=owner_pid, run_id=run_id)
        self._session.close()
        self._closed = True
        return result

    def stop(self):
        if not self._closed:
            try:
                self._call("stop")
            finally:
                self._session.close()
                self._closed = True

    def __enter__(self):
        self.start()
        return self

    def __exit__(self, *_):
        self.stop()


PROJECT_DIR = REPO
LOCK_DIR = Path(os.environ.get("VIBEOS_QEMU_LOCK_DIR", "/tmp/vibeos-qemu-locks"))
SLOT_DIR = LOCK_DIR / "slots"
QEMU_MAX_CONCURRENT = 16
LAUNCH_ATTEMPTS = 3
LAUNCH_RETRY_SECONDS = 2.0


def process_cpu_pct(pid, window=3.0):
    return _call("process_cpu_pct", pid=pid, window=window)


def per_arch_cap(arch):
    return _call("per_arch_cap", arch=arch)


def reap_orphan_qemus(*, dry_run=False):
    return _call("reap_orphan_qemus", dry_run=dry_run)


def launch_with_retry(make_vm, timings, log, label):
    state: dict[str, Any] = {"vm": None, "error": None}

    def make(_):
        state["vm"] = make_vm()
        return None

    def start(_):
        try:
            state["vm"].start()
            return {"timings": state["vm"].launch_timings}
        except Exception as exc:
            state["error"] = exc
            return {"error": str(exc)}

    def stop(_):
        try:
            state["vm"].stop()
        except Exception:
            pass

    with DedicatedSession() as session:
        result = session.call(
            "test_launch_retry",
            {"attempts": LAUNCH_ATTEMPTS, "retry_seconds": LAUNCH_RETRY_SECONDS, "label": label},
            callbacks={
                "launch_make": make,
                "launch_start": start,
                "launch_stop": stop,
                "launch_log": lambda a: log(a["message"]),
            },
        )
    timings.update(result["timings"])
    return state["vm"], result["attempts"], state["error"] if result["error"] is not None else None


HOSTILE_HAZARDS = ("q35", "cpu", "nops2", "novga", "mem", "serial", "a76", "el2")


def _write_sparse_image(path, data):
    _call("sparse_image", path=str(path), data=list(data))


def temp_disk_args(arch, out_dir):
    return _call("temp_disk_args", arch=arch, work_dir=str(out_dir))


def smp_boot_timeout(base, n):
    return _call("smp_boot_timeout", base=base, count=n)


_MP_ENTRY_FOR = {b"LiveCD_Debug": b"LiveCD_SMP", b"ReactOS_Arm64_UP": b"ReactOS_Arm64_MP"}
