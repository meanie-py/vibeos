#!/usr/bin/env python3
"""Robust re-assembler for per-function rustconv caches.

The production stitcher (rustconv/stitcher.py) has a merge-stage bug that, on
large files, (a) drops functions absent from the topological `deps` list and
(b) occasionally orphans a function body (loses its header) during extern-block
merging. The per-function cache files under .responses/<subsys>__<stem>/ are
each individually clean and complete (consts + extern blocks + one fn body), so
this tool re-assembles deterministically from those, GUARANTEEING every cached
function appears exactly once:

  1. order functions by the numeric prefix of the timestamped archive dir
     (00_preamble, 01_Name, ...), pulling COMPLETE content from the name-keyed
     resume dir
  2. split each file into {consts/statics/types, extern blocks, fn body}
  3. dedup consts/types by exact text; dedup extern decls by symbol name and
     DROP any extern decl whose name is a function the file itself defines
     (those are caller forward-decls of file-local exports — redundant + the
     source of the malformed `#[no_mangle] ... fn NAME(...);` merge artifacts)
  4. emit consts, externs, then bodies in source order

Usage:
  python reassemble_from_cache.py <resume_dir> <archive_dir> <out.rs>
"""
import sys, re, pathlib

EXTERN_START = re.compile(r'^\s*(?:unsafe\s+)?extern\s+"[^"]*"\s*\{')
FN_SIG = re.compile(r'\bfn\s+(\w+)\s*\(')
ATTR = re.compile(r'^\s*#\[')
DECL_NAME = re.compile(r'\bfn\s+(\w+)\s*\(')
STATIC_NAME = re.compile(r'\bstatic\s+(?:mut\s+)?(\w+)\s*:')


def _strip_use_lines(text):
    """Split out top-level `use ...;` lines (kept separately to avoid dup imports)."""
    uses, body = [], []
    for ln in text.splitlines():
        if ln.lstrip().startswith("use ") and ln.rstrip().endswith(";"):
            uses.append(ln.strip())
        else:
            body.append(ln)
    return uses, "\n".join(body)


def split_file(text):
    """Return (preamble_items, extern_blocks, body) for one cache file.

    preamble_items: list of standalone const/static/type lines (before the fn)
    extern_blocks:  list of (linkage, [decl_lines]) before the fn
    body:           the function definition text (its attributes + header → EOF)

    The function definition is detected by the first top-level line that carries
    an `fn NAME(` signature and is NOT an extern-block opener (those are consumed
    separately). Any contiguous `#[...]` attribute lines immediately preceding it
    are pulled back out of the preamble into the body.
    """
    lines = text.splitlines()
    i, n = 0, len(lines)
    preamble, externs = [], []
    loose_decls = []   # bare `fn NAME(...);` forward-decls outside extern blocks
    body_start = None
    while i < n:
        line = lines[i]
        s = line.strip()
        if not s:
            i += 1
            continue
        if EXTERN_START.match(line):
            m = re.match(r'^\s*(?:unsafe\s+)?extern\s+"([^"]*)"', line)
            linkage = m.group(1) if m else "C"
            depth = line.count('{') - line.count('}')
            decls, i = [], i + 1
            while i < n and depth > 0:
                decls.append(lines[i])
                depth += lines[i].count('{') - lines[i].count('}')
                i += 1
            externs.append((linkage, decls))
            continue
        if FN_SIG.search(line):
            # Distinguish a real definition (has a `{` body) from a forward
            # declaration (terminates with `;` before any `{`). Scan forward
            # across a possibly multi-line signature.
            depth = 0
            has_body = False
            j = i
            while j < n:
                depth += lines[j].count('{') - lines[j].count('}')
                if '{' in lines[j]:
                    has_body = True
                    break
                if lines[j].rstrip().endswith(';') and depth <= 0:
                    break
                j += 1
            if has_body:
                body_start = i
                break
            # forward declaration: collect as a loose extern "C" decl and skip it
            sig = " ".join(l.strip() for l in lines[i:j + 1]).strip()
            loose_decls.append(sig)
            i = j + 1
            continue
        # standalone const/static/type/use line
        preamble.append(line)
        i += 1
    if loose_decls:
        externs.append(("C", loose_decls))
    # pull contiguous leading attribute/blank lines (in `lines`) into the body,
    # and drop any attribute lines from the preamble (they belong to the fn)
    if body_start is not None:
        j = body_start - 1
        while j >= 0 and (not lines[j].strip() or ATTR.match(lines[j])):
            j -= 1
        body_start = j + 1
        preamble = [p for p in preamble if not ATTR.match(p)]
        body = "\n".join(lines[body_start:])
    else:
        body = ""
    return preamble, externs, body


def main():
    resume_dir = pathlib.Path(sys.argv[1])
    archive_dir = pathlib.Path(sys.argv[2])
    out = pathlib.Path(sys.argv[3])

    # ordering + names from the numeric archive prefixes
    ordered = []  # (idx, name)
    for f in sorted(archive_dir.glob("*.rs")):
        m = re.match(r'(\d+)_(.+)\.(?:rs|partial)$', f.name)
        if not m:
            continue
        ordered.append((int(m.group(1)), m.group(2)))
    ordered.sort()

    defined_fns = set()
    loaded = {}  # name -> text (from complete resume dir, fallback archive)
    for _, name in ordered:
        if name == "preamble" or name == "__preamble__":
            continue
        rp = resume_dir / f"{name}.rs"
        text = rp.read_text() if rp.exists() else ""
        if not text:
            # fall back to whatever the archive has
            cand = list(archive_dir.glob(f"*_{name}.rs")) + list(archive_dir.glob(f"*_{name}.partial"))
            if cand:
                text = cand[0].read_text()
        loaded[name] = text
        defined_fns.add(name)

    all_preamble, seen_pre = [], set()
    extern_by_link = {}   # linkage -> list[decl]
    extern_seen = {}      # linkage -> set[name]
    bodies = []

    for _, name in ordered:
        if name in ("preamble", "__preamble__"):
            continue
        text = loaded.get(name, "")
        if not text:
            print(f"  WARN: no content for {name}", file=sys.stderr)
            continue
        pre, externs, body = split_file(text)
        for ln in pre:
            key = ln.strip()
            if key and key not in seen_pre:
                seen_pre.add(key)
                all_preamble.append(ln)
        for linkage, decls in externs:
            # join multi-line decls into single decls ending in ;
            joined, cur = [], ""
            for d in decls:
                ds = d.strip()
                if not ds:
                    continue
                cur = (cur + " " + ds).strip() if cur else ds
                if cur.endswith(';'):
                    joined.append(cur)
                    cur = ""
            if cur:
                joined.append(cur)
            extern_by_link.setdefault(linkage, [])
            extern_seen.setdefault(linkage, set())
            for d in joined:
                # A valid foreign decl is `fn NAME(...);` / `static NAME: T;`,
                # terminates with `;`, and contains no body braces. Anything
                # else (a stray `}`, a leaked definition fragment) is dropped —
                # robust against brace-counting slips in malformed cache blocks.
                if '{' in d or '}' in d or not d.rstrip().endswith(';'):
                    continue
                m = DECL_NAME.search(d) or STATIC_NAME.search(d)
                dn = m.group(1) if m else None
                if dn is None:
                    continue
                # DROP forward-decls of functions this file defines
                if dn in defined_fns:
                    continue
                if dn in extern_seen[linkage]:
                    continue
                extern_seen[linkage].add(dn)
                extern_by_link[linkage].append(d)
        if body.strip():
            bodies.append(body.strip())

    parts = []
    # Standard substrate imports the production stitcher injects via
    # _generate_imports_from_resolve (the per-function cache files assume them).
    parts.append("use ntdll_sys::nt::*;")
    parts.append("use core::ffi::{c_void, c_char};")
    parts.append("")

    # File-scope TYPE PREAMBLE (structs / typedefs / COM interface vtables /
    # #define→const / static tables) — the synthetic `__preamble__` unit. Emit
    # it before the consts/externs/bodies so the structs they reference exist.
    pre_path = resume_dir / "__preamble__.rs"
    pre_text = pre_path.read_text() if pre_path.exists() else ""
    if pre_text.strip():
        pre_uses, pre_body = _strip_use_lines(pre_text)
        parts.append("// ---- file-scope type preamble (translated) ----")
        parts.append(pre_body.strip())
        parts.append("")
        print(f"  + type preamble: {len(pre_body.splitlines())} lines", file=sys.stderr)
    else:
        print("  WARN: __preamble__ is empty — file-scope structs/types still MISSING",
              file=sys.stderr)

    if all_preamble:
        parts.append("// ---- consts / statics / type aliases (deduped) ----")
        parts.extend(all_preamble)
        parts.append("")
    for linkage in sorted(extern_by_link):
        decls = extern_by_link[linkage]
        if decls:
            parts.append(f'extern "{linkage}" {{')
            for d in decls:
                parts.append(f"    {d}")
            parts.append("}")
            parts.append("")
    for b in bodies:
        parts.append(b)
        parts.append("")

    out.write_text("\n".join(parts) + "\n")
    print(f"  reassembled {len(bodies)} function bodies, "
          f"{len(all_preamble)} preamble items, "
          f"{sum(len(v) for v in extern_by_link.values())} extern decls -> {out}")


if __name__ == "__main__":
    main()
