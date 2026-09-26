"""bank_audit — is every function we PAID for actually in the tree?

`.responses/` is the canonical copy of paid translation output, and the money rule says
a dir holding unintegrated work must never be deleted. That rule needs an oracle, and
until now there wasn't one: the only way to answer "what have we paid for and not
shipped?" was to hand-roll a script, re-derive the two on-disk layouts, and re-discover
the same false positives every time.

The audit runs BACKWARD, which is what makes it different from its neighbours.
`export_audit` goes spec-exports → Rust ("does the DLL still export what it promised?").
`fn_coverage` goes C-source → Rust. This one goes **paid output → tree**: for every
function we bought, is it actually shipping? A gap here is money spent and abandoned,
and unlike a missing export nothing downstream complains — the build is perfectly happy
to link a DLL whose translated helper was silently dropped on the way into the crate.

Two on-disk layouts, both live:

    <dir>/manifest.json          {"FnName": {"file": "FnName.rs", "hash": "..."}}
    <dir>/NN_FnName.rs           legacy, no manifest (the ole32/oleaut32 era)

RESOLUTION IS NOT A NAME COMPARE. A first cut that just asked "is the manifest key in
the tree?" flagged 8 dirs, and *every one* was a naming artifact rather than lost work.
The manifest key is a LABEL — the name of the C construct that was sent for translation
— and four separate things routinely make it differ from the symbol that ships:

  declared_in_body  The key is a MACRO, and the body it expands to holds the real
                    functions. oleaut32's `RETTYP` banks 72 `VarI1FromUI1`-style
                    conversions under one key. The body is ground truth; the key is not.
  varargs_twin      Stable Rust cannot DEFINE a C-variadic (rust-lang/rust#44930), so a
                    variadic C entry point ships as a fixed-arity twin behind a C
                    trampoline. `WLDAP32_ber_printf` is real, shipping, and will never
                    appear under that spelling. Our own decoration is NOT uniform —
                    `FatalError_v` in most modules, `netid_FormatMessageBox_body` in
                    netid — so `_TWIN_SUFFIXES` is the list to extend, not a rule to
                    loosen.
  tchar_macro       `_tmain` is a TCHAR macro; the tree ships `wmain`. Same for
                    `_tWinMain` → `wWinMain`.
  allowlist         A reviewed exception with a written reason (see ALLOW_PATH). The one
                    standing entry is oleaut32's `interface_variant_size`: the C declares
                    it under `#else` and `interface_user_size` under `#ifdef __REACTOS__`,
                    so the ReactOS build never had a function by the banked name.

Each rule is recorded on the Resolution, so a green report still shows its work — an
audit that resolves silently is one you stop believing the first time it's wrong.

Comments and string literals are masked before any name is read, in the banked bodies and
in the tree alike. Every translated file carries the ORIGINAL C SOURCE as a trailing
comment block, so an unmasked scan indexes names that do not exist as Rust items — and a
false name in the TREE index is the dangerous direction, because it reports paid work as
shipped when it is not.

    from rustconv.bank_audit import audit
    for report in audit(responses_root, rust_root):
        if report.absent:
            ...
"""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass, field
from pathlib import Path

from tools.source_walk import source_files

from rustconv.lex_mask import mask_code

try:
    import tomllib  # type: ignore[import-not-found]  # stdlib on 3.11+; tomli backport on 3.10
except ModuleNotFoundError:  # pragma: no cover - depends on interpreter version
    import tomli as tomllib  # type: ignore[no-redef]

# `fn NAME` — the sole shape a Rust function item can take. Masked input only.
_RE_FN = re.compile(r"\bfn\s+(?P<name>[A-Za-z_]\w*)")
# legacy layout orders bodies with a numeric prefix: 07_VarI4FromStr.rs
_RE_SEQ_PREFIX = re.compile(r"^\d+_")
# A banked body that is a C-variadic: `...)` in the signature, or a va_start in the body.
_RE_VARIADIC = re.compile(r"\.\.\.\s*\)|\bva_start\b")
# How the tree decorates a variadic's fixed-arity twin. See `varargs_twins`.
# `_dispatch` is msvcrt's: `_sopen`'s trampoline in rust_file_varargs_shim.c tail-calls
# the Rust `_sopen_dispatch`.
_TWIN_SUFFIXES = ("_v", "_body", "_impl", "_dispatch")
# A macro INVOCATION and its argument region: `isc!(_isalnum_l, isalnum, ...)`.
_RE_MACRO_CALL = re.compile(r"\b[A-Za-z_]\w*!\s*\(")
# A macro argument that is nothing but an identifier — the only shape we will trust.
_RE_BARE_IDENT = re.compile(r"^[A-Za-z_]\w*$")

# Manifest keys that are pipeline bookkeeping, never a translated function.
_NON_FN_KEYS = frozenset({"preamble"})

ALLOW_PATH = Path(__file__).resolve().parent.parent / "bank_audit_allow.toml"


@dataclass(frozen=True)
class BankedFn:
    """One function we paid for, and the file its body landed in."""

    name: str
    path: Path
    truncated: bool = False


@dataclass(frozen=True)
class Resolution:
    """How a banked function was matched against the tree (or that it wasn't)."""

    fn: BankedFn
    found_as: str | None
    via: str

    @property
    def integrated(self) -> bool:
        return self.found_as is not None


@dataclass(frozen=True)
class DirReport:
    """One `.responses/<dir>`: what it holds and what of it is missing."""

    name: str
    total: int
    resolutions: list[Resolution] = field(default_factory=list)

    @property
    def absent(self) -> list[Resolution]:
        return [r for r in self.resolutions if not r.integrated]

    @property
    def aliased(self) -> list[Resolution]:
        return [r for r in self.resolutions if r.integrated and r.via != "direct"]

    @property
    def status(self) -> str:
        if not self.absent:
            return "INTEGRATED"
        return "ABSENT" if len(self.absent) == self.total else "PARTIAL"


def _is_bookkeeping(key: str) -> bool:
    """Manifest/legacy names that never denote a translated function."""
    return key.startswith("__") or key in _NON_FN_KEYS


def _has_body(masked: str, after_name: int) -> bool:
    """Does the `fn` starting here DEFINE a function, or merely declare one?

    The distinction is the whole audit. `extern "C" { pub fn Foo(); }` in a crate prelude
    is a promise that Foo exists elsewhere — counting it as a definition would report a
    function as shipped on the strength of the declaration that it is missing.

    Walks from the end of the name to the first `{` (definition) or `;` (declaration) at
    bracket depth zero, so a `;` inside a `-> [u8; 4]` return type doesn't read as one.
    """
    depth = 0
    for i in range(after_name, len(masked)):
        c = masked[i]
        if c in "([":
            depth += 1
        elif c in ")]":
            depth -= 1
        elif depth == 0:
            if c == "{":
                return True
            if c == ";":
                return False
    return False


def declared_names(path: Path) -> set[str]:
    """Every Rust function DEFINED in one file, comments and literals masked.

    Declarations (`extern` blocks, trait signatures) are deliberately excluded — see
    `_has_body`.
    """
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return set()
    masked = mask_code(text)
    return {m.group("name") for m in _RE_FN.finditer(masked) if _has_body(masked, m.end("name"))}


def tree_fn_names(rust_root: Path) -> set[str]:
    """Index every Rust function item under a crate root."""
    names: set[str] = set()
    for path in rust_root.rglob("*.rs"):
        names |= declared_names(path)
    return names


def truncated_fns(resp_dir: Path) -> list[BankedFn]:
    """`NN_Name.partial` — a call that was billed but cut off mid-stream.

    Scanned separately because the remedy differs: a truncated body cannot be integrated
    as it stands, so one whose function is absent from the tree means "this was paid for
    and never landed — re-translate it", not "go wire it up". Every `.partial` in the bank
    today belongs to a call that was retried successfully, which is exactly why the rule
    needs to exist: without it the audit is structurally blind to the one shape of paid
    output that has no complete body to look for.
    """
    out = []
    for path in sorted(resp_dir.glob("*.partial")):
        stem = _RE_SEQ_PREFIX.sub("", path.stem)
        if _is_bookkeeping(stem):
            continue
        out.append(BankedFn(stem, path, truncated=True))
    return out


def banked_fns(resp_dir: Path) -> list[BankedFn]:
    """Read one response dir in whichever of the two layouts it uses."""
    manifest = resp_dir / "manifest.json"
    if manifest.exists():
        try:
            entries = json.loads(manifest.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            return []
        out = []
        for key, meta in entries.items():
            if _is_bookkeeping(key):
                continue
            fname = meta.get("file") if isinstance(meta, dict) else None
            out.append(BankedFn(key, resp_dir / (fname or f"{key}.rs")))
        return out

    out = []
    for path in sorted(resp_dir.glob("*.rs")):
        stem = _RE_SEQ_PREFIX.sub("", path.stem)
        if _is_bookkeeping(stem):
            continue
        out.append(BankedFn(stem, path))
    return out


def is_variadic_body(path: Path) -> bool:
    """Was the banked body a C-variadic? `...` in the signature, or a `va_start` in it."""
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return False
    return _RE_VARIADIC.search(mask_code(text)) is not None


def varargs_twins(name: str, tree: set[str]) -> list[str]:
    """Tree definitions that could be `name`'s fixed-arity twin, tightest first.

    Stable Rust cannot DEFINE a C-variadic, so a variadic entry point ships as a C
    trampoline that tail-calls a fixed-arity Rust twin — and the tree spells that twin
    two ways: `FatalError_v` (most modules) and `netid_FormatMessageBox_body` (netid).

    A twin must therefore be the name plus a KNOWN decoration, optionally under a module
    prefix. A looser "contains the name" test is not usable here: dhcp banks variadics
    called `debug` and `error`, and mere containment matched them to `debug_block_stamp`
    and `WDML_Initialize_error` — unrelated functions in other crates. Short generic
    names are exactly where a sloppy rule reports missing work as shipped.
    """
    exact = [f"{name}{suffix}" for suffix in _TWIN_SUFFIXES if f"{name}{suffix}" in tree]
    # The C stdio convention: the va_list twin of `fprintf` is `vfprintf`, of
    # `_fwprintf_p_l` is `_vfwprintf_p_l`. The `v` goes after any leading underscores,
    # never at the front. This is an EXACT name, so it is as tight as the suffix rules —
    # the looseness the docstring warns about is containment, not a second spelling.
    lead = len(name) - len(name.lstrip("_"))
    v_twin = f"{name[:lead]}v{name[lead:]}"
    if v_twin in tree and v_twin not in exact:
        exact.append(v_twin)
    prefixed = sorted(
        n
        for n in tree
        if n not in exact and any(n.endswith(f"_{name}{suffix}") for suffix in _TWIN_SUFFIXES)
    )
    return exact + prefixed


def macro_arg_names(path: Path) -> set[str]:
    """Identifiers passed as bare arguments to a macro invocation in one file.

    A whole family of shipped symbols has no `fn NAME` anywhere, because a macro writes
    the items: msvcrt's ctype predicates are 24 lines of
    `isc!(_isalnum_l, isalnum, (sys::_ALPHA | sys::_DIGIT) as i32);` in lib.rs, and every
    one of them was reported as unintegrated paid work while shipping and graded MATCH.

    Kept in a SEPARATE index from `tree_fn_names`, and consulted only after every other
    rule, because the dangerous direction here is a false name in the TREE index — that
    reports paid work as shipped when it is not. Hence "bare identifier argument" and
    nothing looser: an expression argument, a path, a literal and anything inside nested
    parens are all ignored, so a name merely MENTIONED near a macro cannot match.
    """
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return set()
    masked = mask_code(text)
    found: set[str] = set()
    for m in _RE_MACRO_CALL.finditer(masked):
        depth, i = 1, m.end()
        arg_start, args = i, []
        while i < len(masked) and depth:
            c = masked[i]
            if c in "([{":
                depth += 1
            elif c in ")]}":
                depth -= 1
                if not depth:
                    args.append(masked[arg_start:i])
                    break
            elif c == "," and depth == 1:
                args.append(masked[arg_start:i])
                arg_start = i + 1
            i += 1
        found |= {a.strip() for a in args if _RE_BARE_IDENT.match(a.strip())}
    return found


#: A `#[no_mangle]` / `#[export_name = "..."]` function item — the definitions that win
#: the link and displace the C. Ordinary `fn`s are internal scaffolding.
_RE_EXPORTED_FN = re.compile(
    r"#\[\s*(?:no_mangle|export_name\s*=\s*\"(?P<alias>[^\"]+)\")\s*\]"
    r"(?:\s*#\[[^\]]*\])*"
    r"\s*(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+\"[^\"]*\"\s+)?fn\s+(?P<name>\w+)"
)


def exported_symbols(crate_root: Path) -> dict[str, Path]:
    """{exported symbol: the .rs that defines it} for one crate.

    Lives here rather than beside its caller because it is the same question the audit
    asks — "what does this crate actually ship?" — and because one module importing
    `tools.source_walk` is enough. `rustconv translate` uses it to refuse, before
    spending anything, to re-buy a TU the crate already serves.
    """
    out: dict[str, Path] = {}
    for rs in source_files(crate_root, "*.rs"):
        try:
            text = rs.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        masked = mask_code(text)
        for m in _RE_EXPORTED_FN.finditer(masked):
            # Scan the MASKED text so a commented-out item cannot match, but read the
            # alias back out of the ORIGINAL: masking blanks string CONTENTS, which is
            # exactly where `#[export_name = "alpha"]` keeps the symbol. mask_code is
            # length-preserving, so the span is the same in both.
            alias = text[m.start("alias") : m.end("alias")] if m.group("alias") else None
            out.setdefault(alias or m.group("name"), rs)
    return out


def tree_macro_arg_names(rust_root: Path) -> set[str]:
    """Index every bare macro-invocation argument under a crate root."""
    names: set[str] = set()
    for path in source_files(rust_root, "*.rs"):
        names |= macro_arg_names(path)
    return names


def member_body(name: str, tree: set[str]) -> str | None:
    """`CDefView_Refresh` → `CDefView_Refresh_body`, the C++ member lowering.

    rustconv lowers a C++ member function into a free `Class_Method_body` plus a separate
    `extern "system"` vtable thunk, so the banked LABEL (the C++ construct that was sent)
    never appears verbatim. `_body` is already in `_TWIN_SUFFIXES`, but only the variadic
    path consults it, so a non-variadic member resolved as unintegrated — 25 of shell32's
    130 "unintegrated" entries were this one spelling, and every `.partial` in that lane
    was a retried call whose successful body is shipping under the suffix.

    EXACT name plus the suffix, like the twin rules and for the same reason: a containment
    test here would match `CDefView_Refresh` to `CDefView_RefreshGhostedState_body`, which
    is a different function that happens to share a prefix.
    """
    cand = f"{name}_body"
    return cand if cand in tree else None


def tchar_variants(name: str) -> list[str]:
    """`_tmain` → `wmain`, `main`. The TCHAR macro pair, both build flavours."""
    if not name.startswith("_t") or len(name) <= 2:
        return []
    bare = name[2:]
    return [f"w{bare}", bare]


def load_allowlist(path: Path = ALLOW_PATH) -> dict[str, str]:
    """Reviewed exceptions: {function name: written reason}.

    Two spellings, both requiring a reason. `[allow]` gives one function its own, which is
    right when the reason is specific to it. `[groups]` gives one reason to a named list,
    which is right when a whole CLASS was superseded at once — shell32 retired
    `CFolderOptions`'s eleven banked members in a single rewrite, and eleven copies of one
    paragraph is a file nobody re-reads. A group with no `functions`, or a function in no
    group, is dropped rather than allowed silently: the invariant is that every allowed
    name carries a written reason, not that the file parses.
    """
    if not path.exists():
        return {}
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError):
        return {}
    out: dict[str, str] = {}
    groups = data.get("groups", {})
    if isinstance(groups, dict):
        for group in groups.values():
            if not isinstance(group, dict):
                continue
            reason = group.get("reason")
            names = group.get("functions")
            if not isinstance(reason, str) or not isinstance(names, list):
                continue
            for name in names:
                if isinstance(name, str):
                    out[name] = reason
    allowed = data.get("allow", {})
    if isinstance(allowed, dict):
        out.update({k: str(v) for k, v in allowed.items()})
    return out


def _body_name_ok(name: str, tree: set[str], allow: dict[str, str]) -> bool:
    """Is one name declared inside a banked body accounted for?

    In the tree directly, under the `_body` member lowering, or allowlisted. The
    allowlist is keyed by the C++ LABEL, so a body name is checked with `_body` stripped
    as well — `CFolderOptions_ctor_body` is the shipping-side spelling of the allowlisted
    `CFolderOptions_ctor`, and requiring the allowlist to carry both spellings would be a
    second place to forget one.
    """
    return (
        name in tree
        or member_body(name, tree) is not None
        or name in allow
        or (name.endswith("_body") and name[: -len("_body")] in allow)
    )


def resolve(
    fn: BankedFn,
    tree: set[str],
    allow: dict[str, str],
    macro_args: set[str] | None = None,
) -> Resolution:
    """Match one banked function against the tree, cheapest rule first."""
    if fn.name in tree:
        return Resolution(fn, fn.name, "direct")

    # The key may be a macro or a stale label; the body holds what actually shipped.
    # Each declared name goes through the SAME rules, not a raw set test. A
    # content-addressed memo entry (`translation-memory-exact-v1/<sha>.rs`) is keyed by a
    # hash, so its body is the only thing that can identify it — and its body holds
    # `Class_Method_body` names and superseded members, which only resolve via the rules
    # below. Requiring EVERY name to resolve is what keeps this tight: a body whose 5
    # definitions became 1 is partially-lost paid work, and accepting the first hit would
    # call it integrated. Report the count rather than a representative name — picking one
    # from a multi-function body names an arbitrary symbol and reads as if it were the match.
    body = declared_names(fn.path)
    if body and all(n in tree or member_body(n, tree) for n in body):
        return Resolution(fn, f"{len(body)} body fn(s)", "declared_in_body")

    if is_variadic_body(fn.path):
        twins = varargs_twins(fn.name, tree)
        if twins:
            return Resolution(fn, twins[0], "varargs_twin")

    for cand in tchar_variants(fn.name):
        if cand in tree:
            return Resolution(fn, cand, "tchar_macro")

    body_twin = member_body(fn.name, tree)
    if body_twin:
        return Resolution(fn, body_twin, "member_body")

    # Last of the mechanical rules, because it reads a looser index than the others.
    if macro_args and fn.name in macro_args:
        return Resolution(fn, fn.name, "macro_generated")

    if fn.name in allow:
        return Resolution(fn, fn.name, "allowlist")

    # A content-addressed memo entry (`translation-memory-exact-v1/<sha>.rs`) is keyed by a
    # hash, so its BODY is the only thing that can identify it. Checked last, and reported
    # as `allowlist` rather than `declared_in_body`, because a reviewed exception must stay
    # visible as one — laundering it into a mechanical verdict is how an allowlist stops
    # being re-read.
    if body and all(_body_name_ok(n, tree, allow) for n in body):
        return Resolution(fn, f"{len(body)} body fn(s)", "allowlist")

    return Resolution(fn, None, "unresolved")


def audit(
    responses_root: Path,
    rust_root: Path,
    allow: dict[str, str] | None = None,
) -> list[DirReport]:
    """Audit every response dir under `responses_root` against the shipped tree."""
    if allow is None:
        allow = load_allowlist()
    tree = tree_fn_names(rust_root)
    macro_args = tree_macro_arg_names(rust_root)

    reports = []
    for resp_dir in sorted(p for p in responses_root.iterdir() if p.is_dir()):
        # A retried call can leave its `.partial` beside the completed body. Both would
        # resolve, so this is cosmetic — but reporting one function twice reads as two.
        complete = banked_fns(resp_dir)
        done = {fn.name for fn in complete}
        fns = complete + [fn for fn in truncated_fns(resp_dir) if fn.name not in done]
        if not fns:
            continue
        resolutions = [resolve(fn, tree, allow, macro_args) for fn in fns]
        reports.append(DirReport(resp_dir.name, len(fns), resolutions))
    return reports


#: A `RunArchive` dir — one `translate`/`submit` invocation, streamed as it was bought.
_RE_RUN_ARCHIVE = re.compile(r"^\d{8}-\d{6}-")


@dataclass(frozen=True)
class Redundancy:
    """One banked function present in several dirs, and whether the copies AGREE."""

    name: str
    dirs: list[str]
    integrated: bool
    identical: bool
    #: Any copy is a `.partial` — a rejected or truncated body. Then the divergence is a
    #: RETRY, and a retry after a wrong deterministic audit is legitimate (the second
    #: purchase is the one you keep). Divergent copies that are ALL completed `.rs` are
    #: the class that was bought again with nothing rejected in between.
    retry: bool
    module: str


def bank_module(dir_name: str) -> str:
    """The MODULE a response dir belongs to — `20260919-022222-file.c` and
    `msvcrt__file__factseal_v1__callseal_v1` are both `file`.

    Getting this wrong is the single most productive bug in this area, and it was found
    twice in one afternoon from opposite directions. Splitting `<group>__<module>__<flags>`
    and taking `parts[0]` buckets by GROUP, collapsing 37 distinct shell32 modules into
    one — and every cross-module pair that invents is two different modules' completed
    bodies, which lands in the "bought again" class by construction. Keeping the `.cpp`
    on the timestamped side is the mirror-image error: `utils.cpp` and `utils` then read
    as different modules and real duplicate pairs are discarded as collisions.
    """
    dir_name = re.sub(r"-run-[0-9a-f]{12}$", "", dir_name)
    m = re.match(r"^\d{8}-\d{6}-(.+?)(?:\.\w+)?$", dir_name)
    if m:
        return m.group(1)
    parts = dir_name.split("__")
    return parts[1] if len(parts) > 1 else dir_name


def _body_hash(path: Path) -> str | None:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError:
        return None


def redundancies(reports: list[DirReport]) -> tuple[list[Redundancy], list[str]]:
    """Functions banked in MORE THAN ONE dir, and run archives that duplicate the cache.

    **Two dirs holding a function is normally ONE purchase, not two.** Every `translate`
    writes each body twice: to a timestamped `RunArchive` (`20260919-022222-file.c`) as
    it streams, and to the namespaced `ResponseCache`
    (`msvcrt__file__factseal_v1__callseal_v1__wideseal_v1`) that makes a replay free. The
    first draft of this function called that "bought more than once" and was wrong about
    all 235 of msvcrt's — they are byte-identical pairs from a single run. So the copies
    are HASHED, and only genuinely DIVERGENT ones are reported as separate purchases.
    A divergent pair is the real re-buy signal, and its usual cause is a re-run under a
    different seal-flag namespace.

    Returns (duplicates, run-archive dirs that are safe to archive). The second list is
    deliberately never a `ResponseCache` dir: **that directory is the live replay cache,
    and deleting it makes the next run re-buy the file.** Only a timestamped RunArchive
    whose every function is byte-identical to a cached copy — and integrated — qualifies.

    An ALLOWLISTED resolution does not count as integrated here, though `integrated` is
    true for it. "Allowlisted" means correctly ABSENT — superseded by a rewrite, or a body
    that must never ship — so a dir full of them is the record of a decision, not a
    redundant copy of shipping code, and the report's own sentence ("every function in
    them is integrated") would be false of it. `20260718-205701-CEnumIDListBase.cpp` is
    the live example: seven bodies that must never be defined in Rust, because eight C++
    classes still derive from the live base they would capture.

    REPORTS ONLY. `.responses/` is paid artifact; removing even a provably redundant
    copy is a human's call, not a tool's.
    """
    where: dict[str, dict[str, Path]] = {}
    resolved: dict[str, bool] = {}
    for rep in reports:
        for res in rep.resolutions:
            where.setdefault(res.fn.name, {})[rep.name] = res.fn.path
            resolved[res.fn.name] = resolved.get(res.fn.name, False) or res.integrated

    dupes: list[Redundancy] = []
    agree: dict[tuple[str, str], bool] = {}
    for name, by_dir in sorted(where.items()):
        if len(by_dir) < 2:
            continue
        hashes = {d: _body_hash(path) for d, path in by_dir.items()}
        identical = len(set(hashes.values())) == 1 and None not in hashes.values()
        for d in by_dir:
            agree[(d, name)] = identical
        # Group by MODULE before calling anything a duplicate: shell32 has an
        # `IsRegItem` in CDrivesFolder, CDesktopFolder and CControlPanelFolder, and
        # those are three different functions that share a name, not one bought thrice.
        mods: dict[str, list[str]] = {}
        for d in by_dir:
            mods.setdefault(bank_module(d), []).append(d)
        for mod, dirs in sorted(mods.items()):
            if len(dirs) < 2:
                continue
            same = len({hashes[d] for d in dirs}) == 1 and all(hashes[d] for d in dirs)
            dupes.append(
                Redundancy(
                    name,
                    sorted(dirs),
                    resolved[name],
                    same,
                    any(by_dir[d].suffix == ".partial" for d in dirs),
                    mod,
                )
            )

    cache_dirs = {r.name for r in reports if not _RE_RUN_ARCHIVE.match(r.name)}
    safe_archives = [
        rep.name
        for rep in reports
        if _RE_RUN_ARCHIVE.match(rep.name)
        and rep.resolutions
        and all(
            res.integrated
            and res.via != "allowlist"
            and agree.get((rep.name, res.fn.name), False)
            and any(d in cache_dirs for d in where[res.fn.name])
            for res in rep.resolutions
        )
    ]
    return dupes, safe_archives


def format_report(reports: list[DirReport], verbose: bool = False) -> str:
    """Human-readable summary. Absences first — they are the reason to run this."""
    lines = []
    absent = [r for r in reports if r.absent]
    total_fns = sum(r.total for r in reports)
    total_absent = sum(len(r.absent) for r in reports)

    if absent:
        lines.append(f"UNINTEGRATED PAID WORK — {total_absent} function(s):")
        for rep in absent:
            lines.append(f"  {rep.status:<10} {rep.name}")
            for res in rep.absent:
                # A truncated body has no complete text to wire up: the remedy is to
                # re-translate, not to integrate. Say which, or the reader guesses.
                verb = "RE-TRANSLATE (truncated)" if res.fn.truncated else "INTEGRATE"
                lines.append(f"      {verb:<24} {res.fn.name}  ({res.fn.path})")
        lines.append("")

    if verbose:
        aliased = [r for r in reports if r.aliased]
        if aliased:
            lines.append("RESOLVED VIA ALIAS:")
            for rep in aliased:
                for res in rep.aliased:
                    lines.append(f"  {rep.name}: {res.fn.name} -> {res.found_as} ({res.via})")
            lines.append("")

    dupes, safe_archives = redundancies(reports)
    divergent = [d for d in dupes if not d.identical]
    rebought = [d for d in divergent if not d.retry]
    retried = [d for d in divergent if d.retry]
    mirrored = [d for d in dupes if d.identical]
    if rebought:
        by_mod: dict[str, int] = {}
        for d in rebought:
            by_mod[d.module] = by_mod.get(d.module, 0) + 1
        lines.append(
            f"BOUGHT AGAIN — {len(rebought)} function(s) with two COMPLETED bodies in one "
            "module and nothing rejected in between:"
        )
        shown = rebought if verbose else rebought[:12]
        for dup in shown:
            flag = "" if dup.integrated else "  [UNINTEGRATED]"
            lines.append(f"  {dup.module}/{dup.name}  x{len(dup.dirs)}{flag}")
        if not verbose and len(rebought) > 12:
            lines.append(f"  ... and {len(rebought) - 12} more (--verbose for all)")
        lines.append(
            "  by module: "
            + ", ".join(f"{m}={n}" for m, n in sorted(by_mod.items(), key=lambda kv: -kv[1])[:8])
        )
        lines.append(
            "  Usual cause: a re-run under a different SEAL-FLAG namespace re-buys the "
            "whole file.\n  Pass --seal-facts --seal-calls --seal-wide from the START.\n"
            "  NOTE: paid more than once is not yet paid NEEDLESSLY — the outcome ledger "
            "says which."
        )
        lines.append("")
    if retried:
        # Separated because conflating the two overstated shell32's re-buys by 4x.
        lines.append(
            f"({len(retried)} further divergent function(s) have a `.partial` among their "
            "copies — a RETRY after a reject, where the second purchase is the one you "
            "keep. Legitimate; not counted above.)"
        )
        lines.append("")
    if mirrored:
        lines.append(
            f"({len(mirrored)} function(s) appear in several dirs with IDENTICAL bytes "
            "— one purchase written to both its run archive and the replay cache.)"
        )
        lines.append("")
    if safe_archives:
        lines.append(
            f"RUN ARCHIVE FULLY MIRRORED BY THE CACHE — {len(safe_archives)} dir(s); every "
            "function in them is integrated AND byte-identical to a copy in a replay-cache "
            "dir, so archiving loses nothing. NEVER remove the namespaced cache dir itself: "
            "that is what makes a replay free. Deleting paid artifact is a HUMAN's call:"
        )
        lines.extend(f"  {name}" for name in safe_archives)
        lines.append("")

    lines.append(
        f"{len(reports)} response dir(s), {total_fns} banked function(s), "
        f"{total_absent} unintegrated"
    )
    return "\n".join(lines)
