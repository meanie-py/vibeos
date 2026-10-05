"""Generic payload gate — interprets tools/payloads/<name>/payload.yaml (`test:`).

Design doc: docs/engineering/payload-harness-design.md. A payload declares its
binaries (staged onto the media at build time by tools/payloads/compile.py) and
its test contract: a launch marker, a renders probe, and an input-drive script
with an oracle. This module is the TEST-TIME consumer: it reads the same YAML
and runs the three gates against one booted VM per session.

Doom was the hand-written original (test_doom.py, retired 2026-07-07 — its
click/keys/frame-delta logic became the interpreter verbs here); it is now just
tools/payloads/doom/payload.yaml. Adding an app = adding a YAML file.

OFF by default: requires `--payload <name>` (or the legacy `--doom` alias) AND
media built with the payload staged:
    VIBEOS_PAYLOAD=doom ./vibeos-cli build --arch <arch>
    pytest tests/qemu/test_payload.py --arch arm64 --payload doom
Front door: ./vibeos-cli smoke --arch arm64 --payload doom  (alias: --doom)

Verb vocabulary (grown on demand — see design doc; keep it small):
    drive steps:  {keys: [qcodes], gap: s} | {wait: s} | {focus: click-center}
    oracles:      frame_delta_min / frame_delta_static_max (+ gap), survives
                  (QMP query-status running). launch_marker covers serial.
"""

from __future__ import annotations

import re
import time
from pathlib import Path
from typing import Any

import pytest
import yaml

from .vm import PROJECT_DIR, QemuVM

PAYLOADS_DIR = PROJECT_DIR / "tools" / "payloads"

# Default thresholds (a payload's `test:` section can override per-gate).
RENDER_DELTA_MIN = 0.02
STATIC_DELTA_MAX = 0.012


def _selected_payloads(config: pytest.Config) -> list[str]:
    names = [n.strip() for n in (config.getoption("--payload") or "").split(",") if n.strip()]
    if config.getoption("--doom") and "doom" not in names:
        names.append("doom")
    return names


def load_spec(name: str) -> dict[str, Any]:
    path = PAYLOADS_DIR / name / "payload.yaml"
    if not path.is_file():
        pytest.fail(f"unknown payload {name!r}: no {path}")
    spec = yaml.safe_load(path.read_text())
    assert spec.get("name") == name, f"{path}: name != directory"
    return spec


def pytest_generate_tests(metafunc: pytest.Metafunc) -> None:
    if "payload" in metafunc.fixturenames:
        names = _selected_payloads(metafunc.config)
        # No selection -> parametrize empty; the fixtureless skip in
        # test_* below never triggers, pytest just collects zero cases.
        metafunc.parametrize("payload", names, ids=names)


@pytest.fixture(scope="session")
def specs() -> dict[str, dict[str, Any]]:
    return {}


@pytest.fixture
def spec(payload: str, specs: dict[str, dict[str, Any]]) -> dict[str, Any]:
    if payload not in specs:
        specs[payload] = load_spec(payload)
    return specs[payload]


# ---------------------------------------------------------------------------
# Host-side probe/drive primitives (ported verbatim from test_doom.py — the
# QMP input model + frame-delta oracle validated by the Doom bring-up).
# ---------------------------------------------------------------------------


def _frame_delta(vm: QemuVM, gap: float) -> float:
    """Fraction of pixels differing between two screendumps `gap` seconds apart."""
    from PIL import Image, ImageChops

    a_p = vm.work_dir / "payload_a.ppm"
    b_p = vm.work_dir / "payload_b.ppm"
    vm.screendump(a_p)
    time.sleep(gap)
    vm.screendump(b_p)
    a = Image.open(a_p).convert("RGB")
    b = Image.open(b_p).convert("RGB")
    diff = ImageChops.difference(a, b).convert("L").point([255 if v > 8 else 0 for v in range(256)])
    hist = diff.histogram()
    changed = sum(hist[1:])
    return changed / float(a.size[0] * a.size[1])


def _send(vm: QemuVM, events: list) -> None:
    r = vm.qmp.execute("input-send-event", events=events)
    if "error" in r:
        pytest.fail(f"QMP input-send-event failed: {r['error']}")


def _click_center(vm: QemuVM) -> None:
    """Focus-click the app window: sweep the relative mouse to the top-left
    corner (saturate), recenter into the client area, left-click. Exercises
    the full mouse chain (class driver -> win32k hit-test -> activation)."""
    for _ in range(25):  # pin to top-left corner (rel mouse: saturate)
        _send(
            vm,
            [
                {"type": "rel", "data": {"axis": "x", "value": -50}},
                {"type": "rel", "data": {"axis": "y", "value": -40}},
            ],
        )
    for _ in range(10):  # move to ~screen centre — inside a maximized client area
        _send(
            vm,
            [
                {"type": "rel", "data": {"axis": "x", "value": 40}},
                {"type": "rel", "data": {"axis": "y", "value": 28}},
            ],
        )
    _send(vm, [{"type": "btn", "data": {"button": "left", "down": True}}])
    _send(vm, [{"type": "btn", "data": {"button": "left", "down": False}}])
    time.sleep(1.5)  # let activation settle


def _send_key(vm: QemuVM, qcode: str) -> None:
    for down in (True, False):
        _send(
            vm, [{"type": "key", "data": {"down": down, "key": {"type": "qcode", "data": qcode}}}]
        )


def _run_drive(vm: QemuVM, steps: list[dict[str, Any]]) -> None:
    """Interpret a drive script. Unknown verbs fail loud (house rule) — grow
    the vocabulary in ONE place here + document it in the module docstring."""
    for step in steps:
        if "keys" in step:
            gap = float(step.get("gap", 0.5))
            for key in step["keys"]:
                _send_key(vm, str(key))
                time.sleep(gap)
        elif "wait" in step:
            time.sleep(float(step["wait"]))
        elif "focus" in step:
            if step["focus"] != "click-center":
                pytest.fail(f"unknown focus mode {step['focus']!r}")
            _click_center(vm)
        else:
            pytest.fail(f"unknown drive step {step!r} — add the verb to test_payload.py")


def _apply_xfail(spec: dict[str, Any], arch: str, request: pytest.FixtureRequest) -> None:
    import os

    # Debug knob: force-run on xfail arches to capture live serial evidence
    # ([WIN32K] KEYDROP traces etc.) of *where* the chain breaks.
    if os.environ.get("VIBEOS_PAYLOAD_FORCE") == "1":
        return
    for prefix, reason in (spec.get("test", {}).get("xfail") or {}).items():
        if arch.startswith(prefix):
            pytest.xfail(reason)


def _require_net(spec: dict[str, Any], arch: str, payload: str) -> None:
    """Payloads with `net: true` need the -net arch variant (NIC + the payload
    hostfwd). Fail loud with the right invocation rather than timing out."""
    if spec.get("test", {}).get("net") and not arch.endswith("-net"):
        pytest.fail(
            f"payload {payload!r} declares net: true — boot the -net variant "
            f"(e.g. --arch {arch}-net) so the NIC + payload hostfwd exist"
        )


# ---------------------------------------------------------------------------
# The gates (one boot per session; gates run in declaration order).
# ---------------------------------------------------------------------------


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
    """Drain serial until `marker` appears, `fail_marker` appears, or
    `budget_secs` of WALL CLOCK elapse.

    Extracted from test_payload_launched so the budget semantics can be unit
    tested. They were wrong for a long time and nothing could see it: the loop
    was `for _ in range(budget)` around a `read(timeout=1.0)` that returns the
    instant bytes are already waiting, so one "second" cost 1s while the guest
    was silent and ~20ms while it was talking. The declared budget therefore
    shrank in proportion to how chatty the payload was — exactly backwards,
    since the payloads needing the longest drain are the ones producing the most
    output. procstorm hit it first (2400 "seconds" spent in 175s) and worked
    around it by inflating its own number to 60000; the apitest sweep then walked
    into the same trap, a declared 7200s drain ending after 12-28 minutes and
    truncating every run mid-sweep. The expensive part was not the truncation but
    the ATTRIBUTION: the driver above blamed whichever subtest was in flight when
    the harness stopped reading, and quarantined innocent tests as crashers.

    `on_silence(seconds_silent)` is called every `silence_secs` of an unbroken
    serial silence, without ending the drain: a guest that goes quiet and then
    RECOVERS leaves no hang capture, and its vCPUs can only be read during the
    quiet.
    """
    deadline = now() + budget_secs
    last_progress = now()
    next_silence = silence_secs
    line_deadline = None
    while True:
        if marker in output:
            # A marker is usually a PREFIX of the line that carries the verdict
            # ("[APPSTORM] done" vs "... done VERDICT=PASS"). Returning on the
            # prefix alone let a read that ended mid-line hand the caller
            # "[APPSTORM] done VER", and a 38/38 sweep failed its pass check.
            # Finish the line first, briefly bounded.
            if "\n" in output[output.index(marker) :]:
                return output, "marker"
            if line_deadline is None:
                line_deadline = now() + 10.0
            elif now() >= line_deadline:
                return output, "marker"
        if fail_marker and fail_marker in output:
            return output, "fail"
        if now() >= deadline:
            return output, "budget"
        if idle_secs and now() - last_progress >= idle_secs:
            return output, "idle"
        if on_silence is not None and silence_secs and now() - last_progress >= next_silence:
            on_silence(now() - last_progress)
            next_silence += silence_secs
        chunk = read(timeout=1.0)
        if chunk:
            output += chunk
            last_progress = now()
            next_silence = silence_secs
            if on_output is not None:
                on_output(output)


class SilenceSampler:
    """Where every vCPU is while the payload is silent, appended to
    silence-samples.txt in the run's artifact dir. Unlike save_hang_state this
    ends nothing: it exists for the stall that recovers. arm64 kmtest goes
    silent for ~95 s inside NpfsConnect on every round -- the guest clock keeps
    ticking but not one thread, the 10 s census watchdog included, is scheduled
    -- and then carries on, so no hang path ever fires and the culprit PC is
    only readable during the silence. Never raises."""

    def __init__(self, vm: Any, limit: int = 12) -> None:
        self.vm, self.limit, self.taken = vm, limit, 0

    def __call__(self, silent_for: float) -> None:
        if self.taken >= self.limit:
            return
        self.taken += 1
        try:
            state = self.vm.capture_cpu_state()
        except Exception as exc:
            state = f"<capture failed: {exc!r}>\n"
        try:
            from .cli.triage import cpu_pcs

            pcs = " ".join(f"CPU#{i}=0x{pc}" for i, pc in cpu_pcs(state)) or "?"
        except Exception:
            pcs = "?"
        head = (
            f"=== silence sample #{self.taken}: serial silent {silent_for:.0f}s  pcs: {pcs} ===\n"
        )
        print(f"[payload] {head.strip()}", flush=True)
        dest = Path(self.vm.artifact_dir or self.vm.work_dir) / "silence-samples.txt"
        try:
            with dest.open("a") as fh:
                fh.write(head + state + "\n")
        except OSError:
            pass


class ShotTaker:
    """A payload's screenshot requests: each complete serial line containing
    `<shot_prefix><name>` gets one QMP screendump, saved as shot-NN-<name>.png in the run's
    artifact dir. The guest waits a few seconds after printing it; a screendump is immediate.
    This is how a payload that drives GUI apps shows what they actually DREW, which no
    in-guest check can see."""

    def __init__(self, vm: Any, prefix: str | None) -> None:
        self.vm, self.prefix, self.taken = vm, prefix, 0

    def __call__(self, output: str) -> None:
        if not self.prefix:
            return
        # Only complete lines: a name split across two serial reads must not be shot twice.
        requests = re.findall(re.escape(self.prefix) + r"([A-Za-z0-9_.\-]+)\r?\n", output)
        for name in requests[self.taken :]:
            self.taken += 1
            dest_dir = self.vm.artifact_dir or self.vm.work_dir
            ppm = Path(dest_dir) / f"shot-{self.taken:02d}-{name}.ppm"
            try:
                # QMP screendump writes PPM whatever the name says; convert as test_smoke does.
                self.vm.screendump(ppm)
                path = ppm
                try:
                    from PIL import Image

                    path = ppm.with_suffix(".png")
                    Image.open(ppm).save(path, "PNG")
                    ppm.unlink()
                except ImportError:
                    pass
                print(f"[payload] screenshot {path}", flush=True)
            except Exception as e:  # a lost screenshot must not end the run
                print(f"[payload] screenshot {name} FAILED: {e}", flush=True)


def test_payload_launched(vm: QemuVM, arch: str, payload: str, spec: dict[str, Any]) -> None:
    """The payload's exe was spawned (media staged + launcher ran)."""
    t = spec.get("test", {})
    _require_net(spec, arch, payload)
    marker = t.get("launch_marker")
    if not marker:
        pytest.skip(f"payload {payload!r} declares no launch_marker")
    # A crashed guest produces no more serial (or reboots into the same crash),
    # so draining the full launch_drain_secs budget after a BUGCHECK is pure
    # waste — critical for long conformance sweeps where one wedged test would
    # otherwise cost the whole ~90-min drain. Break early on the fail marker; the
    # assert below then fails loud with the crash in the serial tail (and the
    # conformance verb still parses the partial log for the CRASH attribution).
    fail_marker = t.get("fail_marker", "*** BUGCHECK")
    output = vm._boot_output or ""
    shooter = ShotTaker(vm, t.get("shot_prefix"))
    shooter(output)
    if marker not in output:
        # Launcher may fire slightly after the boot marker — drain a little more.
        # WALL-CLOCK bounded; see drain_for_marker for why that is not obvious.
        output, reason = drain_for_marker(
            vm.serial_output,
            output,
            marker,
            fail_marker,
            float(t.get("launch_drain_secs", 30)),
            float(t.get("launch_idle_secs", 0)),
            on_output=shooter,
            on_silence=SilenceSampler(vm),
            silence_secs=float(t.get("silence_sample_secs", 0)),
        )
        # Printed so a DRIVER above can tell WHY the boot stopped. "idle" is the
        # one reason that positively implicates the guest: the payload went
        # silent while the rig was still happily reading. "budget" is the rig
        # running out of time and implicates nobody.
        print(f"[payload] drain-end: {reason}", flush=True)
    if marker not in output and not (fail_marker and fail_marker in output):
        # Drained the whole budget with no marker and no crash: the guest is
        # wedged with the VM still alive, which is the ONE moment its vCPU PCs
        # can be read. wait_for_boot's capture does not cover this — the boot
        # marker was already seen, so nothing timed out there. Without this a
        # stalled sweep yields only "everything stopped at <last serial line>",
        # and every recurrence produces the same no-information capture.
        vm.save_hang_state(f"payload {payload}: launch_marker never appeared")
    assert marker in output, (
        f"--payload {payload} was passed but {marker!r} never appeared. Either "
        f"the media was not built with VIBEOS_PAYLOAD={payload} (fix: "
        f"VIBEOS_PAYLOAD={payload} ./vibeos-cli build --arch <arch>) or the "
        f"launcher regressed (grep serial for CreateProcessW errors).\n"
        f"serial tail: {output[-800:]!r}"
    )
    # A payload that grades itself says so in its output; without this a run whose own
    # verdict was FAIL passed the harness because its launch marker appeared.
    pass_marker = t.get("pass_marker")
    if pass_marker:
        verdict = next((ln for ln in output.splitlines() if marker in ln), "")
        assert pass_marker in output, (
            f"--payload {payload} finished but its output lacks {pass_marker!r}: "
            f"{verdict.strip()!r}"
        )


def test_payload_renders(
    vm: QemuVM, arch: str, payload: str, spec: dict[str, Any], request: pytest.FixtureRequest
) -> None:
    """The app is actually drawing frames."""
    r = spec.get("test", {}).get("renders")
    if not r:
        pytest.skip(f"payload {payload!r} declares no renders gate")
    pytest.importorskip("PIL")
    time.sleep(float(r.get("settle", 3)))
    threshold = float(r.get("frame_delta_min", RENDER_DELTA_MIN))
    delta = _frame_delta(vm, gap=float(r.get("gap", 2.0)))
    assert delta >= threshold, (
        f"screen delta {delta:.4f} < {threshold} — {payload} launched but "
        "nothing is animating (GDI blit path / app loop wedged?). Check "
        f"qemu/{arch}_smoke.png and the serial log."
    )
    print(f"\n[{payload}] rendering confirmed — frame delta {delta:.1%}")


def test_payload_input_reaches_app(
    vm: QemuVM, arch: str, payload: str, spec: dict[str, Any], request: pytest.FixtureRequest
) -> None:
    """PLAYABLE gate: injected input demonstrably changes app behavior.

    The spec's `input.drive` script runs against a focused window, then
    `input.oracle` judges the result (for Doom: menu-driving a new game turns
    the ~50%-delta attract demo into a <1.2%-delta idle first-person view — a
    clean binary signal). Retries with `on_retry` absorb post-boot focus
    settling.

    Real catches so far (all found by this gate's Doom incarnation,
    2026-07-06/07): acpi-rust's invented CmResourceType table (PS/2 ISRs never
    connected), i8042prt's UB enum transmute (mouse stalled at wheel-detect),
    winlogon's arm64-only layout load (x86 keys dropped at 'NO keyboard
    layout'), arm64input's DO_BUFFERED_IO=0x10 invented constant (every input
    packet memcpy'd to NULL).
    """
    t = spec.get("test", {})
    inp = t.get("input")
    if not inp:
        pytest.skip(f"payload {payload!r} declares no input gate")
    pytest.importorskip("PIL")
    _apply_xfail(spec, arch, request)

    oracle = inp.get("oracle") or {}
    static_max = float(oracle.get("frame_delta_static_max", STATIC_DELTA_MAX))
    oracle_gap = float(oracle.get("gap", 1.5))
    render_min = float((t.get("renders") or {}).get("frame_delta_min", RENDER_DELTA_MIN))

    time.sleep(float(inp.get("settle", 3)))
    if inp.get("focus", "click-center") == "click-center":
        _click_center(vm)
    pre = _frame_delta(vm, gap=oracle_gap)
    assert pre >= render_min, (
        f"pre-input frame delta {pre:.4f} too low — app not animating, "
        "input test would be meaningless"
    )

    post = 1.0
    for attempt in range(int(inp.get("retries", 1))):
        _run_drive(vm, inp.get("drive") or [])
        post = _frame_delta(vm, gap=oracle_gap)
        if post <= static_max:
            break
        print(f"[{payload}] input attempt {attempt + 1}: delta {post:.4f} — retry")
        _run_drive(vm, inp.get("on_retry") or [])

    if post > static_max:
        # Persist a screenshot for postmortem (who actually has foreground?).
        shot = PROJECT_DIR / "qemu" / f"{arch}_{payload}_input_fail.ppm"
        vm.screendump(shot)
        pytest.fail(
            f"post-drive frame delta {post:.4f} > {static_max} — the screen kept "
            f"animating, so the drive keys never reached {payload} "
            f"({inp.get('retries', 1)} attempts). Input chain is broken between "
            f"the class driver and the foreground window (win32k routing/focus). "
            f"Screenshot: {shot}"
        )

    if oracle.get("survives", True):
        status = vm.qmp.execute("query-status")
        assert status["return"]["running"] is True
    print(f"\n[{payload}] PLAYABLE — drive script confirmed (delta {pre:.1%} -> {post:.1%})")


def test_payload_tcp_hammer(vm: QemuVM, arch: str, payload: str, spec: dict[str, Any]) -> None:
    """Load gate: hammer the payload's TCP server from the host through the
    slirp hostfwd and verify every response byte.

    This is the netstack's first real concurrency workout — the self-tests are
    polite single connections; this is N threads × M requests of socket churn
    (accept backlog, afd handle churn, per-connection guest threads, TIME_WAIT
    recycling). Bodies are deterministic (`/data?n=SIZE`, byte i = (i*7+13)&0xff)
    and checksum-verified; `/status` at the end proves the guest's own request
    counter covers every 200 we received (no phantom successes); the OS must
    still be running afterwards.
    """
    import socket
    import threading

    t = spec.get("test", {})
    hammer = t.get("tcp_hammer")
    if not hammer:
        pytest.skip(f"payload {payload!r} declares no tcp_hammer gate")
    _require_net(spec, arch, payload)

    host_port = getattr(vm.config, "payload_hostfwd_port", 0)
    assert host_port, "vm config has no payload_hostfwd_port (net config too old?)"

    # Control probe: the netstack self-test's echo listener (guest :7777) is
    # proven-good on this boot (test_net passes against it). If IT refuses,
    # the failure is host-side hostfwd plumbing, not the payload's listener.
    echo_port = getattr(vm.config, "echo_hostfwd_port", 0)
    print(f"[tcp_hammer] payload host_port={host_port} echo control port={echo_port}")
    if echo_port:
        try:
            with socket.create_connection((vm.net_host, echo_port), timeout=10) as cs:
                cs.sendall(b"hammer-control\n")
                got = cs.recv(64)
            print(f"[tcp_hammer] echo control OK ({got!r})")
        except OSError as e:
            print(f"[tcp_hammer] echo control FAILED: {e} — hostfwd plumbing suspect")

    n_threads = int(hammer.get("threads", 16))
    n_requests = int(hammer.get("requests", 20))
    sizes = [int(x) for x in hammer.get("data_sizes", [1, 512, 4096])]

    def expect_body(n: int) -> bytes:
        return bytes((i * 7 + 13) & 0xFF for i in range(n))

    def http_get(path: str, timeout: float = 20.0) -> tuple[int, bytes]:
        with socket.create_connection((vm.net_host, host_port), timeout=timeout) as s:
            s.sendall(f"GET {path} HTTP/1.0\r\nHost: guest\r\n\r\n".encode())
            buf = b""
            while True:
                chunk = s.recv(65536)
                if not chunk:
                    break
                buf += chunk
        head, sep, body = buf.partition(b"\r\n\r\n")
        assert sep, f"no header terminator in response to {path}: {buf[:200]!r}"
        status = int(head.split(b" ", 2)[1])
        return status, body

    ok = [0]
    errors: list[str] = []
    lock = threading.Lock()

    def worker(tid: int) -> None:
        for i in range(n_requests):
            size = sizes[(tid + i) % len(sizes)]
            path = "/" if i % 4 == 0 else f"/data?n={size}"
            try:
                status, body = http_get(path)
                if status != 200:
                    raise AssertionError(f"{path} -> {status}")
                if path.startswith("/data") and body != expect_body(size):
                    want = expect_body(size)
                    diff_at = next(
                        (j for j in range(min(len(body), size)) if body[j] != want[j]), "len"
                    )
                    raise AssertionError(
                        f"{path} body mismatch: got {len(body)} bytes, first diff at {diff_at}"
                    )
                with lock:
                    ok[0] += 1
            except Exception as e:  # collect everything, assert at the end
                with lock:
                    errors.append(f"t{tid} req{i} {path}: {e}")

    threads = [threading.Thread(target=worker, args=(t_,)) for t_ in range(n_threads)]
    for th in threads:
        th.start()
    for th in threads:
        th.join()

    total = n_threads * n_requests
    assert not errors, (
        f"tcp_hammer: {len(errors)}/{total} requests failed "
        f"({ok[0]} ok). First failures:\n" + "\n".join(errors[:10])
    )

    if hammer.get("status_check", True):
        status, body = http_get("/status")
        assert status == 200, f"/status -> {status}"
        counted = int(body.split(b"requests=")[1].split(b" ")[0])
        assert counted >= ok[0], (
            f"guest counted {counted} requests but host got {ok[0]} 200s — "
            "phantom successes (responses without server-side accounting)"
        )

    # And the OS survived the whole exchange.
    qstatus = vm.qmp.execute("query-status")
    assert qstatus["return"]["running"] is True
    print(f"\n[{payload}] tcp_hammer PASS — {ok[0]}/{total} requests verified, OS alive")
