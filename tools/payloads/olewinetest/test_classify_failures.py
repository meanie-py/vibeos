"""classify_failures: todo_wine failures (Wine shares them) vs real ones."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from classify_failures import todo_lines

SRC = """\
static void f(void)
{
    todo_wine
    ok(a, "statement form\\n");
    ok(b, "plain\\n");
    todo_wine {
        ok(c, "block, first\\n");
        if (x) { y(); }
        ok(d, "block, after a nested brace\\n");
    }
    ok(e, "after the block\\n");
    todo_wine_if(z)
    ok(f, "conditional is not todo\\n");
}
""".splitlines()


def test_block_lines_only():
    inside = todo_lines(SRC)
    assert {7, 8, 9}.issubset(inside)
    assert 5 not in inside and 11 not in inside and 13 not in inside
