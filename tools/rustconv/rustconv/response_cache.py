"""Durable per-function translation cache → resume an interrupted run for free.

`translate` makes one PAID API call per function. Before this, a run killed at
function 35/40 (credit exhaustion, Ctrl-C, crash, reboot) lost all 35 completed
functions — the next invocation re-translated and re-PAID for every one. This
module persists each function's Rust the instant it returns, and lets the next
run skip every function whose C body is unchanged (a `$0` cache hit).

Layout (gitignored, in-repo so it survives kills/reboots):

    tools/rustconv/.responses/<subsystem>__<stem>/
        manifest.json          # {func_name: {file, hash}}  — hash = of the C body
        <func>.rs              # the translated Rust for that function
        __preamble__.rs        # the file-scope preamble unit (if any)

Correctness guard: a cached entry is reused ONLY if the sha256 of the current C
function body matches the hash stored when it was translated. Edit the C (or the
split chunks it inlines) and that function re-translates automatically; the rest
still resume. `--no-resume` ignores the cache entirely (force fresh).
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import sys
from pathlib import Path

from .config import RUSTCONV_RESPONSES_DIR
from .outcome_ledger import lexical_fingerprint


def _resolve_responses_dir() -> Path:
    """The ONE durable home for paid responses — project-anchored, never module-anchored.

    This used to be `Path(__file__).parent.parent / ".responses"`, which agreed with
    config's project-anchored `RUSTCONV_RESPONSES_DIR` only by path COINCIDENCE: the
    package sits at `tools/rustconv/rustconv/`, so `parent.parent` happened to equal
    `TOOLS_DIR`. Install or import rustconv from anywhere else (site-packages, a
    standalone checkout) and the two silently diverge: ResponseCache reads and writes
    the TOOL's directory while outcome_ledger / sink / candidate_oracle keep using the
    PROJECT's. Every translation then misses the cache and is re-billed, with no error
    anywhere — the exact silent-degradation class that costs real money.

    So: always follow config. The legacy module-relative location is checked only to
    HARD FAIL when work is genuinely STRANDED there — the legacy bank holds paid work AND
    the active one holds none, so following config would look like a fresh start and
    silently re-buy everything. An explicit RUSTCONV_RESPONSES_DIR is a deliberate choice
    (tests, a relocated bank) and is always honoured without complaint.

    A NON-empty active bank is not a strand, and this distinction matters in practice: the
    vibeos fleet runs many worktrees, each with its own bank, all importing one
    pip-install-e'd package. Every one of those has a populated legacy path that is not
    its own, and failing there would break the whole fleet to warn about the normal case.
    Divergence is only a problem when it would cost money.
    """
    active = RUSTCONV_RESPONSES_DIR
    if os.environ.get("RUSTCONV_RESPONSES_DIR"):
        return active  # explicitly chosen — the caller means it

    def _has_work(path: Path) -> bool:
        return path.is_dir() and any(path.iterdir())

    legacy = Path(__file__).resolve().parent.parent / ".responses"
    if legacy != active and _has_work(legacy) and not _has_work(active):
        raise RuntimeError(
            f"rustconv: paid-response bank is stranded.\n"
            f"  active (project-anchored): {active}  <-- empty\n"
            f"  legacy (module-anchored):  {legacy}  <-- holds work you already paid for\n"
            f"This happens when rustconv is imported from outside the project tree.\n"
            f"fix: point at the bank explicitly, e.g.\n"
            f"  RUSTCONV_RESPONSES_DIR={legacy} <your command>\n"
            f"or move the bank to the active path. Refusing to re-bill silently."
        )
    return active


RESPONSES_DIR = _resolve_responses_dir()

PREAMBLE_KEY = "__preamble__"


def require_no_namespace_rebuy(base_key: str, selected_key: str) -> None:
    """Changing protection defaults may strand purchases; never silently buy them again.

    Deliberately conservative: even a partial/rejected bank requires explicit recovery.
    This does not copy unprotected bodies into a protected namespace or claim that an
    old body passed a new audit. Replay with the original flags, or migrate/review it.
    """
    conflicts = []
    for directory in RESPONSES_DIR.glob(_safe_filename(base_key) + "*"):
        if not directory.is_dir() or directory.name == _safe_filename(selected_key):
            continue
        if directory.name != base_key and not directory.name.startswith(base_key + "__"):
            continue
        if (
            any(directory.glob("*.rs"))
            or any(directory.glob("*.partial"))
            or any((directory / "__rejected__").glob("*.rs"))
        ):
            conflicts.append(str(directory))
    if conflicts:
        raise ValueError(
            "Protection/cache namespace changed while previously purchased output exists. "
            "Refusing automatic re-purchase. Replay using its original seal/cache flags "
            "or review and migrate the bank first:\n  " + "\n  ".join(sorted(conflicts))
        )


def _hash(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8", "replace")).hexdigest()[:16]


def _safe_filename(name: str) -> str:
    """C identifiers are filename-safe, but be defensive about odd symbols."""
    return re.sub(r"[^A-Za-z0-9_.@-]", "_", name)


# A sealed placeholder that survives into a cached entry is a harness bug, not a
# translation: the run stored output it never unsealed. Such an entry then replays
# for $0 on every later run and assembles a module full of `__RUSTCONV_FACT_0000__`,
# which is not valid Rust. Treated exactly like the empty-output case below —
# refused on `put` (loud: it is our bug) and a MISS on `get` (so an already-poisoned
# cache self-heals instead of replaying forever).
_SEAL_TOKEN_RE = re.compile(r"\b__RUSTCONV_(?:CALL|FACT)_[0-9]{4}__\b")

# The contract text ends in a bare enumeration of the tokens it minted:
#   "Call tokens (11): __RUSTCONV_CALL_0000__, __RUSTCONV_CALL_0001__, ..."
# That enumeration is fully DERIVED from (body, seal rules), and the body is already
# in the cache key — so all it contributes is "the seal rules changed", at the price
# of voiding every banked entry in the file whenever they do. Fixing one macro
# exclusion would have re-billed 6,800 lines of accepted, already-unsealed Rust.
# Normalizing it out leaves the part that is a real capability claim: WHICH envelopes
# were active, and the provided-RAII API text a pre-capability output could not have
# used. Seal-set drift is judged separately, and directionally — see `_sealed_ok`.
_TOKEN_ENUMERATION_RE = re.compile(r"(?m)^((?:Call t|T)okens) \(\d+\): .*$")

# The seal envelopes are DETECTORS, not capabilities: their prose tells a worker how
# to satisfy a check whose verdict is binary, and a banked entry is by construction
# one that passed. Its names and values were restored by rustconv at unseal time, not
# written by the model, so rewording the check cannot change a stored byte — while
# hashing that prose re-bills the entire repo for an edit as small as telling `goto`
# bodies that repeating a token is expected. What WAS hidden from the model is
# carried precisely by `sealed`; that is the part with something to prove.
_ENVELOPE_BODY_RE = re.compile(r"(?ms)^(## Deterministic (?:fact|call) envelope)\n.*?(?=\n## |\Z)")


def _capability(contract: str) -> str:
    collapsed = _ENVELOPE_BODY_RE.sub(r"\1 <verification prose>", contract or "")
    return _TOKEN_ENUMERATION_RE.sub(r"\1: <enumeration>", collapsed)


# Every contract block is a `## <title>` section. The two seal envelopes are pure
# verification — they hide names from the model and prove they came back — so an
# entry banked under them carries no capability debt. Any OTHER section (today the
# provided-RAII lowering) hands the worker an API it was required to use, and an
# entry predating it is stale by construction.
_ENVELOPE_SECTIONS = frozenset({"Deterministic fact envelope", "Deterministic call envelope"})


def _capability_titles(contract: str) -> list[str]:
    """The non-envelope section titles — the parts that grant something."""
    return sorted(
        title.strip()
        for title in re.findall(r"(?m)^## (.+)$", contract or "")
        if title.strip() not in _ENVELOPE_SECTIONS
    )


def _envelope_only(contract: str) -> bool:
    return not _capability_titles(contract)


def _legacy_contract_ok(stored_hash: str, contract: str) -> bool:
    """Whether a pre-normalization entry may still be reused.

    Entries banked before the enumeration was normalized out stamped a hash of the
    FULL contract text, so the new capability hash cannot match and the old text
    cannot be reconstructed — every one of them would be re-billed the first time a
    seal rule changed, which is exactly the outcome normalization exists to prevent.

    They are recoverable without weakening the guard, because what the hash was
    protecting is knowable from the current contract alone: if this run's contract is
    nothing but seal envelopes, there is no capability the stored output could be
    missing. A non-empty stored hash additionally proves the entry came from a SEALED
    run (an unsealed one stamped `_hash("")`), so its names went through the model
    guarded.

    The converse — a stored entry that DID carry a capability, replayed into an
    envelope-only run whose output would then reference a type nobody splices — is
    structurally unreachable: the `--local-raii` preflight refuses to translate a
    function with a body-local RAII type unless its lowering is supplied, so an
    envelope-only contract for such a function never gets built at all.

    Applies to any entry without a ``caps`` key, which includes the intermediate
    stampings this rule went through while it was being worked out.
    """
    # Exact agreement with the OLD un-normalized stamping needs no reasoning at all,
    # and is the common case when nothing about the contract moved — including the
    # unsealed-run entry whose contract was, and still is, "".
    if stored_hash == _hash(contract or ""):
        return True
    return bool(stored_hash) and stored_hash != _hash("") and _envelope_only(contract)


def _sealed_ok(stored: list | None, current: frozenset[str] | None) -> bool:
    """True when the stored entry was sealed at least as tightly as this run wants.

    Superset, not equality. Sealing hides a name or value from the model and the
    envelope check proves it came back; the stored Rust was already unsealed with
    rustconv's own names, so an entry sealed MORE tightly than the current run needs
    is still sound — every name this run would protect was protected then. The
    reverse is not: an entry sealed LESS tightly let some name through the model
    unguarded, which is the whole failure sealing exists to catch, so it misses.
    """
    if current is None:
        return True
    if stored is None:
        return True  # pre-superset entry; capability hash already agreed
    return current <= set(stored)


class ResponseCache:
    """One directory per translated file; one .rs per function + a manifest."""

    def __init__(self, key: str):
        self.dir = RESPONSES_DIR / _safe_filename(key)
        self.manifest_path = self.dir / "manifest.json"
        self._manifest: dict[str, dict] = {}
        if self.manifest_path.exists():
            try:
                self._manifest = json.loads(self.manifest_path.read_text())
            except (OSError, ValueError):
                self._manifest = {}

    def get(
        self,
        name: str,
        body_text: str,
        contract: str = "",
        sealed: frozenset[str] | None = None,
    ) -> str | None:
        """Cached Rust for ``name`` IFF the C body hash still matches, else None.

        An EMPTY (or whitespace-only) cached file is treated as a MISS so that a
        previously-poisoned entry (an empty/truncated translation that an older
        pipeline wrongly cached as success) self-heals: the next run re-translates
        it instead of silently reusing nothing. See `put`.

        ``contract`` is the deterministic-contract text the worker prompt carried
        (fact/call seals, provided-RAII API). A stored entry whose CAPABILITY hash
        differs is a MISS — a pre-capability output must not replay after the
        sealed contract changed (the CDrvDefExt --local-raii stale-replay bug).
        The bare token enumeration inside that text is normalized out first and
        judged as ``sealed`` instead; see `_capability` and `_sealed_ok`.
        Entries written before contract stamping (no "contract" key) still hit,
        so previously banked paid work keeps its value.
        """
        entry = self._manifest.get(name)
        if not entry or entry.get("hash") != _hash(body_text):
            return None
        # `caps` marks an entry stamped under the settled rule; only those are held to
        # an exact capability hash. Anything older stamped a different normalization of
        # a text that cannot be reconstructed, so it is judged by what the CURRENT
        # contract can prove instead — see `_legacy_contract_ok`.
        if "caps" in entry:
            if entry["contract"] != _hash(_capability(contract)):
                return None
        elif "contract" in entry and not _legacy_contract_ok(entry["contract"], contract):
            return None
        if not _sealed_ok(entry.get("sealed"), sealed):
            return None
        f = self.dir / entry.get("file", "")
        if not f.exists():
            return None
        try:
            text = f.read_text()
        except OSError:
            return None
        if not text.strip():
            return None
        if _SEAL_TOKEN_RE.search(text):
            return None  # poisoned by a pre-fix run: re-translate instead of replaying
        return text

    def refuse_repurchase(
        self,
        name: str,
        *,
        allow_rejected: bool = False,
        failed_audit: bool = False,
        body_text: str | None = None,
        explicit: bool = False,
    ) -> None:
        """A rejected or incompatible purchase is a repair task, not a fresh cache miss.

        `allow_rejected` (translate --rebuy-rejected) lets a name whose ONLY banked state is
        a body the deterministic audit REJECTED be bought again: that reject was inspected
        and the model was wrong, so a re-roll is the repair. An accepted body is never
        re-bought this way.
        """
        if explicit:
            # translate --rebuy-fn NAME: the caller named this body. The usual case is a seal
            # that got stricter (setuplib's `L"AddReg.NT" INF_ARCH` became one sealed token),
            # so the banked body answers a contract nobody sends any more. Named, never
            # inferred: a contract-format change would otherwise re-buy the whole bank.
            print(f"  {name}: re-buying by request (--rebuy-fn)", file=sys.stderr)
            return
        accepted = name in self._manifest or (self.dir / (_safe_filename(name) + ".rs")).exists()
        entry = self._manifest.get(name)
        if body_text is not None and entry and entry.get("hash") != _hash(body_text):
            # The C INPUT changed since the purchase -- the bought body answers a question
            # nobody is asking any more. setuplib's SetupMoveFile was bought from a body the
            # scanner had cut at `_SEH2_TRY`; once the scanner saw the whole function the
            # guard refused to buy it, which would have kept the truncated translation.
            print(
                f"  {name}: C body changed since its purchase (banked for a different "
                "input); buying it for the current source",
                file=sys.stderr,
            )
            return
        # `failed_audit`: the accepted body fails a check added since it was bought, so it is
        # a reject in all but storage. Its original response stays in the run archive.
        if allow_rejected and (not accepted or failed_audit):
            return
        if accepted or self.get_rejected(name) is not None:
            raise ValueError(
                f"{self.dir}: {name} already has purchased output which did not replay. "
                "Refusing to buy it again automatically; inspect/repair the banked body "
                "and its audit first. If the C is unchanged but a seal got stricter since the "
                f"purchase, re-buy just this body with `--rebuy-fn {name}`; --no-resume is an "
                "explicit paid regeneration of everything."
            )

    def put(
        self,
        name: str,
        body_text: str,
        rust: str,
        contract: str = "",
        sealed: frozenset[str] | None = None,
    ) -> None:
        """Persist one function's Rust + record its body hash. Atomic-ish.

        Refuses to cache EMPTY/whitespace-only output: an empty translation is a
        failure, and caching it as a valid entry would make resume skip the unit
        forever (silently dropping it from the assembled file). Callers should
        only reach here on success, but this is the durable last line of defence.
        """
        if not rust.strip():
            raise ValueError(
                f"refusing to cache empty translation for {name!r} "
                f"(empty output is a failure, not a result)"
            )
        leaked = sorted(set(_SEAL_TOKEN_RE.findall(rust)))
        if leaked:
            raise ValueError(
                f"refusing to cache still-sealed translation for {name!r}: "
                f"{', '.join(leaked[:6])}"
                f"{' ...' if len(leaked) > 6 else ''}. The response was never unsealed — "
                f"caching it would replay un-unsealed Rust for $0 on every later run."
            )
        self.dir.mkdir(parents=True, exist_ok=True)
        fname = _safe_filename(name) + ".rs"
        (self.dir / fname).write_text(rust)
        self._manifest[name] = {
            "file": fname,
            "hash": _hash(body_text),
            "contract": _hash(_capability(contract)),
            "caps": _capability_titles(contract),
            **({"sealed": sorted(sealed)} if sealed is not None else {}),
        }
        tmp = self.manifest_path.with_suffix(".json.tmp")
        tmp.write_text(json.dumps(self._manifest, indent=1, sort_keys=True))
        tmp.replace(self.manifest_path)

    def put_rejected(self, name: str, rust: str, error: str) -> None:
        """Keep a REJECTED worker reply for forensics. Never replayed, never assembled.

        A reject is paid artifact too — one API call bought it — and until this
        existed the tool printed the audit verdict and dropped the evidence on the
        floor. That is the wrong half to keep: the verdict says "the envelope broke",
        the reply says HOW, and diagnosing a seal defect without the reply is
        guesswork. Seven shell32 TUs went unassembled with 226 functions banked and
        34 rejects unexplainable, which is what this is for.

        Deliberately NOT in the manifest and under a `__rejected__/` subdirectory, so
        `get` can never return one: a reply that failed the envelope must not become
        a $0 cache hit later. Overwritten each run; the newest reject is the useful
        one. Best-effort — a forensic write must never fail a translation run.

        `get_rejected` reads it back, and that is not a hole in the rule above: what it
        offers is re-AUDITED before use, through the same envelope the reply failed. A
        reject is paid artifact — an API call bought it — and once a seal defect is fixed
        (or a human repairs the one line the model would not), paying again for a body
        already on disk is waste, not caution.
        """
        try:
            out = self.dir / "__rejected__"
            out.mkdir(parents=True, exist_ok=True)
            (out / (_safe_filename(name) + ".rs")).write_text(
                f"// REJECTED by the deterministic audit — forensic copy, never assembled.\n"
                f"// {error}\n\n{rust}"
            )
        except OSError:
            pass

    def get_rejected(self, name: str) -> str | None:
        """The stored rejected reply for ``name``, forensic header stripped.

        Never trusted on its own — see `put_rejected`. The caller must put it through the
        full audit and bank it only if it passes.
        """
        path = self.dir / "__rejected__" / (_safe_filename(name) + ".rs")
        try:
            text = path.read_text()
        except OSError:
            return None
        body = "\n".join(line for line in text.splitlines() if not line.startswith("// "))
        return body if body.strip() else None

    def count(self) -> int:
        return len(self._manifest)


class ExactBodyTranslationMemory:
    """Reuse an accepted C++ worker for an exact behavior clone.

    This is intentionally narrower than the alpha-normalized duplication census.
    The key includes the method/signature, deterministic class shape, sealing
    namespace, and the prepared behavior after the declarator.  Reuse only
    substitutes the proven owning-class identifier; normal body audit still runs
    before the caller promotes the result into its ordinary response cache.
    """

    def __init__(self, root: Path | None = None):
        self.dir = root or (RESPONSES_DIR / "translation-memory-exact-v1")
        self.manifest_path = self.dir / "manifest.json"
        try:
            self._manifest = json.loads(self.manifest_path.read_text())
        except (OSError, ValueError):
            self._manifest = {}

    @staticmethod
    def _identity(
        *,
        body_text: str,
        class_name: str,
        method: str,
        return_type: str,
        params: list[tuple[str, str]],
        class_shape: str,
        namespace: str,
    ) -> str:
        brace = body_text.find("{")
        behavior = body_text[brace:] if brace >= 0 else body_text
        behavior = behavior.replace(f"{class_name}_", "__RUSTCONV_CLASS___")
        behavior = re.sub(rf"\b{re.escape(class_name)}\b", "__RUSTCONV_CLASS__", behavior)
        canonical_method = (
            "__ctor__"
            if method == class_name
            else "__dtor__"
            if method == f"~{class_name}"
            else method
        )
        payload = {
            "behavior": lexical_fingerprint(behavior),
            "method": canonical_method,
            "return": return_type.replace(class_name, "__RUSTCONV_CLASS__"),
            "params": [c_type.replace(class_name, "__RUSTCONV_CLASS__") for c_type, _ in params],
            "class_shape": class_shape,
            "namespace": namespace,
        }
        return hashlib.sha256(json.dumps(payload, sort_keys=True).encode()).hexdigest()

    def get(self, **identity) -> str | None:
        key = self._identity(**identity)
        entry = self._manifest.get(key)
        if not entry:
            return None
        path = self.dir / entry.get("file", "")
        try:
            rust = path.read_text()
        except OSError:
            return None
        origin = entry.get("class")
        target = identity["class_name"]
        if not origin or not rust.strip():
            return None
        rust = rust.replace(f"{origin}_", f"{target}_")
        return re.sub(rf"\b{re.escape(origin)}\b", target, rust)

    def put(self, rust: str, **identity) -> None:
        if not rust.strip():
            raise ValueError("refusing to remember empty translation")
        key = self._identity(**identity)
        self.dir.mkdir(parents=True, exist_ok=True)
        filename = f"{key}.rs"
        (self.dir / filename).write_text(rust)
        self._manifest[key] = {"file": filename, "class": identity["class_name"]}
        tmp = self.manifest_path.with_suffix(".json.tmp")
        tmp.write_text(json.dumps(self._manifest, indent=1, sort_keys=True))
        tmp.replace(self.manifest_path)
