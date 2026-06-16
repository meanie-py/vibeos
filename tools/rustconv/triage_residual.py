#!/usr/bin/env python3
"""triage_residual.py — bucket a translated crate's residual compile errors into
per-file work packets tiered for Haiku vs Sonnet subagent repair.

Runs AFTER the deterministic pipeline (repair_pass + verify_constants --fix +
fix_type_casts/fix_arg_casts + resolve_imports) has squeezed out everything
mechanical. What remains is hallucination residue + semantic mismatches that
need a model to read the C ground truth. This tool decides WHICH model per file
and emits a JSON packet list the orchestrator turns into parallel Agent calls.

Tiering heuristic (per file):
  HAIKU  — localized, single-shape fixes a fast model nails from the error +
           struct layout alone: wrong bindgen accessor name (E0599 method),
           field path (E0609/E0615/E0610), cast target (E0308), arg count
           (E0061), undeclared name (E0425/E0433). Low error count, no SEH.
  SONNET — tangled: many distinct error codes, control-flow/borrow errors
           (E0382/E0499/E0502/E0506/E0716), trait/lifetime (E0277/E0309/E0310),
           PSEH/_SEH2 remnants, or simply a large blast radius (>~40 errors).

Disjoint file ownership: each packet is one .rs file → at most one subagent.
Never assigns types_*.rs / glue.rs / ddk_shims.rs (substrate — fix call sites).

Usage: triage_residual.py <crate-src-dir> <target-triple> [--c-dir <drivers/.../x>]
                          [--out packets.json] [--max-haiku N] [--sonnet-floor N]
"""
import sys, os, json, subprocess, argparse, re
from collections import defaultdict, Counter

SUBSTRATE = re.compile(r"(types_(i386|amd64|arm64)|glue|ddk_shims|data|guid|globals)\.rs$")

# error codes a fast model handles well from error+layout alone
EASY = {"E0308", "E0599", "E0609", "E0610", "E0615", "E0061", "E0425", "E0433",
        "E0412", "E0560", "E0614", "E0606", "E0605", "E0573", "E0252", "E0255",
        "E0428", "E0432"}
# error codes that signal real semantic/borrow/lifetime reasoning
HARD = {"E0382", "E0499", "E0502", "E0506", "E0716", "E0515", "E0277", "E0309",
        "E0310", "E0391", "E0507", "E0505", "E0271", "E0072", "E0204"}


def cargo_errors(crate_src, target):
    crate = os.path.basename(os.path.dirname(crate_src))
    ws = os.path.abspath(os.path.join(crate_src, "../.."))
    p = subprocess.run(
        ["cargo", "check", "-p", crate, "--target", target,
         "--message-format", "json"],
        cwd=ws, capture_output=True, text=True)
    per_file = defaultdict(list)
    for line in p.stdout.splitlines():
        try:
            m = json.loads(line)
        except json.JSONDecodeError:
            continue
        if m.get("reason") != "compiler-message":
            continue
        msg = m["message"]
        code = (msg.get("code") or {}).get("code")
        if not code or msg.get("level") != "error":
            continue
        for sp in msg.get("spans", []):
            if sp.get("is_primary"):
                per_file[sp["file_name"]].append(
                    {"code": code, "line": sp["line_start"],
                     "msg": msg["message"][:160]})
                break
    return per_file


def seh_density(path):
    try:
        t = open(path).read()
    except OSError:
        return 0
    return t.count("_SEH2_") + t.count("catch_unwind")


def tier(file_path, errs):
    codes = Counter(e["code"] for e in errs)
    n = len(errs)
    distinct = len(codes)
    hard = sum(codes[c] for c in codes if c in HARD)
    seh = seh_density(file_path)
    # Sonnet if any genuine semantic signal, broad blast radius, or SEH-tangled
    if hard > 0 or n > 40 or distinct >= 6 or seh > 8:
        return "sonnet"
    # Haiku if dominated by easy, localized shapes
    easy = sum(codes[c] for c in codes if c in EASY)
    if easy >= 0.8 * n and n <= 40:
        return "haiku"
    return "sonnet"  # default to the stronger model when unsure


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("crate_src")
    ap.add_argument("target")
    ap.add_argument("--c-dir", default="")
    ap.add_argument("--out", default="")
    args = ap.parse_args()

    crate_src = os.path.abspath(args.crate_src)
    ws = os.path.abspath(os.path.join(crate_src, "../.."))
    per_file = cargo_errors(crate_src, args.target)

    packets = []
    for f, errs in sorted(per_file.items(), key=lambda kv: -len(kv[1])):
        base = os.path.basename(f)
        if SUBSTRATE.search(base):
            continue  # never edit substrate
        abspath = f if os.path.isabs(f) else os.path.join(ws, f)
        t = tier(abspath, errs)
        codes = Counter(e["code"] for e in errs)
        c_src = ""
        if args.c_dir:
            cand = os.path.join(args.c_dir, base.replace(".rs", ".c"))
            if os.path.exists(cand):
                c_src = cand
        packets.append({
            "file": f,
            "abspath": abspath,
            "tier": t,
            "n_errors": len(errs),
            "codes": dict(codes.most_common()),
            "seh": seh_density(abspath),
            "c_source": c_src,
            "sample_errors": errs[:6],
        })

    summary = {
        "total_files": len(packets),
        "total_errors": sum(p["n_errors"] for p in packets),
        "haiku_files": sum(1 for p in packets if p["tier"] == "haiku"),
        "sonnet_files": sum(1 for p in packets if p["tier"] == "sonnet"),
        "haiku_errors": sum(p["n_errors"] for p in packets if p["tier"] == "haiku"),
        "sonnet_errors": sum(p["n_errors"] for p in packets if p["tier"] == "sonnet"),
    }
    out = {"summary": summary, "packets": packets}
    dst = args.out or os.path.join(os.path.dirname(crate_src), "..", "_triage.json")
    dst = os.path.abspath(dst)
    with open(dst, "w") as fh:
        json.dump(out, fh, indent=2)
    print(json.dumps(summary, indent=2))
    print(f"\nwrote {dst}")
    for p in packets:
        print(f"  [{p['tier']:6}] {p['file']:42} {p['n_errors']:3} errs  "
              f"{','.join(list(p['codes'])[:5])}")


if __name__ == "__main__":
    main()
