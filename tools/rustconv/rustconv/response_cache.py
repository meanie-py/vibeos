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
import re
from pathlib import Path

from .outcome_ledger import lexical_fingerprint

# tools/rustconv/.responses/  (parent.parent = the rustconv package's parent dir)
RESPONSES_DIR = Path(__file__).resolve().parent.parent / ".responses"

PREAMBLE_KEY = "__preamble__"


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

    def get(self, name: str, body_text: str, contract: str = "") -> str | None:
        """Cached Rust for ``name`` IFF the C body hash still matches, else None.

        An EMPTY (or whitespace-only) cached file is treated as a MISS so that a
        previously-poisoned entry (an empty/truncated translation that an older
        pipeline wrongly cached as success) self-heals: the next run re-translates
        it instead of silently reusing nothing. See `put`.

        ``contract`` is the deterministic-contract text the worker prompt carried
        (fact/call seals, provided-RAII API). A stored entry whose contract hash
        differs is a MISS — a pre-capability output must not replay after the
        sealed contract changed (the CDrvDefExt --local-raii stale-replay bug).
        Entries written before contract stamping (no "contract" key) still hit,
        so previously banked paid work keeps its value.
        """
        entry = self._manifest.get(name)
        if not entry or entry.get("hash") != _hash(body_text):
            return None
        if "contract" in entry and entry["contract"] != _hash(contract or ""):
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

    def put(self, name: str, body_text: str, rust: str, contract: str = "") -> None:
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
            "contract": _hash(contract or ""),
        }
        tmp = self.manifest_path.with_suffix(".json.tmp")
        tmp.write_text(json.dumps(self._manifest, indent=1, sort_keys=True))
        tmp.replace(self.manifest_path)

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
