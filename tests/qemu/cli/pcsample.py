"""Sample every vCPU's PC through a live boot, then name where the time went.

A slow boot is a question a hang capture cannot answer: `save_hang_state` sees one
instant of a wedge, but "this boot took 9x longer" is spread across thousands of
instants, none of them stuck. So, while a boot runs, read `info registers -a` every
PERIOD seconds and append each vCPU's PC, its halted flag (x86 prints `HLT=1` while
a vCPU sits in `hlt`, so idle is told apart from spinning) and the first few return
addresses of its frame-pointer chain. Afterwards `report()` symbolizes the samples
in one batch and ranks functions by share of samples.

Opt-in with VIBEOS_QEMU_PC_SAMPLE=<seconds> (`vibeos-cli stress --sample-pcs`).
Rows go to `pcsamples.jsonl` beside the boot's serial log.

It reads registers over the harness's OWN QMP connection, under the client's lock:
a monitor command is one round trip, and the `cpu N` selection a frame walk needs is
monitor state that must not interleave with another thread's walk.

What a sample is NOT: a cycle profile. Reading registers kicks each vCPU out of its
translated code, and a vCPU the host has descheduled reports the PC it was preempted
at. Under starvation that is exactly the point (where a vCPU is parked when it does
not get to run), but read shares as "where vCPUs were found", not CPU time.
"""

from __future__ import annotations

import contextlib
import json
import os
import re
import threading
import time
from collections import Counter
from pathlib import Path

SAMPLE_ENV_VAR = "VIBEOS_QEMU_PC_SAMPLE"
SAMPLES_FILE = "pcsamples.jsonl"
CALLER_DEPTH = 3

_HLT = re.compile(r"\bHLT=([01])\b")


def env_period() -> float | None:
    """The sampling period in seconds, or None when sampling is off. Refuses junk."""
    raw = os.environ.get(SAMPLE_ENV_VAR, "").strip()
    if not raw:
        return None
    try:
        period = float(raw)
    except ValueError as exc:
        raise ValueError(f"{SAMPLE_ENV_VAR}={raw!r} is not a number of seconds") from exc
    if not 0.05 <= period <= 60:
        raise ValueError(f"{SAMPLE_ENV_VAR}={raw!r}: want 0.05..60 seconds")
    return period


def _per_cpu(state: str) -> dict[int, str]:
    """`info registers -a` split into {cpu_index: that vCPU's block}."""
    parts = re.split(r"^CPU#(\d+)\b", state, flags=re.MULTILINE)
    return {int(parts[i]): parts[i + 1] for i in range(1, len(parts) - 1, 2)}


def parse_sample(state: str) -> list[dict]:
    """[{cpu, pc, halted, fp, sp}] from one `info registers -a` dump."""
    from .triage import cpu_fps, cpu_pcs, cpu_sps

    fps = dict(cpu_fps(state))
    sps = dict(cpu_sps(state))
    blocks = _per_cpu(state)
    rows = []
    for idx, pc in cpu_pcs(state):
        m = _HLT.search(blocks.get(idx, ""))
        rows.append(
            {
                "cpu": idx,
                "pc": pc.lower(),
                "halted": None if m is None else m.group(1) == "1",
                "fp": fps.get(idx),
                "sp": sps.get(idx),
            }
        )
    return rows


def _callers(qmp, cpu: int, fp_hex: str | None, depth: int) -> list[str]:
    """Up to `depth` return addresses up the frame-pointer chain. Never raises."""
    if not fp_hex:
        return []
    wide = len(fp_hex) > 8
    unit = "g" if wide else "w"
    out: list[str] = []
    try:
        qmp.hmp(f"cpu {cpu}")
        fp = int(fp_hex, 16)
        for _ in range(depth):
            body = qmp.hmp(f"x /2{unit}x 0x{fp:x}").strip()
            words = re.findall(r"0x([0-9a-fA-F]+)", body.split(":", 1)[-1])
            if len(words) < 2:
                break
            nxt, ret = int(words[0], 16), int(words[1], 16)
            out.append(f"{ret:x}")
            if nxt <= fp or nxt - fp > 0x100000:
                break
            fp = nxt
    except Exception:
        pass
    finally:
        try:
            qmp.hmp("cpu 0")
        except Exception:
            pass
    return out


STACK_WORDS = 48
KERNEL_HALF = 0xFFFF_8000_0000_0000  # canonical upper half: kernel VAs on amd64/arm64


def _stack_words(qmp, cpu: int, sp_hex: str | None) -> list[str]:
    """Kernel-half words in the top STACK_WORDS slots of the stack. Never raises.

    The frame-pointer chain is the exact answer when every frame keeps one, but an
    optimized Rust function free to use RBP as a scratch base (ki_tb_shootdown is
    one) ends the chain in garbage at its very first link. A scan is the classic
    fallback: return addresses sit on the stack, and the first one that resolves
    to a DIFFERENT function than the PC is very likely the caller. Stale words from
    dead frames are the known cost, so the report calls these "likely" callers.
    """
    if not sp_hex or len(sp_hex) <= 8:
        return []  # 64-bit only for now: the kernel-half filter is the whole trick
    try:
        qmp.hmp(f"cpu {cpu}")
        body = qmp.hmp(f"x /{STACK_WORDS}gx 0x{sp_hex}")
    except Exception:
        return []
    finally:
        try:
            qmp.hmp("cpu 0")
        except Exception:
            pass
    words = []
    for line in body.splitlines():
        for w in re.findall(r"0x([0-9a-fA-F]{16})", line.split(":", 1)[-1]):
            if int(w, 16) >= KERNEL_HALF:
                words.append(w.lower())
    return words


def sample_once(qmp) -> list[dict]:
    """One locked snapshot of every vCPU. Raises if the monitor is gone."""
    with getattr(qmp, "lock", None) or contextlib.nullcontext():
        state = qmp.hmp("info registers -a")
        rows = parse_sample(state)
        for r in rows:
            r["callers"] = _callers(qmp, r["cpu"], r.pop("fp"), CALLER_DEPTH)
            r["stack"] = _stack_words(qmp, r["cpu"], r.pop("sp"))
    return rows


def start(qmp, out: Path, period: float) -> threading.Thread:
    """Sample until the monitor goes away (the VM was torn down)."""

    def _run() -> None:
        t0 = time.monotonic()
        with out.open("a") as fh:
            while True:
                try:
                    rows = sample_once(qmp)
                except Exception:
                    return  # the boot is over; the monitor closed under us
                t = round(time.monotonic() - t0, 2)
                for r in rows:
                    fh.write(json.dumps({"t": t, **r}) + "\n")
                fh.flush()
                time.sleep(period)

    th = threading.Thread(target=_run, name="pc-sample", daemon=True)
    th.start()
    return th


# --- the report ----------------------------------------------------------------


def _resolve_batch(
    repo: Path, arch: str, pcs: list[str], *, mp: bool, reloc: str | None
) -> dict[str, str]:
    """{pc_hex: "symbol+off"} against the kernel index, in-process.

    Same index and discipline as `qemu/addr2sym.sh` (the MP index for an MP boot,
    never the UP one; decline past a sane offset), but one bisect per address
    instead of a python+awk process each: a boot's worth of samples is hundreds of
    distinct PCs. PCs outside the kernel image are simply absent from the result.
    """
    import bisect

    from .triage import _MAX_SANE_SYMBOL_OFFSET

    index = repo / "qemu" / f"symbols-{arch}{'-mp' if mp else ''}.tsv"
    if not index.is_file():
        raise FileNotFoundError(
            f"{index} missing: run qemu/build-index.sh {arch} (an MP boot needs the -mp index)"
        )
    addrs: list[int] = []
    names: list[str] = []
    for line in index.read_text(errors="replace").splitlines():
        cols = line.split("\t")
        if len(cols) >= 2 and cols[0]:
            try:
                addrs.append(int(cols[0], 16))
            except ValueError:
                continue
            names.append(cols[1])
    shift = int(reloc, 16) if reloc else 0
    got: dict[str, str] = {}
    for pc in pcs:
        a = int(pc, 16) - shift
        i = bisect.bisect_right(addrs, a) - 1
        if i < 0 or a - addrs[i] > _MAX_SANE_SYMBOL_OFFSET:
            continue
        got[pc] = f"{names[i]}+{a - addrs[i]:#x}"
    return got


def _func(sym: str) -> str:
    return sym.split("+", 1)[0]


def _where(pc: str, sym: dict[str, str]) -> str:
    if pc in sym:
        return _func(sym[pc])
    # Canonical user addresses sit in the low half on both 32- and 64-bit guests.
    if int(pc, 16) < 0x8000_0000_0000 and (len(pc) > 8 or int(pc, 16) < 0x8000_0000):
        return "(user mode)"
    return "(kernel, not in ntoskrnl index)"


def _likely_caller(row: dict, sym: dict[str, str]) -> str:
    """The first two distinct functions on the stack scan other than the PC's own.
    Falls back to the frame-pointer chain on a sample that carries no scan."""
    own = _where(row["pc"], sym)
    seen: list[str] = []
    for w in row.get("stack") or row.get("callers") or []:
        fn = _where(w, sym)
        if fn.startswith("(") or fn == own or fn in seen:
            continue
        seen.append(fn)
        if len(seen) == 2:
            break
    return " <- ".join(seen) or "?"


def report(repo: Path, arch: str, samples: Path, serial: str, top: int = 15) -> list[str]:
    """Rank where vCPUs were found. Lines meant for a terminal."""
    from .triage import kernel_relocation

    rows = [json.loads(x) for x in samples.read_text().splitlines() if x.strip()]
    if not rows:
        return [f"[pc-sample] no samples in {samples}"]
    mp = "ntkrnlmp.exe" in serial
    reloc = kernel_relocation(arch, serial)
    uniq = sorted(
        {r["pc"] for r in rows}
        | {c for r in rows for c in r.get("callers", [])[:2]}
        | {w for r in rows for w in r.get("stack", [])}
    )
    sym = _resolve_batch(repo, arch, uniq, mp=mp, reloc=reloc)

    out: list[str] = []
    cpus = sorted({r["cpu"] for r in rows})
    for cpu in cpus:
        mine = [r for r in rows if r["cpu"] == cpu]
        halted = sum(1 for r in mine if r.get("halted"))
        out.append(
            f"[pc-sample] CPU#{cpu}: {len(mine)} samples, {100 * halted / len(mine):.0f}% halted"
        )
    busy = [r for r in rows if not r.get("halted")]
    if not busy:
        return out
    by_fn = Counter(_where(r["pc"], sym) for r in busy)
    out.append(f"[pc-sample] where non-halted vCPUs were found ({len(busy)} samples):")
    for fn, n in by_fn.most_common(top):
        out.append(f"  {100 * n / len(busy):5.1f}%  {fn}")
    by_site = Counter(
        (_where(r["pc"], sym), _likely_caller(r, sym))
        for r in busy
        if not _where(r["pc"], sym).startswith("(")
    )
    if by_site:
        out.append(
            "[pc-sample] kernel PC <- likely callers (first 2 other functions found on the stack):"
        )
        for (fn, caller), n in by_site.most_common(top):
            out.append(f"  {100 * n / len(busy):5.1f}%  {fn}  <-  {caller}")
    return out


__all__ = [
    "SAMPLES_FILE",
    "SAMPLE_ENV_VAR",
    "env_period",
    "parse_sample",
    "report",
    "sample_once",
    "start",
]
