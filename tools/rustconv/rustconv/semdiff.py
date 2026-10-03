"""semdiff — C↔Rust semantic-divergence finder.

Pairs each C function (parsed via tree-sitter-c, reusing scanner.py) with its
hand/LLM-translated Rust counterpart (extracted by brace-matching) and asks an
LLM to report *behavioral* differences — places where the Rust would run
differently from the C — ranked by bug risk. Stylistic / idiomatic rewrites
that preserve behavior are ignored.

This is a CORRECTNESS oracle, complementary to the static verify_* suite
(verify_constants / verify_offsets / verify_abi / verify_sizes), which catch
mechanical mismatches but not logic divergence. semdiff catches the logic.

The translations compile and link; the question semdiff answers is "do they do
the same thing at runtime?" — exactly the class of bug that survives the build
and only shows up as a runtime regression (wrong timeout, element-vs-byte
overflow, inverted condition, sync-vs-async, missing early return, ...).

Usage:
    python -m rustconv semdiff <c_file> [--rust <rs_file>] [--model M]
                               [--only fn1,fn2] [--max-fns N] [--no-cache]

If --rust is omitted, the .rs file is located by stem under the candidate
crate src dirs. Output: a ranked JSON + Markdown report under
tools/rustconv/.semdiff/<ts>-<stem>/ (durable, in-repo, gitignored), plus a
bounded summary to stdout.
"""

from __future__ import annotations

import concurrent.futures as _cf
import hashlib
import json
import os
import re
import sys
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path

import anthropic
from anthropic.types import MessageParam
from openai import OpenAI

from . import config
from .scanner import CFunction, scan_file

SEMDIFF_DIR = Path(os.environ.get("RUSTCONV_SEMDIFF_DIR", str(config.TOOLS_DIR / ".semdiff")))

SEVERITY_ORDER = {"critical": 0, "high": 1, "medium": 2, "low": 3}


# --------------------------------------------------------------------------
# Rust function extraction (brace-matching; no tree-sitter-rust dependency)
# --------------------------------------------------------------------------


@dataclass
class RustFunction:
    name: str
    body_text: str
    line_start: int


_FN_RE = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>{};]*>)?\s*\(")


def extract_rust_functions(text: str) -> dict[str, RustFunction]:
    """Return {name: RustFunction} for every fn *definition* (with a body) in
    `text`. extern-block declarations (`fn Foo(...);`) are skipped — they hit a
    `;` before any `{`. The full item text (from `fn`) through the brace-matched
    body is captured; the leading attributes/qualifiers (#[no_mangle], pub,
    unsafe, extern "C") are not strictly needed for a semantic diff."""
    out: dict[str, RustFunction] = {}
    for m in _FN_RE.finditer(text):
        name = m.group(1)
        # Extend the item start backwards to include the qualifiers on the fn's
        # line (pub/unsafe/extern "C"/const/async) and any immediately-preceding
        # attribute lines (#[no_mangle], #[inline], ...). Without this the model
        # can't see the calling convention and emits spurious ABI "divergences".
        item_start = text.rfind("\n", 0, m.start()) + 1
        # Walk back over preceding attribute / doc-comment lines.
        probe = item_start
        while probe > 0:
            prev_end = probe - 1  # the '\n' before this line
            prev_start = text.rfind("\n", 0, prev_end) + 1
            prev_line = text[prev_start:prev_end].strip()
            if (
                prev_line.startswith("#[")
                or prev_line.startswith("#!")
                or prev_line.startswith("///")
                or prev_line.endswith("]")
            ):
                probe = prev_start
            else:
                break
        item_start = probe
        # Find the body-opening brace: the first '{' or ';' after the signature.
        i = m.end()
        depth_paren = 1  # we matched the opening '(' of the param list
        # Walk past the param list first.
        while i < len(text) and depth_paren > 0:
            c = text[i]
            if c == "(":
                depth_paren += 1
            elif c == ")":
                depth_paren -= 1
            i += 1
        # Now scan for '{' (body) or ';' (declaration only).
        body_open = -1
        while i < len(text):
            c = text[i]
            if c == ";":
                break  # declaration, no body
            if c == "{":
                body_open = i
                break
            i += 1
        if body_open < 0:
            continue
        # Brace-match the body.
        depth = 0
        j = body_open
        while j < len(text):
            c = text[j]
            if c == "{":
                depth += 1
            elif c == "}":
                depth -= 1
                if depth == 0:
                    j += 1
                    break
            j += 1
        item_text = text[item_start:j]
        line_start = text.count("\n", 0, item_start) + 1
        # Keep the first definition per name (later dup-cfg variants are similar).
        if name not in out:
            out[name] = RustFunction(name=name, body_text=item_text, line_start=line_start)
    return out


def _pair_functions(
    c_functions: list[CFunction], rust_funcs: dict[str, RustFunction]
) -> tuple[list[tuple[str, CFunction, RustFunction]], list[str]]:
    """Pair each C/C++ function with the Rust fn that replaced it.

    A C++ method `CFoo::Bar` is translated as a free `CFoo_Bar`, so matching on the
    bare name alone pairs nothing for the COM corpus. Bare-name keying is also
    lossy: a TU defining several classes has one `Release`/`AddRef`/`QueryInterface`
    per class, and a {name: fn} dict silently keeps only the last — the pair list is
    therefore built per C function, never through a bare-name dict.

    The `--cpp-com` scaffold splits each method in two: `<Class>_<Iface>_<Method>` is
    the DETERMINISTIC vtable thunk (generated, not worth diffing) and
    `<Class>_<Method>_body` holds the model-written logic. The `_body` form is
    therefore tried first, so the diff lands on the part a model actually wrote.
    Constructors/destructors follow the same convention as `_ctor_body`/`_dtor_body`.

    Returns (pairs, unpaired) where each pair is (display_name, c_fn, rust_fn) and
    display_name is class-qualified so same-named methods stay distinguishable.
    """
    pairs: list[tuple[str, CFunction, RustFunction]] = []
    unpaired: list[str] = []
    for f in c_functions:
        cls = getattr(f, "cpp_class", None)
        candidates: list[str] = []
        if cls:
            if getattr(f, "is_ctor", False):
                candidates += [f"{cls}_ctor_body", f"{cls}_ctor"]
            elif getattr(f, "is_dtor", False):
                candidates += [f"{cls}_dtor_body", f"{cls}_dtor"]
            candidates += [f"{cls}_{f.name}_body", f"{cls}_{f.name}"]
        candidates += [f"{f.name}_body", f.name]
        hit = next((c for c in candidates if c in rust_funcs), None)
        if hit is None:
            unpaired.append(f.display_name)
        else:
            pairs.append((f.display_name, f, rust_funcs[hit]))
    return pairs, unpaired


def locate_rust_file(c_path: Path) -> Path | None:
    """Best-effort: find a .rs with the same stem under known crate src dirs."""
    stem = c_path.stem
    roots = [
        config.REACTOS_ROOT / "sdk" / "lib" / "rust",
    ]
    hits: list[Path] = []
    for root in roots:
        if not root.exists():
            continue
        hits.extend(root.rglob(f"{stem}.rs"))
    # Prefer paths that mention the c file's parent driver name.
    hits = [
        h for h in hits if "/target/" not in h.as_posix() and "old_translations" not in h.as_posix()
    ]
    if not hits:
        return None
    # Heuristic: shortest path / one whose crate dir name overlaps the c parent.
    drv = c_path.parent.name
    hits.sort(key=lambda p: (drv not in p.as_posix(), len(p.as_posix())))
    return hits[0]


# --------------------------------------------------------------------------
# Prompt
# --------------------------------------------------------------------------

SYSTEM_PROMPT = """\
You are an expert in C and Rust systems programming and the Windows/ReactOS
kernel + KMDF/WDM driver model. You are given a C function and its translation
to Rust. The Rust already COMPILES and LINKS. Your task is to find SEMANTIC /
BEHAVIORAL DIFFERENCES — places where the Rust would execute differently from
the C at runtime and thus could cause a bug.

REPORT divergences in these classes:
- wrong constant / enum / timeout / flag value
- off-by-one; loop-bound or termination change
- element-count vs byte-count size errors (memcpy/alloc/add)
- inverted or altered condition; missing/extra branch
- missing early-return / error path; swallowed status
- wrong struct field, wrong union arm, wrong pointer arithmetic
- synchronous-vs-asynchronous change; missing/extra event signal or wait;
  wait timeout that differs (e.g. C uses a timeout, Rust waits forever, or
  vice-versa)
- integer width / sign / truncation differences
- operations performed in a different order when order is observable
- a call present in C but dropped in Rust (or vice-versa), especially
  Ke*/Ex*/Io*/Wdf* calls with side effects

JUDGE RUST BY RUST'S RULES, NOT C'S. The two measured false positives so far both
came from applying C semantics to the Rust side:
- OPERATOR PRECEDENCE IS NOT C's. In Rust `&`/`|`/`^` bind TIGHTER than the
  comparisons, so `flags & FLAG != 0` parses as `(flags & FLAG) != 0` — the
  intended test, and the opposite of the classic C gotcha. It could not parse the
  other way and still compile (`u32 & bool` is a type error), and this Rust is
  known to compile. Never report a precedence divergence on `&`/`|` vs `==`/`!=`.
- A `0` in C passed where the callee's prototype declares a POINTER is a null
  pointer constant; `core::ptr::null_mut()` is its correct translation, not an ABI
  mismatch. Use the CALLEE PROTOTYPES section as authoritative for argument types.

IGNORE (do NOT report):
- naming, formatting, comments, idiomatic rewrites that preserve behavior
- `unsafe`, raw-pointer vs reference style, transmute used for FFI typing
- equivalent control-flow expressed differently
- the mere absence of SEH/PSEH machinery (this project uses DUMMY_PSEH; SEH
  handlers are intentionally dead code) UNLESS the Rust drops a real
  side-effecting statement that the C ran on the normal path
- ADDED debug instrumentation in the Rust that is absent in the C: calls to
  CdromDbgPrint*/DbgPrint/TracePrint/KeQueryPerformanceCounter-for-timing/
  sd_trace and `[TAG]`-style debug prints are intentional diagnostic scaffolding,
  NOT bugs — do not report them at any severity
- `PAGED_CODE()` is a no-op macro in this project; its absence is not a bug
- `debug_assert!(x)` vs `NT_ASSERT(x)`: equivalent (both compile out in release);
  not a divergence
- `KSPIN_LOCK` is `ULONG_PTR`, so casting `&spinlock` to `*mut usize` for
  KeAcquireSpinLock/KeReleaseSpinLock is correct, not a bug
- pointer/reference type casts that are ABI-equivalent (e.g. `as *mut usize`,
  `as *mut c_void`) when the underlying width matches

Be skeptical and precise. If the two are behaviorally equivalent, say so —
do not invent divergences. Output STRICT JSON only, no prose, matching:

{"verdict": "equivalent" | "diverges",
 "divergences": [
   {"severity": "critical"|"high"|"medium"|"low",
    "category": "<short tag>",
    "c_behavior": "<what the C does>",
    "rust_behavior": "<what the Rust does>",
    "impact": "<runtime consequence>"}
 ]}
"""

USER_TEMPLATE = """\
Function: {name}

===== C =====
{c_src}
{prototypes}
===== RUST =====
{rust_src}

Report behavioral divergences as strict JSON.
"""

PROTOTYPE_HEADER = """
===== CALLEE PROTOTYPES (authoritative — from the Rust module's extern blocks) =====
These are the REAL signatures of the functions this body calls. Judge argument
types against THESE, not against a guess from the C spelling. In particular a C
`0` passed where the prototype declares a POINTER is a null pointer constant, and
`core::ptr::null_mut()` is its correct translation — not an ABI mismatch.
{decls}
"""

# `extern "..." { fn Name(...) -> Ret; }` items, plus bare `fn Name(...);` lines
# inside them. Captures the whole declaration so the model sees parameter types.
_EXTERN_FN_RE = re.compile(r"^\s*(?:pub\s+)?fn\s+([A-Za-z_]\w*)\s*\(", re.M)


def extract_extern_prototypes(text: str) -> dict[str, str]:
    """{callee: declaration} for every fn declared (no body) in `text`.

    The false-positive class this exists to kill: semdiff saw the C
    `SHQueryValueExW(hkey, name, 0, ...)` next to Rust `null_mut()` and reported a
    CRITICAL "argument type mismatch — an 8-byte pointer where a 4-byte DWORD is
    expected". The real prototype takes `LPDWORD`, so the Rust was correct and the
    C `0` was a null pointer constant. The model could not know: it saw two bodies
    and no signature. It is in the same file the Rust body came from.
    """
    out: dict[str, str] = {}
    for match in _EXTERN_FN_RE.finditer(text):
        # A declaration ends at the first `;` before any `{`.
        tail = text[match.start() :]
        semi, brace = tail.find(";"), tail.find("{")
        if semi < 0 or (0 <= brace < semi):
            continue  # has a body — a definition, not a prototype
        out.setdefault(match.group(1), " ".join(tail[:semi].split()) + ";")
    return out


# --------------------------------------------------------------------------
# Model call (DeepSeek / Claude, JSON-mode)
# --------------------------------------------------------------------------


@dataclass
class PairResult:
    name: str
    verdict: str = "error"
    divergences: list = field(default_factory=list)
    error: str | None = None
    input_tokens: int = 0
    output_tokens: int = 0
    cached: bool = False


def _resolve_model(model: str) -> tuple[str, str]:
    """Return (backend, api_model_id).

    Model IDs come from the one canonical alias map (`__main__.MODEL_ALIASES`) that
    the paid translate path already uses, rather than a second copy here — this
    module's private copy had gone stale and resolved every deepseek alias to
    `deepseek-reasoner`, which the provisioned key now rejects with a 400.
    """
    from .__main__ import MODEL_ALIASES

    resolved = MODEL_ALIASES.get(model, model)
    if resolved.startswith("claude"):
        return ("claude", resolved)
    if resolved.startswith("gemini"):
        return ("gemini", resolved)
    if resolved.startswith("deepseek"):
        return ("deepseek", resolved)
    if model.startswith("deepseek"):
        return ("deepseek", model)
    if model.startswith("claude"):
        return ("claude", model)
    return ("deepseek", model)


def _require_uncapped_review() -> None:
    if os.environ.get("RUSTCONV_BUDGET_FILE") or os.environ.get("RUSTCONV_BUDGET_USD"):
        raise RuntimeError(
            "semdiff's direct review API does not support persistent budget reservations; "
            "refusing to bypass the configured cap"
        )


def _call_deepseek(model: str, system: str, user: str) -> tuple[str, int, int]:
    _require_uncapped_review()
    api_key = os.environ.get("DEEPSEEK_API_KEY")
    if not api_key:
        raise RuntimeError("DEEPSEEK_API_KEY not set (add it to .env)")
    client = OpenAI(api_key=api_key, base_url=config.DEEPSEEK_BASE_URL)
    resp = client.chat.completions.create(
        model=model,
        messages=[
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        max_tokens=config.DEEPSEEK_MAX_TOKENS,
        temperature=0.0,
        stream=False,
    )
    text = resp.choices[0].message.content or ""
    it = getattr(resp.usage, "prompt_tokens", 0) if resp.usage else 0
    ot = getattr(resp.usage, "completion_tokens", 0) if resp.usage else 0
    return text, it, ot


def _call_claude(model: str, system: str, user: str) -> tuple[str, int, int]:
    _require_uncapped_review()
    client = anthropic.Anthropic()
    # Annotated, not inlined: pyright infers a bare literal as dict[str, str], and the
    # SDK's `role` is Literal["user", "assistant"] -- so the inline form matches no
    # overload of `create` even though it is correct at runtime.
    messages: list[MessageParam] = [{"role": "user", "content": user}]
    # No `temperature=0.0` here: the Messages API dropped the parameter (anthropic
    # 1.5.0 has zero occurrences of it, and `create` takes no **kwargs), so passing it
    # raised TypeError on EVERY call down this path -- pyright's "no overloads match"
    # was reporting a crash, not a typing nit. The DeepSeek path below still pins
    # temperature; it goes through the OpenAI client, which still accepts it. This
    # path therefore samples at the service default and is no longer deterministic.
    msg = client.messages.create(
        model=model,
        max_tokens=8192,
        system=system,
        messages=messages,
    )
    text = "".join(b.text for b in msg.content if isinstance(b, anthropic.types.TextBlock))
    return text, msg.usage.input_tokens, msg.usage.output_tokens


def _parse_json(text: str) -> dict | None:
    """Extract the first JSON object from the response (tolerates ```json fences
    and leading prose)."""
    t = text.strip()
    if "```" in t:
        m = re.search(r"```(?:json)?\s*(.*?)```", t, re.DOTALL)
        if m:
            t = m.group(1).strip()
    # Find the outermost {...}.
    start = t.find("{")
    if start < 0:
        return None
    depth = 0
    for i in range(start, len(t)):
        if t[i] == "{":
            depth += 1
        elif t[i] == "}":
            depth -= 1
            if depth == 0:
                try:
                    return json.loads(t[start : i + 1])
                except json.JSONDecodeError:
                    return None
    return None


# --------------------------------------------------------------------------
# Cache (per C-body+Rust-body+model hash)
# --------------------------------------------------------------------------


def _cache_key(name: str, c_src: str, rust_src: str, model: str) -> str:
    h = hashlib.sha256()
    h.update(model.encode())
    h.update(b"\0")
    h.update(c_src.encode())
    h.update(b"\0")
    h.update(rust_src.encode())
    return f"{name}-{h.hexdigest()[:16]}"


def _cache_dir() -> Path:
    d = SEMDIFF_DIR / "cache"
    d.mkdir(parents=True, exist_ok=True)
    return d


def diff_pair(
    name: str,
    c_src: str,
    rust_src: str,
    model: str,
    use_cache: bool = True,
    prototypes: str = "",
) -> PairResult:
    # Prototypes are part of what the model is shown, so they belong in the key.
    key = _cache_key(name, c_src, rust_src + prototypes, model)
    cache_path = _cache_dir() / f"{key}.json"
    if use_cache and cache_path.exists():
        try:
            data = json.loads(cache_path.read_text())
            return PairResult(
                name=name,
                verdict=data.get("verdict", "error"),
                divergences=data.get("divergences", []),
                cached=True,
            )
        except Exception:
            pass

    backend, api_model = _resolve_model(model)
    user = USER_TEMPLATE.format(name=name, c_src=c_src, rust_src=rust_src, prototypes=prototypes)
    try:
        if backend == "deepseek":
            text, it, ot = _call_deepseek(api_model, SYSTEM_PROMPT, user)
        else:
            text, it, ot = _call_claude(api_model, SYSTEM_PROMPT, user)
    except Exception as e:
        return PairResult(name=name, error=f"{type(e).__name__}: {e}")

    data = _parse_json(text)
    if data is None:
        return PairResult(
            name=name, error="unparseable JSON response", input_tokens=it, output_tokens=ot
        )
    res = PairResult(
        name=name,
        verdict=data.get("verdict", "diverges"),
        divergences=data.get("divergences", []),
        input_tokens=it,
        output_tokens=ot,
    )
    # Cache only well-formed results.
    cache_path.write_text(
        json.dumps({"verdict": res.verdict, "divergences": res.divergences}, indent=2)
    )
    return res


# --------------------------------------------------------------------------
# Driver
# --------------------------------------------------------------------------


def run(
    c_file: str,
    rust_file: str | None = None,
    model: str = "deepseek-reasoner",
    only: list[str] | None = None,
    max_fns: int | None = None,
    use_cache: bool = True,
    workers: int = 6,
) -> int:
    c_path = Path(c_file).resolve()
    if not c_path.exists():
        print(f"[semdiff] C file not found: {c_path}", file=sys.stderr)
        return 2

    rs_path = Path(rust_file).resolve() if rust_file else locate_rust_file(c_path)
    if rs_path is None or not rs_path.exists():
        print(
            f"[semdiff] could not locate Rust file for {c_path.name} (pass --rust <path>)",
            file=sys.stderr,
        )
        return 2

    scan = scan_file(c_path)
    rust_funcs = extract_rust_functions(rs_path.read_text(errors="replace"))

    prototype_index = extract_extern_prototypes(rs_path.read_text(errors="replace"))

    pairs, missing_rust = _pair_functions(scan.functions, rust_funcs)
    if only:
        pairs = [p for p in pairs if p[0] in only or p[1].name in only]
    pairs.sort(key=lambda p: p[0])
    if max_fns:
        pairs = pairs[:max_fns]

    c_funcs = {disp: cf for disp, cf, _ in pairs}
    rust_by_disp = {disp: rf for disp, _, rf in pairs}
    names = [p[0] for p in pairs]

    print(
        f"[semdiff] C={c_path.name} ({len(scan.functions)} fns)  "
        f"Rust={rs_path.name} ({len(rust_funcs)} fns)"
    )
    print(
        f"[semdiff] paired={len(names)}  c-only(no rust match)={len(missing_rust)}  "
        f"model={model}  cache={'on' if use_cache else 'off'}"
    )
    if not names:
        print("[semdiff] no paired functions to diff.")
        return 1

    results: list[PairResult] = []
    tot_in = tot_out = 0
    done = 0
    total = len(names)

    def _work(name: str) -> PairResult:
        rust_body = rust_by_disp[name].body_text
        # Only the callees this body actually names — the whole index would bury
        # the signal and re-pay for tokens on every pair.
        referenced = {
            callee: decl
            for callee, decl in prototype_index.items()
            if re.search(rf"\b{re.escape(callee)}\s*\(", rust_body)
        }
        protos = (
            PROTOTYPE_HEADER.format(decls="\n".join(referenced[k] for k in sorted(referenced)))
            if referenced
            else ""
        )
        return diff_pair(
            name,
            c_funcs[name].body_text,
            rust_body,
            model,
            use_cache,
            prototypes=protos,
        )

    with _cf.ThreadPoolExecutor(max_workers=max(1, workers)) as ex:
        futs = {ex.submit(_work, n): n for n in names}
        for fut in _cf.as_completed(futs):
            res = fut.result()
            results.append(res)
            tot_in += res.input_tokens
            tot_out += res.output_tokens
            done += 1
            ndiv = len(res.divergences)
            tag = "ERR" if res.error else "  ok" if res.verdict == "equivalent" else f"DIV×{ndiv}"
            flag = " [cache]" if res.cached else ""
            cost = config.cost_usd(_resolve_model(model)[1], tot_in, tot_out)
            print(
                f"  [{done}/{total}] {tag:>7}  {res.name}{flag}"
                + (f"  ({res.error})" if res.error else "")
                + (f"   [running ${cost:.2f}]" if not res.cached else "")
            )

    # Rank findings.
    findings = []
    for r in results:
        for d in r.divergences:
            findings.append({"function": r.name, **d})
    findings.sort(key=lambda d: SEVERITY_ORDER.get(d.get("severity", "low"), 3))

    ts = time.strftime("%Y%m%d-%H%M%S")
    out_dir = SEMDIFF_DIR / f"{ts}-{c_path.stem}"
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "report.json").write_text(
        json.dumps(
            {
                "c_file": str(c_path),
                "rust_file": str(rs_path),
                "model": model,
                "paired": len(names),
                "c_only": missing_rust,
                "input_tokens": tot_in,
                "output_tokens": tot_out,
                "cost_usd": config.cost_usd(_resolve_model(model)[1], tot_in, tot_out),
                "results": [asdict(r) for r in results],
                "findings_ranked": findings,
            },
            indent=2,
        )
    )

    md = _render_md(c_path, rs_path, model, names, findings, results, missing_rust)
    (out_dir / "report.md").write_text(md)

    # Bounded stdout summary.
    crit = sum(1 for d in findings if d.get("severity") == "critical")
    high = sum(1 for d in findings if d.get("severity") == "high")
    med = sum(1 for d in findings if d.get("severity") == "medium")
    low = sum(1 for d in findings if d.get("severity") == "low")
    print(
        f"\n[semdiff] findings: {crit} critical, {high} high, {med} medium, {low} low"
        f"  (in={tot_in} out={tot_out} tok, "
        f"${config.cost_usd(_resolve_model(model)[1], tot_in, tot_out):.2f})"
    )
    for d in findings[:12]:
        print(
            f"  {d.get('severity', '?'):>8}  {d['function']}: "
            f"{d.get('category', '')} — {d.get('impact', '')[:90]}"
        )
    print(f"[semdiff] full report: {out_dir / 'report.md'}")
    return 0


def _render_md(c_path, rs_path, model, names, findings, results, missing_rust) -> str:
    lines = [f"# semdiff: {c_path.name} ↔ {rs_path.name}", ""]
    lines.append(f"- model: `{model}`")
    lines.append(f"- paired functions: {len(names)}")
    lines.append(f"- C-only (no Rust match): {len(missing_rust)}")
    lines.append("")
    by_sev = {"critical": [], "high": [], "medium": [], "low": []}
    for d in findings:
        by_sev.setdefault(d.get("severity", "low"), []).append(d)
    for sev in ["critical", "high", "medium", "low"]:
        ds = by_sev.get(sev, [])
        if not ds:
            continue
        lines.append(f"## {sev.upper()} ({len(ds)})")
        lines.append("")
        for d in ds:
            lines.append(f"### `{d['function']}` — {d.get('category', '')}")
            lines.append(f"- **C:** {d.get('c_behavior', '')}")
            lines.append(f"- **Rust:** {d.get('rust_behavior', '')}")
            lines.append(f"- **Impact:** {d.get('impact', '')}")
            lines.append("")
    errs = [r for r in results if r.error]
    if errs:
        lines.append(f"## errors ({len(errs)})")
        for r in errs:
            lines.append(f"- `{r.name}`: {r.error}")
        lines.append("")
    if missing_rust:
        lines.append("## C functions with no Rust match")
        lines.append("")
        lines.append(", ".join(f"`{n}`" for n in sorted(missing_rust)))
        lines.append("")
    return "\n".join(lines)
