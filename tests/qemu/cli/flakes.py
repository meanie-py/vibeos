"""Flake signatures and `vibeos-cli flake-report` -- which failures are FLAKES, ranked.

Two questions a pass/fail tally cannot answer, and both have cost us:

  * WHICH failure is this? The durable ledger recorded verdict + classification
    and nothing else, so "arm64 HANG_TIMEOUT" pooled a GIC wedge with a WoW64
    stall, and naming the flake meant scraping run dirs under /tmp -- which a
    reboot deletes. `boot_signature`/`conformance_signatures` reduce a failure to
    a stable cluster key that goes INTO the ledger row, so the row alone names
    it after the artifacts are gone.

  * Is it a flake at all? A deterministic fault with a variable observable looks
    exactly like a flake in a tally, and a lane iterating on an uncommitted tree
    looks like a flaky commit. The grade therefore comes from one definition:
    the SAME signature at the SAME commit on the SAME arch, on a CLEAN tree, both
    present and absent across runs. Everything else is reported as what it is --
    deterministic at that commit, a single sample, or dirty and ungradeable.

A run-level failure reported per item manufactures N findings from one event
(95 kmtests "never launched" in one round is one lost boot, not 95 flakes), so
any transition shared by several unrelated items in one round collapses to a
single `mass:` signature. The tell is the shared constant.
"""

from __future__ import annotations

import json
import re
import time
from collections import defaultdict
from pathlib import Path

FARM_DIR = Path.home() / "vibeos-flake-farm"
PRESERVED = FARM_DIR / "fails" / "auto"
CONF_ROUNDS = FARM_DIR / "conformance.jsonl"

# Refusals, not boots: the harness declined to run, so nothing about the OS was sampled.
NOT_A_BOOT = frozenset({"STALE_MEDIA", "LADDER_NOT_STAGED", "RUN_INTERRUPTED"})

_HEX = re.compile(r"0x[0-9a-fA-F]+|\b[0-9a-fA-F]{8,16}\b")
_NUM = re.compile(r"\b\d+\b")
_BUGCHECK = re.compile(r"BugCheck (0x[0-9A-Fa-f]+)")
_MAX_SYM_OFFSET = 0x10000
_MASS_MIN = 3  # this many unrelated items sharing one transition = one run-level event


def norm(s: str, width: int = 70) -> str:
    """Strip what varies between instances of one failure: addresses, counters, seq ids."""
    s = _HEX.sub("#", s)
    s = _NUM.sub("#", s)
    return " ".join(s.split())[:width]


def _pc_symbol(line: str) -> str | None:
    """`hang: CPU#0 0xfff… -> 00661544<TAB>KeRaiseIrql+14<TAB>file:line` -> `KeRaiseIrql`.

    None when the symbolicator had nothing (`<no symbol>`, `(no symbol: …)`): a
    missing symbol must not become a shared cluster key of its own."""
    parts = line.split("\t")
    if len(parts) < 2:
        return None
    sym = parts[1].strip()
    if not sym or sym.startswith("<") or sym.startswith("("):
        return None
    name, _, off = sym.partition("+")
    # `XHF_NOEXECUTE+314197`, `EFLAGS_ID_MASK+148214`: a CONSTANT, hundreds of KB from
    # the PC -- the symbolicator resolved against the wrong image. As a cluster key it
    # would pose as a cause; a real function is never that far from its own code.
    try:
        if off and int(off, 0) > _MAX_SYM_OFFSET:
            return None
    except ValueError:
        pass
    return name


def boot_signature(boot: dict) -> str:
    """One failed boot (a `boots[]` entry of a smoke summary) -> a stable cluster key."""
    cls = boot.get("classification") or boot.get("outcome") or "FAIL"
    sal = [str(x) for x in (boot.get("salient") or [])]
    joined = "\n".join(sal)
    pc = next((_pc_symbol(x) for x in sal if "\t" in x and "->" in x), None)
    m = _BUGCHECK.search(joined)
    if m:
        code = f"0x{int(m.group(1), 16):X}"
        post = " post-marker" if "(after boot marker)" in joined else ""
        return f"{cls} {code}{post}" + (f" @ {pc}" if pc else "")
    if cls in ("HANG_TIMEOUT",) or boot.get("outcome") == "timeout":
        kind = boot.get("hang_kind") or "?"
        if pc:
            return f"{cls} {kind} @ {pc}"
        last = next((x.split("last serial:", 1)[1] for x in sal if "last serial:" in x), "")
        return f"{cls} {kind} | {norm(last, 40)}" if last else f"{cls} {kind}"
    if cls == "FAIL_MARKER":
        mk = next((x.split("fail marker:", 1)[1] for x in sal if "fail marker:" in x), "")
        return f"{cls} {norm(mk, 50)}"
    if cls == "LADDER_FAIL":
        guests = sorted({x.split()[0] for x in sal if re.match(r"^\w+32\.exe\b", x)})
        return f"{cls} {','.join(guests) or '?'}"
    return f"{cls} | {norm(sal[0], 60)}" if sal else cls


def run_signature(summary: dict) -> str | None:
    """A whole smoke/gate summary -> the signature of its FIRST failure (None if green)."""
    if summary.get("verdict") in ("OK", "WARN"):
        return None
    for b in summary.get("boots") or []:
        if not b.get("ok"):
            return boot_signature(b)
    cls = summary.get("classification") or "FAIL"
    st = next((s for s in summary.get("stages") or [] if s.get("status") == "fail"), None)
    if st:
        return f"{cls} {st.get('name', '')}: {norm(str(st.get('detail', '')), 50)}"
    return cls


def conformance_signatures(rec: dict) -> list[str]:
    """One conformance round -> its distinct failure signatures (empty when clean)."""
    shape = str(rec.get("shape") or "")
    if shape == "clean":
        return []
    if shape.startswith("A:bugcheck"):
        code = shape.split()[-1]
        try:
            code = f"0x{int(code, 16):X}"
        except ValueError:
            pass
        return [f"BUGCHECK {code}"]
    if shape in ("B:no-launch", "L:no-guest-serial", "stalled/DID-NOT-FINISH"):
        return [shape]
    # Grouped on the VERBATIM detail: the one-event tell is a shared CONSTANT
    # ("never launched", "OK -> FAILSTART"). Three tests each dropping a different
    # count ("pass 679 -> 678", "pass 40 -> 37") share nothing and stay separate.
    by_tail: dict[str, list[str]] = defaultdict(list)
    for r in rec.get("regressions") or []:
        name, _, detail = str(r).partition(": ")
        by_tail[detail].append(name)
    mass_tails = {
        t for t, ns in by_tail.items() if len(ns) >= _MASS_MIN and "failing assertion" not in t
    }
    sigs: set[str] = {f"mass: {norm(t, 60)} (N tests)" for t in mass_tails}
    per_test: dict[str, list[str]] = defaultdict(list)
    for r in rec.get("regressions") or []:
        name, _, detail = str(r).partition(": ")
        if detail not in mass_tails:
            per_test[name].append(detail)
    for name, details in per_test.items():
        found = (re.search(r"failing assertion (\S+)", d) for d in details)
        asserts = sorted({m.group(1) for m in found if m})
        if len(asserts) >= _MASS_MIN:
            # One test losing several assertions in one round is one event in that test.
            sigs.add(f"{name}: several new failing assertions")
        elif asserts:
            sigs.update(f"assert {a}" for a in asserts)  # its count drift is the same event
        elif any("(dropped)" in d or "(grew)" in d for d in details):
            sigs.add(f"{name}: count drift")
        else:
            sigs.update(f"{name}: {norm(d, 40)}" for d in details)
    return sorted(sigs)


# --------------------------------------------------------------------------- report
#
# A SAMPLE is one run. It carries the signatures it showed (`sigs`) and how much it
# could have seen (`sees`):
#   "all"    it completed: any signature it lacks, it OBSERVED to be absent
#   "shape"  it died early: only whole-run signatures (lost boot, bugcheck) are
#            observed absent -- it never reached the assertion a test-level one is about
#   "own"    a failure whose signature we have, or a historical failure whose we do not:
#            evidence FOR its own signature, never evidence AGAINST any other
# Without this a lost boot at a commit "proves" every assertion there intermittent,
# which is how a deterministic failure gets graded FLAKE.

_COMPLETED = frozenset({"clean", "clean-but-ratchet-regression"})


def _run_level(sig: str) -> bool:
    return sig.startswith(("B:", "L:", "BUGCHECK", "stalled", "mass:"))


def _ledger_path(repo: Path) -> Path:
    from . import cachecfg

    return cachecfg.vibeos_root(repo) / "vibeos-ledger.jsonl"


def _read_jsonl(path: Path) -> list[dict]:
    out: list[dict] = []
    try:
        with open(path) as f:
            for line in f:
                try:
                    out.append(json.loads(line))
                except ValueError:
                    continue
    except OSError:
        pass
    return out


def _run_id_epoch(run_id: str) -> float:
    try:
        return time.mktime(time.strptime("-".join(run_id.split("-")[:2]), "%Y%m%d-%H%M%S"))
    except (TypeError, ValueError):
        return 0.0


def collect(repo: Path, since: float) -> list[dict]:
    """Every smoke and conformance run since `since`, as samples (see above)."""
    samples: list[dict] = []
    ledger = [r for r in _read_jsonl(_ledger_path(repo)) if r.get("ts", 0) >= since]

    # --- smoke. The ledger has every run; rows written before `sig` existed name no
    # signature, so their failures come from the preserved evidence dirs instead.
    have_sig: set[str] = set()
    for r in ledger:
        if r.get("cmd") != "smoke" or r.get("classification") in NOT_A_BOOT:
            continue
        green = r.get("verdict") in ("OK", "WARN")
        sig = r.get("sig")
        if sig and r.get("run_id"):
            have_sig.add(r["run_id"])
        samples.append(
            {
                "source": "smoke",
                "arch": r.get("arch"),
                "head": r.get("head"),
                "dirty": r.get("dirty"),
                "ts": r.get("ts", 0),
                "sigs": [] if green else [sig] if sig else [],
                "sees": "all" if green else "own",
                "where": r.get("preserved"),
                "count": True,
            }
        )
    for d in sorted(PRESERVED.glob("*")):
        try:
            man = json.loads((d / "manifest.json").read_text())
            summ = json.loads((d / "summary.json").read_text())
        except (OSError, ValueError):
            continue
        rid = str(man.get("run_id", ""))
        ts = _run_id_epoch(rid)
        if rid in have_sig or ts < since or summ.get("command") != "smoke":
            continue
        if summ.get("classification") in NOT_A_BOOT:
            continue
        sig = run_signature(summ)
        if sig:
            # Its ledger row is already counted (as "own", sig unknown); this one only
            # contributes the signature, so it must not count as a second run.
            samples.append(
                {
                    "source": "smoke",
                    "arch": man.get("arch"),
                    "head": man.get("head"),
                    "dirty": man.get("dirty"),
                    "ts": ts,
                    "sigs": [sig],
                    "sees": "own",
                    "where": str(d),
                    "count": False,
                }
            )

    # --- conformance. Ledger rows carry `sigs` from the commit that added this module;
    # before that the flake farm's round corpus is the only record.
    conf_rows = [r for r in ledger if r.get("cmd") == "conformance" and "sigs" in r]
    cutoff = min((r["ts"] for r in conf_rows), default=float("inf"))
    for r in _read_jsonl(CONF_ROUNDS):
        ts = _run_id_epoch(str(r.get("ts", "")))
        if not since <= ts < cutoff:
            continue
        samples.append(
            {
                "source": "conformance",
                "arch": r.get("arch"),
                "head": r.get("sha"),
                "dirty": None,
                "ts": ts,
                "sigs": conformance_signatures(r),
                "sees": "all" if r.get("shape") in _COMPLETED else "shape",
                "where": r.get("run_dir"),
                "count": True,
            }
        )
    for r in conf_rows:
        samples.append(
            {
                "source": "conformance",
                "arch": r.get("arch"),
                "head": r.get("head"),
                "dirty": r.get("dirty"),
                "ts": r["ts"],
                "sigs": r.get("sigs") or [],
                "sees": "all" if r.get("classification") in _COMPLETED else "shape",
                "where": r.get("preserved"),
                "count": True,
            }
        )
    return samples


def _observed_absent(s: dict, sig: str) -> bool:
    if sig in s["sigs"]:
        return False
    return s["sees"] == "all" or (s["sees"] == "shape" and _run_level(sig))


def grade(samples: list[dict]) -> list[dict]:
    """Per (source, arch, signature), graded at the commits it was seen on:

    FLAKE          present in one run and OBSERVED absent in another, at one commit
    DETERMINISTIC  present in every observing run (n>=2) at every commit it was seen on
    UNGRADED       never more than one observing run at a commit where it occurred
    Dirty-tree runs are set aside entirely: their commit does not name their code.
    """
    clean = [s for s in samples if s.get("dirty") is not True]
    at: dict[tuple, list[dict]] = defaultdict(list)
    for s in clean:
        at[(s["source"], s["arch"], s["head"])].append(s)
    runs: dict[tuple, int] = defaultdict(int)
    for s in clean:
        if s["count"]:
            runs[(s["source"], s["arch"])] += 1
    hits: dict[tuple, list[dict]] = defaultdict(list)
    dirty: dict[tuple, int] = defaultdict(int)
    for s in samples:
        for sig in s["sigs"]:
            if s.get("dirty") is True:
                dirty[(s["source"], s["arch"], sig)] += 1
            else:
                hits[(s["source"], s["arch"], sig)].append(s)
    out = []
    for key in set(hits) | set(dirty):
        src, arch, sig = key
        hs = hits.get(key, [])
        heads: dict[str, int] = defaultdict(int)
        for s in hs:
            heads[s["head"]] += 1
        flaky = det = single = 0
        for h, k in heads.items():
            absent = sum(_observed_absent(s, sig) for s in at[(src, arch, h)])
            if absent and k:
                flaky += 1
            elif k >= 2:
                det += 1
            else:
                single += 1
        g = "FLAKE" if flaky else "DETERMINISTIC" if det else "UNGRADED"
        latest = max(hs or [{"ts": 0, "where": None}], key=lambda s: s.get("ts", 0))
        out.append(
            {
                "source": src,
                "arch": arch,
                "sig": sig,
                "grade": g,
                "count": len(hs),
                "dirty": dirty.get(key, 0),
                "rate": len(hs) / max(1, runs[(src, arch)]),
                "flaky_heads": flaky,
                "deterministic_heads": det,
                "single_heads": single,
                "last_seen": time.strftime("%m-%d %H:%M", time.localtime(latest.get("ts", 0))),
                "where": latest.get("where"),
            }
        )
    order = {"FLAKE": 0, "DETERMINISTIC": 1, "UNGRADED": 2}
    out.sort(key=lambda c: (order[c["grade"]], -c["count"]))
    return out


def _where(path: str | None) -> str:
    if not path:
        return "(no evidence recorded)"
    if not Path(path).exists():
        return f"(evidence GONE — it was in {Path(path).parent})"
    return path


def flake_report(args, repo: Path) -> int:
    """`vibeos-cli flake-report` -- the fleet's failures clustered by signature and graded."""
    days = float(getattr(args, "days", 14) or 14)
    samples = collect(repo, time.time() - days * 86400)
    rows = grade(samples)
    arch = getattr(args, "arch", None)
    source = getattr(args, "source", "all") or "all"
    rows = [
        r
        for r in rows
        if (not arch or r["arch"] == arch) and (source == "all" or r["source"] == source)
    ]
    if getattr(args, "json", False):
        print(json.dumps(rows, indent=2))
        return 0
    runs: dict[tuple, int] = defaultdict(int)
    for s in samples:
        if s["count"] and s.get("dirty") is not True:
            runs[(s["source"], s["arch"])] += 1
    print(f"flake-report: last {days:g} days. Runs graded (clean or unknown tree):")
    print("  " + "   ".join(f"{src}:{a}={n}" for (src, a), n in sorted(runs.items())))
    print("  FLAKE = present in one run and observed ABSENT in another at the same commit.")
    min_count = int(getattr(args, "min", 2))
    shown = [r for r in rows if r["count"] >= min_count]
    cur = None
    for r in shown:
        if r["grade"] != cur:
            cur = r["grade"]
            print(f"\n{cur}")
        extra = f"  +{r['dirty']} on dirty trees" if r["dirty"] else ""
        print(
            f"  {r['count']:4d} {100 * r['rate']:5.1f}%  {r['source']:11s} {r['arch']:6s} "
            f"{r['sig'][:72]}"
        )
        print(
            f"        commits: {r['flaky_heads']} flaky, {r['deterministic_heads']} "
            f"deterministic, {r['single_heads']} single-sample{extra}; last {r['last_seen']}"
        )
        print(f"        {_where(r['where'])}")
    # Seen ONLY on uncommitted trees: real failures, but the commit does not name the
    # code that ran, so nothing can be graded. Hiding them would make a lane that
    # iterates dirty look like it never fails.
    dirty_only = sorted(
        (r for r in rows if not r["count"] and r["dirty"] >= min_count), key=lambda r: -r["dirty"]
    )
    if dirty_only:
        print("\nSEEN ONLY ON DIRTY TREES (ungradeable: the commit does not name the code)")
        for r in dirty_only:
            print(f"  {r['dirty']:4d}         {r['source']:11s} {r['arch']:6s} {r['sig'][:72]}")
    rest = len(rows) - len(shown) - len(dirty_only)
    if rest:
        print(f"\n({rest} more signatures below --min {min_count}; --min 1 or --json for all)")
    return 0
