"""Which .rs files a crate actually compiles — the one implementation.

`verify_match_consts` grew this to suppress findings in files nothing compiles;
`scan_orphan_modules` needs the same answer in the other direction (a file nothing
compiles is itself the finding). Two copies of a reachability walk is the same
"two copies of the truth" shape this repo keeps getting burned by, so both import
from here.

The walk follows, from each crate root:
  * `mod X;`  ->  X.rs | X/mod.rs, relative to the enclosing module's directory
  * `#[path = "..."] mod X;`
  * inline `mod X { ... }` nesting (a file module inside one lives a dir deeper)
  * `include!("...")`  ->  per-arch generated files are pulled in this way
Roots are src/lib.rs, src/main.rs, and every `path =` in Cargo.toml's
[lib]/[[bin]] tables.

Everything here fails OPEN: an unparseable crate reports no orphans rather than
declaring live code dead. That direction matters — verify_match_consts SUPPRESSES
findings in files this calls dead, so a false "dead" hides real bugs.
"""

import os
import re
from pathlib import Path

# `r#` is not optional decoration: ws2_32 translates upstream's async.c to a module
# that can only be spelled `pub mod r#async;` (a keyword since edition 2018). Without
# this, its 1,941 live lines read as an orphan.
RE_MOD_ANY = re.compile(
    r"(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?\bmod\s+(?:r#)?(\w+)\s*([;{])"
)
RE_PATH_MOD = re.compile(r'#\[path\s*=\s*"([^"]+)"\]\s*(?:pub\s+)?mod\s+(?:r#)?(\w+)\s*;')
# The per-arch selector is `#[cfg_attr(target_arch = "x86", path = "x_i386.rs")]`,
# stacked three deep over one `mod generated;` — ole32 routes 34K lines of generated
# COBJMACROS wrappers that way. A `#[path = ...]`-only regex misses every one of
# them and calls all three arch files dead. Match the whole attribute run and take
# every `path` in it: a file live on ANY arch is live.
RE_MOD_WITH_ATTRS = re.compile(
    r"((?:#\[[^\]]*\]\s*)+)(?:pub(?:\([^)]*\))?\s+)?mod\s+(?:r#)?(\w+)\s*;"
)
RE_ATTR_PATH = re.compile(r'\bpath\s*=\s*"([^"]+)"')
RE_INCLUDE = re.compile(r'include!\s*\(\s*"([^"]+)"\s*\)')
# `include!(env!("WINREG_STUB_RS"))` — the path is a build.rs output we cannot
# resolve from source. Seeing one makes the whole crate fail open.
RE_INCLUDE_ENV = re.compile(r"include!\s*\(\s*(?:env|concat)!")
RE_CARGO_TARGET_PATH = re.compile(r'^\s*path\s*=\s*"([^"]+)"', re.M)


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


def strip_comments(text):
    """Blank out Rust comments, single-pass, string-aware.

    A two-regex version (block-comments first, then line-comments) is wrong in the
    dangerous direction and was wrong here: a `/*` inside a LINE comment opens a
    phantom block comment that runs to the next `*/` anywhere in the file. freeldr's
    `// (called from arch/*/multiboot.S asm)` swallowed 53 of its 87 `mod`
    declarations, so the walk called 42,750 lines of live boot loader dead. 20+
    crate roots carry the same `path/*`-in-a-comment shape.

    Rust block comments NEST, and `"http://x"` / `"/*"` in a string must not open
    one, so this walks the text once rather than composing regexes.
    """
    out = []
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        if c == '"':
            out.append(c)
            i += 1
            while i < n:
                if text[i] == "\\":
                    out.append(text[i : i + 2])
                    i += 2
                    continue
                out.append(text[i])
                if text[i] == '"':
                    i += 1
                    break
                i += 1
            continue
        if text.startswith("//", i):
            while i < n and text[i] != "\n":
                i += 1
            continue
        if text.startswith("/*", i):
            depth, i = 1, i + 2
            while i < n and depth:
                if text.startswith("/*", i):
                    depth, i = depth + 1, i + 2
                elif text.startswith("*/", i):
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
            out.append(" ")
            continue
        out.append(c)
        i += 1
    return "".join(out)


def decl_sites(text):
    """[(module_name, [enclosing inline-module names])] for every `mod X;` in `text`.

    Two shapes broke a line-anchored regex, and BOTH of them mark live files dead:
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
    text = strip_comments(text)
    sites = []
    stack = []  # [(name, depth_at_open)]
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


def path_mod_targets(text):
    """[(mod_name, [target paths])] for every `mod X;` carrying a `path = "..."`,
    in either the plain `#[path]` or the cfg_attr-selected form."""
    out = []
    for m in RE_MOD_WITH_ATTRS.finditer(text):
        paths = RE_ATTR_PATH.findall(m.group(1))
        if paths:
            out.append((m.group(2), paths))
    return out


def crate_roots(crate_dir):
    """Compilation roots: src/lib.rs, src/main.rs, and Cargo.toml `path =` targets."""
    roots = []
    src = os.path.join(crate_dir, "src")
    for n in ("lib.rs", "main.rs"):
        f = os.path.join(src, n)
        if os.path.exists(f):
            roots.append(f)
    cargo = os.path.join(crate_dir, "Cargo.toml")
    try:
        text = Path(cargo).read_text(encoding="utf-8", errors="replace")
    except OSError:
        return roots
    # Only [lib]/[[bin]]/[[test]]/... target tables carry a `path` that names a
    # crate root; a `path` under [dependencies] names a sibling CRATE directory
    # and must not be mistaken for one.
    for block in re.split(r"^\s*\[", text, flags=re.M)[1:]:
        head, _, body = block.partition("]")
        head = head.strip().strip("[")
        if head.split(".")[0] not in ("lib", "bin", "test", "bench", "example"):
            continue
        for m in RE_CARGO_TARGET_PATH.finditer(body):
            f = os.path.normpath(os.path.join(crate_dir, m.group(1)))
            if os.path.exists(f):
                roots.append(f)
    return roots


def crate_live_files(crate_dir, _cache={}):  # noqa: B006 — deliberate memo cache
    # The mutable default IS the cache: it is read and written across calls (see the
    # `if crate_dir in _cache` below). Ruff's suggested fix (default None, init inside)
    # would hand back a fresh dict per call and silently turn every hit into a full
    # re-walk of the crate's module tree.
    """Every .rs REACHABLE from a crate root via `mod` / `#[path]` / `include!`.

    Empty set means "could not establish" — callers must fail open on it.
    """
    if crate_dir in _cache:
        return _cache[crate_dir]
    _cache[crate_dir] = set()
    live, queue = set(), [(f, []) for f in crate_roots(crate_dir)]
    while queue:
        f, inline = queue.pop()
        if f in live or not os.path.exists(f):
            continue
        live.add(f)
        try:
            raw = Path(f).read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        text = strip_comments(raw)
        fdir = os.path.dirname(f)
        stem = os.path.splitext(os.path.basename(f))[0]
        base = fdir if stem in ("lib", "main", "mod") else os.path.join(fdir, stem)
        for _name, paths in path_mod_targets(text):
            for rel in paths:
                queue.append((os.path.normpath(os.path.join(fdir, rel)), []))
        for inc in RE_INCLUDE.finditer(text):
            queue.append((os.path.normpath(os.path.join(fdir, inc.group(1))), []))
        for name, encl in decl_sites(raw):
            d = os.path.join(base, *(inline + encl))
            for cand in (os.path.join(d, name + ".rs"), os.path.join(d, name, "mod.rs")):
                if os.path.exists(cand):
                    queue.append((cand, []))
                    break
    _cache[crate_dir] = live
    return live


def crate_is_opaque(crate_dir, _cache={}):  # noqa: B006 — intentional memo cache
    """True when a crate root resolves a module path we cannot follow from source.

    `include!(env!("WINREG_STUB_RS"))` names a build.rs output; the file it pulls
    in is real and live, but nothing in the tree says which. Report no orphans for
    such a crate rather than guess.
    """
    if crate_dir in _cache:
        return _cache[crate_dir]
    opaque = False
    for f in crate_live_files(crate_dir):
        try:
            if RE_INCLUDE_ENV.search(
                strip_comments(Path(f).read_text(encoding="utf-8", errors="replace"))
            ):
                opaque = True
                break
        except OSError:
            continue
    _cache[crate_dir] = opaque
    return opaque


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
