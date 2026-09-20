#!/usr/bin/env python3
"""
Sweep FP..upstream/master for bug-fix-SHAPED diffs, independent of commit subject line.

Method (re-runnable):
  1. List every commit in the upstream range: `git log --format=%H FP..upstream/master`.
  2. For each commit, get its changed C/C++/header files via
     `git diff-tree --no-commit-id --name-only -r <sha>`.
  3. Keep only commits touching a file with a known Rust twin: either
       (a) an exact c_path hit in tools/cohab-registry.json, or
       (b) a path under win32ss/ (the deleted-C-tree case — twin is found later,
           by function name, not by this script).
  4. Skip commits already in the keyword-sweep queue (docs/engineering/active-lanes/
     upstream-sync-bug-leads.md) and the explicitly-excluded already-fixed SHAs.
  5. Pull the full diff (`git show --unified=0 <sha> -- <files>`), split into per-file
     hunks, and pattern-match each hunk against a fixed list of "bug fix shapes"
     (comparison operator tightened/loosened, off-by-one +-1, added NULL/bounds check,
     added lock/unlock, added Probe/_SEH2_TRY, changed constant literal, changed
     signedness, &|/&&|| swap, added ! or removed !, added early return/goto).
  6. Keep only SMALL diffs (default: <=10 changed lines in the touching hunk(s) for the
     matched file) — large diffs are triaged separately as "too big for this sweep".

Output: JSON lines to stdout, one candidate per line, plus a summary of shape counts to
stderr. This script does not touch the working tree and makes no git writes.
"""

import json
import pathlib
import re
import subprocess
import sys
from collections import Counter

FP = "5213cf717cff6946945922e6a7e03d87b0a58e4f"
UPSTREAM = "upstream/master"
REPO = "/home/curi/Projects/os-playground/vibeos-upstream-sync"

KNOWN_SHAS_SHORT = set(["39ea28d4c6", "c0f18d059a", "ae0e4ee305", "e1bf662d9a", "e7e1465812"])

# full known-queue shas pulled live from the leads doc, see load_known_shas()


def run(*args):
    p = subprocess.run(["git", "-C", REPO, *args], capture_output=True, check=True)
    return p.stdout.decode("utf-8", errors="replace")


def load_known_shas():
    known = set(KNOWN_SHAS_SHORT)
    for fn in [
        "docs/engineering/active-lanes/upstream-sync-bug-leads.md",
        "docs/engineering/active-lanes/upstream-sync-verified-findings.md",
    ]:
        try:
            txt = pathlib.Path(f"{REPO}/{fn}").read_text()
        except FileNotFoundError:
            continue
        known |= set(re.findall(r"`([0-9a-f]{8,12})`", txt))
    return known


def load_cohab_paths():
    with pathlib.Path(f"{REPO}/tools/cohab-registry.json").open() as fh:
        return json.load(fh)  # c_path -> record


SRC_EXT = re.compile(r"\.(c|cpp|cxx|cc|h|hpp)$", re.IGNORECASE)


def list_commits():
    out = run("log", "--format=%H", f"{FP}..{UPSTREAM}")
    return [line for line in out.splitlines() if line]


def files_of(sha):
    out = run("diff-tree", "--no-commit-id", "--name-only", "-r", sha)
    return [line for line in out.splitlines() if line and SRC_EXT.search(line)]


# ---- shape detectors: operate on a single hunk's (removed_lines, added_lines) ----

CMP_OPS = ["<=", ">=", "==", "!=", "<", ">"]
CMP_RE = re.compile(r"(<=|>=|==|!=|<|>)")


def strip(line):
    return line.rstrip("\n")


def normalize_for_cmp_diff(line):
    return CMP_RE.sub("<CMP>", line)


def shape_cmp_operator_change(removed, added):
    if len(removed) != 1 or len(added) != 1:
        return None
    r, a = removed[0], added[0]
    rops = CMP_RE.findall(r)
    aops = CMP_RE.findall(a)
    if not rops or not aops:
        return None
    if rops == aops:
        return None
    if normalize_for_cmp_diff(r) == normalize_for_cmp_diff(a):
        return f"comparison operator changed: {''.join(rops)} -> {''.join(aops)}"
    return None


NUM_RE = re.compile(r"(?<![\w.])([+-]?\s*\d+)(?![\w.])")


def shape_off_by_one(removed, added):
    if len(removed) != 1 or len(added) != 1:
        return None
    r, a = removed[0], added[0]
    # same line except a "- 1"/"+ 1" (or similar small literal) added/removed near an
    # index/length/size expression
    r2 = re.sub(r"\s*[+-]\s*1\b", "", r)
    a2 = re.sub(r"\s*[+-]\s*1\b", "", a)
    if ((r != a and r2.strip() == a2.strip() and r2 != r) or a2 != a) and (
        r2.strip() == a2.strip()
    ):
        return "±1 literal added/removed (potential off-by-one)"
    return None


CONST_LITERAL_RE = re.compile(r"\b0x[0-9a-fA-F]+\b|\b\d{2,}\b")


def shape_constant_change(removed, added):
    if len(removed) != 1 or len(added) != 1:
        return None
    r, a = removed[0], added[0]
    rl = CONST_LITERAL_RE.findall(r)
    al = CONST_LITERAL_RE.findall(a)
    if not rl or not al:
        return None
    if rl == al:
        return None
    if CONST_LITERAL_RE.sub("<N>", r) == CONST_LITERAL_RE.sub("<N>", a):
        return f"constant literal changed: {rl} -> {al}"
    return None


LOGIC_OPS = {"&&": "AND", "||": "OR", "&": "BITAND", "|": "BITOR"}


def shape_logic_swap(removed, added):
    if len(removed) != 1 or len(added) != 1:
        return None
    r, a = removed[0], added[0]
    for op_r in LOGIC_OPS:
        for op_a in LOGIC_OPS:
            if op_r == op_a:
                continue
            if op_r in r and op_a in a:
                rr = r.replace(op_r, "<OP>", 1)
                aa = a.replace(op_a, "<OP>", 1)
                if rr == aa:
                    return f"logic operator swapped: {op_r} -> {op_a}"
    return None


def shape_bang_change(removed, added):
    if len(removed) != 1 or len(added) != 1:
        return None
    r, a = removed[0], added[0]
    if r.replace("!", "") == a.replace("!", "") and r != a:
        return "'!' added/removed"
    return None


NULLCHECK_RE = re.compile(r"if\s*\(\s*!?\s*\w[\w.\->]*\s*(==|!=)?\s*(NULL|0)?\s*\)")
ADDED_CHECK_RE = re.compile(
    r"if\s*\(.*(==\s*NULL|!=\s*NULL|^\s*!\s*\w|>=\s*\w|<=\s*\w|>\s*\w+Size|<\s*\w+Size)"
)


def shape_added_check(removed, added):
    if removed:
        return None
    for a in added:
        s = a.strip()
        if (
            s.startswith("if")
            and ("NULL" in s or re.search(r"[<>]=?", s))
            and ("return" not in s and "goto" not in s)
        ):
            return "bare 'if' condition added (bounds/NULL check line)"
    return None


LOCK_WORDS = re.compile(
    r"(KeAcquireSpinLock|ExAcquireFastMutex|ExAcquireResource|Lock\w*\(|EnterCriticalSection|_SEH2_TRY|ProbeForRead|ProbeForWrite|RtlEnterCriticalSection)"
)
UNLOCK_WORDS = re.compile(
    r"(KeReleaseSpinLock|ExReleaseFastMutex|ExReleaseResource|Unlock\w*\(|LeaveCriticalSection|_SEH2_END|RtlLeaveCriticalSection)"
)


def shape_lock_added(removed, added):
    if removed:
        return None
    joined_add = "\n".join(added)
    m = LOCK_WORDS.search(joined_add)
    if m:
        return f"locking/Probe/SEH primitive added: {m.group(1)}"
    return None


EARLY_RETURN_RE = re.compile(r"^\s*(return\b|goto\b)")


def shape_early_return_added(removed, added):
    if removed:
        return None
    for a in added:
        if EARLY_RETURN_RE.match(a.strip()):
            return "early return/goto added"
    return None


SIGN_RE = re.compile(
    r"\b(unsigned|signed|ULONG|LONG|USHORT|SHORT|UCHAR|CHAR|SIZE_T|SSIZE_T|INT|UINT|DWORD|"
    r"ULONG_PTR|LONG_PTR|ULONGLONG|LONGLONG|INT64|UINT64|int32_t|uint32_t|int64_t|uint64_t)\b"
)


def shape_sign_or_width_change(removed, added):
    if len(removed) != 1 or len(added) != 1:
        return None
    r, a = removed[0], added[0]
    rt = SIGN_RE.findall(r)
    at = SIGN_RE.findall(a)
    if rt and at and rt != at:
        rest_r = SIGN_RE.sub("<T>", r)
        rest_a = SIGN_RE.sub("<T>", a)
        if rest_r == rest_a:
            return f"declared type changed: {rt} -> {at}"
    return None


SHAPES = [
    ("cmp_operator_change", shape_cmp_operator_change),
    ("off_by_one", shape_off_by_one),
    ("constant_change", shape_constant_change),
    ("logic_swap", shape_logic_swap),
    ("bang_change", shape_bang_change),
    ("added_check", shape_added_check),
    ("lock_or_probe_added", shape_lock_added),
    ("early_return_added", shape_early_return_added),
    ("sign_or_width_change", shape_sign_or_width_change),
]


def parse_diff_hunks(diff_text):
    """Yield (file, removed_lines, added_lines, hunk_header) per hunk, unified=0 diff."""
    cur_file = None
    removed, added = [], []
    hunk_header = None

    def flush():
        if cur_file is not None and (removed or added):
            yield_list.append((cur_file, removed[:], added[:], hunk_header))

    yield_list = []
    for line in diff_text.splitlines():
        if line.startswith("+++ b/"):
            cur_file = line[6:]
            removed, added = [], []
        elif line.startswith("@@"):
            if removed or added:
                yield_list.append((cur_file, removed[:], added[:], hunk_header))
            removed, added = [], []
            hunk_header = line
        elif line.startswith("+") and not line.startswith("+++"):
            added.append(line[1:])
        elif line.startswith("-") and not line.startswith("---"):
            removed.append(line[1:])
    if removed or added:
        yield_list.append((cur_file, removed[:], added[:], hunk_header))
    return yield_list


def main():
    max_hunk_lines = 10
    known = load_known_shas()
    cohab = load_cohab_paths()
    cohab_paths = set(cohab.keys())

    commits = list_commits()
    sys.stderr.write(f"total commits in range: {len(commits)}\n")

    candidates = []
    shape_counts = Counter()
    scanned = 0
    touched_relevant = 0

    for sha in commits:
        short = sha[:10]
        if short in known or any(short.startswith(k[:10]) or k.startswith(short) for k in known):
            continue
        files = files_of(sha)
        if not files:
            continue
        relevant = [f for f in files if f in cohab_paths or f.startswith("win32ss/")]
        if not relevant:
            continue
        touched_relevant += 1
        scanned += 1
        try:
            diff = run("show", "--unified=0", "--no-color", sha, "--", *relevant)
        except subprocess.CalledProcessError:
            continue
        hunks = parse_diff_hunks(diff)
        subject = run("log", "-1", "--format=%s", sha).strip()
        for f, removed, added, header in hunks:
            if f is None or (f not in relevant and not any(f.endswith(r) for r in relevant)):
                # +++ b/ path should match one of relevant; loose check
                pass
            total_lines = len(removed) + len(added)
            if total_lines == 0 or total_lines > max_hunk_lines:
                continue
            for name, fn in SHAPES:
                res = fn(removed, added)
                if res:
                    shape_counts[name] += 1
                    candidates.append(
                        {
                            "sha": sha,
                            "short": short,
                            "subject": subject,
                            "file": f,
                            "hunk": header,
                            "shape": name,
                            "detail": res,
                            "removed": removed,
                            "added": added,
                            "in_cohab_registry": f in cohab_paths,
                        }
                    )
                    break  # one shape tag per hunk is enough

    sys.stderr.write(
        f"commits touching a cohab/win32ss file (not already known): {touched_relevant}\n"
    )
    sys.stderr.write(f"candidate hunks found: {len(candidates)}\n")
    for k, v in shape_counts.most_common():
        sys.stderr.write(f"  {k}: {v}\n")

    for c in candidates:
        print(json.dumps(c))


if __name__ == "__main__":
    main()
