"""Pre-seed the resume cache (.responses/<subsystem>__<stem>/) from a banked
stitched translation, so finishing a partial file re-translates ONLY the
missing functions and reuses the already-paid-for ones for $0.

The banked *_out.rs files predate the resume cache, so .responses/ is empty for
them; a plain `translate` would re-pay every function. This walks the banked
file, extracts each top-level `fn NAME { .. }`, pairs it with the C body that
rustconv's scanner derives for NAME (same body_text → same hash the cache keys
on), and writes a cache entry. On the next `translate <file>.c`, those names
resume for $0; the missing ones translate fresh.

Usage:  preseed_cache.py <c_file> <banked_out.rs> <subsystem> <stem>
A mis-extracted function just fails to hash-match later (it re-translates) or is
caught by the crate/flip-link/smoke gates — never silently shipped.
"""

import re
import sys
from pathlib import Path

from rustconv.scanner import scan_file
from rustconv.callgraph import extract_call_graph
from rustconv.response_cache import ResponseCache


def extract_rust_functions(text: str) -> dict[str, str]:
    """Extract each top-level `fn NAME(...) { ... }` (with its attribute stack)
    from a stitched Rust file, by brace-matching. Best-effort: generated code
    rarely has unbalanced braces inside string literals."""
    out: dict[str, str] = {}
    lines = text.splitlines()
    i, n = 0, len(lines)
    while i < n:
        m = re.search(r'\bfn\s+([A-Za-z_]\w*)\s*[(<]', lines[i])
        if m:
            name = m.group(1)
            # Walk back to absorb the attribute / doc-comment stack above the fn.
            start = i
            while start > 0:
                prev = lines[start - 1].lstrip()
                if prev.startswith('#[') or prev.startswith('//') or prev.startswith('#!'):
                    start -= 1
                else:
                    break
            # Brace-match from the signature to the closing brace.
            depth, started, j, end = 0, False, i, None
            while j < n:
                for ch in lines[j]:
                    if ch == '{':
                        depth += 1
                        started = True
                    elif ch == '}':
                        depth -= 1
                if started and depth == 0:
                    end = j
                    break
                j += 1
            if end is not None:
                out[name] = "\n".join(lines[start:end + 1])
                i = end + 1
                continue
        i += 1
    return out


def main() -> None:
    c_file, banked_rs, subsystem, stem = sys.argv[1:5]
    scan = scan_file(Path(c_file))
    deps = extract_call_graph(scan)
    bodies = {d.name: d.body_text for d in deps}

    funcs = extract_rust_functions(Path(banked_rs).read_text())
    cache = ResponseCache(f"{subsystem}__{stem}")

    seeded, skipped = 0, []
    for name, rust in funcs.items():
        if name in bodies:
            cache.put(name, bodies[name], rust)
            seeded += 1
        else:
            skipped.append(name)

    total_c = len(bodies)
    print(f"{stem}: seeded {seeded}/{len(funcs)} extracted fns "
          f"(C file has {total_c} fns) → {total_c - seeded} will translate fresh")
    if skipped:
        print(f"  not matched to a C fn (skipped): {', '.join(skipped[:8])}"
              + (" ..." if len(skipped) > 8 else ""))


if __name__ == "__main__":
    main()
