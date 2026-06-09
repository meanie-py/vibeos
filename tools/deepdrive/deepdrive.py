#!/usr/bin/env python3
"""deepdrive — drive DeepCode (DeepSeek terminal agent) as a tool any Claude can call.

Architecture (see docs/engineering/deepdrive-controller-design.md): **tmux-in, files-out**.
- INPUT goes through a tmux PTY (DeepCode is an Ink TUI; it has no headless mode).
- OUTPUT + STATE come from DeepCode's structured on-disk files under
  ~/.deepcode/projects/<projectCode>/  (sessions-index.json + <sessionId>.jsonl),
  NOT from scraping the ANSI pane.

`run` is one BLOCKING call per bounded turn: it sends the prompt, waits internally until
the session leaves `status:"processing"`, then returns a compact JSON snapshot (reply,
tool calls, optional --verify error count, git diff stat). The caller never polls.

Invoked in the CALLER'S directory (--dir) — no worktree is created; we inherit the
caller's already-working build env. The handoff is synchronous, so there is no concurrent
mutation of the shared tree. Review the returned diff every turn.
"""
import argparse
import fcntl
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
import time

HOME = os.path.expanduser("~")
PROJECTS = os.path.join(HOME, ".deepcode", "projects")
SETTINGS = os.path.join(HOME, ".deepcode", "settings.json")
LOCK_DIR = os.path.join(HOME, ".deepdrive", "locks")
PROMPT_DIR = os.path.join(HOME, ".deepdrive", "prompts")
STATE_DIR = os.path.join(HOME, ".deepdrive", "state")
POLL = 1.5            # seconds between status polls
READY_TIMEOUT = 40    # seconds to wait for the TUI to accept input

# APPROXIMATE DeepSeek v4 prices (USD per 1M tokens). The DeepSeek dashboard is the
# source of truth; these are for a relative per-turn estimate only. cache_hit is the
# dominant term in agentic loops (~99% of input is a cache read). Edit if rates change.
PRICES = {
    "deepseek-v4-pro":   {"cache_hit": 0.01, "input": 0.28, "output": 1.10},
    "deepseek-v4-flash": {"cache_hit": 0.005, "input": 0.14, "output": 0.55},
}
DEFAULT_PRICE = PRICES["deepseek-v4-pro"]


def estimate_cost(model, prompt, cached, completion):
    p = PRICES.get(model, DEFAULT_PRICE)
    cached = cached or 0
    uncached = max(0, (prompt or 0) - cached)
    return round(cached * p["cache_hit"] / 1e6
                 + uncached * p["input"] / 1e6
                 + (completion or 0) * p["output"] / 1e6, 4)


# ---------- small helpers ----------
def sh(args, cwd=None, timeout=None):
    """Run argv, return (rc, combined_output)."""
    try:
        p = subprocess.run(args, cwd=cwd, capture_output=True, text=True, timeout=timeout)
        return p.returncode, (p.stdout or "") + (p.stderr or "")
    except subprocess.TimeoutExpired as e:
        return 124, f"(timeout) {e}"
    except Exception as e:  # noqa: BLE001
        return 1, f"(error) {e}"


def abspath(d):
    return os.path.abspath(os.path.expanduser(d))


def project_code(d):
    # DeepCode encodes the project root as the abs path with '/' -> '-'.
    return abspath(d).replace("/", "-")


def project_dir(d):
    return os.path.join(PROJECTS, project_code(d))


def session_name(d):
    return "deepdrive-" + hashlib.sha1(abspath(d).encode()).hexdigest()[:12]


def die(obj, code=1):
    print(json.dumps(obj, ensure_ascii=False, indent=2))
    sys.exit(code)


# ---------- tmux ----------
def tmux(*args):
    return sh(["tmux", *args])


def has_session(sess):
    return tmux("has-session", "-t", sess)[0] == 0


def pane(sess, lines=4000):
    return tmux("capture-pane", "-t", sess, "-p", "-S", f"-{lines}")[1]


def launch_cfg(model, effort):
    """(env-prefix string, signature) for a deepcode launch. model/effort override
    settings via systemEnv (highest precedence in DeepCode). effort 'none'/'off' =>
    thinking disabled; otherwise REASONING_EFFORT."""
    parts = []
    if model:
        parts.append(f"MODEL={model}")
    if effort:
        if effort.lower() in ("none", "off", "no"):
            parts.append("THINKING_ENABLED=false")
        else:
            parts.append(f"REASONING_EFFORT={effort}")
    return " ".join(parts), f"{model or 'default'}|{effort or 'default'}"


def cfg_path(sess):
    return os.path.join(STATE_DIR, sess + ".cfg")


def read_cfg(sess):
    try:
        return open(cfg_path(sess)).read().strip()
    except OSError:
        return None


def write_cfg(sess, sig):
    os.makedirs(STATE_DIR, exist_ok=True)
    open(cfg_path(sess), "w").write(sig)


def start_session(sess, d, env_prefix=""):
    cmd = (env_prefix + " " if env_prefix else "") + "deepcode 2>&1"
    tmux("new-session", "-d", "-s", sess, "-x", "220", "-y", "55", "-c", d, cmd)
    t0 = time.time()
    while time.time() - t0 < READY_TIMEOUT:
        if "Type your message" in pane(sess, 60):
            time.sleep(1.0)
            return True
        time.sleep(1.0)
    return "Type your message" in pane(sess, 60)


def submit(sess, text):
    """Type `text` into the TUI and submit. Uses a tmux buffer (paste) so quotes/
    backticks/newlines never get mangled or auto-submit early. `text` must have NO
    trailing newline; the explicit Enter is the submit."""
    with tempfile.NamedTemporaryFile("w", delete=False, suffix=".txt") as f:
        f.write(text)
        tmp = f.name
    try:
        buf = "deepdrive"
        tmux("load-buffer", "-b", buf, tmp)
        tmux("paste-buffer", "-t", sess, "-b", buf, "-d")
        time.sleep(0.4)
        tmux("send-keys", "-t", sess, "Enter")
    finally:
        os.unlink(tmp)


# ---------- DeepCode state (files-out) ----------
def read_index(d):
    p = os.path.join(project_dir(d), "sessions-index.json")
    try:
        return json.load(open(p, encoding="utf-8")).get("entries", [])
    except Exception:  # noqa: BLE001
        return []


def newest_jsonl(d, after=None):
    """Newest session jsonl by mtime. If `after` is given, only return it when its
    mtime is newer than `after` (i.e. it was written as a result of our submit) —
    this reliably finds the active session whether reused or freshly created."""
    pd = project_dir(d)
    try:
        js = [os.path.join(pd, f) for f in os.listdir(pd) if f.endswith(".jsonl")]
    except FileNotFoundError:
        return None
    if not js:
        return None
    newest = max(js, key=os.path.getmtime)
    if after is not None and os.path.getmtime(newest) <= after:
        return None
    return newest


def entry_for(d, sid):
    for e in read_index(d):
        if e.get("id") == sid:
            return e
    return None


def parse_last_turn(jsonl):
    """From a session jsonl, return (reply, tool_calls) for the most recent user turn."""
    recs = []
    for line in open(jsonl, encoding="utf-8"):
        line = line.strip()
        if line:
            try:
                recs.append(json.loads(line))
            except Exception:  # noqa: BLE001
                pass
    last_user = max((i for i, r in enumerate(recs) if r.get("role") == "user"), default=-1)
    turn = recs[last_user + 1:]
    # map tool_call_id -> ok (from role:tool results)
    ok_by_id = {}
    for r in turn:
        if r.get("role") == "tool":
            tcid = (r.get("messageParams") or {}).get("tool_call_id")
            try:
                ok_by_id[tcid] = json.loads(r.get("content") or "{}").get("ok")
            except Exception:  # noqa: BLE001
                ok_by_id[tcid] = None
    tool_calls = []
    reply = ""
    for r in turn:
        if r.get("role") != "assistant":
            continue
        mp = r.get("messageParams") or {}
        for tc in (mp.get("tool_calls") or []):
            fn = tc.get("function") or {}
            try:
                a = json.loads(fn.get("arguments") or "{}")
            except Exception:  # noqa: BLE001
                a = {}
            arg = a.get("command") or a.get("file_path") or a.get("path") or ""
            tool_calls.append({"name": fn.get("name"), "arg": str(arg)[:200],
                               "ok": ok_by_id.get(tc.get("id"))})
        if (r.get("content") or "").strip():
            reply = r["content"]
    return reply, tool_calls


# ---------- verify / git ----------
def count_errors(out):
    m = re.search(r"due to (\d+) previous error", out)
    if m:
        return int(m.group(1))
    return len(re.findall(r"(?m)^\s*error(\[E\d+\])?[:\s]", out))


def run_verify(cmd, d):
    rc, out = sh(["bash", "-lc", cmd], cwd=d, timeout=1800)
    return {"cmd": cmd, "exit": rc, "error_count": count_errors(out),
            "tail": "\n".join(out.splitlines()[-25:])}


def run_metric(cmd, d):
    """Run a command whose output ends in an integer = remaining-units convergence
    signal (e.g. `grep -rc old_trampoline | wc -l`). Returns that int."""
    rc, out = sh(["bash", "-lc", cmd], cwd=d, timeout=600)
    ints = re.findall(r"-?\d+", out)
    return {"cmd": cmd, "exit": rc, "value": int(ints[-1]) if ints else None,
            "tail": "\n".join(out.splitlines()[-5:])}


def settings_model():
    try:
        return (json.load(open(SETTINGS)).get("env") or {}).get("MODEL")
    except Exception:  # noqa: BLE001
        return None


def git_snapshot(d):
    _, diff = sh(["git", "-C", d, "diff", "--stat"])
    _, st = sh(["git", "-C", d, "--no-optional-locks", "status", "--short"])
    return {"diff_stat": diff.strip()[:4000], "status_short": st.strip()[:4000]}


# ---------- lock ----------
def acquire_lock(d):
    os.makedirs(LOCK_DIR, exist_ok=True)
    lp = os.path.join(LOCK_DIR, session_name(d) + ".lock")
    fh = open(lp, "w")
    try:
        fcntl.flock(fh, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        die({"status": "error", "error": f"another deepdrive run holds the lock for {d}"}, 2)
    return fh


# ---------- commands ----------
def cmd_run(a):
    d = abspath(a.dir)
    if not os.path.isdir(d):
        die({"status": "error", "error": f"--dir not a directory: {d}"}, 2)
    if not os.path.exists(SETTINGS):
        die({"status": "error", "error": f"no DeepCode config at {SETTINGS}"}, 2)
    prompt = a.prompt
    if a.prompt_file:
        prompt = open(a.prompt_file, encoding="utf-8").read()
    if not prompt:
        die({"status": "error", "error": "need --prompt or --prompt-file"}, 2)

    lock = acquire_lock(d)  # noqa: F841 (held until process exit)
    sess = session_name(d)
    env_prefix, sig = launch_cfg(a.model, a.effort)
    # Restart the session if model/effort was explicitly requested and differs from the
    # running session's launch config (env is only read at launch). Otherwise reuse.
    if (a.model or a.effort) and has_session(sess) and read_cfg(sess) != sig:
        tmux("kill-session", "-t", sess)
    started_fresh = False
    if not has_session(sess):
        if not start_session(sess, d, env_prefix):
            die({"status": "error", "error": "deepcode TUI did not become ready"}, 1)
        write_cfg(sess, sig)
        started_fresh = True
    if a.new and not started_fresh:
        submit(sess, "/new")
        time.sleep(3)
    # snapshot pre-turn cumulative usage so we can report THIS turn's delta (DeepCode's
    # sessions-index usage is cumulative per session, not per-turn).
    pre_usage = {e.get("id"): (e.get("usage") or {}) for e in read_index(d)}

    # Write the full (possibly multi-line) prompt to a file DeepCode READS, and submit a
    # single-line directive. Keeps multi-line/code prompts out of the keystroke path and
    # out of the caller's repo.
    os.makedirs(PROMPT_DIR, exist_ok=True)
    ppath = os.path.join(PROMPT_DIR, f"{sess}-{int(time.time())}.md")
    open(ppath, "w", encoding="utf-8").write(prompt)
    directive = (f"Read the file {ppath} and carry out its instructions fully and "
                 f"autonomously. When the task is complete or you are blocked, stop and "
                 f"give a short summary.")

    t0 = time.time()
    submit(sess, directive)

    # Wait: the active session is the jsonl TOUCHED AFTER submit (reused or fresh).
    # Done when its index status leaves "processing"/None, confirmed over a 1-cycle
    # settle. mtime-after-submit avoids the stale-prior-completion and file-switch traps.
    status, sid, jl, settle = "timeout", None, None, 0
    while time.time() - t0 < a.timeout:
        time.sleep(POLL)
        cand = newest_jsonl(d, after=t0)
        if not cand:
            continue  # our turn hasn't registered on disk yet
        jl = cand
        sid = os.path.splitext(os.path.basename(jl))[0]
        st = (entry_for(d, sid) or {}).get("status")
        if st == "processing" or st is None:
            settle = 0
            continue
        settle += 1
        if settle >= 2:
            status = st
            break

    e = entry_for(d, sid) or {}
    reply, tool_calls = ("", [])
    if jl and os.path.exists(jl):
        reply, tool_calls = parse_last_turn(jl)
    # per-turn delta = cumulative now - cumulative before this turn (same session)
    post = e.get("usage") or {}
    pre = pre_usage.get(sid, {})
    d_prompt = (post.get("prompt_tokens") or 0) - (pre.get("prompt_tokens") or 0)
    d_compl = (post.get("completion_tokens") or 0) - (pre.get("completion_tokens") or 0)
    d_cached = ((post.get("prompt_tokens_details") or {}).get("cached_tokens") or 0) \
        - ((pre.get("prompt_tokens_details") or {}).get("cached_tokens") or 0)
    cfg = read_cfg(sess) or ""
    cfg_model = cfg.split("|")[0] if cfg else ""
    if cfg_model == "default":
        cfg_model = ""
    eff_model = a.model or cfg_model or settings_model() or "deepseek-v4-pro"
    result = {
        "status": status,
        "session_id": sid,
        "model": eff_model,
        "elapsed_s": round(time.time() - t0, 1),
        "turn": {"prompt_tokens": d_prompt, "completion_tokens": d_compl,
                 "cached_tokens": d_cached, "n_tool_calls": len(tool_calls),
                 "cost_estimate_usd": estimate_cost(eff_model, d_prompt, d_cached, d_compl)},
        "tool_calls": tool_calls,
        "reply": (reply or "")[:2500],
        "diff": git_snapshot(d),
    }
    if a.verify:
        result["verify"] = run_verify(a.verify, d)
    if a.metric:
        result["metric"] = run_metric(a.metric, d)
    print(json.dumps(result, ensure_ascii=False, indent=2))


def cmd_status(a):
    d = abspath(a.dir)
    jl = newest_jsonl(d)
    sid = os.path.splitext(os.path.basename(jl))[0] if jl else None
    e = entry_for(d, sid) or {}
    print(json.dumps({"session_id": sid, "status": e.get("status"),
                      "usage": e.get("usage"), "tmux_up": has_session(session_name(d)),
                      "summary": e.get("summary")}, ensure_ascii=False, indent=2))


def cmd_tail(a):
    d = abspath(a.dir)
    jl = newest_jsonl(d)
    if not jl:
        die({"error": "no session jsonl found"}, 1)
    reply, tool_calls = parse_last_turn(jl)
    print(json.dumps({"session_id": os.path.splitext(os.path.basename(jl))[0],
                      "tool_calls": tool_calls, "reply": reply}, ensure_ascii=False, indent=2))


def cmd_interrupt(a):
    d = abspath(a.dir)
    tmux("send-keys", "-t", session_name(d), "Escape")
    print(json.dumps({"interrupted": True}))


def cmd_stop(a):
    d = abspath(a.dir)
    tmux("kill-session", "-t", session_name(d))
    print(json.dumps({"stopped": session_name(d)}))


def cmd_sessions(a):
    print(json.dumps(read_index(abspath(a.dir)), ensure_ascii=False, indent=2))


def main():
    p = argparse.ArgumentParser(prog="deepdrive", description="Drive DeepCode as a tool.")
    sub = p.add_subparsers(dest="cmd", required=True)

    r = sub.add_parser("run", help="send one bounded prompt, block until done, return JSON")
    r.add_argument("--dir", required=True, help="ABSOLUTE working dir (the caller's worktree)")
    r.add_argument("--prompt", help="prompt text")
    r.add_argument("--prompt-file", help="file with the prompt")
    r.add_argument("--verify", help="shell cmd run after the turn; its error count is returned")
    r.add_argument("--metric", help="shell cmd whose trailing integer = remaining-units "
                                    "convergence signal (e.g. grep -c of callsites left)")
    r.add_argument("--model", help="override model (e.g. deepseek-v4-flash); restarts session if changed")
    r.add_argument("--effort", help="reasoning effort: max|high|medium|low|none (none=no thinking)")
    r.add_argument("--timeout", type=int, default=1800, help="max seconds to wait (default 1800)")
    r.add_argument("--new", action="store_true", help="reset DeepCode's conversation (/new) first")
    r.set_defaults(fn=cmd_run)

    for name, fn, h in [("status", cmd_status, "current session status + usage"),
                        ("tail", cmd_tail, "structured transcript of the last turn"),
                        ("interrupt", cmd_interrupt, "send Esc to the active turn"),
                        ("stop", cmd_stop, "kill the tmux session"),
                        ("sessions", cmd_sessions, "list sessions-index entries")]:
        s = sub.add_parser(name, help=h)
        s.add_argument("--dir", required=True)
        s.set_defaults(fn=fn)

    a = p.parse_args()
    a.fn(a)


if __name__ == "__main__":
    main()
