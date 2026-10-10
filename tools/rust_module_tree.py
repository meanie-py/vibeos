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
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from tools.lint_facts import source_facts

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


# Necessary (not sufficient) tokens for the module grammar below. Comments and
# strings can produce false positives, which simply take the full parser path.
# No parsed declaration can exist when these literal keywords are absent.
RE_MODULE_KEYWORD = re.compile(r"\bmod\b|include")
_COMMENT_START = re.compile(r'//|/\*|"')
_STRING_DELIMITER = re.compile(r'["\\]')
_BLOCK_DELIMITER = re.compile(r"/\*|\*/")


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
        match = _COMMENT_START.search(text, i)
        if match is None:
            out.append(text[i:])
            break
        out.append(text[i : match.start()])
        token = match.group()
        i = match.end()
        if token == "//":
            end = text.find("\n", i)
            i = n if end < 0 else end
        elif token == "/*":
            depth = 1
            while depth:
                marker = _BLOCK_DELIMITER.search(text, i)
                if marker is None:
                    i = n
                    break
                depth += 1 if marker.group() == "/*" else -1
                i = marker.end()
            out.append(" ")
        else:
            while i < n:
                marker = _STRING_DELIMITER.search(text, i)
                if marker is None:
                    i = n
                    break
                i = marker.end()
                if marker.group() == '"':
                    break
                i += 1
            out.append(text[match.start() : i])
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
    if not RE_MODULE_KEYWORD.search(text):
        return []
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
        for brace in re.finditer(r"[{}]", text[i:upto]):
            c = brace.group()
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


@source_facts()
def module_facts(raw):
    if not RE_MODULE_KEYWORD.search(raw):
        return [], [], [], False
    text = strip_comments(raw)
    return (
        path_mod_targets(text),
        RE_INCLUDE.findall(text),
        decl_sites(raw),
        bool(RE_INCLUDE_ENV.search(text)),
    )


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
        if not RE_MODULE_KEYWORD.search(raw):
            continue  # reachable leaf: no module/include edges to discover
        paths_by_name, includes, declarations, _opaque = module_facts(raw)
        fdir = os.path.dirname(f)
        stem = os.path.splitext(os.path.basename(f))[0]
        base = fdir if stem in ("lib", "main", "mod") else os.path.join(fdir, stem)
        # A `#[path]` inside an INLINE module resolves against that module's directory (the
        # Rust reference: the file's directory, plus `stem/` for a non-mod-rs file, plus the
        # inline module names). ndis's `pub mod ndis { #[path = "30stubs.rs"] pub mod
        # _30stubs; }` is src/ndis/30stubs.rs, and read as src/30stubs.rs it was an orphan.
        encl_of = {name: encl for name, encl in declarations}
        for name, paths in paths_by_name:
            encl = inline + encl_of.get(name, [])
            pdir = os.path.join(base, *encl) if encl else fdir
            for rel in paths:
                queue.append((os.path.normpath(os.path.join(pdir, rel)), []))
        for include in includes:
            queue.append((os.path.normpath(os.path.join(fdir, include)), []))
        for name, encl in declarations:
            d = os.path.join(base, *(inline + encl))
            for cand in (os.path.join(d, name + ".rs"), os.path.join(d, name, "mod.rs")):
                if os.path.exists(cand):
                    queue.append((cand, []))
                    break
    _cache[crate_dir] = live
    return live


def crate_phantom_mods(crate_dir):
    """Every `mod X;` that resolves to NO file, as (declaring_file, name, [searched paths]).

    The dual of `crate_live_files`, and deliberately the same walk: that function drops an
    unresolvable `mod` on the floor (the `break`-less fall-through below its candidate loop),
    so a module declared against a file that does not exist is invisible to every gate built
    on it. It is invisible to the rest of the tree too — a crate parked out of
    `workspace.members` is never compiled, and no scanner opens a path that is absent. desk
    carried `pub mod guid;` with no guid.rs, and separately shipped 17 of 20 TUs while its
    lib.rs still claimed all 20; both survived a full green `check` and a green `check-tooling`.

    Reuses `crate_live_files` for traversal so the resolution rules cannot drift between the
    two directions: any file the walk actually reached is re-read here with the same
    `decl_sites`/`base` logic that reached it."""
    phantom = []
    for f in sorted(crate_live_files(crate_dir)):
        try:
            raw = Path(f).read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        fdir = os.path.dirname(f)
        stem = os.path.splitext(os.path.basename(f))[0]
        base = fdir if stem in ("lib", "main", "mod") else os.path.join(fdir, stem)
        # A `#[path = "..."]` module names its own file and is resolved by path_mod_targets,
        # not by the name-based search below — excluding it here keeps those from reading as
        # phantoms (the per-arch `types_i386.rs` selector is exactly this shape).
        paths_by_name, _includes, declarations, _opaque = module_facts(raw)
        by_path = {name for name, _ in paths_by_name}
        for name, encl in declarations:
            if name in by_path:
                continue
            d = os.path.join(base, *encl)
            cands = [os.path.join(d, name + ".rs"), os.path.join(d, name, "mod.rs")]
            if not any(os.path.exists(c) for c in cands):
                phantom.append((f, name, cands))
    return phantom


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
            raw = Path(f).read_text(encoding="utf-8", errors="replace")
            if "include" not in raw:
                continue
            if module_facts(raw)[3]:
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
