"""Pytest entry points for native payload contracts and execution.

The shared spec parser lives in vibeos-build; vibeos-test owns draining,
render/input/survival checks and TCP load. These adapters supply VM handles and
translate native outcomes into pytest presentation.
"""

from __future__ import annotations

import os
import time
from pathlib import Path
from typing import Any

import pytest

from tools.qemu_infra_compat import DedicatedSession, NativeError, call

from .vm import PROJECT_DIR, QemuVM

PAYLOADS_DIR = PROJECT_DIR / "tools" / "payloads"
RENDER_DELTA_MIN = 0.02
STATIC_DELTA_MAX = 0.012


def _selected_payloads(config: pytest.Config) -> list[str]:
    return call(
        "payload_select",
        {"payload": config.getoption("--payload") or "", "doom": bool(config.getoption("--doom"))},
    )


def load_spec(name: str) -> dict[str, Any]:
    return call("payload_load_spec", {"name": name})


def pytest_generate_tests(metafunc: pytest.Metafunc) -> None:
    if "payload" in metafunc.fixturenames:
        names = _selected_payloads(metafunc.config)
        metafunc.parametrize("payload", names, ids=names)


@pytest.fixture(scope="session")
def specs() -> dict[str, dict[str, Any]]:
    return {}


@pytest.fixture
def spec(payload: str, specs: dict[str, dict[str, Any]]) -> dict[str, Any]:
    if payload not in specs:
        specs[payload] = load_spec(payload)
    return specs[payload]


def _callbacks(vm: Any) -> dict[str, Any]:
    def capture(_):
        try:
            return {"state": vm.capture_cpu_state()}
        except Exception as error:
            return {"error": str(error)}

    def screenshot(a):
        try:
            vm.screendump(Path(a["path"]))
            return {}
        except Exception as error:
            return {"error": str(error)}

    return {
        "payload_read": lambda a: vm.serial_output(timeout=a["timeout"]),
        "payload_screendump": screenshot,
        "payload_qmp": lambda a: vm.qmp.execute(a["command"], **a["arguments"]),
        "payload_cpu_state": capture,
        "payload_save_hang": lambda a: vm.save_hang_state(a["phase"]),
        "payload_sleep": lambda a: time.sleep(a["seconds"]),
        "payload_print": lambda a: print(a["message"], flush=True),
    }


def _run(operation: str, arguments: dict, callbacks: dict) -> Any:
    try:
        with DedicatedSession() as session:
            result = session.call(operation, arguments, callbacks)
    except NativeError as error:
        if error.kind == "AssertionError":
            raise AssertionError(str(error)) from error
        raise
    if isinstance(result, dict):
        if result.get("status") == "skip":
            pytest.skip(result["reason"])
        if result.get("status") == "xfail":
            pytest.xfail(result["reason"])
    return result


def _context(vm: Any, arch: str = "", payload: str = "", spec: Any = None) -> dict:
    work = getattr(vm, "work_dir", PROJECT_DIR / "qemu")
    config = getattr(vm, "config", None)
    return {
        "repo": str(PROJECT_DIR),
        "arch": arch,
        "name": payload,
        "spec": spec,
        "work_dir": str(work),
        "artifact_dir": str(getattr(vm, "artifact_dir", None) or work),
        "boot_output": getattr(vm, "_boot_output", None) or "",
        "force": os.environ.get("VIBEOS_PAYLOAD_FORCE") == "1",
        "host_port": getattr(config, "payload_hostfwd_port", 0),
        "echo_port": getattr(config, "echo_hostfwd_port", 0),
        "net_host": getattr(vm, "net_host", "127.0.0.1"),
    }


def _frame_delta(vm: QemuVM, gap: float) -> float:
    return _run("payload_frame_delta", {**_context(vm), "gap": gap}, _callbacks(vm))


def _send(vm: QemuVM, events: list) -> None:
    _run("payload_send", {"events": events}, _callbacks(vm))


def _click_center(vm: QemuVM) -> None:
    _run("payload_click", {}, _callbacks(vm))


def _send_key(vm: QemuVM, qcode: str) -> None:
    _run("payload_key", {"key": qcode}, _callbacks(vm))


def _run_drive(vm: QemuVM, steps: list[dict[str, Any]]) -> None:
    _run("payload_drive", {"steps": steps}, _callbacks(vm))


def _require_net(spec: dict[str, Any], arch: str, payload: str) -> None:
    _run("payload_require_net", {"spec": spec, "arch": arch, "name": payload}, {})


def drain_for_marker(
    read: Any,
    output: str,
    marker: str,
    fail_marker: str,
    budget_secs: float,
    idle_secs: float = 0.0,
    now: Any = time.monotonic,
    on_output: Any = None,
    on_silence: Any = None,
    silence_secs: float = 0.0,
) -> tuple[str, str]:
    result = _run(
        "payload_drain",
        {
            "output": output,
            "marker": marker,
            "fail_marker": fail_marker,
            "budget_secs": budget_secs,
            "idle_secs": idle_secs,
            "silence_secs": silence_secs,
            "callback_clock": True,
            "on_output": on_output is not None,
            "on_silence": on_silence is not None,
        },
        {
            "payload_read": lambda a: read(timeout=a["timeout"]),
            "payload_clock": lambda _: now(),
            "payload_on_output": lambda a: on_output(a["output"]),
            "payload_on_silence": lambda a: on_silence(a["seconds"]),
        },
    )
    return result["output"], result["reason"]


class SilenceSampler:
    def __init__(self, vm: Any, limit: int = 12) -> None:
        self.vm, self.limit, self.taken = vm, limit, 0

    def __call__(self, silent_for: float) -> None:
        self.taken = _run(
            "payload_sample",
            {
                **_context(self.vm),
                "seconds": silent_for,
                "limit": self.limit,
                "taken": self.taken,
            },
            _callbacks(self.vm),
        )


class ShotTaker:
    def __init__(self, vm: Any, prefix: str | None) -> None:
        self.vm, self.prefix, self.taken = vm, prefix, 0

    def __call__(self, output: str) -> None:
        self.taken = _run(
            "payload_shots",
            {
                **_context(self.vm),
                "prefix": self.prefix,
                "taken": self.taken,
                "output": output,
            },
            _callbacks(self.vm),
        )


def test_payload_launched(vm: QemuVM, arch: str, payload: str, spec: dict[str, Any]) -> None:
    _run("payload_launch", _context(vm, arch, payload, spec), _callbacks(vm))


def test_payload_renders(
    vm: QemuVM, arch: str, payload: str, spec: dict[str, Any], request: pytest.FixtureRequest
) -> None:
    _run("payload_renders", _context(vm, arch, payload, spec), _callbacks(vm))


def test_payload_input_reaches_app(
    vm: QemuVM, arch: str, payload: str, spec: dict[str, Any], request: pytest.FixtureRequest
) -> None:
    _run("payload_input", _context(vm, arch, payload, spec), _callbacks(vm))


def test_payload_tcp_hammer(vm: QemuVM, arch: str, payload: str, spec: dict[str, Any]) -> None:
    _run("payload_tcp_hammer", _context(vm, arch, payload, spec), _callbacks(vm))
