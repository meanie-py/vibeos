#!/usr/bin/env python3
"""repair_pass.py — rustconv Rust->Rust repair pass for LLM-translated driver crates.

The agentic grind wanders; a from-scratch re-translation throws away good structure. This
instead REPAIRS each broken function in place: it hands DeepSeek a tightly-constrained task —
the (broken) Rust function, its EXACT cargo errors (rustc's `expected X, found Y` + field
hints), the STRUCT LAYOUTS of the types it touches, and the original C as semantic ground
truth — and asks it to fix only the errors while preserving logic. The structural translation
DeepSeek already got right is kept; only the localized casts/paths are repaired.

Deterministic tools (resolve_imports, harvest_consts, fix_field_paths) should run FIRST so the
residual the model sees is minimal (ideally cast-only). The model never has to discover types
or imports — those are handed over.

Usage: repair_pass.py <crate-src-dir> <target-triple> [--files a.rs,b.rs] [--limit N]
                      [--model deepseek-v4-pro] [--dry-run]
"""
import sys, re, os, json, subprocess, argparse
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


def brace_block(text, open_idx):
    """From the `{` at open_idx, return index just past the matching `}` (string/comment aware)."""
    depth, i, n = 0, open_idx, len(text)
    while i < n:
        c = text[i]
        if c == '"':
            i += 1
            while i < n and text[i] != '"':
                i += 2 if text[i] == '\\' else 1
        elif c == '/' and i + 1 < n and text[i + 1] == '/':
            while i < n and text[i] != '\n':
                i += 1
        elif c == '{':
            depth += 1
        elif c == '}':
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return n


def functions(src):
    """Yield (name, start, end) for each top-level fn in the Rust source."""
    for m in re.finditer(r"(?:#\[no_mangle\]\s*)?(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+\"[A-Za-z]+\"\s+)?fn\s+([A-Za-z0-9_]+)\s*[(<]", src):
        name = m.group(1)
        ob = src.find("{", m.end())
        if ob == -1:
            continue
        # ensure the { is the body, not a where-clause/type — find first { after the signature's )
        end = brace_block(src, ob)
        # rewind start to include attributes/doc lines above
        ls = src.rfind("\n", 0, m.start()) + 1
        yield name, ls, end


def line_to_func(src):
    """Map 1-based line number -> enclosing function name."""
    fns = list(functions(src))
    line_starts = [0]
    for ch in src.split("\n")[:-1]:
        line_starts.append(line_starts[-1] + len(ch) + 1)
    out = {}
    for name, s, e in fns:
        ls = src.count("\n", 0, s) + 1
        le = src.count("\n", 0, e) + 1
        for ln in range(ls, le + 1):
            out[ln] = (name, s, e)
    return out


def rustc_messages(crate, target, ws):
    r = subprocess.run(["cargo", "check", "-p", crate, "--target", target,
                        "--message-format=json"], cwd=ws, capture_output=True, text=True)
    for ln in r.stdout.splitlines():
        try:
            m = json.loads(ln)
        except ValueError:
            continue
        if m.get("reason") == "compiler-message":
            yield m["message"]


def c_function(c_src, name):
    """Best-effort extract the C definition of `name` (signature line at col 0 ... matching })."""
    m = re.search(r"(?m)^" + re.escape(name) + r"\s*\(", c_src)
    if not m:
        # also try `TYPE name(` on its own line (return type on prior line)
        m = re.search(r"(?m)^[A-Za-z_].*\b" + re.escape(name) + r"\s*\(", c_src)
    if not m:
        return None
    ob = c_src.find("{", m.start())
    if ob == -1:
        return None
    # back up to the return-type line
    sline = c_src.rfind("\n", 0, m.start())
    sline = c_src.rfind("\n", 0, sline) + 1 if sline != -1 else 0
    return c_src[sline:brace_block(c_src, ob)]


def type_defs(types_src, type_names):
    """Return concatenated `pub struct` defs (+_underscored) for the named types."""
    out = []
    for t in sorted(set(type_names)):
        for nm in (t, "_" + t):
            m = re.search(r"pub struct " + re.escape(nm) + r"\b[^{]*\{", types_src)
            if m:
                end = brace_block(types_src, m.end() - 1)
                out.append(types_src[m.start():end])
                break
    return "\n\n".join(out)


_FN_DECL = re.compile(r"(?:pub\s+)?(?:unsafe\s+)?(?:extern\s+\"[A-Za-z]+\"\s+)?fn\s+([A-Za-z0-9_]+)\s*\(")


def sig_index(crate_src, extra_dirs):
    """name -> one-line `fn NAME(...) -> R` signature, harvested from every .rs we can see.

    The crate's own files declare their callee shims in `extern \"C\"`/`extern \"system\"` blocks
    (e.g. `fn NT_ASSERT(expr: i32)`), and ntkrnl_sys declares the kernel ones. Without these the
    model GUESSES the callee ABI — it assumes NT_ASSERT takes bool, strips the (correct) `as i32`,
    and invents E0308s. Handing over the exact signature turns the guess into a lookup."""
    idx = {}
    files = []
    for d in [crate_src] + extra_dirs:
        if os.path.isdir(d):
            files += [os.path.join(d, f) for f in os.listdir(d) if f.endswith(".rs")]
    for p in files:
        try:
            s = open(p).read()
        except OSError:
            continue
        for m in _FN_DECL.finditer(s):
            nm = m.group(1)
            # capture the signature: from `fn` keyword to the terminating `;` or `{`
            kw = s.rfind("fn ", 0, m.end())
            tail = s[m.end() - 1:]  # starts at '('
            depth, i = 0, 0
            while i < len(tail):  # balance the param parens
                if tail[i] == "(":
                    depth += 1
                elif tail[i] == ")":
                    depth -= 1
                    if depth == 0:
                        i += 1
                        break
                i += 1
            ret = re.match(r"\s*->\s*[^;{]+", tail[i:])
            sig = "fn " + nm + tail[:i] + (ret.group(0).rstrip() if ret else "")
            sig = re.sub(r"\s+", " ", sig).strip()
            # prefer the shortest decl (extern shims are terse; defs carry bodies/where-clauses)
            if nm not in idx or len(sig) < len(idx[nm]):
                idx[nm] = sig
    return idx


def callee_sigs(rust_fn, idx):
    """Signatures of every function CALLED in the broken body that we have a declaration for."""
    called = set(re.findall(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(", rust_fn))
    out = [idx[n] for n in sorted(called) if n in idx]
    return "\n".join(out)


def _calls(src):
    """Multiset of called function names — `NAME(` occurrences (skips Rust keywords)."""
    kw = {"if", "while", "for", "match", "fn", "return", "as", "let", "else", "loop"}
    from collections import Counter
    return Counter(n for n in re.findall(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(", src) if n not in kw)


def validate_repair(orig, new, name):
    """Reject LLM 'repairs' that bullshit. Returns None if OK, else a reason string.

    The PoC caught DeepSeek (a) dropping a masking call — `SRB_STATUS(x)==..` -> `x==..` — and
    (b) emitting a C ternary `cond ? 0 : 1` with a rambling comment. These are logic regressions
    a compiler may NOT catch (the mangled line still type-checks), so we gate on the source itself
    rather than trusting cargo. Cheap, deterministic, runs before the splice."""
    if f"fn {name}" not in new:
        return "output lost the function signature"
    for bad in ("todo!()", "unimplemented!()", "unreachable!()"):
        if bad in new and bad not in orig:
            return f"introduced stub {bad}"
    # C ternary: a `?` with a `:` before the next stmt boundary (Rust `?` is postfix, no `:`).
    if re.search(r"\?[^;{}\n]*[^:]:[^:]", new) and not re.search(r"\?[^;{}\n]*[^:]:[^:]", orig):
        return "introduced C ternary `? :` (invalid Rust)"
    # dropped calls: every function the original CALLED must still be called (>= count).
    co, cn = _calls(orig), _calls(new)
    dropped = [k for k in co if cn[k] < co[k]]
    if dropped:
        return "dropped call(s): " + ", ".join(sorted(dropped))
    return None


SYS_PROMPT = """You repair a single Rust function that was mechanically translated from C and
fails to compile. You are given: the original C function, the current broken Rust, the exact
rustc errors, the Rust struct layouts of the types involved, and the EXACT signatures of every
function it calls.

HARD RULES:
1. Fix ONLY what the errors require — add `as` casts to the EXPECTED type, fix bindgen field
   paths (`.__bindgen_anon_N.field`, `.as_ref()`/`.as_mut()` for union fields), fix arg counts.
2. PRESERVE the logic, control flow, names, and structure exactly. Do not rewrite, simplify,
   re-order, or drop anything. The C original is the semantic ground truth.
3. CALLEE SIGNATURES ARE LAW. Every callee's exact signature is given below. Match each call to
   it. These shims are deliberately C-shaped: `NT_ASSERT(expr: i32)` takes an i32, NOT a bool, so
   `NT_ASSERT(cond as i32)` is CORRECT — do not strip that cast. Helpers like `SRB_STATUS(u8)->u8`
   mask bits; NEVER drop a call to one. If a call's args mismatch, add casts to fit the SIGNATURE,
   do not remove the call or change which function is called.
4. Do NOT remove existing `as T` casts unless a provided signature proves the cast is wrong — the
   original transliteration's casts usually match these C-shaped shims.
5. NO stubs, no `todo!()`/`unimplemented!()`/`panic!()`, no dummy returns. NO new helper functions.
6. Do NOT change the function signature or its `#[no_mangle]`/`extern` attributes.
7. Do NOT add `use` statements or glob imports — imports are already resolved at module level. A
   `cannot find function X` error is an import gap, NOT your problem: leave the call as-is.
8. Output ONLY the corrected function (with its attributes), nothing else — no prose, no fences.
"""


def build_prompt(name, rust_fn, errs, c_fn, types, sigs):
    parts = [f"# Repair function `{name}`\n"]
    if c_fn:
        parts.append("## Original C (semantic ground truth)\n```c\n" + c_fn.strip() + "\n```\n")
    parts.append("## Current broken Rust\n```rust\n" + rust_fn.strip() + "\n```\n")
    parts.append("## rustc errors\n" + "\n".join(errs) + "\n")
    if types:
        parts.append("## Relevant Rust struct layouts (use these exact field paths)\n```rust\n" + types + "\n```\n")
    if sigs:
        parts.append("## Callee signatures (call these EXACTLY — args/casts must fit)\n```rust\n" + sigs + "\n```\n")
    parts.append("\nOutput the corrected `" + name + "` function only.")
    return "\n".join(parts)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("crate_src")
    ap.add_argument("target")
    ap.add_argument("--files", default="")
    ap.add_argument("--limit", type=int, default=0)
    ap.add_argument("--model", default="deepseek-v4-pro")
    ap.add_argument("--dry-run", action="store_true")
    a = ap.parse_args()

    crate_src = os.path.abspath(a.crate_src)
    crate = os.path.basename(os.path.dirname(crate_src))
    ws = os.path.abspath(os.path.join(crate_src, "../.."))
    arch = {"i686-pc-windows-gnu": "i386", "x86_64-pc-windows-gnu": "amd64",
            "aarch64-pc-windows-gnu": "arm64"}[a.target]
    types_src = open(os.path.join(crate_src, f"types_{arch}.rs")).read()
    # repo root for the C sources (src->crate->rust->lib->sdk->root)
    root = os.path.abspath(os.path.join(crate_src, "../../../../.."))
    cdir_map = {"classpnp-rust": "drivers/storage/class/classpnp",
                "cdfs-rust": "drivers/filesystems/cdfs",
                "cdrom-rust": "drivers/storage/class/cdrom"}
    cdir = os.path.join(root, cdir_map.get(crate, ""))

    # callee-signature index: crate's own extern shims + ntkrnl_sys kernel decls.
    ntk = os.path.join(ws, "ntkrnl-sys/src")
    SIGS = sig_index(crate_src, [ntk])

    only = set(f.strip() for f in a.files.split(",") if f.strip())
    # group errors by (file, function)
    per_fn = defaultdict(lambda: {"errs": [], "types": set()})
    src_cache, l2f_cache = {}, {}
    for msg in rustc_messages(crate, a.target, ws):
        if not (msg.get("code") or {}).get("code", "").startswith("E"):
            continue
        sp = next((s for s in msg["spans"] if s.get("is_primary")), None)
        if not sp:
            continue
        fname = os.path.basename(sp["file_name"])
        if fname.startswith("types_") or fname in ("glue.rs", "lib.rs"):
            continue
        if only and fname not in only:
            continue
        path = os.path.join(crate_src, fname)
        if not os.path.exists(path):
            continue
        if fname not in src_cache:
            src_cache[fname] = open(path).read()
            l2f_cache[fname] = line_to_func(src_cache[fname])
        info = l2f_cache[fname].get(sp["line_start"])
        if not info:
            continue
        key = (fname, info[0])
        per_fn[key]["errs"].append(f"L{sp['line_start']}: {msg['message']}")
        for tm in re.findall(r"`[^`]*?([A-Z][A-Za-z0-9_]+)`", msg["message"]):
            per_fn[key]["types"].add(tm)

    items = sorted(per_fn.items(), key=lambda kv: -len(kv[1]["errs"]))
    if a.limit:
        items = items[:a.limit]
    print(f"{len(per_fn)} broken functions; processing {len(items)}")

    if not a.dry_run:
        from rustconv.batch_runner import submit_single
    repaired = 0
    for (fname, name), info in items:
        src = src_cache[fname]
        span = next(((s, e) for n, s, e in functions(src) if n == name), None)
        if not span:
            continue
        rust_fn = src[span[0]:span[1]]
        c_fn = c_function(open(os.path.join(cdir, fname[:-3] + ".c")).read(), name) \
            if os.path.exists(os.path.join(cdir, fname[:-3] + ".c")) else None
        types = type_defs(types_src, info["types"])
        sigs = callee_sigs(rust_fn, SIGS)
        prompt = build_prompt(name, rust_fn, info["errs"], c_fn, types, sigs)
        if a.dry_run:
            print(f"\n===== {fname}::{name}  ({len(info['errs'])} errors, "
                  f"{len(rust_fn)} chars, c_fn={'yes' if c_fn else 'NO'}, "
                  f"types={len(info['types'])}, sigs={sigs.count(chr(10))+1 if sigs else 0}) =====")
            print(prompt[:1600])
            continue
        res = submit_single(prompt, model=a.model, system_prompt=SYS_PROMPT)
        out = (res.rust_code or "").strip()
        out = re.sub(r"^```[a-z]*\n|\n```$", "", out).strip()
        if not out:
            print(f"  SKIP {name}: empty output")
            continue
        reason = validate_repair(rust_fn, out, name)
        if reason:
            print(f"  REJECT {name}: {reason}  (kept original)")
            continue
        # splice back (re-read fresh in case of overlap; functions don't overlap so offsets hold)
        src2 = open(os.path.join(crate_src, fname)).read()
        sp2 = next(((s, e) for n, s, e in functions(src2) if n == name), None)
        if not sp2:
            continue
        open(os.path.join(crate_src, fname), "w").write(src2[:sp2[0]] + out + "\n" + src2[sp2[1]:])
        src_cache[fname] = open(os.path.join(crate_src, fname)).read()
        repaired += 1
        print(f"  repaired {fname}::{name} ({len(info['errs'])} errors)")
    print(f"repaired {repaired} functions")


if __name__ == "__main__":
    main()
