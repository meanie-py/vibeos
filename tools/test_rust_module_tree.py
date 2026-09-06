"""Differential oracle for the chunk-based comment scanner.

The reference is the preceding shipping implementation, deliberately retaining
its treatment of malformed strings/comments. This is a speed change, not a new
Rust grammar or a relaxation of module reachability checks.
"""

import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from rust_module_tree import strip_comments


def reference(text):
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


def test_randomized_equivalence_to_previous_scanner():
    rng = random.Random(348901)
    atoms = ["mod", "/*", "*/", "//", '"', "\\", "\n", "\r", " ", "α", "'", "#", ";", "abc"]
    for _ in range(10000):
        text = "".join(rng.choices(atoms, k=rng.randrange(250)))
        assert strip_comments(text) == reference(text), repr(text)


def test_large_plain_chunks_and_escaped_string_tail():
    text = "pub const LONG_BINDING: usize = 42;\n" * 20000
    for suffix in ("/* nested /* block */ comment */", "// tail\r\nmod live;", '"unterminated\\'):
        assert strip_comments(text + suffix) == reference(text + suffix)
