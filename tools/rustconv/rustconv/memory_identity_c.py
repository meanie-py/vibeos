"""Exact-body translation memory for plain C — replay a paid worker for $0.

WHY THIS EXISTS
---------------
`ExactBodyTranslationMemory` (response_cache.py) already banks an accepted C++
worker and replays it for a structurally identical sibling class; on two shell
folder classes that reused 19 of 22 workers (86% fewer paid calls, ~83% less
cost).  Its identity key is built only where ``dep.cpp_class is not None``
(__main__.py ``_memory_identity``), so **plain C never reaches it** and rustconv
re-pays for byte-identical bodies.  Stub-heavy C is exactly where that hurts:
`drivers/network/ndis` alone has a 10-function group whose bodies are all
``{ UNIMPLEMENTED; return NDIS_STATUS_FAILURE; }``.

This module is the C-side identity key plus the memory that uses it.

DESIGN LAW: A MISS IS FREE, A FALSE HIT IS CATASTROPHIC
------------------------------------------------------
A miss costs one API call we were going to make anyway.  A false hit ships the
wrong function body with **no cost signal and no review prompt** — it looks
like a saving.  So every ambiguity here resolves toward MISS: an AST extraction
that fell back, a module-scope name whose definition cannot be isolated, a
rename whose origin cannot be found, a collision with an existing symbol — all
refuse rather than guess.

EXACT ONLY, NEVER ALPHA
-----------------------
`dup_census` deliberately treats alpha-renamed (identifier-renamed) matches as
census *evidence* and never as a reuse source, because scope, signature,
substrate and identifier re-instantiation are unproven.  This module honors
that: the behavior component is the EXACT prepared body (via the same
`dup_census.behavior_source` extraction, so census and memory agree on what
"the body" is), and the only identifier ever substituted is the function's own
name — the direct analogue of the sanctioned owning-class substitution on the
C++ path.  **Parameter names are part of the key**, not renamed: the C path has
no deterministic signature re-imposer (`normalize_worker_body` is cpp_com-only),
so a body whose params happen to be spelled differently must MISS.

THE KEY COMPONENTS, AND WHY EACH ONE CHANGES THE CORRECT OUTPUT
--------------------------------------------------------------
``schema``
    Key-format version.  Bumping it retires every banked entry instead of
    letting a stale entry answer a question the new key asks differently.
``behavior`` / ``behavior_complete``
    Lexical fingerprint of the source *after the declarator* with the
    function's own name canonicalized to `SELF_SENTINEL`.  This IS the thing
    being reused.  `lexical_fingerprint` drops comments and insignificant
    whitespace but keeps literal bytes and every code token, so a body
    differing by one constant is a different behavior.  ``behavior_complete``
    is False when tree-sitter could not isolate the declarator; then we do not
    know where the behavior starts, so the identity is unusable (never stored,
    never served).
``return_type`` / ``param_types`` / ``param_names``
    The emitted Rust `fn` signature is a pure function of these.  Param names
    are included because nothing re-imposes the target's spelling (see above).
``calling_convention`` + ``arches``
    The correct `extern "…"` string is a function of BOTH.  `extern "system"`
    is stdcall on i386 and cdecl on amd64/arm64, and i386 `FASTCALL` is
    `extern "fastcall"`; a crate built for one arch can also need a different
    `#[cfg]` shape than a 3-arch crate.  Convention alone is not enough.
``is_static`` / ``is_exported`` / ``export_convention``
    Decide the item shape: a private `fn` versus
    `#[no_mangle] pub unsafe extern "…"`, and which decorated symbol the
    linker must see.
``substrate``
    The target crate's available import surface for THIS body: the resolved
    `(module, symbol)` pairs, the gap-function set, the **unresolved** set, the
    target's substrate routing (`sys_crate` / `sys_umbrella` / `crate_module`),
    and — for names routed to the crate-local bindgen island — the island's
    own definition text.  This is what makes "a body referencing a type the
    other crate does not expose must MISS" true: in that crate the name lands
    in a different module or in `unresolved`, and the digest changes.  The
    unresolved set matters on its own — an unresolved name is one the model
    was told to invent a declaration for, so two crates with different
    unresolved sets received different prompts.
``callees``
    Per-callee resolution, including the C signature of a same-file callee.
    Two byte-identical bodies can still need different Rust when a callee is a
    crate import in one module and a local function with a different signature
    in the other.
``preamble``
    The C analogue of the C++ ``class_shape``: for every module-scope name the
    body references, the module-scope text that mentions it.  `#define X 5` and
    `#define X 6` are different substrate under the same name, and a file-scope
    `static HINSTANCE hDllInstance;` is different substrate from a
    `static HMODULE hDllInstance;`.  A hinted name we cannot isolate is bound to
    the digest of the WHOLE preamble, which keeps same-module reuse working
    while forcing a cross-module miss.  Two calibrations that matter (both were
    real false-separator bugs found while measuring this against the tree):
    the projected set is names DEFINED at module scope, not merely mentioned
    (`module_scope_names`), and the function's OWN name is excluded, because it
    routinely appears in a module-scope vtable (`substrate_digest`).
``contract``
    Hash of the deterministic contract text (fact seal / call seal / provided
    RAII).  A sealed prompt and an unsealed prompt are different prompts with
    different accepted outputs — this is the guard `ResponseCache.get` already
    carries for the stale-replay bug.
``namespace``
    The pipeline preparation that shapes the output: mode, cache tag, seal
    flags, whether a c2rust anchor was supplied, whether debug statements were
    stripped.  An anchored translation must not answer for an unanchored run.

A HIT IS A CANDIDATE, NOT AN ACCEPTANCE
---------------------------------------
`CExactBodyTranslationMemory.candidate()` never hands back Rust.  It returns a
`CMemoryCandidate` whose only way out is `accept(audit)`, mirroring the C++
path where a memory hit is re-normalized and re-audited before it is promoted
into the ordinary response cache.  There is no attribute holding the text, so
the integrator cannot accidentally bypass the gate.
"""

from __future__ import annotations

import hashlib
import json
import re
from collections.abc import Callable, Collection, Iterable, Mapping, Sequence
from dataclasses import dataclass, field
from pathlib import Path

from .config import Target
from .dup_census import behavior_source
from .outcome_ledger import lexical_fingerprint
from .resolver import ResolveResult
from .response_cache import RESPONSES_DIR
from .scanner import CFunction

SCHEMA = "c-exact-v1"

#: The token the function's own name is canonicalized to inside the behavior.
SELF_SENTINEL = "__RUSTCONV_SELF__"

#: Every field of `CIdentity.payload()`. Tests assert each one is load-bearing.
KEY_COMPONENTS = (
    "schema",
    "behavior",
    "return_type",
    "param_types",
    "param_names",
    "calling_convention",
    "arches",
    "is_static",
    "is_exported",
    "export_convention",
    "substrate",
    "callees",
    "preamble",
    "contract",
    "namespace",
)

_COMMENT_OR_LITERAL = re.compile(
    r'/\*.*?\*/|//[^\n]*|"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'',
    re.S,
)
_OPENERS = "{(["
_CLOSERS = "})]"


class CMemoryError(RuntimeError):
    """The C translation memory was asked to do something it cannot prove."""


def _sha(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8", "replace")).hexdigest()


def _digest(payload: object) -> str:
    return _sha(json.dumps(payload, sort_keys=True, default=str))


def _collapse(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


def _mask(source: str) -> str:
    """Blank comments and string/char literals, preserving every offset."""

    def blank(match: re.Match[str]) -> str:
        return "".join("\n" if char == "\n" else " " for char in match.group(0))

    return _COMMENT_OR_LITERAL.sub(blank, source)


def top_level_chunks(source: str) -> list[str]:
    """Split module-scope source into brace/semicolon-delimited items.

    Deliberately syntax-light so it serves both a C preamble and a generated
    Rust bindgen island.  Over-inclusion (a chunk carrying more context than
    the one name) only makes the projection stricter, which is the safe
    direction; under-inclusion falls back to the whole-source digest.
    """
    masked = _mask(source)
    chunks: list[str] = []
    depth = 0
    start = 0
    for index, char in enumerate(masked):
        if char in _OPENERS:
            depth += 1
        elif char in _CLOSERS:
            depth = max(0, depth - 1)
            if depth == 0 and char == "}":
                chunks.append(source[start : index + 1])
                start = index + 1
        elif depth == 0 and char == ";":
            chunks.append(source[start : index + 1])
            start = index + 1
        elif depth == 0 and char == "\n":
            pending = masked[start:index]
            if pending.lstrip().startswith("#") and not pending.rstrip().endswith("\\"):
                chunks.append(source[start : index + 1])
                start = index + 1
    tail = source[start:]
    if tail.strip():
        chunks.append(tail)
    return chunks


def _defines_name(chunk: str, name: str) -> bool:
    """Does this module-scope chunk DEFINE ``name`` (rather than just use it)?"""
    escaped = re.escape(name)
    return any(
        re.search(pattern, chunk) is not None
        for pattern in (
            rf"#\s*define\s+{escaped}\b",  # #define NAME / NAME(args)
            rf"\b(?:struct|union|enum|typedef)\b[^;{{]*\b{escaped}\b",  # tag / typedef name
            # declarator or enumerator: obj; obj[8]; obj = …; LAST_ENUMERATOR }
            rf"\b{escaped}\b\s*(?:\[[^\]]*\])*\s*(?:=|;|,|\}})",
        )
    )


def module_scope_names(source: str, identifiers: Iterable[str]) -> set[str]:
    """Which referenced identifiers are DEFINED at module scope in ``source``.

    Derived from the TEXT on purpose.  `ScanResult.globals` is empty for plenty
    of real files (rasdlg.c / rasman.c / netid.c each declare a file-scope
    ``static HINSTANCE hDllInstance;`` that never reaches it), so a projection
    driven by ``scan.globals`` would silently see nothing and drop the substrate
    component to ``{}`` — a false-hit vector dressed up as a safety check.

    "Defined", not merely "mentioned": a body referencing `HINSTANCE` or `BOOL`
    makes those names *appear* in the preamble (they are used in declarations
    there), and projecting them would drag unrelated module-scope declarations
    into the key.  That is safe but arbitrarily strict — it made two identical
    `DllMain` bodies miss over a neighbouring `BOOL` field — and an arbitrary
    key is a key nobody can reason about.  SDK type names belong to the import
    surface, which the `substrate` component already covers.
    """
    chunks = [_mask(chunk) for chunk in top_level_chunks(source)]
    return {
        name for name in identifiers if name and any(_defines_name(chunk, name) for chunk in chunks)
    }


def module_scope_projection(source: str, names: Iterable[str]) -> dict[str, str]:
    """Fingerprint the module-scope shape of each name the body references.

    A name we can isolate maps to the fingerprint of every module-scope chunk
    mentioning it — so a differing `#define` value, field type, or initializer
    changes the projection.  A name we CANNOT isolate maps to the digest of the
    whole source instead of to a shared "unknown" marker: same-module reuse
    still matches (identical source), while two different modules can never
    agree.  That is the fail-safe; there is no third outcome where we guess.
    """
    wanted = sorted({name for name in names if name})
    if not wanted:
        return {}
    fallback = f"unlocated@{_sha(source)[:16]}"
    chunks = top_level_chunks(source)
    masked_chunks = [(chunk, _mask(chunk)) for chunk in chunks]
    projection: dict[str, str] = {}
    for name in wanted:
        pattern = re.compile(rf"\b{re.escape(name)}\b")
        mentions = [chunk for chunk, masked in masked_chunks if pattern.search(masked)]
        projection[name] = lexical_fingerprint("\n".join(mentions)) if mentions else fallback
    return projection


def canonical_behavior(body_text: str, self_name: str) -> tuple[str, bool]:
    """Fingerprint the source after the declarator, self-name canonicalized.

    Returns ``(fingerprint, complete)``.  ``complete`` is False when
    tree-sitter could not isolate the declarator — the caller must then treat
    the identity as unusable rather than fingerprint a guessed span.
    """
    probe = CFunction(
        name=self_name,
        return_type="",
        calling_convention=None,
        params=[],
        is_static=False,
        code_seg=None,
        seh_count=0,
        line_start=0,
        line_end=0,
        body_text=body_text,
    )
    behavior, complete = behavior_source(probe, ".c")
    if self_name:
        behavior = re.sub(rf"\b{re.escape(self_name)}\b", SELF_SENTINEL, behavior)
    return lexical_fingerprint(behavior), complete


def preparation_namespace(
    *,
    mode: str,
    cache_tag: str | None = None,
    fact_sealed: bool = False,
    call_sealed: bool = False,
    wide_sealed: bool = False,
    c2rust_anchored: bool = False,
    debug_stripped: bool = False,
) -> str:
    """Stable name for the prompt-shaping pipeline state (see module docstring).

    Each flag contributes an INDEPENDENT token, which is what makes a new protection
    free to add: `--seal-wide` appends `wideseal_v1` and leaves every already-banked
    `mechanical__factseal_v1__callseal_v1` entry addressable exactly as it was. Adding a
    protection to an EXISTING token's meaning would instead re-bill every paid body under
    it — see fact_seal's frozen `cpp` dialect for what that costs.
    """
    parts = [mode or "default"]
    if cache_tag:
        parts.append(cache_tag)
    if fact_sealed:
        parts.append("factseal_v1")
    if call_sealed:
        parts.append("callseal_v1")
    if wide_sealed:
        parts.append("wideseal_v1")
    if c2rust_anchored:
        parts.append("c2rust")
    if debug_stripped:
        parts.append("nodebug")
    return "__".join(parts)


def import_pairs(resolution: ResolveResult) -> list[tuple[str, str]]:
    """Sorted ``(module, symbol)`` pairs the body resolved to."""
    return sorted(
        (module, symbol.name)
        for module, group in resolution.imports.items()
        for symbol in group.symbols
    )


def callee_resolutions(
    *,
    callees: Iterable[str],
    resolution: ResolveResult,
    local_signatures: Mapping[str, str],
) -> dict[str, str]:
    """Label how each callee resolves, so a same-name/different-thing MISSes.

    A same-file callee carries its C signature: a local `Helper` returning
    NTSTATUS in one module and VOID in another needs different Rust from a
    byte-identical caller body.  C scoping shadows imports, so a local wins.
    """
    modules = {symbol: module for module, symbol in import_pairs(resolution)}
    labels: dict[str, str] = {}
    for callee in sorted({name for name in callees if name}):
        if callee in local_signatures:
            labels[callee] = f"local:{_collapse(local_signatures[callee])}"
        elif callee in modules:
            labels[callee] = f"import:{modules[callee]}"
        elif callee in resolution.gap_functions:
            labels[callee] = "gap"
        elif callee in resolution.unresolved:
            labels[callee] = "unresolved"
        else:
            labels[callee] = "builtin"
    return labels


def substrate_digest(
    *,
    target: Target,
    self_name: str,
    identifiers: Collection[str],
    resolution: ResolveResult,
    preamble_text: str,
    preamble_names: Collection[str] = (),
    island_source: str = "",
) -> tuple[str, str]:
    """Digest the target crate's import surface + island shape for this body.

    Returns ``(substrate, preamble)`` — two separate key components so a test
    can prove each is load-bearing, and so a diagnostic can say which one
    forced a miss.

    The projected name set is DERIVED from ``preamble_text`` (see
    `module_scope_names`).  ``preamble_names`` is an optional additive hint for
    names the caller knows are module-scope (``scan.defines`` is a good one); a
    hinted name absent from the text falls back to the whole-preamble digest,
    so a hint can only ever tighten the key.

    ``self_name`` is excluded from the projection for the same reason
    `canonical_behavior` canonicalizes it away: a function's own name routinely
    appears at module scope in a COM vtable or dispatch table, and projecting
    it makes every such function self-separate — three `{ return E_NOTIMPL; }`
    netcfgx vtbl entries could never match each other.  The function's own
    module-scope declaration is what the memory RE-INSTANTIATES, not substrate
    the body depends on.
    """
    referenced = {name for name in identifiers if name and name != self_name}
    projected = module_scope_names(preamble_text, referenced) | (
        referenced & {name for name in preamble_names if name}
    )
    pairs = import_pairs(resolution)
    island_names = [symbol for module, symbol in pairs if module == target.crate_module]
    substrate = _digest(
        {
            "target": target.name,
            "sys_crate": target.sys_crate,
            "sys_umbrella": target.sys_umbrella,
            "crate_module": target.crate_module,
            "imports": pairs,
            "gaps": sorted(resolution.gap_functions),
            "unresolved": sorted(resolution.unresolved),
            "island": module_scope_projection(
                island_source, [name for name in island_names if name in referenced]
            ),
        }
    )
    preamble = _digest(module_scope_projection(preamble_text, projected))
    return substrate, preamble


@dataclass(frozen=True)
class CIdentity:
    """Everything that changes the correct Rust for one C function body.

    ``self_name`` is deliberately NOT part of `payload()` — it is what the
    memory re-instantiates, exactly as the C++ path re-instantiates the owning
    class name.  Everything else is in the key.
    """

    behavior: str
    behavior_complete: bool
    self_name: str
    return_type: str
    param_types: tuple[str, ...]
    param_names: tuple[str, ...]
    calling_convention: str | None
    arches: tuple[str, ...]
    is_static: bool
    is_exported: bool
    export_convention: str | None
    substrate: str
    callees: str
    preamble: str
    contract: str
    namespace: str
    schema: str = SCHEMA

    @property
    def usable(self) -> bool:
        """False when the identity cannot be trusted to separate two bodies."""
        return bool(self.behavior_complete and self.self_name and self.behavior)

    def payload(self) -> dict[str, object]:
        """The key's components, one entry per `KEY_COMPONENTS` name."""
        return {
            "schema": self.schema,
            "behavior": self.behavior,
            "return_type": _collapse(self.return_type),
            "param_types": [_collapse(item) for item in self.param_types],
            "param_names": list(self.param_names),
            "calling_convention": self.calling_convention,
            "arches": sorted(self.arches),
            "is_static": self.is_static,
            "is_exported": self.is_exported,
            "export_convention": self.export_convention,
            "substrate": self.substrate,
            "callees": self.callees,
            "preamble": self.preamble,
            "contract": self.contract,
            "namespace": self.namespace,
        }

    def key(self) -> str:
        return _digest(self.payload())


def build_identity(
    *,
    dep,
    target: Target,
    resolution: ResolveResult,
    local_signatures: Mapping[str, str],
    preamble_text: str,
    namespace: str,
    preamble_names: Collection[str] = (),
    island_source: str = "",
    contract: str = "",
) -> CIdentity:
    """Assemble a `CIdentity` from one pipeline `FunctionDep` + its resolution.

    ``dep`` is a `callgraph.FunctionDep`; it is untyped here so this module
    stays importable without the call-graph machinery (and so the C++ path's
    `replace()`-derived variants pass through unchanged).
    """
    behavior, complete = canonical_behavior(dep.body_text, dep.name)
    export = resolution.exports.get(dep.name)
    substrate, preamble = substrate_digest(
        target=target,
        self_name=dep.name,
        identifiers=dep.identifiers,
        resolution=resolution,
        preamble_text=preamble_text,
        preamble_names=preamble_names,
        island_source=island_source,
    )
    return CIdentity(
        behavior=behavior,
        behavior_complete=complete,
        self_name=dep.name,
        return_type=dep.return_type,
        param_types=tuple(c_type for c_type, _ in dep.params),
        param_names=tuple(name for _, name in dep.params),
        calling_convention=dep.calling_convention,
        arches=tuple(target.arches),
        is_static=bool(dep.is_static),
        is_exported=bool(dep.is_exported),
        export_convention=export.convention if export is not None else None,
        substrate=substrate,
        callees=_digest(
            callee_resolutions(
                callees=dep.callees,
                resolution=resolution,
                local_signatures=local_signatures,
            )
        ),
        preamble=preamble,
        contract=_sha(contract or ""),
        namespace=namespace,
    )


def reinstantiate(rust: str, origin: str, target: str) -> tuple[str | None, str]:
    """Rename the banked worker's own symbol, or refuse.

    Returns ``(rust, reason)``; ``rust`` is None when the rename cannot be
    proven, with ``reason`` naming the guard that fired:

    * the banked text does not mention ``origin`` as a whole word — we would
      emit a function still named after the wrong C function;
    * the banked text already mentions ``target`` as a whole word — renaming
      would fuse two distinct symbols into one.

    Both are silent-wrong-code shapes, so both resolve to MISS.
    """
    if not origin or not rust.strip():
        return None, "empty origin or banked text"
    if origin == target:
        return rust, ""
    boundary = re.compile(rf"\b{re.escape(origin)}\b")
    if not boundary.search(rust):
        return None, f"banked worker never names its origin {origin!r}"
    if re.search(rf"\b{re.escape(target)}\b", rust):
        return None, f"banked worker already names the rename target {target!r}"
    return boundary.sub(target, rust), ""


@dataclass(frozen=True)
class MemoryAcceptance:
    """Outcome of gating a memory hit. ``rust`` is None iff it was rejected."""

    rust: str | None
    findings: tuple[str, ...]

    @property
    def accepted(self) -> bool:
        return self.rust is not None


@dataclass(frozen=True)
class CMemoryCandidate:
    """A banked worker that MIGHT fit — reachable only through `accept`.

    There is intentionally no attribute holding the Rust text: the promotion
    path runs the caller's normalize+audit closure or it gets nothing.
    """

    key: str
    origin: str
    target: str
    _load: Callable[[], str | None] = field(repr=False)

    def accept(self, audit: Callable[[str], Sequence[str]]) -> MemoryAcceptance:
        """Re-instantiate, then gate through ``audit`` before releasing text.

        ``audit`` takes the candidate Rust and returns findings — empty means
        accepted.  It is the C-path equivalent of the C++ path's
        `normalize_worker_body` + `normalize_worker_semantics` +
        `audit_worker_body` chain, and it is mandatory: this is the only way
        the banked bytes leave the memory.  Exceptions from ``audit``
        propagate (fail loud) rather than degrading into a quiet miss.
        """
        if audit is None:  # pyright: ignore[reportUnnecessaryComparison]
            raise CMemoryError(
                "a memory hit is a candidate, not an acceptance: accept() requires an "
                "audit callable (normalize + verify) before banked bytes are released"
            )
        banked = self._load()
        if banked is None:
            return MemoryAcceptance(None, ("banked worker unreadable",))
        rust, reason = reinstantiate(banked, self.origin, self.target)
        if rust is None:
            return MemoryAcceptance(None, (reason,))
        findings = tuple(audit(rust))
        return MemoryAcceptance(None if findings else rust, findings)


class CExactBodyTranslationMemory:
    """Cross-module exact-body memory for plain C translations.

    Same on-disk shape as `ExactBodyTranslationMemory` (one `.rs` per key plus
    a manifest), separate directory so the C and C++ key spaces can never
    answer each other's questions.
    """

    def __init__(self, root: Path | None = None):
        self.dir = root or (RESPONSES_DIR / f"translation-memory-{SCHEMA}")
        self.manifest_path = self.dir / "manifest.json"
        try:
            self._manifest: dict[str, dict] = json.loads(self.manifest_path.read_text())
        except (OSError, ValueError):
            self._manifest = {}

    def count(self) -> int:
        return len(self._manifest)

    def candidate(self, identity: CIdentity) -> CMemoryCandidate | None:
        """A gated candidate for ``identity``, or None for a (free, safe) miss."""
        if not identity.usable:
            return None
        key = identity.key()
        entry = self._manifest.get(key)
        if not entry:
            return None
        origin = entry.get("self")
        filename = entry.get("file")
        if not origin or not filename:
            return None
        path = self.dir / filename

        def load() -> str | None:
            try:
                text = path.read_text()
            except OSError:
                return None
            return text if text.strip() else None

        return CMemoryCandidate(key=key, origin=origin, target=identity.self_name, _load=load)

    def remember(self, rust: str, identity: CIdentity) -> bool:
        """Bank an ACCEPTED worker. False when the identity is unusable.

        Returns False (rather than raising) for an unusable identity because
        banking is opportunistic — the run must not fail over a cache write —
        but the boolean is there so the caller can log the skip instead of
        assuming it landed.  An empty body IS a programming error and raises.
        """
        if not rust.strip():
            raise ValueError("refusing to remember empty translation")
        if not identity.usable:
            return False
        key = identity.key()
        self.dir.mkdir(parents=True, exist_ok=True)
        filename = f"{key}.rs"
        (self.dir / filename).write_text(rust)
        self._manifest[key] = {"file": filename, "self": identity.self_name}
        tmp = self.manifest_path.with_suffix(".json.tmp")
        tmp.write_text(json.dumps(self._manifest, indent=1, sort_keys=True))
        tmp.replace(self.manifest_path)
        return True
