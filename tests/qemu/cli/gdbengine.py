"""Native-Python GDB engine for the windows-host (and native) QEMU backend.

The bash kernel-debug toolkit (qemu/start.sh + inspect.sh + run-until.sh) is
native-only: it launches a *Linux* qemu, hardcodes `-gdb tcp:127.0.0.1`, and
forces KVM (where software breakpoints don't reliably fire). This module is the
full Python port — it drives the SAME proven gdb-python helpers (qemu/gdb-helpers/
*.py, which are transport-agnostic) but owns the QEMU lifecycle and the
gdb-multiarch orchestration in Python, so it works over the default windows-host
TCG backend (breakpoints fire; QemuVM already emits a gdbstub — see vm.py
_gdb_args / gdb_host).

Model (mirrors the bash toolkit's run-id + manifest so `qemu/ps` etc. interop):
  - `start(arch)` launches a DETACHED QEMU with the gdbstub + serial->file, writes
    qemu/runs/<run-id>/manifest.json, returns a DebugRun. On windows-host the
    QEMU.exe persists past this process (separate Windows process); we address it
    later by run-id and kill it by its uid tag.
  - `wait_for(run, pattern)` polls the serial log file.
  - `run_until(run, brk, timeout, then=[...])` / `inspect(run, verb)` spawn
    `gdb-multiarch -batch`, connect to gdb_host:gdb_port, source the helpers, and
    run the verb — one batch per call (stateless, like inspect.sh).
  - `stop(run)` / `status(run)` manage the run.
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import time
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path

from ..vm import (
    PROJECT_DIR,
    _acpitable_relative,
    console_uart_args,
    get_arch_config,
    smp_count,
    temp_disk_args,
)
from . import archcfg

RUNS_DIR = PROJECT_DIR / "qemu" / "runs"
GDB_HELPERS = PROJECT_DIR / "qemu" / "gdb-helpers"
GDB_BIN = "gdb-multiarch"


def _gdb_bin() -> str:
    """The gdb to drive, or a hard failure that says how to get it.

    Without this, a host lacking gdb-multiarch surfaced as `[Errno 13] Permission denied`
    (the exec search tripping over an unreadable PATH entry), which reads like a sandbox
    problem. Host `gdb` is no substitute: it has no aarch64 support.
    """
    path = shutil.which(GDB_BIN)
    if path is None:
        raise RuntimeError(
            f"{GDB_BIN} not found; install it: sudo apt-get install -y gdb-multiarch"
        )
    return path


# Sourced into every gdb batch, in dependency order (preamble first). Same set the
# bash inspect.sh/run-until.sh source; they're pure gdb-python and transport-agnostic.
HELPER_FILES = [
    "preamble.gdb",
    "find-kbase.py",
    "bt-tagged.py",
    "decode-bugcheck.py",
    "dump-thread.py",
    "dump-trapframe.py",
    "pool-tag.py",
    "identify-object.py",
    "triage-bugcheck.py",
    "walk-waitlist.py",
    "list-threads.py",
    "who-holds.py",  # after list-threads.py + dump-thread.py: uses their offset tables
    "sample-pc.py",
    "run-until-bp.py",
    "lm.py",
    # DWARF-free decoders over VIBEOS_DEBUG_BLOCK (sync-instrumentation §5):
    # vibeos-debug-block / vibeos-synctrace / vibeos-threads / vibeos-flightrec.
    "debug_block.py",
]

# gdb `set architecture` name per base arch (mirror qemu/lib/common.sh arch_gdb_name).
_GDB_ARCH = {"i386": "i386", "amd64": "i386:x86-64", "arm64": "aarch64"}

# --then / inspect verb -> the gdb command the helpers register (mirror run-until.sh
# expand_then + inspect.sh). `{rest}` is the forwarded argument tail.
_VERB_MAP = {
    "bt": "bt-tagged {rest}",
    "bt-all-cpus": "bt-tagged {rest}",
    "regs": "info registers",
    "decode-bugcheck": "decode-bugcheck",
    "triage-bugcheck": "triage-bugcheck",
    "walk-waitlist": "walk-waitlist {rest}",
    "dump-thread": "dump-thread {rest}",
    "dump-trapframe": "dump-trapframe {rest}",
    "pool-tag": "pool-tag {rest}",
    "identify-object": "identify-object {rest}",
    "list-threads": "list-threads {rest}",
    "who-holds": "who-holds {rest}",
    "stack-scan": "stack-scan {rest}",
    "sample-pc": "sample-pc {rest}",
    "symbolicate-pc": "info symbol {rest}",
    "regs-raw": "info registers {rest}",
    "lm": "lm {rest}",
}


def _gdb_arch(arch: str) -> str:
    return _GDB_ARCH.get(arch, arch)


def _kernel_image(arch: str) -> Path:
    arch = archcfg.base_arch(arch)
    kernel_dir = PROJECT_DIR / f"build-{arch}" / "ntoskrnl"
    # Every media boots the MP kernel by default (DefaultOS=LiveCD_SMP / ReactOS_Arm64_MP since
    # 2026-09-12), with or without VIBEOS_QEMU_SMP. Naming the UP image here loaded its DWARF at
    # the MP kernel's base, so every symbol and struct offset in a gdb session was wrong.
    mp = kernel_dir / "ntkrnlmp" / "ntkrnlmp.exe"
    if mp.is_file():
        return mp
    return kernel_dir / "ntoskrnl.exe"


def _now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


@dataclass
class DebugRun:
    run_id: str
    arch: str  # base arch (i386/amd64/arm64) — for gdb arch + kernel image
    backend: str
    gdb_host: str
    gdb_port: int
    serial_log: str  # Linux-readable path (/mnt/... on windows-host) of the serial file
    kernel_image: str
    win_uid: str = ""  # windows-host: -name tag for be.kill_by_uid
    qemu_pid: int = 0  # native: the local qemu pid
    staging: str = ""
    start_time: str = ""
    kernel_runtime_base: str = ""  # cached kbase (filled by run_until on first find)
    accel: str = "tcg"  # resolved accelerator; an i386 guest under whpx runs on
    # qemu-system-x86_64.exe whose gdbstub speaks i386:x86-64 (see _run_gdb)

    # ---- persistence ----
    @property
    def dir(self) -> Path:
        return RUNS_DIR / self.run_id

    def manifest_path(self) -> Path:
        return self.dir / "manifest.json"

    def save(self) -> None:
        self.dir.mkdir(parents=True, exist_ok=True)
        self.manifest_path().write_text(json.dumps(asdict(self), indent=2))

    @classmethod
    def load(cls, run_id: str) -> DebugRun:
        p = RUNS_DIR / run_id / "manifest.json"
        if not p.is_file():
            raise FileNotFoundError(f"no debug run '{run_id}' (looked at {p})")
        data = json.loads(p.read_text())
        # qemu/runs/ is shared with the bash toolkit, whose manifests have a different
        # schema. Keep only our fields; a manifest missing our required ones (i.e. a
        # bash-toolkit run) raises ValueError so callers can skip it.
        fields = {k: data[k] for k in cls.__dataclass_fields__ if k in data}
        try:
            return cls(**fields)
        except TypeError as e:
            raise ValueError(f"'{run_id}' is not a gdbengine run manifest ({e})") from e


def _win_alive(uid: str) -> bool:
    """Is the detached Windows QEMU with this -name uid still running? Liveness MUST
    be checked this way, NOT by connecting to the gdbstub: opening a TCP connection to
    QEMU's gdbstub HALTS the vCPUs (it's waiting for the gdb client). A connect-probe
    therefore freezes the guest mid-boot — which is exactly what wedged early runs
    (serial stuck at 0, PC parked in OVMF). Only inspect()/run_until() may connect."""
    import re

    ps = (
        "@(Get-CimInstance Win32_Process -Filter \"Name like 'qemu-system-%'\" "
        f"| Where-Object {{ $_.CommandLine -like '*{uid}*' }}).Count"
    )
    try:
        out = subprocess.run(
            ["powershell.exe", "-NoProfile", "-Command", ps],
            capture_output=True,
            text=True,
            timeout=15,
        ).stdout
        return int(re.sub(r"\D", "", out) or "0") > 0
    except (OSError, subprocess.SubprocessError, ValueError):
        return False


def _tripwire_fw_cfg() -> list[str]:
    """VIBEOS_TRIPWIRE for a debug boot, as smoke passes it (vm.QemuVM._tripwire_args).
    Without it a special-pool fault seen in smoke could not be caught under gdb."""
    cfg = os.environ.get("VIBEOS_TRIPWIRE", "").strip()
    if not cfg:
        return []
    if "," in cfg:
        raise ValueError(f"VIBEOS_TRIPWIRE must not contain commas (got {cfg!r})")
    return ["-fw_cfg", f"name=opt/org.vibeos.tripwire,string={cfg}"]


def _build_windows_cmd(
    arch: str, config, win_cfg: dict, work_dir: Path, staging: Path, *, gdb_wait: bool
) -> tuple[list[str], dict]:
    """Assemble the DETACHED windows-host qemu cmd: gdbstub bound 0.0.0.0 on the
    Windows stack, serial -> a staged file (readable across CLI invocations), media
    staged + accel rewritten. Mirrors vm._build_windows_cmd but swaps the TCP dial-
    back serial/QMP (which can't outlive the launching process) for -serial file:."""
    from .. import backend as be

    is_arm64 = arch.startswith("arm64")
    configured = win_cfg.get("accel", "tcg")
    whpx_ok = be.whpx_available() if (configured == "whpx" and not is_arm64) else None
    accel = be.resolve_accel(arch, configured, whpx_ok)

    scratch = staging / "scratch.qcow2"
    subprocess.run(
        ["qemu-img", "create", "-f", "qcow2", str(scratch), "64M"],
        check=True,
        capture_output=True,
    )
    serial_log = staging / "serial.log"
    serial_log.write_bytes(b"")  # pre-create so wait_for can poll immediately
    debugcon = staging / "debugcon.bin"
    gdb_port = be.windows_free_tcp_port()
    uid = f"vibeos-dbg-{os.getpid()}-{gdb_port}"
    win_bin = be.windows_qemu_bin(config.qemu_bin, win_cfg["qemu_dir"], accel)

    raw = config.qemu_args(work_dir)
    staged = _acpitable_relative(be.stage_media_paths(raw, staging))
    arch_args = staged if is_arm64 else be.rewrite_accel(staged, accel, arch)

    scratch_dev = [
        "-drive",
        f"file={scratch},format=qcow2,if=none,id=scratch",
        *(
            []
            if arch.startswith("i386")
            else [
                "-device",
                "virtio-blk-device,drive=scratch" if is_arm64 else "virtio-blk-pci,drive=scratch",
            ]
        ),
    ]
    scratch_dev = be.stage_media_paths(scratch_dev, staging)
    # The same \\TEMP disk smoke attaches: a debug boot that differs from the smoke
    # it is reproducing cannot reproduce a disk-path crash at all.
    temp_dev = be.stage_media_paths(temp_disk_args(arch, staging), staging)
    smp = smp_count()

    cmd = [
        win_bin,
        "-name",
        uid,
        "-chardev",
        f"file,id=dbgser,path={be.win_path(serial_log)}",
        *console_uart_args(getattr(config, "arch", arch), "dbgser"),
        "-monitor",
        "none",
        "-gdb",
        f"tcp:0.0.0.0:{gdb_port}",
        *(["-S"] if gdb_wait else []),
        *([] if is_arm64 else ["-debugcon", f"file:{be.win_path(debugcon)}"]),
        *scratch_dev,
        *temp_dev,
        *(["-smp", str(smp)] if smp is not None else []),
        *_tripwire_fw_cfg(),
        *arch_args,
    ]
    meta = {
        "gdb_host": be.gateway_ip(),
        "gdb_port": gdb_port,
        "serial_log": str(serial_log),
        "win_uid": uid,
        "staging": str(staging),
        "accel": accel,
    }
    return cmd, meta


def start(
    arch: str,
    *,
    backend: str,
    win_cfg: dict,
    gdb_wait: bool = False,
) -> DebugRun:
    from ..portlease import launch_lock

    with launch_lock():
        return _start_locked(arch, backend=backend, win_cfg=win_cfg, gdb_wait=gdb_wait)


def _start_locked(
    arch: str,
    *,
    backend: str,
    win_cfg: dict,
    gdb_wait: bool = False,
) -> DebugRun:
    """Launch a detached debug QEMU with a gdbstub; write + return the run manifest.
    windows-host only for now (the whole point — native breakpoints don't fire under
    KVM; the native port reuses this shell once its accel is forced to tcg)."""
    if backend != "windows-host":
        raise NotImplementedError(
            f"gdbengine backend '{backend}' not supported yet (windows-host only); "
            "for native use the bash toolkit (qemu/start.sh) until it is ported."
        )
    vm_arch = archcfg.vm_arch(arch)
    config = get_arch_config(vm_arch)
    run_id = f"{arch}-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}"
    run_dir = RUNS_DIR / run_id
    if run_dir.exists():
        run_id = f"{run_id}-{os.getpid()}"
        run_dir = RUNS_DIR / run_id
    work_dir = run_dir / "work"
    work_dir.mkdir(parents=True, exist_ok=True)

    staging_root = Path(win_cfg.get("staging_dir", "/mnt/c/temp/vibeos-qemu"))
    from ..windows_staging import auto_cleanup_locked

    auto_cleanup_locked(staging_root)
    staging = staging_root / f"dbg-{run_id}"
    shutil.rmtree(staging, ignore_errors=True)
    staging.mkdir(parents=True, exist_ok=True)

    cmd, meta = _build_windows_cmd(arch, config, win_cfg, work_dir, staging, gdb_wait=gdb_wait)

    launch_log = (run_dir / "launch.log").open("wb")
    proc = subprocess.Popen(
        cmd,
        stdout=launch_log,
        stderr=subprocess.STDOUT,
        stdin=subprocess.DEVNULL,
        start_new_session=True,  # detach: survives this CLI process exiting
        cwd=staging,  # `-acpitable` names its file relative to here (vm._acpitable_relative)
    )
    run = DebugRun(
        run_id=run_id,
        arch=arch,
        backend=backend,
        gdb_host=meta["gdb_host"],
        gdb_port=meta["gdb_port"],
        serial_log=meta["serial_log"],
        kernel_image=str(_kernel_image(arch)),
        win_uid=meta["win_uid"],
        qemu_pid=proc.pid,
        staging=meta["staging"],
        start_time=_now_iso(),
        accel=meta.get("accel", "tcg"),
    )
    run.save()
    (run_dir / "qemu_cmd.txt").write_text(" ".join(cmd))
    return run


def wait_for(run: DebugRun, pattern: str, timeout: float = 120.0) -> bool:
    """Poll the serial log for a regex, up to `timeout`s. Returns True on match."""
    import re

    rx = re.compile(pattern)
    log = Path(run.serial_log)
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            text = log.read_text(errors="replace")
        except OSError:
            text = ""
        if rx.search(text):
            return True
        time.sleep(0.5)
    return False


def _helper_source_args() -> list[str]:
    out: list[str] = []
    for h in HELPER_FILES:
        p = GDB_HELPERS / h
        if p.is_file():
            out += ["-ex", f"source {p}"]
    return out


def _run_gdb(run: DebugRun, verb_ex: list[str], *, timeout: float, relocate: bool = True) -> str:
    """Core: one `gdb-multiarch -batch` invocation — connect, source helpers, find
    kbase (relocate symbols), then the caller's verb -ex commands, always detach+quit.
    Returns combined stdout/stderr. The host timeout is a hard safety net."""
    # An i386 guest under whpx runs on the 64-bit emulator (whpx lives only in
    # qemu-system-x86_64.exe) — its gdbstub reports i386:x86-64 and sends 64-bit
    # 'g' packets, so the i386 arch name would fail every register read.
    gdb_arch = (
        "i386:x86-64" if (run.arch == "i386" and run.accel == "whpx") else _gdb_arch(run.arch)
    )
    ex = [
        "-ex",
        f"set architecture {gdb_arch}",
        "-ex",
        f"file {run.kernel_image}",
        "-ex",
        f"target remote {run.gdb_host}:{run.gdb_port}",
        # target remote may leave vCPUs running; force a stop so break/scan are stable.
        "-ex",
        "interrupt",
        *_helper_source_args(),
    ]
    if relocate:
        kbase = run.kernel_runtime_base
        ex += ["-ex", f"find-kbase {kbase}" if kbase else "find-kbase"]
    ex += verb_ex
    ex += ["-ex", "delete breakpoints", "-ex", "detach", "-ex", "quit"]

    env = {**os.environ, "VIBEOS_ARCH": run.arch, "VIBEOS_KERNEL": run.kernel_image}
    try:
        cp = subprocess.run(
            [_gdb_bin(), "-batch", "-nx", *ex],
            capture_output=True,
            text=True,
            timeout=timeout,
            env=env,
        )
        return cp.stdout + cp.stderr
    except subprocess.TimeoutExpired as e:
        out = (e.stdout or b"") if isinstance(e.stdout, bytes) else (e.stdout or "")
        if isinstance(out, bytes):
            out = out.decode(errors="replace")
        return f"{out}\n=== gdb hard-timeout after {timeout}s ==="


def _expand_verb(token: str) -> str:
    """Map a verb token ('bt 50', 'triage-bugcheck', 'raw <gdb cmd>') to a gdb command."""
    head, _, rest = token.partition(" ")
    if head == "raw":
        return rest
    tmpl = _VERB_MAP.get(head)
    if tmpl is None:
        raise ValueError(f"unknown verb: {head}")
    return tmpl.format(rest=rest).rstrip()


_KRNLBASE = r"\[KRNLBASE\] ntoskrnl runtime base=(?:0x)?([0-9a-fA-F]+)"


def _await_kernel_base(run: DebugRun, timeout: float) -> bool:
    """Cache the kernel's runtime base from its own [KRNLBASE] line, letting the guest
    boot until it prints one.

    A symbol breakpoint needs the kernel relocated, and find-kbase can only scan for an
    image that is already mapped. A run fresh from `gdb start` is parked at reset, so the
    scan found nothing, symbols stayed at the PE link address, and `--break KeBugCheckEx`
    armed 0x49f942 — an address the guest never executes, so it silently never fired.
    The guest prints its base (ARM3/sysldr.rs) on every arch before any driver loads.
    """
    import re

    if run.kernel_runtime_base:
        return True
    rx = re.compile(_KRNLBASE)
    log = Path(run.serial_log)

    def base() -> str:
        try:
            m = rx.findall(log.read_text(errors="replace"))
        except OSError:
            return ""
        return hex(int(m[-1], 16)) if m else ""

    found = base()
    if not found:
        # Connecting halts the vCPUs (and a `-S` start never ran them); detaching
        # resumes both, so the boot proceeds while the serial log is polled.
        _run_gdb(run, [], timeout=60, relocate=False)
        if wait_for(run, _KRNLBASE, timeout=timeout):
            found = base()
    if not found:
        return False
    run.kernel_runtime_base = found
    run.save()
    return True


def _decorated(run: DebugRun, brk: str) -> str:
    """The breakpoint spec for `brk`. i386 PE symbols are decorated (`_KeBugCheckEx@20`) and
    gdb does not load them as minimal symbols, while the Rust DWARF names the function
    `ntoskrnl_rust::ke::bug::KeBugCheckEx` — so a bare `--break KeBugCheckEx` was "not
    defined" and never armed. Resolve it from the image's own table instead and break on
    the relocated address (file VA - PE ImageBase + runtime base)."""
    import re

    if not run.arch.startswith("i386") or not re.fullmatch(r"[A-Za-z_]\w*", brk):
        return brk
    nm = PROJECT_DIR / ".build-tools/llvm-mingw/bin/llvm-nm"
    try:
        out = subprocess.run(
            [str(nm), run.kernel_image], capture_output=True, text=True, timeout=60
        ).stdout
    except (OSError, subprocess.SubprocessError):
        return brk
    text = {}
    for ln in out.splitlines():
        parts = ln.split()
        if len(parts) == 3 and parts[1] in ("T", "t"):
            text[parts[2]] = int(parts[0], 16)
    exact = [f"_{brk}", *sorted(n for n in text if n.startswith(f"_{brk}@"))]
    hit = next((n for n in exact if n in text), None)
    if hit is None or not run.kernel_runtime_base:
        return brk
    return f"*{text[hit] - _PE_IMAGE_BASE + int(run.kernel_runtime_base, 16):#x}"


_PE_IMAGE_BASE = 0x400000


def run_until(
    run: DebugRun, brk: str, *, timeout: float = 180.0, then: list[str] | None = None
) -> str:
    """Set a breakpoint, bounded-continue until it hits (or the in-gdb budget lapses),
    then run each `then` verb against the stopped target. Mirrors run-until.sh."""
    if not _await_kernel_base(run, timeout):
        return (
            "=== BREAKPOINT NOT SET: the kernel never printed [KRNLBASE] within "
            f"{timeout:.0f}s, so a symbol would resolve to its unrelocated link address ==="
        )
    verb_ex = ["-ex", f"break {_decorated(run, brk)}", "-ex", f"bounded-continue {timeout}"]
    for t in then or []:
        if not t:
            continue
        verb_ex += ["-ex", f"echo --- {t} ---\\n", "-ex", _expand_verb(t)]
    # host net = the in-gdb budget + teardown headroom
    return _run_gdb(run, verb_ex, timeout=timeout + 30)


def inspect(run: DebugRun, verb: str, *, timeout: float = 60.0) -> str:
    """Run a single inspection verb against the (already stopped/running) target."""
    return _run_gdb(run, ["-ex", _expand_verb(verb)], timeout=timeout)


def probe(run: DebugRun, script: Path, *, timeout: float) -> dict:
    """Run a GDB Python probe from reset, without scanning unloaded kernel memory.

    The script must set VIBEOS_PROBE_RESULT = {"hit": bool, "checks": positive int}.
    Missing coverage or a Python exception is an error, never a negative result.
    """
    result_path = run.dir / "probe-result.json"
    saved_script = run.dir / "probe.py"
    shutil.copy2(script, saved_script)
    # Execute inside one GDB Python command: `source` alone reports Python errors
    # while batch GDB can still exit zero. The result file is our success contract.
    bootstrap = (
        "import json, pathlib\n"
        f"scope = {{'VIBEOS_PROBE_DIR': {str(run.dir)!r}, 'VIBEOS_PROBE_TIMEOUT': {timeout!r}}}\n"
        f"path = pathlib.Path({str(saved_script)!r})\n"
        "exec(compile(path.read_text(), str(path), 'exec'), scope)\n"
        "result = scope['VIBEOS_PROBE_RESULT']\n"
        f"pathlib.Path({str(result_path)!r}).write_text(json.dumps(result))\n"
    )
    gdb_arch = (
        "i386:x86-64"
        if run.arch.startswith("i386") and run.accel == "whpx"
        else _gdb_arch(archcfg.base_arch(run.arch))
    )
    command = [
        _gdb_bin(),
        "-q",
        "-nx",
        "-batch",
        "-ex",
        f"set architecture {gdb_arch}",
        "-ex",
        f"target remote {run.gdb_host}:{run.gdb_port}",
        "-ex",
        f"source {GDB_HELPERS / 'run-until-bp.py'}",
        "-ex",
        f"python exec({bootstrap!r})",
        "-ex",
        "delete breakpoints",
        "-ex",
        "detach",
    ]
    (run.dir / "probe-command.json").write_text(json.dumps(command, indent=2))
    with (run.dir / "probe.log").open("w") as log:
        cp = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, timeout=timeout + 30)
    if cp.returncode != 0 or not result_path.is_file():
        raise RuntimeError(f"probe failed (gdb exit {cp.returncode}); read {run.dir / 'probe.log'}")
    result = json.loads(result_path.read_text())
    if (
        not isinstance(result, dict)
        or type(result.get("hit")) is not bool
        or type(result.get("checks")) is not int
        or result["checks"] <= 0
    ):
        raise RuntimeError(f"probe did not establish coverage: {result!r}")
    return result


def cmd_probe(args, repo: Path) -> int:
    script = Path(args.script).resolve()
    if not script.is_file() or args.repeat < 1 or args.timeout <= 0:
        raise ValueError("probe needs an existing Python script, positive repeat and timeout")
    backend, win_cfg = _cfg(repo)
    kernel = _kernel_image(args.arch)
    kernel_hash = hashlib.sha256(kernel.read_bytes()).hexdigest()
    media = repo / archcfg.arch_build(args.arch).boot_media_rel
    media_stat = media.stat()
    media_identity = (media_stat.st_size, media_stat.st_mtime_ns)
    for i in range(args.repeat):
        if hashlib.sha256(kernel.read_bytes()).hexdigest() != kernel_hash:
            raise RuntimeError(
                "kernel changed during probe soak; stop building and restart the probe"
            )
        current_stat = media.stat()
        if (current_stat.st_size, current_stat.st_mtime_ns) != media_identity:
            raise RuntimeError("boot media changed during probe soak; restart the probe")
        run = start(args.arch, backend=backend, win_cfg=win_cfg, gdb_wait=True)
        verdict = {"round": i + 1, "kernel_sha256": kernel_hash, "verdict": "PROBE_ERROR"}
        try:
            result = probe(run, script, timeout=args.timeout)
            verdict.update(result)
            verdict["verdict"] = "HIT" if result["hit"] else "NO_HIT"
        except (OSError, ValueError, RuntimeError, subprocess.TimeoutExpired) as exc:
            verdict["error"] = str(exc)
        except KeyboardInterrupt:
            verdict["verdict"] = "CANCELLED"
            verdict["error"] = "interrupted by caller"
        finally:
            stop(run)
            (run.dir / "probe-summary.json").write_text(json.dumps(verdict, indent=2))
        print(f"probe {i + 1}/{args.repeat}: {verdict['verdict']} {run.dir}", flush=True)
        if verdict["verdict"] == "PROBE_ERROR":
            print(verdict.get("error", "unknown probe error"), flush=True)
            return 2
        if verdict["verdict"] == "CANCELLED":
            return 130
        if verdict["verdict"] == "HIT":
            return 0
    return 1


def stop(run: DebugRun) -> None:
    """Kill the debug QEMU and mark the run stopped."""
    # A VM kept by `smoke --attach-on-fail` is also registered as a lock (so `ps`,
    # `kill` and the reaper see it); unregister it with the VM, or `ps` lists a dead VM
    # until its deadline. Read it FIRST: run.save() below rewrites the manifest with the
    # dataclass fields only, which drops "lock".
    mp = run.manifest_path()
    lock = json.loads(mp.read_text()).get("lock") if mp.is_file() else None
    if run.backend == "windows-host" and run.win_uid:
        from .. import backend as be

        # Unconfirmed, leave the staging and lock in place: they are what lets a later
        # `gdb stop`, `kill --stale` or the reaper find this qemu.exe again.
        left = be.kill_by_uid(run.win_uid)
        if left != 0:
            raise RuntimeError(
                f"could not confirm {run.win_uid} stopped "
                f"({'PowerShell gave no answer' if left is None else f'{left} still running'}); "
                f"run left registered -- retry `vibeos-cli gdb stop {run.run_id}`"
            )
    # Serial lives inside staging on windows-host. Preserve the experiment's
    # evidence before deleting that directory; a stopped run must stay readable.
    serial = Path(run.serial_log)
    if serial.is_file():
        archived = run.dir / "serial.log"
        run.dir.mkdir(parents=True, exist_ok=True)
        if serial.resolve() != archived.resolve():
            shutil.copy2(serial, archived)
        run.serial_log = str(archived)
        run.save()
    if run.staging:
        shutil.rmtree(run.staging, ignore_errors=True)
    if lock:
        Path(lock).unlink(missing_ok=True)


def status(run: DebugRun, *, check_alive: bool = True) -> dict:
    """Liveness snapshot. Liveness = the Windows QEMU process being alive + serial-log
    growth. It deliberately does NOT probe the gdbstub — connecting to it halts the
    guest (see _win_alive). Pass check_alive=False to skip the (PowerShell) process
    query when polling tightly for serial growth."""
    log = Path(run.serial_log)
    out = {
        "run_id": run.run_id,
        "arch": run.arch,
        "backend": run.backend,
        "gdb": f"{run.gdb_host}:{run.gdb_port}",
        "serial_bytes": log.stat().st_size if log.is_file() else 0,
        "serial_log": run.serial_log,
    }
    if check_alive:
        out["alive"] = _win_alive(run.win_uid) if run.win_uid else False
    return out


def list_runs() -> list[str]:
    if not RUNS_DIR.is_dir():
        return []
    return sorted(p.name for p in RUNS_DIR.iterdir() if (p / "manifest.json").is_file())


# --- CLI handlers (`vibeos-cli gdb <verb>`) --------------------------------


def _cfg(repo: Path):
    from . import config as cli_config

    cli_cfg = cli_config.load(repo)
    return cli_cfg.get("qemu.backend", "native"), cli_cfg.get("qemu.windows", {})


def cmd_start(args, repo: Path) -> int:
    backend, win_cfg = _cfg(repo)
    if getattr(args, "tripwire", ""):
        os.environ["VIBEOS_TRIPWIRE"] = args.tripwire
    run = start(args.arch, backend=backend, win_cfg=win_cfg, gdb_wait=args.wait)
    print(f"run-id : {run.run_id}")
    print(f"gdb    : {run.gdb_host}:{run.gdb_port}  (gdb-multiarch target remote)")
    print(f"serial : {run.serial_log}")
    print(f"kernel : {run.kernel_image}")
    print(f"\nnext: vibeos-cli gdb wait-for {run.run_id} --pattern '<marker>'")
    print(f"      vibeos-cli gdb run-until {run.run_id} --break KeBugCheckEx")
    print(f"      vibeos-cli gdb stop {run.run_id}")
    return 0


def cmd_status(args, repo: Path) -> int:
    run = DebugRun.load(args.run_id)
    print(json.dumps(status(run), indent=2))
    return 0


def cmd_wait_for(args, repo: Path) -> int:
    run = DebugRun.load(args.run_id)
    hit = wait_for(run, args.pattern, timeout=args.timeout)
    print(f"{'HIT' if hit else 'MISS'}: {args.pattern!r} (timeout={args.timeout}s)")
    return 0 if hit else 1


def cmd_run_until(args, repo: Path) -> int:
    run = DebugRun.load(args.run_id)
    out = run_until(run, args.brk, timeout=args.timeout, then=args.then or [])
    print(out)
    return 0 if "BREAKPOINT HIT" in out else 1


def cmd_inspect(args, repo: Path) -> int:
    run = DebugRun.load(args.run_id)
    print(inspect(run, " ".join(args.verb), timeout=args.timeout))
    return 0


def cmd_stop(args, repo: Path) -> int:
    run = DebugRun.load(args.run_id)
    try:
        stop(run)
    except RuntimeError as e:
        print(f"NOT stopped {run.run_id}: {e}")
        return 1
    print(f"stopped {run.run_id} (uid={run.win_uid})")
    return 0


def cmd_ps(args, repo: Path) -> int:
    for rid in list_runs():
        try:
            st = status(DebugRun.load(rid))
        except (OSError, ValueError):
            continue
        live = "up" if st.get("alive") else "down"
        print(f"{rid:32} {live:5} gdb={st['gdb']:24} serial={st['serial_bytes']}B")
    return 0
