"""Flake farm — soak boots overnight, harvest every outcome into a durable corpus.

`vibeos-cli farm` loops independent smoke boots (per arch, in parallel threads —
the hostlock CPU-lease pool is the real throttle) and appends one JSONL record
per boot to ~/vibeos-flake-farm/corpus.jsonl. Failures get their artifacts
(full.log + screenshot) copied out of /tmp (reboot-volatile) into the farm dir.

`vibeos-cli farm-report` clusters the corpus by (arch, classification,
stop code, top symbol) so a 1-in-50 flake becomes a ranked table with sample
paths instead of lore.

Pairs with the tripwire channel: `--tripwire "SPECIALPOOL=*"` runs every boot
with special pool armed so silent corruption faults at the corrupting store
(symbolicated on serial by the bugcheck banner) instead of surfacing as a
downstream mystery. See ntoskrnl-rust src/tripwire.rs.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import threading
import time
from collections import defaultdict
from pathlib import Path

from . import archcfg, hostlock, provenance
from . import config as cli_config  # noqa: F401  (parity with smoke; config read in _one_boot)
from .artifacts import Run
from .smoke import _one_boot

FARM_DIR = Path(os.path.expanduser("~/vibeos-flake-farm"))
CORPUS = FARM_DIR / "corpus.jsonl"

# Serial/salient patterns worth carrying into the corpus record.
_STOP_RE = r"BUGCHECK: 0x([0-9A-Fa-f]{8})"


def _git_state(repo: Path) -> tuple[str, bool]:
    try:
        sha = subprocess.run(
            ["git", "rev-parse", "--short", "HEAD"], cwd=repo, capture_output=True, text=True
        ).stdout.strip()
        dirty = bool(
            subprocess.run(
                ["git", "status", "--porcelain"], cwd=repo, capture_output=True, text=True
            ).stdout.strip()
        )
        return sha, dirty
    except Exception:
        return "unknown", False


def _extract_evidence(run_dir: Path, salient: list[str]) -> dict:
    """Pull stop code + symbolicated banner lines from the boot's full.log
    (the self-symbolicating bugcheck banner prints `***` lines on serial)."""
    import re

    ev: dict = {"stop_code": None, "symbols": []}
    text = ""
    full = run_dir / "full.log"
    if full.exists():
        try:
            text = full.read_text(errors="replace")
        except Exception:
            pass
    hay = text + "\n" + "\n".join(salient)
    m = re.search(_STOP_RE, hay)
    if m:
        ev["stop_code"] = "0x" + m.group(1).upper()
    # Banner lines: "***   P3 = ntoskrnl.exe+0x..." / "***   TrapFrame.Rip = ..."
    # / "***   stack[+0x..] = ..." — keep the deduped first handful.
    seen: list[str] = []
    for ln in text.splitlines():
        ln = ln.strip()
        if ln.startswith("***") and ("=" in ln) and ln not in seen:
            seen.append(ln)
        if len(seen) >= 12:
            break
    ev["symbols"] = seen
    # Prefer the most causal symbol for clustering: TrapFrame.Rip, else P-param.
    top = next((s for s in seen if "TrapFrame.Rip" in s), None) or next(
        (s for s in seen if ".exe+" in s or ".sys+" in s), None
    )
    if top and "=" in top:
        ev["top_symbol"] = top.split("=")[-1].strip()
    else:
        ev["top_symbol"] = None
    return ev


def _preserve_failure(run_dir: Path, rec: dict) -> str | None:
    """Copy the failure's artifacts out of /tmp into the farm dir (durable)."""
    tag = rec["classification"] or rec["outcome"]
    dst = FARM_DIR / "fails" / f"{rec['ts']}-{rec['arch']}-{tag}"
    try:
        dst.mkdir(parents=True, exist_ok=True)
        for name in ("full.log", "summary.json"):
            src = run_dir / name
            if src.exists():
                shutil.copy2(src, dst / name)
        # The SERIAL is the evidence -- bugcheck banner, trap frame, flight
        # recorder. Preserving a failure without it kept the verdict and dropped
        # everything that explains it.
        for src in run_dir.glob("*serial*.log"):
            shutil.copy2(src, dst / src.name)
        shot = rec.get("screenshot")
        if shot and Path(shot).exists():
            shutil.copy2(shot, dst / Path(shot).name)
        return str(dst)
    except Exception:
        return None


_corpus_lock = threading.Lock()


def _append(rec: dict) -> None:
    with _corpus_lock:
        FARM_DIR.mkdir(parents=True, exist_ok=True)
        with CORPUS.open("a") as f:
            f.write(json.dumps(rec) + "\n")


POLITE_POLL_S = 30.0


def _wait_for_quiet_host(arch: str, deadline: float) -> bool:
    """`--polite`: block until nobody else is landing or booting. Checked BEFORE
    each boot, never during one, so a soak only ever fills idle time and yields
    the pool back at the next boot boundary. False if the deadline passed."""
    announced = ""
    while time.monotonic() < deadline:
        holder = hostlock.land_holder()
        others = hostlock.foreign_boot_holders(os.getpid())
        if not holder and not others:
            return True
        why = (
            f"a land is gating ({holder.get('kind', '?')})"
            if holder
            else f"{len(others)} other boot(s) running"
        )
        if why != announced:
            print(f"[farm {arch}] polite: waiting, {why}", flush=True)
            announced = why
        time.sleep(POLITE_POLL_S)
    return False


def _farm_arch(
    repo: Path,
    arch: str,
    deadline: float,
    max_boots: int,
    sha: str,
    dirty: bool,
    tripwire: str,
    stats: dict,
    polite: bool = False,
) -> None:
    i = 0
    while time.monotonic() < deadline and i < max_boots:
        if polite and not _wait_for_quiet_host(arch, deadline):
            break
        i += 1
        run = Run("farm", arch)
        t0 = time.monotonic()
        try:
            r = _one_boot(run, repo, arch, until="", fail_on=None, idx=0, total=1)
        except Exception as e:  # a farm worker must never die silently
            r = {
                "ok": False,
                "outcome": "farm_error",
                "classification": "FARM_ERROR",
                "salient": [repr(e)[:200]],
            }
        dur = time.monotonic() - t0
        run.classification = r.get("classification", "") or ""
        try:
            run.finalize(repo)
        except Exception:
            pass
        rec = {
            "ts": time.strftime("%Y%m%d-%H%M%S"),
            "arch": arch,
            "sha": sha,
            "dirty": dirty,
            "tripwire": tripwire,
            # Self-describing corpus: an MP failure record is not comparable to a
            # UP one, and reading that off the surrounding shell history later is
            # exactly the sort of provenance gap that produces a retraction.
            "smp": int(os.environ.get("VIBEOS_QEMU_SMP", 0) or 0),
            "ahci": os.environ.get("VIBEOS_AHCI_CD", "") == "1",
            "ok": r.get("ok", False),
            "outcome": r.get("outcome", ""),
            "classification": r.get("classification", ""),
            "hang_kind": r.get("hang_kind"),
            "census_stuck": r.get("census_stuck"),
            "lockdep_hits": r.get("lockdep_hits"),
            "cswitch_hits": r.get("cswitch_hits"),
            "census_lines": (r.get("census_lines") or [])[:6],
            "salient": r.get("salient", [])[:8],
            "screenshot": r.get("screenshot"),
            "run_dir": str(run.dir),
            "duration_s": round(dur, 1),
            "timings": r.get("timings", {}),
        }
        rec.update(_extract_evidence(run.dir, rec["salient"]))
        if not rec["ok"]:
            rec["preserved"] = _preserve_failure(run.dir, rec)
        _append(rec)
        with _corpus_lock:
            stats["total"] += 1
            stats["fails"] += 0 if rec["ok"] else 1
            key = "ok" if rec["ok"] else (rec["classification"] or rec["outcome"])
            stats["by"][key] = stats["by"].get(key, 0) + 1
        state = "ok" if rec["ok"] else (rec["classification"] or rec["outcome"])
        print(
            f"[farm {arch}] boot {i}: {state} ({dur:.0f}s)"
            + (f" stop={rec['stop_code']}" if rec.get("stop_code") else "")
            + (f" @ {rec['top_symbol']}" if rec.get("top_symbol") else ""),
            flush=True,
        )


def farm(args, repo: Path) -> int:
    arches = [a.strip() for a in (args.arches or "amd64").split(",") if a.strip()]
    minutes = float(args.minutes or 0)
    max_boots = int(args.boots or (10**9 if minutes else 5))
    # monotonic: "soak for N minutes" is an elapsed-time question, and a
    # wall-clock step would silently lengthen or shorten the soak.
    deadline = time.monotonic() + (minutes * 60 if minutes else 10**9)
    tripwire = (getattr(args, "tripwire", "") or "").strip()
    if tripwire:
        os.environ["VIBEOS_TRIPWIRE"] = tripwire

    # Same env knob smoke/conformance use; every boot goes through smoke's
    # `_one_boot`, so `_smp_check` proves per boot that the MP kernel actually
    # ran. That guard is why this can be a bare env var: a soak that silently
    # booted the UP kernel would otherwise bank hours of meaningless green.
    smp = int(getattr(args, "smp", 0) or 0)
    if smp:
        os.environ["VIBEOS_QEMU_SMP"] = str(smp)

    # Same shape for --ahci: smoke's `_ahci_check` fails any boot that did not
    # actually run an AHCI driver, so the soak cannot bank IDE boots as AHCI.
    polite = bool(getattr(args, "polite", False))
    ahci = bool(getattr(args, "ahci", False))
    if ahci:
        os.environ["VIBEOS_AHCI_CD"] = "1"

    # Farming stale media produces garbage data — same hard boundary as smoke.
    for arch in arches:
        st = provenance.check_staleness(repo, archcfg.base_arch(arch))
        if st.stale and not args.allow_stale:
            print(f"[farm] {arch}: STALE media — build first (or --allow-stale)")
            return 1

    # Same for a soak whose whole point is a probe: check ONCE up front rather
    # than per boot — the media is fixed for the run, and a farm is exactly the
    # shape (hours of boots, one frozen image) that made this failure expensive.
    require = [s for s in (getattr(args, "require_probe", None) or []) if s.strip()]
    if require:
        for arch in arches:
            missing = provenance.probes_missing_from_media(repo, archcfg.base_arch(arch), require)
            if missing:
                print(f"[farm] {arch}: PROBE NOT IN MEDIA — refusing to spend the boots")
                for m in missing:
                    print(f"[farm]   {m}")
                return 1
        print(f"[farm] probe-check OK: {len(require)} literal(s) in every boot image")

    sha, dirty = _git_state(repo)
    print(
        f"[farm] arches={','.join(arches)} sha={sha}{' DIRTY' if dirty else ''}"
        + (f" tripwire={tripwire!r}" if tripwire else "")
        + (f" -smp {smp}" if smp else "")
        + (" ahci" if ahci else "")
        + (" polite" if polite else "")
        + (f" budget={minutes:.0f}min" if minutes else f" budget={max_boots} boots/arch")
        + f" -> {CORPUS}"
    )

    stats: dict = {"total": 0, "fails": 0, "by": {}}
    threads = [
        threading.Thread(
            target=_farm_arch,
            args=(
                repo,
                archcfg.base_arch(a),
                deadline,
                max_boots,
                sha,
                dirty,
                tripwire,
                stats,
                polite,
            ),
            daemon=True,
        )
        for a in arches
    ]
    for t in threads:
        t.start()
    try:
        for t in threads:
            t.join()
    except KeyboardInterrupt:
        print("[farm] interrupted — corpus records written so far are durable")
    print(f"[farm] done: {stats['total']} boots, {stats['fails']} failures — {dict(stats['by'])}")
    print("[farm] report: ./vibeos-cli farm-report")
    return 0


def farm_report(args, repo: Path) -> int:
    if not CORPUS.exists():
        print(f"no corpus at {CORPUS} — run `vibeos-cli farm` first")
        return 1
    since = getattr(args, "since", "") or ""
    sha_filter = getattr(args, "sha", "") or ""
    recs = []
    for line in CORPUS.read_text().splitlines():
        try:
            r = json.loads(line)
        except Exception:
            continue
        if since and r.get("ts", "") < since:
            continue
        if sha_filter and r.get("sha") != sha_filter:
            continue
        recs.append(r)
    if not recs:
        print("no records match")
        return 1

    total = len(recs)
    fails = [r for r in recs if not r.get("ok")]
    print(f"corpus: {total} boots, {len(fails)} failures ({100 * len(fails) / total:.1f}%)")
    per_arch: dict[str, list] = defaultdict(list)
    for r in recs:
        per_arch[r.get("arch", "?")].append(r)
    for arch, rs in sorted(per_arch.items()):
        f = [r for r in rs if not r.get("ok")]
        print(f"  {arch}: {len(rs)} boots, {len(f)} failures ({100 * len(f) / len(rs):.1f}%)")

    # Cluster failures by family: (arch, classification, stop_code, top_symbol).
    fams: dict[tuple, list] = defaultdict(list)
    for r in fails:
        key = (
            r.get("arch", "?"),
            r.get("classification") or r.get("outcome") or "?",
            r.get("stop_code") or "-",
            (r.get("top_symbol") or "-").split("+")[0],  # cluster by module+fn, not offset
        )
        fams[key].append(r)
    if fams:
        print("\nfailure families (most frequent first):")
        for key, rs in sorted(fams.items(), key=lambda kv: -len(kv[1])):
            arch, cls, stop, sym = key
            latest = rs[-1]
            print(f"  {len(rs):3d}x  {arch:6s} {cls:14s} stop={stop:12s} {sym}")
            top_syms = latest.get("symbols") or []
            for s in top_syms[:3]:
                print(f"          {s}")
            where = latest.get("preserved") or latest.get("run_dir")
            print(f"          latest: {where}")

    # Tripwire hits (lockdep/cswitchdep) fire on OTHERWISE-GREEN boots — the
    # detector pins the violation without a BUGCHECK, so they never appear in
    # the failure families above. Surface them separately or a soak that
    # exists precisely to catch context-switch violations reports nothing.
    tfams: dict[tuple, list] = defaultdict(list)
    for r in recs:
        for kind in ("lockdep", "cswitch"):
            if r.get(f"{kind}_hits") or 0:
                tfams[(r.get("arch", "?"), kind)].append(r)
    if tfams:
        print("\ntripwire hits (on green + failed boots — detector fired):")
        for key, rs in sorted(tfams.items(), key=lambda kv: -len(kv[1])):
            arch, kind = key
            hits = sum((r.get(f"{kind}_hits") or 0) for r in rs)
            latest = rs[-1]
            print(f"  {len(rs):3d} boots {arch:6s} {kind:8s} {hits} total hits")
            for s in (latest.get("census_lines") or [])[:3]:
                print(f"          {s}")
            where = latest.get("preserved") or latest.get("run_dir")
            print(f"          latest: {where}")
    return 0
