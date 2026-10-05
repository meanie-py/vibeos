"""Find the first point where a replay's guest state leaves its recording's.

A replay that wedges has usually diverged much EARLIER, silently: QEMU only
notices when the diverged guest takes an exception the log does not contain,
and then it waits forever. The wedge marks where the damage became visible,
not where it happened (docs/engineering/qemu-record-replay.md, "Determinism
findings"). So the useful question is not "what is at the wedge" but "what is
the last instruction count at which record and replay still agreed".

That needs state from BOTH sides at the SAME icount, and the recording is the
only chance to capture its half. So:

  record  `rr record --sample SECS` pauses the recording every SECS of wall
          time, reads the exact icount, and writes a fingerprint of the guest
          (every vCPU's registers + a hash per MiB of guest RAM) to
          samples.jsonl. Pausing is free to the guest: under icount its clock
          is the instruction counter, which does not advance while stopped.
  replay  `rr diverge` replays with a `replay-break` at each sampled icount,
          fingerprints the guest there, and stops at the first mismatch. It
          reports the last matching and first mismatching icount, the register
          lines that differ, and which MiB of physical RAM differ.

The fingerprints are taken over a SECOND QMP socket owned by this module, so the
boot harness's own QMP connection is never shared between threads.
"""

from __future__ import annotations

import difflib
import hashlib
import itertools
import json
import os
import re
import sys
import threading
import time
import zlib
from pathlib import Path

from ..qmp_client import QmpClient, QmpError

SAMPLE_ENV_VAR = "VIBEOS_QEMU_RR_SAMPLE"
SAMPLES_FILE = "samples.jsonl"
VERDICT_FILE = "diverge.json"
# Content-addressed RAM: every distinct MiB either side has seen, zlib'd, named by
# its hash. Hashes alone say THAT memory differs; a verdict that cannot show the
# bytes cannot tell a real divergence from a DMA buffer filled at a different host
# moment. Deduplication keeps it small: most of RAM does not change between samples.
RAM_STORE = "ram"

CHUNK = 1 << 20  # one hash per MiB: the granularity the verdict names RAM at

# x86 legacy hole: VGA memory and option ROMs. pmemsave reads through the memory
# API, so it would call device read handlers here, not just copy RAM.
_X86_HOLE = (0xA0000, 0x100000)
# arm64 `-M virt` places RAM at 1 GiB.
_ARM64_RAM_BASE = 0x40000000


def env_sample() -> str | None:
    """`<seconds>` on a recording, `check` on a replay, or None."""
    raw = os.environ.get(SAMPLE_ENV_VAR, "").strip()
    return raw or None


def _ram_bytes(argv: list[str]) -> int:
    try:
        spec = argv[argv.index("-m") + 1]
    except (ValueError, IndexError) as exc:
        raise RuntimeError("no -m in the QEMU argv; cannot size guest RAM") from exc
    spec = spec.split(",")[0].removeprefix("size=")
    m = re.fullmatch(r"(\d+)([KMGT]?)", spec, re.IGNORECASE)
    if not m:
        raise RuntimeError(f"cannot parse -m {spec!r}")
    scale = {"": 1 << 20, "K": 1 << 10, "M": 1 << 20, "G": 1 << 30, "T": 1 << 40}
    return int(m.group(1)) * scale[m.group(2).upper()]


def ram_ranges(argv: list[str]) -> list[tuple[int, int]]:
    """(base, length) of guest RAM that is safe to read with pmemsave."""
    ram = _ram_bytes(argv)
    if "aarch64" in argv[0]:
        return [(_ARM64_RAM_BASE, ram)]
    # x86: RAM is contiguous from 0 below the PCI hole as long as it fits in the
    # low 2 GiB, which every VM this harness launches does. Refuse rather than
    # silently hash a partial picture if that ever stops being true.
    if ram > 0x80000000:
        raise RuntimeError(f"x86 guest RAM {ram:#x} spans the PCI hole; ranges not modelled")
    lo, hi = _X86_HOLE
    return [(0, lo), (hi, ram - hi)]


class _Fingerprinter:
    """Owns the second QMP connection and takes fingerprints over it."""

    def __init__(
        self, sock_path: Path, ranges: list[tuple[int, int]], work: Path, store: Path
    ) -> None:
        self.store = store
        store.mkdir(exist_ok=True)
        self.sock_path = sock_path
        self.ranges = ranges
        self.work = work
        self.qmp = QmpClient()
        self._seq = itertools.count()

    def connect(self, timeout: float = 60.0) -> None:
        deadline = time.monotonic() + timeout
        while True:
            try:
                self.qmp.connect(str(self.sock_path), timeout=30.0)
                return
            except OSError:
                if time.monotonic() > deadline:
                    raise
                time.sleep(0.2)

    def icount(self) -> int:
        return int(self.qmp.execute("query-replay")["return"]["icount"])

    def _keep(self, chunk: bytes) -> str:
        h = hashlib.blake2b(chunk, digest_size=8).hexdigest()
        path = self.store / f"{h}.z"
        if not path.exists():
            tmp = path.with_suffix(".tmp")
            tmp.write_bytes(zlib.compress(chunk, 1))
            tmp.replace(path)
        return h

    def _hash_range(self, base: int, length: int) -> list[str]:
        fifo = self.work / f"rrmem-{os.getpid()}-{next(self._seq)}.fifo"
        os.mkfifo(fifo)
        out: list[str] = []
        err: list[BaseException] = []

        def _reader() -> None:
            try:
                with open(fifo, "rb", buffering=0) as fh:
                    buf = bytearray()
                    while True:
                        b = fh.read(CHUNK)
                        if not b:
                            break
                        buf += b
                        while len(buf) >= CHUNK:
                            out.append(self._keep(bytes(buf[:CHUNK])))
                            del buf[:CHUNK]
                    if buf:
                        out.append(self._keep(bytes(buf)))
            except BaseException as exc:  # surfaced below, never swallowed
                err.append(exc)

        t = threading.Thread(target=_reader, daemon=True)
        t.start()
        try:
            self.qmp.execute("pmemsave", val=base, size=length, filename=str(fifo))
        except BaseException:
            # QEMU never opened the fifo: open the write end ourselves so the
            # reader's open() returns and the thread ends.
            with open(fifo, "wb"):
                pass
            raise
        finally:
            t.join(timeout=120)
            fifo.unlink(missing_ok=True)
        if err:
            raise err[0]
        want = (length + CHUNK - 1) // CHUNK
        if len(out) != want:
            raise RuntimeError(f"pmemsave {base:#x}+{length:#x}: {len(out)} chunks, wanted {want}")
        return out

    def fingerprint(self) -> dict:
        regs = self.qmp.hmp("info registers -a")
        mem = []
        for base, length in self.ranges:
            mem.append({"base": base, "hashes": self._hash_range(base, length)})
        return {"icount": self.icount(), "regs": regs, "mem": mem}

    def wait_stop_or_stall(self, window: float = 30.0, stall_windows: int = 4) -> bool:
        """True once the guest STOPs; False if the icount stops moving first.

        A wedged replay never fires its break; it sits with the vCPU spinning and
        the icount frozen. Waiting on STOP alone would hold until the harness's
        boot timeout, so watch the icount too.
        """
        last, still = -1, 0
        while True:
            if self.qmp.wait_event("STOP", timeout=window) is not None:
                return True
            now = self.icount()
            if self.qmp.saw_event("STOP") is not None:
                return True
            still = still + 1 if now == last else 0
            if still >= stall_windows:
                return False
            last = now

    def close(self) -> None:
        self.qmp.close()


def compare(rec: dict, rep: dict, store: Path | None = None) -> dict | None:
    """None when the two fingerprints agree, else what differs."""
    diff: dict = {}
    if rec["icount"] != rep["icount"]:
        diff["icount"] = [rec["icount"], rep["icount"]]
    if rec["regs"] != rep["regs"]:
        diff["regs"] = [
            line
            for line in difflib.unified_diff(
                rec["regs"].splitlines(),
                rep["regs"].splitlines(),
                "record",
                "replay",
                n=0,
                lineterm="",
            )
            if not line.startswith("@@")
        ]
    bad = []
    for a, b in zip(rec["mem"], rep["mem"], strict=True):
        for i, (ha, hb) in enumerate(zip(a["hashes"], b["hashes"], strict=True)):
            if ha != hb:
                bad.append((a["base"] + i * CHUNK, ha, hb))
    if bad:
        diff["mem_mib"] = [f"{x:#x}" for x, _, _ in bad]
        if store is not None:
            diff["mem_bytes"] = [byte_diff(store, base, ha, hb) for base, ha, hb in bad[:16]]
    return diff or None


EVENT_INTERRUPT = 1  # QEMU v12 log id
_REG = re.compile(r"\b([A-Z][A-Z0-9_]*)=([0-9a-fA-F]+)\b")


def _regs(text: str) -> dict[str, str]:
    return dict(_REG.findall(text))


def artifact(
    d: dict, io_at_pause: bool, irq_at_pause: bool, rec_regs: str, rep_regs: str
) -> str | None:
    """Why a mismatch is the SAMPLER's doing rather than the guest's, else None.

    Two shapes, both measured, both only possible when the trace logs an event at
    exactly the sampled icount — so a mismatch at an icount with no such event is
    always reported as real:

    * I/O: QMP `stop` drains block I/O, so the recording's pause can complete a
      request and log that completion AT the pause; its fingerprint then shows the
      device's writes (a virtqueue index one ahead, a flag flipped) that the
      replay, broken before the logged completion is applied, does not have yet.
      Registers equal, RAM differs.
    * IRQ entry: an interrupt logged at the sampled icount is taken without
      executing an instruction, and one side fingerprinted before entry and the
      other after. RAM equal; only PC and PSTATE differ (PC at a vector).
    """
    if "icount" in d:
        return None
    if io_at_pause and "regs" not in d:
        return "record pause completed I/O"
    if irq_at_pause and "mem_mib" not in d:
        a, b = _regs(rec_regs), _regs(rep_regs)
        if {k for k in a.keys() | b.keys() if a.get(k) != b.get(k)} <= {"PC", "PSTATE"}:
            return "interrupt entry on one side of the pause only"
    return None


def _load(store: Path, h: str) -> bytes | None:
    p = store / f"{h}.z"
    return zlib.decompress(p.read_bytes()) if p.is_file() else None


def byte_diff(store: Path, base: int, rec_hash: str, rep_hash: str, runs: int = 24) -> dict:
    """Which bytes of one MiB differ, as physical-address runs with both sides' bytes."""
    a, b = _load(store, rec_hash), _load(store, rep_hash)
    if a is None or b is None:
        return {"base": f"{base:#x}", "error": "chunk not in the RAM store (older trace?)"}
    out: list[dict] = []
    total, i, n = 0, 0, min(len(a), len(b))
    while i < n:
        if a[i] == b[i]:
            i += 1
            continue
        j = i
        while j < n and a[j] != b[j]:
            j += 1
        total += j - i
        if len(out) < runs:
            lo, hi = i & ~0xF, min(n, max(j, i + 16) + 0xF & ~0xF)
            out.append(
                {
                    "addr": f"{base + i:#x}",
                    "len": j - i,
                    "record": a[lo:hi].hex(" ", 8),
                    "replay": b[lo:hi].hex(" ", 8),
                    "at": f"{base + lo:#x}",
                }
            )
        i = j
    return {"base": f"{base:#x}", "bytes_differing": total, "runs": out}


def record_loop(fp: _Fingerprinter, period: float, out: Path) -> None:
    """Pause the recording every `period` seconds and write a fingerprint."""
    fp.connect()
    n = 0
    with out.open("a") as fh:
        while True:
            try:
                fp.qmp.execute("stop")
                s = fp.fingerprint()
                fp.qmp.execute("cont")
            except (OSError, ConnectionError, QmpError) as exc:
                # QEMU exiting at the end of the boot is the normal way out.
                print(f"[rr-sample] recording ended after {n} samples ({exc})", file=sys.stderr)
                return
            s["t"] = round(time.time(), 1)
            fh.write(json.dumps(s) + "\n")
            fh.flush()
            n += 1
            time.sleep(period)


def replay_loop(fp: _Fingerprinter, trace: Path) -> None:
    """Break at each sampled icount, compare, stop at the first mismatch."""
    from . import recordreplay as rr

    verdict_path = trace / VERDICT_FILE
    samples = [json.loads(x) for x in (trace / SAMPLES_FILE).read_text().splitlines() if x]
    want = {s["icount"] for s in samples}
    bounds: set[int] = set()
    io_at: set[int] = set()
    irq_at: set[int] = set()
    for ev in rr.walk_events(trace):
        if ev.icount in want:
            if ev.eid == 0:
                bounds.add(ev.icount)
            elif ev.eid in rr._V12_ASYNC_IDS:
                io_at.add(ev.icount)
            elif ev.eid == EVENT_INTERRUPT:
                irq_at.add(ev.icount)
    verdict: dict = {
        "samples": len(samples),
        "compared": 0,
        "skipped": [],
        "inconclusive": [],
        "last_match": None,
    }

    def _write() -> None:
        verdict_path.write_text(json.dumps(verdict, indent=2) + "\n")

    fp.connect()
    try:
        for s in samples:
            ic = s["icount"]
            # QEMU 9.0.2 aborted a replay-break that fell mid-chunk ("Playback
            # shouldn't have to iowait"); 8.2.2 does not even carry that message.
            # Attempt every sample and say which were off-boundary, so an abort is
            # attributed to the right cause instead of being assumed.
            off = ic not in bounds
            verdict["attempting"] = {"icount": ic, "mid_chunk": off}
            _write()
            now = fp.icount()
            if now > ic:
                verdict["skipped"].append({"icount": ic, "why": f"replay already at {now}"})
                continue
            fp.qmp.events.clear()
            fp.qmp.execute("replay-break", icount=ic)
            if not fp.wait_stop_or_stall():
                verdict["error"] = (
                    f"replay WEDGED before reaching the break at {ic}: icount stopped "
                    "advancing. Compare from the last match; the divergence is between it "
                    "and the wedge."
                )
                verdict["wedged_at"] = fp.icount()
                # Where the diverged guest is stuck: the instruction QEMU refuses
                # to let it take an exception at.
                verdict["wedge_regs"] = fp.qmp.hmp("info registers -a")
                break
            r = fp.fingerprint()
            verdict["compared"] += 1
            verdict.setdefault("mid_chunk_ok", 0)
            verdict["mid_chunk_ok"] += off
            d = compare(s, r, fp.store)
            why = artifact(d, ic in io_at, ic in irq_at, s["regs"], r["regs"]) if d else None
            if why:
                verdict["inconclusive"].append(
                    {"icount": ic, "why": why, "mem_mib": d.get("mem_mib") if d else None}
                )
                d = None
            elif d:
                verdict["first_mismatch"] = {"icount": ic, **d}
                break
            if not any(x["icount"] == ic for x in verdict["inconclusive"]):
                verdict["last_match"] = ic
            _write()
            fp.qmp.execute("cont")
        else:
            verdict["all_matched"] = True
    except (OSError, ConnectionError, QmpError) as exc:
        verdict["error"] = f"{type(exc).__name__}: {exc}"
    _write()
    if "first_mismatch" in verdict or "error" in verdict:
        # Nothing further to learn from this replay; end it instead of letting
        # the harness wait out a 30-minute boot timeout on a paused guest.
        try:
            fp.qmp.execute("quit")
        except (OSError, ConnectionError, QmpError):
            pass
    print(f"[rr-diverge] verdict written to {verdict_path}", file=sys.stderr)


def start(mode: str, value: str, trace: Path, argv: list[str], sock_path: Path, work: Path):
    """Start the sampler/checker thread for this VM. Returns the thread."""
    fp = _Fingerprinter(sock_path, ram_ranges(argv), work, trace / RAM_STORE)
    if mode == "record":
        target, args = record_loop, (fp, float(value), trace / SAMPLES_FILE)
    else:
        target, args = replay_loop, (fp, trace)
    t = threading.Thread(target=target, args=args, name=f"rr-sample-{mode}", daemon=True)
    t.start()
    return t


def qmp_args(sock_path: Path) -> list[str]:
    return ["-qmp", f"unix:{sock_path},server,nowait"]


__all__ = [
    "SAMPLES_FILE",
    "SAMPLE_ENV_VAR",
    "VERDICT_FILE",
    "compare",
    "env_sample",
    "qmp_args",
    "ram_ranges",
    "start",
]
