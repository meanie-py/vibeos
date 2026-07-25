#!/usr/bin/env python3
"""
verify_match_consts.py — catch the "match on an undefined constant silently
becomes a catch-all variable binding" bug class in the Rust ntoskrnl translation.

THE BUG (caused a real BUGCHECK 0x1E):
    C:    switch (x) { case FOO: ...; case BAR: ...; }
    Rust: match x { FOO => ..., BAR => ..., }
If `FOO` is NOT an in-scope `const` / `static` / imported enum variant, Rust does
NOT compare by value. It treats `FOO` as a *fresh irrefutable binding* that matches
EVERYTHING. The first such arm swallows every input; later arms become dead. rustc
emits only a `unreachable_patterns` warning (plus maybe `non_upper_case_globals`),
never an error — so it compiles green and silently does the wrong thing at runtime.

WHAT THIS TOOL FLAGS:
A `match` arm whose pattern is a single bare path-free identifier in CONST_CASE or
PascalCase (^[A-Z][A-Za-z0-9_]*$), used as a pattern (left of `=>`), that is NOT
resolvable to an in-scope const/static/enum-variant. Such an arm is a silent binding.

RESOLUTION / CONFIDENCE (tuned to minimize false positives):
  - In-file `const IDENT` / `static IDENT` / `let IDENT` / explicit `use ...::IDENT`
    / local `enum { ... IDENT ... }`  -> RESOLVED, not flagged. (This is what makes
    pnpnotify.rs's `EventCategoryDeviceInterfaceChange =>` SAFE: it has a local
    `const EventCategoryDeviceInterfaceChange` at the top of the file.)
  - File has glob imports (`use X::*;`) AND the identifier exists in this crate's
    DEPENDENCY CLOSURE as a `pub const` / enum variant  -> LOW confidence (probably
    resolved through a glob; reported only with --include-low).
  - Not resolved in-file, and NOT defined anywhere in the crate's dependency closure
    -> HIGH confidence hazard (this is the real bug shape).
  - Not resolved in-file, found in the closure, but file has NO glob imports
    -> MEDIUM confidence (can't reach it without an import).

SCOPE (both axes were wrong before 2026-07-24, and each hid live bugs):
  - Files scanned: the whole Rust workspace. The default used to be
    `ntoskrnl-rust/src` alone, so the entire driver fleet and every usermode DLL
    were never looked at — serial.sys had 37 dead IOCTL arms while this reported ✓.
  - Constant resolution: each crate's own sources + its transitive *path*
    dependencies. Resolving against the whole workspace treats a constant defined
    in an unrelated crate as reachable; classpnp's `SCSI_ADSENSE_*` dispatch arms
    (defined only in the unrelated `atapi-rust`) were filed LOW and ignored.
    `--workspace-scope` restores the old, laxer behaviour.

CORROBORATING SMELL:
A risky arm immediately followed by additional arms (which rustc would call
`unreachable`) raises confidence — that's the exact runtime footprint of the bug.

Patterns that are NORMAL Rust bindings and are NEVER flagged:
  Some(x), Ok/Err/None, tuple/struct/slice patterns, `n @ range`, ranges `a..=b`,
  guards `if`, `|` alternations containing literals/paths, `_`, lowercase idents,
  paths with `::`, calls `IDENT(...)`, struct literals `IDENT { .. }`.
"""

import argparse
import os
import re
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Identifier classification
# ---------------------------------------------------------------------------

# Leading underscores are part of the class, not an exclusion from it. c2rust and
# bindgen both emit `_FSCTL_LOCK_VOLUME`-style names, and `_NAME` in a match arm
# binds exactly as `NAME` does — the underscore only silences `unused_variables`.
# That makes the underscored form the MORE dangerous spelling and it is why an
# oracle built on `--force-warn unused_variables` cannot see this variant at all:
# cdfs's `_FSCTL_LOCK_VOLUME` arm swallowed the following 8 FSCTL dispatches with
# no warning of any kind except `unreachable_patterns` on its victims.
CONSTISH = re.compile(r"^_*[A-Z][A-Za-z0-9_]*$")

# Prelude enum variants / unit types that are always in scope and are NOT the
# undefined-const bug (they really do match by value). Never flag these.
ALLOW_BINDINGS = {
    "None",
    "Some",
    "Ok",
    "Err",
    "Ordering",
    "Less",
    "Greater",
    "Equal",
    "Cow",
    "Borrowed",
    "Owned",
}


def looks_const(ident: str) -> bool:
    return bool(CONSTISH.match(ident))


# ---------------------------------------------------------------------------
# Comment / string stripping (line-oriented, good enough for arm detection)
# ---------------------------------------------------------------------------


def strip_line_comment(line: str) -> str:
    # remove // ... but not inside a string. Simple state machine.
    out = []
    i = 0
    in_str = False
    in_char = False
    n = len(line)
    while i < n:
        c = line[i]
        if in_str:
            out.append(c)
            if c == "\\" and i + 1 < n:
                out.append(line[i + 1])
                i += 2
                continue
            if c == '"':
                in_str = False
            i += 1
            continue
        if in_char:
            out.append(c)
            if c == "\\" and i + 1 < n:
                out.append(line[i + 1])
                i += 2
                continue
            if c == "'":
                in_char = False
            i += 1
            continue
        if c == '"':
            in_str = True
            out.append(c)
            i += 1
            continue
        if c == "/" and i + 1 < n and line[i + 1] == "/":
            break
        out.append(c)
        i += 1
    return "".join(out)


# ---------------------------------------------------------------------------
# Build the workspace const/static/enum-variant universe
# ---------------------------------------------------------------------------

RE_PUB_CONST = re.compile(
    r"\b(?:pub(?:\([^)]*\))?\s+)?(?:const|static)\s+(?:mut\s+)?(_*[A-Z][A-Za-z0-9_]*)\b"
)


ENUM_BODY = re.compile(r"enum\s+\w+[^{]*\{(.*?)\}", re.DOTALL)

RE_PUB_USE_ITEM = re.compile(r"^[ \t]*(?:#\[[^\]]*\]\s*)*pub\s+use\s+([^;]+);", re.M)


def pub_use_names(text):
    """Names a module RE-EXPORTS by explicit `pub use` (as opposed to `pub use X::*`,
    which is followed separately).

    This is not a nicety: the house prelude pattern aliases bindgen's mangled enum
    variants back to their bare C spelling —

        pub use samsrv_sys::{
            _DOMAIN_INFORMATION_CLASS_DomainPasswordInformation as DomainPasswordInformation,
        };

    — precisely so the translated `match` arms are const patterns rather than
    irrefutable bindings. Not harvesting these made every such alias look
    unreachable, i.e. the tool reported the bug the prelude already fixed.
    """
    names = set()
    for m in RE_PUB_USE_ITEM.finditer(text):
        clause = m.group(1)
        for grp in re.finditer(r"\{([^}]*)\}", clause):
            for tok in grp.group(1).split(","):
                tok = tok.strip()
                if not tok or "*" in tok:
                    continue
                # `A::B as C` re-exports under C; a bare `A::B` under B.
                last = tok.split(" as ")[-1].split("::")[-1].strip()
                if CONSTISH.match(last):
                    names.add(last)
        head = clause.split("{")[0]
        if head.strip() and "*" not in head:
            last = head.split(" as ")[-1].split("::")[-1].strip()
            if CONSTISH.match(last):
                names.add(last)
    return names


def _enum_variant_names(body):
    """Variant names in an enum body.

    Split on commas rather than scanning with a `(^|,)…[,(={]` regex: that
    pattern CONSUMES the trailing comma, so the next match had no delimiter left
    to anchor on and every OTHER variant was silently dropped. `I8042_DEVICE_TYPE
    { Unknown, Keyboard, Mouse, PhysicalDeviceObject }` yielded Unknown and Mouse
    but not Keyboard — so a correct `Keyboard =>` arm was reported as a HIGH
    catch-all, in a file that glob-imports the enum's variants.
    """
    names = set()
    for item in body.split(","):
        m = re.match(r"^\s*(_*[A-Z][A-Za-z0-9_]*)\s*(?:[=({]|$)", item.strip() + "\n")
        if m:
            names.add(m.group(1))
    return names


def names_in_file(path):
    """const/static names, enum-variant names, and explicit `pub use` re-exports
    visible from one .rs file."""
    consts = set()
    try:
        text = Path(path).read_text(encoding="utf-8", errors="replace")
    except OSError:
        return consts
    for m in RE_PUB_CONST.finditer(text):
        consts.add(m.group(1))
    for em in ENUM_BODY.finditer(text):
        consts |= _enum_variant_names(em.group(1))
    consts |= pub_use_names(text)
    return consts


def harvest_universe(roots):
    """Return a set of all const/static names and enum-variant names defined
    anywhere under the given roots. Over-approximates resolvability (safe for FP)."""
    consts = set()
    for root in roots:
        for dp, _dn, fns in os.walk(root):
            if "target" in dp.split(os.sep):
                continue
            for fn in fns:
                if fn.endswith(".rs"):
                    consts |= names_in_file(os.path.join(dp, fn))
    return consts


# ---------------------------------------------------------------------------
# Per-CRATE universes (the dependency closure, not the whole workspace)
# ---------------------------------------------------------------------------
#
# Resolving against the *workspace* is what made this lint miss real bugs: a
# constant defined in some unrelated crate is NOT in scope here, but the old
# model saw "exists somewhere + this file has glob imports" and filed it LOW.
# Concrete miss: classpnp's `SCSI_ADSENSE_*` dispatch arms resolve nowhere —
# those constants live only in `atapi-rust`, which classpnp does not depend on —
# so 119 live catch-all arms sat in the ignored LOW bucket. Scope the universe to
# the crate's own sources plus its transitive *path* dependencies, and an
# unreachable constant correctly lands in HIGH.

RE_PATH_DEP = re.compile(r'path\s*=\s*"([^"]+)"')


def crate_root_of(path):
    """Nearest ancestor directory containing a Cargo.toml."""
    d = os.path.dirname(os.path.abspath(path))
    while True:
        if os.path.exists(os.path.join(d, "Cargo.toml")):
            return d
        parent = os.path.dirname(d)
        if parent == d:
            return None
        d = parent


def crate_path_deps(crate_dir, _cache={}):
    """Direct path-dependencies of a crate, as absolute dirs."""
    if crate_dir in _cache:
        return _cache[crate_dir]
    deps = []
    try:
        text = Path(os.path.join(crate_dir, "Cargo.toml")).read_text(
            encoding="utf-8", errors="replace"
        )
    except OSError:
        _cache[crate_dir] = deps
        return deps
    for m in RE_PATH_DEP.finditer(text):
        cand = os.path.normpath(os.path.join(crate_dir, m.group(1)))
        if os.path.isdir(cand):
            deps.append(cand)
    _cache[crate_dir] = deps
    return deps


def crate_own_names(crate_dir, _cache={}):
    """const/enum-variant names defined in one crate's own sources."""
    if crate_dir in _cache:
        return _cache[crate_dir]
    names = set()
    src = os.path.join(crate_dir, "src")
    walk_root = src if os.path.isdir(src) else crate_dir
    for dp, _dn, fns in os.walk(walk_root):
        if "target" in dp.split(os.sep):
            continue
        for fn in fns:
            if fn.endswith(".rs"):
                names |= names_in_file(os.path.join(dp, fn))
    _cache[crate_dir] = names
    return names


# ---------------------------------------------------------------------------
# Glob-import resolution
# ---------------------------------------------------------------------------
#
# "This file has some glob import, so the name is probably fine" is far too lax:
# the globs in translated code are always *specific* modules
# (`use ntkrnl_sys::ke::*;`, `use crate::shims::*;`), and the constant a dispatch
# arm needs usually lives in a module that is NOT globbed — classpnp's
# `SCSI_ADSENSE_*` live in `crate::types`, which no affected file globs. So we
# resolve each glob to its actual module and harvest only those names. If ANY
# glob target cannot be resolved we fall back to the old lax behaviour for that
# file, so sharpening can never manufacture a false HIGH.

# A `use` may carry attributes on the SAME line — `#[allow(unused_imports)] use
# crate::glue::*;` is the house style. Anchoring on `^\s*use` skipped those globs
# entirely, and an UNSEEN glob is worse than an unresolved one: `all_resolved`
# stayed True, so the fail-open guard never fired and the tool emitted a
# CONFIDENT HIGH for names the glob really did bring in (all 8 of scsiport's
# SCSI_NOTIFICATION_TYPE arms, which glue.rs has aliased since it was written).
_ATTRS = r"(?:#\[[^\]]*\]\s*)*"
RE_GLOB_USE = re.compile(
    r"^\s*" + _ATTRS + r"(?:pub\s+)?use\s+([A-Za-z_][A-Za-z0-9_:]*)::\*\s*;", re.M
)
RE_PATH_MOD = re.compile(r'#\[path\s*=\s*"([^"]+)"\]\s*(?:pub\s+)?mod\s+(\w+)\s*;')
RE_PUB_USE_GLOB = re.compile(
    r"^\s*" + _ATTRS + r"pub\s+use\s+([A-Za-z_][A-Za-z0-9_:]*)::\*\s*;", re.M
)


def _crate_dir_for_extern(crate_dir, extern_name):
    """Map a Rust extern-crate name (`ntkrnl_sys`) to a path-dep directory."""
    want = extern_name.replace("_", "-")
    for d in crate_path_deps(crate_dir):
        if os.path.basename(d) in (want, extern_name):
            return d
    return None


def _module_file(crate_dir, parts):
    """Resolve a module path (list of segments) to a .rs file inside a crate."""
    src = os.path.join(crate_dir, "src")
    if not parts:
        for cand in (os.path.join(src, "lib.rs"), os.path.join(crate_dir, "lib.rs")):
            if os.path.exists(cand):
                return cand
        return None
    direct = os.path.join(src, *parts)
    for cand in (direct + ".rs", os.path.join(direct, "mod.rs")):
        if os.path.exists(cand):
            return cand
    # `#[path = "other.rs"] mod name;` indirection (the types_<arch>.rs pattern)
    parent = _module_file(crate_dir, parts[:-1])
    if parent and os.path.exists(parent):
        try:
            ptext = Path(parent).read_text(encoding="utf-8", errors="replace")
        except OSError:
            return None
        for pm in RE_PATH_MOD.finditer(ptext):
            if pm.group(2) == parts[-1]:
                cand = os.path.normpath(os.path.join(os.path.dirname(parent), pm.group(1)))
                if os.path.exists(cand):
                    return cand
    return None


def _names_from_file(crate_dir, f, parts, _seen, depth):
    """Names exported by module file `f`, following its `pub use ...::*;` chain.

    Submodules declared in `f` itself with `#[path = "..."] mod X;` are resolved
    relative to `f`'s directory — the declaration lives in the *current* file, not
    in its parent. Missing that is what made `use crate::types::*` fail to resolve
    (types.rs declares `#[path="types_<arch>.rs"] mod types_arch;` then re-exports
    it), which produced false HIGHs on constants that really are in scope.

    All three `#[cfg(target_arch)]` variants are unioned: a constant present on any
    arch is not the catch-all bug we are hunting (a genuinely arch-asymmetric gap is
    a different, narrower finding).
    """
    # An unresolvable target (module declared inline in lib.rs, generated file we
    # cannot find, …) must report NOT-resolved so the caller falls back to the lax
    # rule. Claiming "resolved" for a file we never read is how the tool produced
    # confident false HIGHs (e.g. `ntdll_sys::nt`, which has no nt.rs on disk).
    if f is None or depth > 6:
        return set(), False
    if f in _seen:
        return set(), True
    _seen.add(f)
    names = names_in_file(f)
    try:
        text = Path(f).read_text(encoding="utf-8", errors="replace")
    except OSError:
        return names, True

    fdir = os.path.dirname(f)
    local_path_mods = {}
    for pm in RE_PATH_MOD.finditer(text):
        local_path_mods.setdefault(pm.group(2), []).append(
            os.path.normpath(os.path.join(fdir, pm.group(1)))
        )

    all_ok = True
    for m in RE_PUB_USE_GLOB.finditer(text):
        segs = [s for s in m.group(1).split("::") if s and s != "self"]
        if not segs:
            all_ok = False
            continue
        if segs[0] in local_path_mods and len(segs) == 1:
            # `#[path="..."] mod X;` declared right here (possibly once per arch)
            for cand in local_path_mods[segs[0]]:
                if os.path.exists(cand):
                    sub, ok = _names_from_file(crate_dir, cand, parts, _seen, depth + 1)
                    names |= sub
                    all_ok = all_ok and ok
            continue
        if segs[0] == "crate":
            sub, ok = _names_from_file(
                crate_dir, _module_file(crate_dir, segs[1:]), segs[1:], _seen, depth + 1
            )
        elif segs[0] == "super":
            tgt = parts[:-1] + segs[1:]
            sub, ok = _names_from_file(
                crate_dir, _module_file(crate_dir, tgt), tgt, _seen, depth + 1
            )
        else:
            dep = _crate_dir_for_extern(crate_dir, segs[0])
            if dep is not None:
                sub, ok = _names_from_file(
                    dep, _module_file(dep, segs[1:]), segs[1:], set(), depth + 1
                )
            else:
                # sibling module, or a submodule file next to this one
                cand = _module_file(crate_dir, parts[:-1] + segs)
                if cand is None:
                    for ext in (".rs", "/mod.rs"):
                        c2 = os.path.join(fdir, segs[0].rstrip("/") + ext)
                        if os.path.exists(c2):
                            cand = c2
                            break
                sub, ok = _names_from_file(crate_dir, cand, parts[:-1] + segs, _seen, depth + 1)
        names |= sub
        all_ok = all_ok and ok
    return names, all_ok


def module_names(crate_dir, parts, _seen=None, depth=0):
    """Names a `use <module>::*;` actually brings into scope, following the
    module's own `pub use ...::*;` re-export chain (e.g. types.rs -> types_<arch>.rs)."""
    if depth > 4:
        return set(), False
    f = _module_file(crate_dir, parts)
    if f is None:
        return set(), False  # unresolved
    return _names_from_file(crate_dir, f, parts, set() if _seen is None else _seen, depth)


def glob_scope(path, crate_dir):
    """(names reachable via this file's glob imports, all_globs_resolved)."""
    try:
        text = Path(path).read_text(encoding="utf-8", errors="replace")
    except OSError:
        return set(), False
    names = set()
    all_resolved = True
    for m in RE_GLOB_USE.finditer(text):
        segs = [s for s in m.group(1).split("::") if s and s != "self"]
        if not segs:
            all_resolved = False
            continue
        if segs[0] == "crate":
            sub, ok = module_names(crate_dir, segs[1:])
        else:
            dep = _crate_dir_for_extern(crate_dir, segs[0])
            if dep is None:
                sub, ok = set(), False
            else:
                sub, ok = module_names(dep, segs[1:])
        names |= sub
        all_resolved = all_resolved and ok
    return names, all_resolved


def crate_universe(crate_dir, _cache={}):
    """Names reachable from a crate: its own sources + transitive path deps."""
    if crate_dir is None:
        return set()
    if crate_dir in _cache:
        return _cache[crate_dir]
    _cache[crate_dir] = set()  # cycle guard
    names = set(crate_own_names(crate_dir))
    seen = {crate_dir}
    stack = list(crate_path_deps(crate_dir))
    while stack:
        d = stack.pop()
        if d in seen:
            continue
        seen.add(d)
        names |= crate_own_names(d)
        stack.extend(crate_path_deps(d))
    _cache[crate_dir] = names
    return names


# ---------------------------------------------------------------------------
# Per-file local-scope resolution
# ---------------------------------------------------------------------------


def file_local_names(text: str):
    """Names that are definitely resolvable inside this file:
    local const/static, local enum variants, explicit `use`d names, and `let`
    bindings (shadowing). Returns a set."""
    names = set()
    has_glob = False

    for m in RE_PUB_CONST.finditer(text):
        names.add(m.group(1))

    # local enum variants
    for em in re.finditer(r"enum\s+\w+[^{]*\{(.*?)\}", text, re.DOTALL):
        names |= _enum_variant_names(em.group(1))

    # explicit `use` imports (capture identifiers, ignore globs but record presence)
    for um in re.finditer(r"\buse\s+([^;]+);", text):
        clause = um.group(1)
        if "*" in clause:
            has_glob = True
        # braced group: use a::b::{X, Y as Z, ...}
        for grp in re.finditer(r"\{([^}]*)\}", clause):
            for tok in grp.group(1).split(","):
                tok = tok.strip()
                if not tok:
                    continue
                # handle `X as Y`
                parts = [p.strip() for p in tok.split(" as ")]
                last = parts[-1].split("::")[-1].strip()
                if CONSTISH.match(last):
                    names.add(last)
                first = parts[0].split("::")[-1].strip()
                if CONSTISH.match(first):
                    names.add(first)
        # tail of an unbraced path: use a::b::CONST;  or use a::b::CONST as C;
        tail = clause.split("{")[0]
        if tail.strip() and "*" not in tail:
            tparts = [p.strip() for p in tail.split(" as ")]
            cand = tparts[-1].split("::")[-1].strip()
            if CONSTISH.match(cand):
                names.add(cand)
            cand0 = tparts[0].split("::")[-1].strip()
            if CONSTISH.match(cand0):
                names.add(cand0)

    # let bindings that shadow with a const-looking name (rare but real)
    for lm in re.finditer(r"\blet\s+(?:mut\s+)?(_*[A-Z][A-Za-z0-9_]*)\b", text):
        names.add(lm.group(1))

    return names, has_glob


# ---------------------------------------------------------------------------
# Match-arm scanning
# ---------------------------------------------------------------------------

# A bare-identifier arm pattern. We require the *whole* pattern (everything left
# of the top-level `=>`) to be exactly one const-ish identifier, optionally with a
# trailing guard? No — guards make it refutable-ish but a bare ident binding +
# guard is still a binding; however to be conservative we skip arms with `if`
# guards and `|`. We also require it not be a path (no `::`), not a call, not a
# struct literal, not `@`, not a range.


def iter_match_arms(lines):
    """Yield (lineno, pattern_text, raw_line) for candidate arm lines.
    Heuristic line-based: a line containing `=>` whose left side (after stripping
    comments and leading whitespace) is a single bare identifier."""
    for i, raw in enumerate(lines, start=1):
        line = strip_line_comment(raw)
        if "=>" not in line:
            continue
        left = line.split("=>", 1)[0].strip()
        if not left:
            continue
        yield i, left, raw


def is_risky_pattern(left: str):
    """Return (ident, reason) if `left` is a single bare const-ish binding pattern,
    else (None, None)."""
    # Disqualifiers that mean it's a legit pattern, not a lone binding:
    if left == "_" or left.endswith(","):
        # trailing comma can appear; strip it
        pass
    s = left.rstrip(",").strip()
    if not s:
        return None, None
    # guards / alternations / ranges / bindings / paths / calls / structs / refs
    if any(
        tok in s
        for tok in (
            "::",
            "|",
            "@",
            "..",
            "(",
            ")",
            "{",
            "}",
            "[",
            "]",
            " if ",
            "&",
            '"',
            "'",
            " as ",
        )
    ):
        return None, None
    if " " in s:
        return None, None
    if not CONSTISH.match(s):
        return None, None
    if s == "_":
        return None, None
    if s in ALLOW_BINDINGS:
        return None, None
    return s, "bare const-ish identifier used as match pattern"


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------


def scan_file(path, universe, use_crate_scope=True):
    text = Path(path).read_text(encoding="utf-8", errors="replace")
    lines = text.splitlines()
    local_names, has_glob = file_local_names(text)
    findings = []

    # Resolvability is a per-crate question, not a per-workspace one: only the
    # crate's own sources + its transitive path deps can actually be in scope.
    glob_names, globs_resolved = set(), False
    if use_crate_scope:
        cdir = crate_root_of(path)
        universe = crate_universe(cdir)
        glob_names, globs_resolved = glob_scope(path, cdir)

    arms = list(iter_match_arms(lines))
    # index arm linenos for "followed-by-more-arms" smell
    arm_linenos = [a[0] for a in arms]

    for lineno, left, raw in arms:
        ident, _reason = is_risky_pattern(left)
        if ident is None:
            continue
        if ident in local_names:
            continue  # resolved in-file -> SAFE, not the bug
        if ident in glob_names:
            continue  # a glob import really does bring it in -> SAFE

        in_universe = ident in universe

        # confidence model
        if not in_universe:
            confidence = "HIGH"
            why = (
                "not defined in-file and not defined anywhere in this crate's "
                "dependency closure -> silent catch-all binding"
            )
        elif globs_resolved:
            # Every glob in this file resolved to a real module and none of them
            # export this name, and there is no explicit `use` for it either.
            # It is definitely not in scope, however many globs the file has.
            confidence = "HIGH"
            why = (
                "defined in this crate but NOT exported by any of this file's "
                "(fully resolved) glob imports and never explicitly `use`d "
                "-> silent catch-all binding"
            )
        elif has_glob:
            confidence = "LOW"
            why = (
                "defined in this crate's dependency closure and this file has glob "
                "imports -> probably resolved through a `use ...::*` (likely FP)"
            )
        else:
            confidence = "MEDIUM"
            why = (
                "defined in this crate's dependency closure but this file has NO "
                "glob import that could bring it into scope -> may be a silent binding"
            )

        # corroborating smell: are there subsequent arms in the same match block?
        # heuristic: another arm within the next ~80 lines at >= same indent.
        followed = False
        indent = len(raw) - len(raw.lstrip())
        for ln2 in arm_linenos:
            if lineno < ln2 <= lineno + 120:
                raw2 = lines[ln2 - 1]
                ind2 = len(raw2) - len(raw2.lstrip())
                if ind2 == indent:
                    followed = True
                    break

        findings.append(
            {
                "path": path,
                "line": lineno,
                "ident": ident,
                "confidence": confidence,
                "why": why,
                "followed_by_arms": followed,
                "pattern": left,
            }
        )
    return findings


# ---------------------------------------------------------------------------
# Module-tree reachability
# ---------------------------------------------------------------------------

RE_MOD_ANY = re.compile(
    r"(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?\bmod\s+(\w+)\s*([;{])"
)


def _decl_sites(text):
    """[(module_name, [enclosing inline-module names])] for every `mod X;` in `text`.

    Two shapes broke a line-anchored regex, and BOTH of them mark live files dead
    (the dangerous direction — a hidden catch-all is the bug we are hunting):
      * several declarations on one line — win32k's
        `pub mod dcstate; pub mod dcobjs; pub mod dclife; pub mod dcutil;`
      * file modules nested in an INLINE module — wkssvc's
        `pub mod wkssvc { pub mod domain; ... }`, where `domain` lives at
        src/wkssvc/domain.rs, not src/domain.rs.
    So scan the whole text and track inline-module nesting by brace depth.
    """
    # Comments must go first: dhcpcsvc's lib.rs documents the exclusion as
    # `// pub mod tree;`, and counting that as a declaration would resurrect the
    # very dead file this function exists to recognise.
    text = re.sub(r"/\*.*?\*/", " ", text, flags=re.S)
    text = re.sub(r"//[^\n]*", "", text)
    sites = []
    stack = []            # [(name, depth_at_open)]
    depth = 0
    i = 0
    while i < len(text):
        m = RE_MOD_ANY.search(text, i)
        upto = m.start() if m else len(text)
        for c in text[i:upto]:
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                while stack and stack[-1][1] >= depth:
                    stack.pop()
        if not m:
            break
        if m.group(2) == ";":
            sites.append((m.group(1), [s[0] for s in stack]))
            i = m.end()
        else:
            depth += 1
            stack.append((m.group(1), depth))
            i = m.end()
    return sites


def crate_live_files(crate_dir, _cache={}):
    """Every .rs REACHABLE from the crate root via `mod` declarations.

    A file that no `mod` reaches is not compiled, so a catch-all arm inside it
    cannot execute — reporting it as a live hazard is noise (dhcpcsvc keeps
    `dhcp/tree.rs`, a translation of upstream-dead source, deliberately excluded
    in lib.rs). The same computation is worth having in the other direction too:
    an ORPHANED file that looks live to grep is its own bug — classpnp's
    `class_gap3.rs`/`class_gap4.rs` are in no module tree, so edits to them
    passed `cargo check` while the link still failed.
    """
    if crate_dir in _cache:
        return _cache[crate_dir]
    _cache[crate_dir] = set()
    src = os.path.join(crate_dir, "src")
    live, queue = set(), []
    for n in ("lib.rs", "main.rs"):
        f = os.path.join(src, n)
        if os.path.exists(f):
            queue.append((f, []))
    while queue:
        f, inline = queue.pop()
        if f in live or not os.path.exists(f):
            continue
        live.add(f)
        try:
            text = Path(f).read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        fdir = os.path.dirname(f)
        stem = os.path.splitext(os.path.basename(f))[0]
        base = fdir if stem in ("lib", "main", "mod") else os.path.join(fdir, stem)
        for pm in RE_PATH_MOD.finditer(text):
            queue.append((os.path.normpath(os.path.join(fdir, pm.group(1))), []))
        for name, encl in _decl_sites(text):
            d = os.path.join(base, *(inline + encl))
            for cand in (os.path.join(d, name + ".rs"), os.path.join(d, name, "mod.rs")):
                if os.path.exists(cand):
                    queue.append((cand, []))
                    break
    _cache[crate_dir] = live
    return live


def is_live_file(path):
    """False only when we positively established the crate's module tree and this
    file is not in it. Unknown crate layout -> treat as live (fail open)."""
    crate = crate_root_of(path)
    if crate is None:
        return True
    live = crate_live_files(crate)
    if not live:
        return True
    # Compare REAL paths: scan targets arrive relative, the module walk builds
    # absolutes from crate_root_of(). Comparing the two spellings marked every
    # file orphaned.
    return os.path.realpath(path) in {os.path.realpath(x) for x in live}


def main():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument(
        "paths",
        nargs="*",
        help="files/dirs to scan (default: the whole Rust workspace, sdk/lib/rust)",
    )
    ap.add_argument(
        "--workspace-scope",
        action="store_true",
        help="resolve constants against the whole workspace instead of each crate's "
        "dependency closure (legacy behaviour; hides real bugs — see module docstring)",
    )
    ap.add_argument(
        "--universe-root",
        action="append",
        default=None,
        help="root(s) to harvest the const/enum-variant universe from (default: sdk/lib/rust)",
    )
    ap.add_argument(
        "--include-orphan",
        action="store_true",
        help="also list findings in files no `mod` declaration reaches (dead source)",
    )
    ap.add_argument(
        "--include-low",
        action="store_true",
        help="also report LOW-confidence (likely-FP, glob-resolvable) hits",
    )
    ap.add_argument(
        "--context", type=int, default=0, help="print N lines of context around each hit"
    )
    ap.add_argument(
        "--check",
        action="store_true",
        help="exit 1 if any HIGH (or MEDIUM) confidence hazard is found",
    )
    args = ap.parse_args()

    repo = os.getcwd()
    targets = args.paths or [os.path.join("sdk", "lib", "rust")]
    uni_roots = args.universe_root or [os.path.join("sdk", "lib", "rust")]

    # collect .rs files to scan
    scan_files = []
    for t in targets:
        if os.path.isfile(t):
            scan_files.append(t)
        else:
            for dp, _dn, fns in os.walk(t):
                if "target" in dp.split(os.sep):
                    continue
                for fn in fns:
                    if fn.endswith(".rs"):
                        scan_files.append(os.path.join(dp, fn))

    sys.stderr.write(f"[verify_match_consts] harvesting const universe from {uni_roots} ...\n")
    universe = harvest_universe(uni_roots)
    sys.stderr.write(
        f"[verify_match_consts] {len(universe)} const/enum-variant names known; "
        f"scanning {len(scan_files)} files ...\n"
    )

    all_findings = []
    for path in sorted(scan_files):
        fs = scan_file(path, universe, use_crate_scope=not args.workspace_scope)
        if fs and not is_live_file(path):
            for f in fs:
                f["confidence"] = "ORPHAN"
                f["why"] += " [in a file NO `mod` declaration reaches — not compiled]"
        all_findings.extend(fs)

    order = {"HIGH": 0, "MEDIUM": 1, "ORPHAN": 2, "LOW": 3}
    all_findings.sort(
        key=lambda f: (order[f["confidence"]], not f["followed_by_arms"], f["path"], f["line"])
    )

    # ORPHAN is suppressed with LOW by default. A finding in a file no `mod`
    # declaration reaches cannot execute, so listing it would spend this lint's
    # STRICT (empty) baseline on non-actionable entries — and an empty baseline is
    # the whole point: it fails on the first NEW live catch-all. `--include-orphan`
    # lists them, which doubles as the orphaned-translation-file report.
    hidden = {"LOW"} if args.include_low else {"LOW", "ORPHAN"}
    if args.include_orphan:
        hidden.discard("ORPHAN")
    if args.include_low:
        hidden.discard("LOW")
    shown = [f for f in all_findings if f["confidence"] not in hidden]

    counts = {"HIGH": 0, "MEDIUM": 0, "ORPHAN": 0, "LOW": 0}
    for f in all_findings:
        counts[f["confidence"]] += 1

    for f in shown:
        rel = os.path.relpath(f["path"], repo)
        smell = "  [+followed-by-arms: unreachable smell]" if f["followed_by_arms"] else ""
        print(f"{rel}:{f['line']}: [{f['confidence']}] `{f['ident']}` -- {f['why']}{smell}")
        if args.context:
            try:
                lines = Path(f["path"]).read_text(encoding="utf-8", errors="replace").splitlines()
            except OSError:
                lines = []
            lo = max(1, f["line"] - args.context)
            hi = min(len(lines), f["line"] + args.context)
            for ln in range(lo, hi + 1):
                marker = ">>" if ln == f["line"] else "  "
                print(f"    {marker} {ln}: {lines[ln - 1]}")
            print()

    print()
    print(
        f"SUMMARY: HIGH={counts['HIGH']}  MEDIUM={counts['MEDIUM']}  "
        f"ORPHAN(dead file)={counts['ORPHAN']}  "
        f"LOW(likely-FP)={counts['LOW']}  shown={len(shown)}"
    )

    if args.check and (counts["HIGH"] + counts["MEDIUM"]) > 0:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
