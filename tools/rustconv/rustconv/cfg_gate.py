"""Carry a C `#ifdef` build-configuration guard across into the translated Rust.

WHY this exists: a Wine-derived module is often compiled with an optional backend
switched OFF. wldap32 is the live case — `config.h` says `/* #undef HAVE_LDAP */`, so
all 19 PCH'd TUs compile to `return WLDAP32_LDAP_NOT_SUPPORTED` and the real LDAP
marshalling is never built. The translator faithfully renders the *whole* C, guard arm
included, because the guarded arm is the interesting code and it is what a future
`HAVE_LDAP=1` build needs. The result compiles against a libldap that is not there:
1087 rustc errors, 778 of them plain "cannot find `ldap_*`".

Deleting the guarded arm is not an option — it is paid artifact, and it is the arm that
makes the translation worth more than the C. So instead we do what the C does: keep both
arms and let a cfg feature pick. The Rust that ships is then *byte-for-byte equivalent in
behaviour* to the C that ships, and the guarded arm stays on disk, compiling the day
someone turns the feature on.

The boundary is recovered from the C, never guessed:

  UNGATED       the guard is not mentioned — leave the Rust alone.
  WHOLLY_GATED  the whole definition sits inside a file-scope `#ifdef GUARD`
                (wldap32.h:244 puts 43 of its 62 marshalling helpers there).
                -> prepend `#[cfg(feature = "f")]`. No twin: nothing outside the
                   guarded arm can call it, because its callers are gated too.
  BODY_GATED    the body is exactly `<decl> #ifdef GUARD ... #endif return <decl-var>;`
                — 150 of wldap32's 153 guarded bodies have precisely this shape.
                -> gate the translated fn and emit a `#[cfg(not(feature = "f"))]` twin
                   whose body is the initializer the C would have returned. The twin
                   carries a COPY of the original's attributes, because `#[no_mangle]`
                   on the export must survive with the feature off.
  COMPLEX       anything else. REPORTED, never transformed. The three wldap32 cases are
                `map_error` (guard nested inside a `switch`) and the two
                `ldap_parse_*_controlW` (three separate guarded regions each).

The twin's return value is lifted from the *Rust* `let mut ret: T = <expr>;`, not
re-derived from the C constant. The translation already rendered that constant once and
its rendering was audited by fact sealing; re-deriving it here would be a second,
unaudited opinion about the same value, and the two could disagree silently.

Guards are DERIVED, not named on the command line: `guards_for_module` intersects the
macros the module actually branches on with the `/* #undef X */` set in wine's
config.h. wldap32 branches on EIGHT (HAVE_LDAP, HAVE_LDAP_H and six per-function
probes like HAVE_LDAP_PARSE_SORT_CONTROL); each maps to its own lowercase feature, so
gating on the obvious one alone would have left `map_error` and six others silently
wrong. Pass --guard to override.

Usage:
    python -m rustconv.cfg_gate --c-dir dll/win32/wldap32 \
        --rust-dir sdk/lib/rust/wldap32-rust/src/wldap32     # report
    ... then re-run with --apply.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

import tree_sitter_rust as tsrust
from tree_sitter import Language, Parser

from rustconv.profile_derive import _wine_config_undefs
from rustconv.scanner import (
    C_LANG,
    _blank_memmodel,
    _normalize_callbacks,
    _strip_sal,
)

RUST_LANG = Language(tsrust.language())

UNGATED = "UNGATED"
WHOLLY_GATED = "WHOLLY_GATED"
BODY_GATED = "BODY_GATED"
COMPLEX = "COMPLEX"


class CfgGateError(Exception):
    """A transform could not be carried out safely. Never degrade -- fail loudly."""


@dataclass
class CVerdict:
    """What the C says about one function."""

    name: str
    kind: str
    source: str = ""
    reason: str = ""
    ret_var: str | None = None
    guard: str | None = None  # which macro gated it; None when UNGATED

    @property
    def feature(self) -> str:
        """Cargo feature name for this verdict's guard. `HAVE_LDAP_H` -> `have_ldap_h`."""
        return (self.guard or "").lower()


@dataclass
class Applied:
    """What the transform did to one Rust file."""

    path: Path
    gated: list[str] = field(default_factory=list)
    twinned: list[str] = field(default_factory=list)
    skipped: list[tuple[str, str]] = field(default_factory=list)


def _txt(src: bytes, node) -> str:
    """Decode a node's span.

    `src` is BYTES on purpose. tree-sitter reports byte offsets, and slicing a `str`
    with them drifts by one position per non-ASCII character already seen. The first
    version of this module sliced `str` and a single em-dash in helpers.rs's own doc
    comment shifted every name after it by two — `bvarraydup` came out `arraydup(b`,
    matched nothing, and 43 functions were silently skipped rather than failing.
    """
    return src[node.start_byte : node.end_byte].decode("utf-8", "replace")


def _guard_name(src: bytes, node) -> str | None:
    """The macro a `preproc_ifdef` tests, or None if it is an `#ifndef`."""
    name = node.child_by_field_name("name")
    if name is None:
        return None
    # tree-sitter-c uses one node type for both; the keyword distinguishes them.
    kw = _txt(src, node.children[0]) if node.children else ""
    if "ifndef" in kw:
        return None
    return _txt(src, name)


def _declarator_name(src: bytes, node) -> str | None:
    """Innermost identifier of a declarator, peeling pointers/arrays/params."""
    cur = node
    seen = 0
    while cur is not None and seen < 32:
        seen += 1
        if cur.type == "identifier":
            return _txt(src, cur)
        nxt = cur.child_by_field_name("declarator")
        if nxt is None:
            for c in cur.named_children:
                if c.type in (
                    "identifier",
                    "pointer_declarator",
                    "array_declarator",
                    "function_declarator",
                    "init_declarator",
                    "parenthesized_declarator",
                ):
                    nxt = c
                    break
        cur = nxt
    return None


def _fn_name(src: bytes, fn_node) -> str | None:
    return _declarator_name(src, fn_node.child_by_field_name("declarator"))


def _guard_alt(guards: frozenset[str]) -> str:
    # Longest-first so the alternation cannot prefix-match; `\b` then pins the end.
    return "|".join(re.escape(g) for g in sorted(guards, key=len, reverse=True))


def _mentions_guard(text: str, guards: frozenset[str]) -> bool:
    return re.search(rf"\b(?:{_guard_alt(guards)})\b", text) is not None


def _count_ifdefs(text: str, guards: frozenset[str]) -> int:
    """How many `#ifdef <guard>` openers `text` contains.

    Word-anchored on purpose. A plain substring count says `#ifdef HAVE_LDAP` occurs
    inside `#ifdef HAVE_LDAP_PARSE_REFERENCE`, which made this function report a nested
    region for two bodies that have exactly one -- so they were refused as COMPLEX and
    silently left ungated. Same class of bug as slicing a `str` with byte offsets: a
    match that is nearly right is worse than none.
    """
    return len(re.findall(rf"#\s*ifdef\s+(?:{_guard_alt(guards)})\b", text))


def _classify_body(src: bytes, fn_node, guards: frozenset[str]) -> CVerdict:
    """Classify one root-level `function_definition` against every macro in `guards`."""
    name = _fn_name(src, fn_node) or "?"
    body = fn_node.child_by_field_name("body")
    if body is None:
        return CVerdict(name, UNGATED)

    whole = _txt(src, body)
    if not _mentions_guard(whole, guards):
        return CVerdict(name, UNGATED)

    direct = [c for c in body.named_children if c.type != "comment"]
    guarded = [c for c in direct if c.type == "preproc_ifdef" and _guard_name(src, c) in guards]
    if len(guarded) != 1:
        return CVerdict(name, COMPLEX, reason=f"{len(guarded)} direct guarded region(s) in body")

    region = guarded[0]
    guard = _guard_name(src, region)
    if region.child_by_field_name("alternative") is not None:
        # `#else` -- the OTHER arm is the one this build compiles. Gating the translated
        # arm would ship the wrong half, so refuse.
        return CVerdict(name, COMPLEX, reason=f"`#ifdef {guard}` has an `#else` arm", guard=guard)

    # A guard mentioned anywhere deeper than the one direct region means the shape is not
    # the simple "whole body is optional" case -- wldap32's ldap_parse_sort_controlW nests
    # HAVE_LDAP_PARSE_SORT_CONTROL inside HAVE_LDAP, and the two switch independently.
    nested = _count_ifdefs(whole, guards)
    if nested > 1:
        return CVerdict(
            name, COMPLEX, reason="nested guarded region(s) inside the region", guard=guard
        )

    idx = direct.index(region)
    before = direct[:idx]
    after = direct[idx + 1 :]

    if len(before) > 1 or len(after) > 1:
        return CVerdict(
            name,
            COMPLEX,
            reason=f"{len(before)} stmt(s) before / {len(after)} after the guarded region",
            guard=guard,
        )

    # void-ish: nothing outside the region at all.
    if not before and not after:
        return CVerdict(name, BODY_GATED, ret_var=None, guard=guard)

    if len(before) != 1 or len(after) != 1:
        return CVerdict(
            name,
            COMPLEX,
            reason="guarded region is not flanked by exactly decl + return",
            guard=guard,
        )

    decl, tail = before[0], after[0]
    if decl.type != "declaration" or tail.type != "return_statement":
        return CVerdict(
            name,
            COMPLEX,
            guard=guard,
            reason=f"flanked by {decl.type} / {tail.type}, not declaration / return",
        )

    var = None
    for c in decl.named_children:
        if c.type == "init_declarator":
            var = _declarator_name(src, c)
            break
    if var is None:
        return CVerdict(name, COMPLEX, guard=guard, reason="leading declaration has no initializer")

    ret_expr = [c for c in tail.named_children if c.type != "comment"]
    if len(ret_expr) != 1 or _txt(src, ret_expr[0]).strip() != var:
        got = _txt(src, ret_expr[0]).strip() if ret_expr else "<none>"
        return CVerdict(
            name, COMPLEX, guard=guard, reason=f"returns `{got}`, not the declared `{var}`"
        )

    return CVerdict(name, BODY_GATED, ret_var=var, guard=guard)


def classify_c(path: Path, guards: frozenset[str] | str) -> list[CVerdict]:
    """Every top-level function in `path`, classified against every macro in `guards`."""
    if isinstance(guards, str):
        guards = frozenset({guards})
    raw = path.read_text(errors="replace")
    src = _blank_memmodel(_normalize_callbacks(_strip_sal(raw))).encode()
    tree = Parser(C_LANG).parse(src)
    out: list[CVerdict] = []

    def visit(node, gated: str | None) -> None:
        for ch in node.named_children:
            if ch.type == "function_definition":
                if gated is not None:
                    out.append(
                        CVerdict(
                            _fn_name(src, ch) or "?", WHOLLY_GATED, source=path.name, guard=gated
                        )
                    )
                else:
                    v = _classify_body(src, ch, guards)
                    v.source = path.name
                    out.append(v)
            elif ch.type == "preproc_ifdef":
                own = _guard_name(src, ch)
                inner = gated or (own if own in guards else None)
                if ch.child_by_field_name("alternative") is not None and gated is None:
                    # Functions in the `#else` arm are LIVE. Walking with `inner` would
                    # gate them backwards, so descend ungated and let the body classifier
                    # (which sees no guard) call them UNGATED.
                    visit(ch, gated)
                else:
                    visit(ch, inner)
            elif ch.type in (
                "preproc_if",
                "preproc_ifndef",
                "linkage_specification",
                "declaration_list",
            ):
                visit(ch, gated)

    visit(tree.root_node, None)
    return out


_GUARD_USE_RE = re.compile(r"#\s*if(?:n?def)?\s+([A-Za-z_]\w*)")


def guards_for_module(c_dir: Path, repo_root: Path) -> frozenset[str]:
    """Macros this module branches on that wine's `config.h` says are UNDEFINED.

    config.h is the build-configuration oracle: autoconf writes an off case as
    `/* #undef HAVE_LDAP */`, which is an assertion about THIS build, not a guess. Any
    such macro the module actually tests is a live gate that has to reach the Rust.
    Deriving the set beats naming one guard on the command line -- wldap32 branches on
    EIGHT (`HAVE_LDAP`, `HAVE_LDAP_H`, and six per-function feature probes), and passing
    only the obvious one leaves the rest silently ungated.
    """
    undefs = _wine_config_undefs(repo_root)
    used: set[str] = set()
    for f in sorted(list(c_dir.glob("*.c")) + list(c_dir.glob("*.h"))):
        used |= set(_GUARD_USE_RE.findall(f.read_text(errors="replace")))
    return frozenset(used & undefs)


# --------------------------------------------------------------------------- Rust side


@dataclass
class RustFn:
    name: str
    attr_start: int  # byte offset of the first attribute in the run preceding the fn
    fn_start: int  # byte offset of the `function_item` itself
    body_start: int  # byte offset of the `{`
    end: int


def _rust_fns(src: bytes) -> list[RustFn]:
    """Top-level `fn` items, each with the span of its preceding attribute run."""
    tree = Parser(RUST_LANG).parse(src)
    kids = list(tree.root_node.children)
    out: list[RustFn] = []
    for i, node in enumerate(kids):
        if node.type != "function_item":
            continue
        name_node = node.child_by_field_name("name")
        body = node.child_by_field_name("body")
        if name_node is None or body is None:
            continue
        name = _txt(src, name_node)
        if not name.isidentifier():
            # Cheap invariant, expensive absence: a name that is not an identifier means
            # the offsets are lying (see _txt). Without this the mismatch is SILENT --
            # the garbage name simply matches no C verdict and the function is skipped.
            raise CfgGateError(
                f"extracted Rust fn name {name!r} is not an identifier -- offsets are wrong"
            )
        out.append(
            RustFn(
                name=name,
                attr_start=attr_start_of(kids, i, node),
                fn_start=node.start_byte,
                body_start=body.start_byte,
                end=node.end_byte,
            )
        )
    return out


def attr_start_of(kids, i, node) -> int:
    """Start of the contiguous attribute run immediately above `node`."""
    attr_start = node.start_byte
    j = i - 1
    while j >= 0 and kids[j].type == "attribute_item":
        attr_start = kids[j].start_byte
        j -= 1
    return attr_start


def _lift_init(src: bytes, fn: RustFn, var: str) -> str | None:
    """The initializer of the translated `let mut <var>: T = <expr>;`.

    Taken from the Rust rather than re-derived from the C constant: fact sealing already
    audited this rendering, and a second opinion here could disagree silently.
    """
    body = src[fn.body_start : fn.end]
    tree = Parser(RUST_LANG).parse(body)

    found: list[str] = []

    def visit(node) -> None:
        if node.type == "let_declaration":
            pat = node.child_by_field_name("pattern")
            val = node.child_by_field_name("value")
            if pat is not None and val is not None:
                text = _txt(body, pat).replace("mut ", "").strip()
                if text == var:
                    found.append(_txt(body, val))
                    return
        for c in node.named_children:
            visit(c)

    visit(tree.root_node)
    return found[0] if found else None


def apply_file(rust_path: Path, verdicts: dict[str, CVerdict], *, write: bool = True) -> Applied:
    """Gate one translated Rust module. Each function is gated on the feature derived
    from ITS OWN guard, so a module branching on several config probes stays faithful.
    Edits are applied back-to-front so earlier offsets stay valid."""
    src = rust_path.read_bytes()
    res = Applied(path=rust_path)

    edits: list[tuple[int, int, bytes]] = []
    for fn in _rust_fns(src):
        v = verdicts.get(fn.name)
        if v is None or v.kind == UNGATED:
            continue
        if v.kind == COMPLEX:
            res.skipped.append((fn.name, v.reason))
            continue

        gate_on = f'#[cfg(feature = "{v.feature}")]\n'.encode()
        gate_off = f'#[cfg(not(feature = "{v.feature}"))]\n'.encode()

        if (
            gate_on in src[fn.attr_start : fn.fn_start]
            or gate_on in src[max(0, fn.attr_start - len(gate_on)) : fn.attr_start]
        ):
            continue  # already gated -- idempotent

        if v.kind == WHOLLY_GATED:
            edits.append((fn.attr_start, fn.attr_start, gate_on))
            res.gated.append(fn.name)
            continue

        # BODY_GATED: gate the translated arm and mint the off-arm twin.
        if v.ret_var is None:
            off_body = b"{}"
        else:
            init = _lift_init(src, fn, v.ret_var)
            if init is None:
                res.skipped.append(
                    (fn.name, f"no `let {v.ret_var} = ...` in the translated body to lift")
                )
                continue
            off_body = b"{\n    " + init.encode() + b"\n}"

        attrs = src[fn.attr_start : fn.fn_start]
        signature = src[fn.fn_start : fn.body_start].rstrip()
        twin = attrs + gate_off + signature + b" " + off_body + b"\n\n"
        edits.append((fn.attr_start, fn.attr_start, twin + gate_on))
        res.gated.append(fn.name)
        res.twinned.append(fn.name)

    for start, end, text in sorted(edits, key=lambda e: -e[0]):
        src = src[:start] + text + src[end:]

    if write and edits:
        rust_path.write_bytes(src)
    return res


def run(
    c_dir: Path, rust_dir: Path, guards: frozenset[str], *, apply: bool
) -> tuple[list[CVerdict], list[Applied]]:
    verdicts: list[CVerdict] = []
    for c in sorted(list(c_dir.glob("*.c")) + list(c_dir.glob("*.h"))):
        verdicts.extend(classify_c(c, guards))

    by_name: dict[str, CVerdict] = {}
    for v in verdicts:
        # A name defined in two TUs is `static`; both copies get the same verdict or the
        # transform is ambiguous, so refuse rather than pick one.
        prev = by_name.get(v.name)
        if prev is not None and prev.kind != v.kind:
            raise CfgGateError(
                f"`{v.name}` classified {prev.kind} in {prev.source} but {v.kind} in {v.source}"
            )
        by_name[v.name] = v

    applied: list[Applied] = []
    for r in sorted(rust_dir.glob("*.rs")):
        applied.append(apply_file(r, by_name, write=apply))
    return verdicts, applied


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(prog="rustconv.cfg_gate", description=__doc__)
    ap.add_argument("--c-dir", required=True, type=Path)
    ap.add_argument("--rust-dir", required=True, type=Path)
    ap.add_argument(
        "--guard",
        action="append",
        default=[],
        help="C macro to gate on, repeatable. Default: every macro the module tests that "
        "wine's config.h marks `#undef`.",
    )
    ap.add_argument("--repo-root", type=Path, default=Path.cwd())
    ap.add_argument("--apply", action="store_true", help="write; default is report-only")
    args = ap.parse_args(argv)

    guards = frozenset(args.guard) or guards_for_module(args.c_dir, args.repo_root)
    if not guards:
        raise CfgGateError(
            f"no guards: {args.c_dir} tests no macro that config.h marks `#undef`. "
            "Pass --guard explicitly if that is wrong."
        )
    print("guards: " + ", ".join(f"{g} -> feature {g.lower()}" for g in sorted(guards)))

    verdicts, applied = run(args.c_dir, args.rust_dir, guards, apply=args.apply)

    counts: dict[str, int] = {}
    for v in verdicts:
        counts[v.kind] = counts.get(v.kind, 0) + 1
    print(f"C functions scanned: {len(verdicts)}")
    for k in (UNGATED, WHOLLY_GATED, BODY_GATED, COMPLEX):
        print(f"  {k:<14} {counts.get(k, 0)}")

    hard = [v for v in verdicts if v.kind == COMPLEX]
    if hard:
        print(f"\nCOMPLEX -- not transformed, handle by hand ({len(hard)}):")
        for v in hard:
            print(f"  {v.source}: {v.name}  -- {v.reason}")

    tot_g = sum(len(a.gated) for a in applied)
    tot_t = sum(len(a.twinned) for a in applied)
    print(
        f"\nRust: {tot_g} fn(s) gated, {tot_t} off-arm twin(s)"
        f"{'' if args.apply else '  (REPORT ONLY -- pass --apply to write)'}"
    )
    for a in applied:
        if a.gated or a.skipped:
            print(
                f"  {a.path.name}: {len(a.gated)} gated, {len(a.twinned)} twinned"
                + (f", {len(a.skipped)} skipped" if a.skipped else "")
            )
        for name, why in a.skipped:
            print(f"      skip {name}: {why}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
