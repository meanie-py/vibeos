"""Durable, stream-to-disk archiving + cost guardrails for PAID rustconv API calls.

Why this module exists (read before "simplifying" it away):

  1. STREAM TO DISK, NOT RAM. The Anthropic stream in batch_runner accumulated
     the whole response in a Python string and only returned it at the very end.
     Kill the process mid-stream and every output token you were billed for is
     gone. A `ResponseSink` writes each delta to disk and flushes immediately, so
     a kill loses nothing already generated.

  2. NEVER /tmp. Agents kept pointing `-o /tmp/foo.rs` at expensive translations;
     when the box rebooted or /tmp was reaped, the only copy vanished. EVERY paid
     call now also lands, unconditionally, in an in-repo (gitignored) run dir
     under RUSTCONV_RESPONSES_DIR — independent of whatever `-o` says.

  3. PER-FUNCTION PERSISTENCE. `translate` makes one paid call PER FUNCTION and
     used to buffer them all, writing once at the end. Now each function's Rust
     is committed to the run dir the instant it returns, so a kill at function
     35/40 keeps all 35.

  4. LOUD WARNING + KILL GUARD. Every invocation prints a banner (this costs real
     money; do NOT Ctrl-C mid-call) and installs a SIGINT/SIGTERM handler that
     points at the preserved partial work instead of letting a panicked kill feel
     like it threw everything away.
"""

import json
import os
import re
import signal
import sys
import uuid
from datetime import datetime
from pathlib import Path

from .config import RUSTCONV_RESPONSES_DIR, cost_usd, model_pricing


def _slug(s: str) -> str:
    return re.sub(r"[^A-Za-z0-9._-]+", "-", str(s)).strip("-")[:80] or "call"


def _is_under_tmp(p: Path) -> bool:
    tmp_roots = ["/tmp", "/var/tmp", os.environ.get("TMPDIR", "")]
    rp = str(p.resolve())
    return any(t and (rp == t or rp.startswith(t.rstrip("/") + "/")) for t in tmp_roots)


def warn_if_tmp_output(output: str | None) -> None:
    """Loudly flag `-o /tmp/...` — the durable archive already holds the output,
    so a /tmp path is at best redundant and at worst the only (ephemeral) copy."""
    if not output:
        return
    if _is_under_tmp(Path(output)):
        print(
            f"\n  ⚠️  WARNING: --output points into a temp dir ({output}).\n"
            f"     Temp files are wiped on reboot/tmp-reaping and are easy to lose.\n"
            f"     The durable copy is saved in the .responses archive regardless;\n"
            f"     prefer --assemble (writes into the crate) or an in-repo path.\n",
            file=sys.stderr,
        )


class ResponseSink:
    """A single open file that the streaming loop writes deltas into, flushing
    each one so the bytes are on disk before the next delta arrives."""

    def __init__(self, path: Path):
        self.path = path
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self._fh = open(self.path, "w", encoding="utf-8")  # noqa: SIM115  # long-lived streaming handle, closed via close()
        self.bytes_written = 0

    def write(self, text: str) -> None:
        self._fh.write(text)
        self._fh.flush()  # land on disk now — a kill after this keeps it
        try:
            os.fsync(self._fh.fileno())
        except (OSError, ValueError):
            pass
        self.bytes_written += len(text)

    def close(self) -> None:
        try:
            self._fh.close()
        except Exception:
            pass


class RunArchive:
    """One directory per `translate`/`submit` invocation, holding every paid
    response as it streams. Survives kills, reboots, and tmp-reaping."""

    def __init__(self, label: str, model: str, target: str):
        ts = datetime.now().strftime("%Y%m%d-%H%M%S")
        self.dir = RUSTCONV_RESPONSES_DIR / f"{ts}-{_slug(label)}-run-{uuid.uuid4().hex[:12]}"
        self.dir.mkdir(parents=True, exist_ok=False)
        self.model = model
        self.label = label
        self.committed: list[str] = []  # human-facing list of saved files
        self._meta = {
            "label": label,
            "model": model,
            "target": target,
            "started": ts,
            "calls": [],
        }
        self._flush_meta()

    def _flush_meta(self) -> None:
        (self.dir / "meta.json").write_text(json.dumps(self._meta, indent=2))

    def stream_path(self, idx: int, name: str) -> Path:
        """Live partial-stream file for an in-flight call (kept on kill)."""
        return self.dir / f"{uuid.uuid4().int}_{_slug(name)}.partial"

    def commit(
        self,
        idx: int,
        name: str,
        code: str,
        input_tokens: int,
        output_tokens: int,
        *,
        attempt_id: str | None = None,
    ) -> Path:
        """Persist a completed unit's final Rust + record token usage."""
        final = self.dir / f"{uuid.uuid4().int}_{_slug(name)}.rs"
        final.write_text(code, encoding="utf-8")
        # Raw streams are paid evidence; immutable attempts own their authoritative copy.
        self.committed.append(final.name)
        self._meta["calls"].append(
            {
                "idx": idx,
                "attempt_id": attempt_id,
                "name": name,
                "file": final.name,
                "input_tokens": input_tokens,
                "output_tokens": output_tokens,
                "cost_usd": round(cost_usd(self.model, input_tokens, output_tokens), 4),
            }
        )
        self._flush_meta()
        return final


# --- invocation banner + kill guard --------------------------------------

_GUARD_INSTALLED = False


def paid_invocation_banner(
    model: str, n_calls: int, est_input_tokens: int, archive: RunArchive
) -> None:
    """Print the up-front cost warning. n_calls is the number of paid API calls
    this command will make (one per function for `translate`)."""
    pin, pout = model_pricing(model)
    # Output is typically 0.4–0.7x of input for C→Rust (Rust expands the source);
    # use 0.6 and show a 1x–3x band because retries / large fns / re-sent context
    # routinely push the real bill well past a point estimate.
    est_output = int(est_input_tokens * 0.6)
    low = cost_usd(model, est_input_tokens, est_output)
    print(
        "\n" + "═" * 68 + "\n"
        "  💸  PAID API CALL — this spends real money.\n"
        f"      model: {model}  (${pin:.2f}/${pout:.2f} per 1M tok, FULL sync price)\n"
        f"      ~{n_calls} call(s), ~{est_input_tokens:,} input tok est.\n"
        f"      ROUGH estimate: ${low:.2f}–${low * 3:.2f}  "
        "(real cost is shown live below as it accrues)\n"
        "\n"
        "  ⛔  Do NOT Ctrl-C mid-call. Output streams to disk as it generates;\n"
        "      a kill still BILLS you but partial work IS preserved here:\n"
        f"      {archive.dir}\n" + "═" * 68 + "\n",
        file=sys.stderr,
    )


def install_kill_guard(archive: RunArchive) -> None:
    """On SIGINT/SIGTERM, point at the preserved partial work and exit. Data is
    already flushed per-delta, so this is about not panicking — and not killing
    again on the assumption everything was lost."""
    global _GUARD_INSTALLED
    if _GUARD_INSTALLED:
        return
    _GUARD_INSTALLED = True

    def _handler(signum, frame):
        print(
            "\n\n" + "─" * 68 + "\n"
            f"  ⚠️  Interrupted ({signal.Signals(signum).name}).\n"
            f"      {len(archive.committed)} completed unit(s) ARE SAVED — plus any\n"
            f"      in-flight partial stream — under:\n"
            f"      {archive.dir}\n"
            "      Re-run the SAME command to resume; already-saved units are reused\n"
            "      and are NOT re-billed. Do not delete that dir.\n" + "─" * 68 + "\n",
            file=sys.stderr,
        )
        sys.exit(130)

    for sig in (signal.SIGINT, signal.SIGTERM):
        try:
            signal.signal(sig, _handler)
        except (ValueError, OSError):
            pass  # not on the main thread; best-effort
