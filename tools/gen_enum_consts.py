#!/usr/bin/env python3
"""
gen_enum_consts.py — deterministically transcribe a C enum's members into Rust
consts, with the values COMPUTED from the header.

WHY THIS EXISTS
    A translated `match` arm naming an enum member that is not in scope is not a
    compile error: Rust reads the bare identifier as a fresh irrefutable binding
    that matches everything, so the first such arm swallows the whole dispatch
    (field manual §9.112 / §9.112a). The usual root fix is to widen the crate's
    bindgen island, but that is not always available — the enum may live in a
    header the island cannot parse, in a *secondary* island the crate does not
    import, or (scsiport) in a legacy header whose member list differs from the
    modern one bindgen would pick up.

    Transcription is then correct, but transcription BY HAND is how §9.113
    happened (61 invented kernel stop codes). So compute: walk the enum body,
    track implicit increments, evaluate explicit initialisers, and emit the
    result with its provenance. Anything unevaluable is a hard error, never a
    guess.

    Note the ordering trap this closes: an enum whose members are all implicit
    depends on POSITION, so a member list that silently differs between two
    headers yields different values for the same name. scsiport's
    SCSI_NOTIFICATION_TYPE is exactly that — srb.h has CallDisableInterrupts /
    CallEnableInterrupts at 4/5 where storport.h has _obsolete1 / _obsolete2.
    Always name the header the module actually compiles against.

USAGE
    python3 tools/gen_enum_consts.py sdk/include/ddk/srb.h _SCSI_NOTIFICATION_TYPE
    python3 tools/gen_enum_consts.py <header> <tag> --type u32 --only A,B,C
    python3 tools/gen_enum_consts.py <header> <tag> --check <file.rs>
"""

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent


class Unevaluable(Exception):
    pass


INT_RE = re.compile(r"^[-+]?(0[xX][0-9a-fA-F]+|\d+)[uUlL]*$")


def evaluate(expr, syms, depth=0):
    if depth > 12:
        raise Unevaluable(f"recursion limit on {expr!r}")
    e = expr.strip()
    while e.startswith("(") and e.endswith(")"):
        inner = e[1:-1]
        d = 0
        ok = True
        for c in inner:
            d += (c == "(") - (c == ")")
            if d < 0:
                ok = False
                break
        if not ok or d != 0:
            break
        e = inner.strip()
    if INT_RE.match(e):
        return int(e.rstrip("uUlL"), 0)
    if re.fullmatch(r"[A-Za-z_]\w*", e):
        if e in syms:
            return syms[e]
        raise Unevaluable(f"undefined symbol {e}")
    if not re.fullmatch(r"[0-9A-Za-z_\s()<>|&+\-*/^~]*", e):
        raise Unevaluable(f"unsupported syntax in {e!r}")
    subbed = re.sub(r"\b([A-Za-z_]\w*)\b", lambda m: str(evaluate(m.group(1), syms, depth + 1)), e)
    subbed = re.sub(r"\b(\d+)[uUlL]+\b", r"\1", subbed)
    try:
        return int(eval(subbed, {"__builtins__": {}}, {}))  # noqa: S307
    except Exception as exc:  # noqa: BLE001
        raise Unevaluable(f"cannot evaluate {e!r} ({exc})") from exc


def strip_comments(text):
    text = re.sub(r"/\*.*?\*/", " ", text, flags=re.S)
    return re.sub(r"//[^\n]*", " ", text)


def _body_at(text, open_brace_end):
    i = open_brace_end
    depth = 1
    start = i
    while i < len(text) and depth:
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
        i += 1
    return text[start : i - 1]


def find_enum_body(text, tag):
    """Return the brace body of `enum <tag> { ... }`, or None."""
    m = re.search(rf"\benum\s+{re.escape(tag)}\s*\{{", text)
    return None if m is None else _body_at(text, m.end())


def find_enum_body_by_member(text, member):
    """Locate an enum by one of its MEMBERS. Plenty of SDK enums are anonymous —
    `typedef enum { BM_x555RGB = 0x00, ... } BMFORMAT;` has no tag to name — and
    those are just as capable of producing a catch-all arm. Ambiguity is an error,
    not a coin flip: two enums sharing a member name would give different values."""
    hits = []
    for m in re.finditer(r"\benum\b[^{;]*\{", text):
        body = _body_at(text, m.end())
        if re.search(rf"(^|,)\s*{re.escape(member)}\s*(=|,|$)", body):
            hits.append(body)
    if not hits:
        return None
    if len(hits) > 1:
        raise Unevaluable(f"{member!r} appears in {len(hits)} enums here — name the tag instead")
    return hits[0]


def parse_members(body):
    """[(name, value)] with C's implicit-increment semantics."""
    out = []
    syms = {}
    nxt = 0
    for raw in body.split(","):
        item = raw.strip()
        if not item:
            continue
        m = re.match(r"^([A-Za-z_]\w*)\s*(?:=\s*(.+))?$", item, re.S)
        if not m:
            raise Unevaluable(f"cannot parse enumerator {item!r}")
        name = m.group(1)
        if m.group(2) is not None:
            val = evaluate(m.group(2), syms)
        else:
            val = nxt
        out.append((name, val))
        syms[name] = val
        nxt = val + 1
    return out


def main():
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("header")
    ap.add_argument(
        "tag",
        help="enum tag (e.g. _SCSI_NOTIFICATION_TYPE), or a member name with --by-member",
    )
    ap.add_argument("--by-member", action="store_true",
                    help="treat `tag` as a MEMBER name and find the enum containing it")
    ap.add_argument("--type", default="u32")
    ap.add_argument("--only", help="comma-separated subset of members to emit")
    ap.add_argument("--check", metavar="RSFILE", help="verify RSFILE instead of emitting")
    args = ap.parse_args()

    hdr = Path(args.header)
    if not hdr.is_absolute():
        hdr = REPO / hdr
    if not hdr.is_file():
        sys.exit(f"FATAL: header not found: {hdr}")

    text = strip_comments(hdr.read_text(encoding="utf-8", errors="replace"))
    try:
        body = (find_enum_body_by_member(text, args.tag) if args.by_member
                else find_enum_body(text, args.tag))
    except Unevaluable as exc:
        sys.exit(f"FATAL: {exc}")
    if body is None:
        kind = "member of any enum" if args.by_member else "`enum %s`" % args.tag
        sys.exit(f"FATAL: no {kind} named {args.tag} in {hdr}")
    try:
        members = parse_members(body)
    except Unevaluable as exc:
        sys.exit(f"FATAL: {exc}")

    if args.only:
        want = [w.strip() for w in args.only.split(",") if w.strip()]
        have = {n for n, _ in members}
        missing = [w for w in want if w not in have]
        if missing:
            # Loud, not silent: a member that is not in THIS header is the
            # ordering trap above — you are looking at the wrong enum.
            sys.exit(f"FATAL: not members of {args.tag} in {hdr.name}: {', '.join(missing)}")
        members = [(n, v) for n, v in members if n in want]

    rel = hdr.relative_to(REPO) if hdr.is_relative_to(REPO) else hdr

    if args.check:
        rs = Path(args.check)
        if not rs.is_absolute():
            rs = REPO / rs
        text = rs.read_text(encoding="utf-8", errors="replace")
        bad = 0
        checked = 0
        for name, value in members:
            m = re.search(rf"\b{re.escape(name)}\s*:\s*\w+\s*=\s*([^;]+);", text)
            if not m:
                continue
            checked += 1
            try:
                have = evaluate(m.group(1).replace("_", ""), {})
            except Unevaluable:
                print(f"  ?  {name}: cannot parse {m.group(1).strip()}")
                continue
            if have != value:
                print(f"  MISMATCH {name}: file={have} header={value}")
                bad += 1
        print(f"checked {checked} member(s) of {args.tag} against {rel}: {bad} mismatch(es)")
        return 1 if bad else 0

    print(f"// enum {args.tag} ({rel}), transcribed by tools/gen_enum_consts.py.")
    print("// Values COMPUTED from the header — never hand-typed (§9.113). These arms")
    print("// were bare identifiers with no definition in scope, i.e. catch-all")
    print("// bindings that made the whole dispatch take the first arm (§9.112a).")
    for name, value in members:
        print(f"pub const {name}: {args.type} = {value};")
    return 0


if __name__ == "__main__":
    sys.exit(main())
