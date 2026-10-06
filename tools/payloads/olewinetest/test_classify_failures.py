"""classify_failures: todo_wine failures (Wine shares them) vs real ones."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from classify_failures import TODO_IF_RE, classify, todo_lines

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
    todo_wine_if(!(p == 1
                 || q(2)))
    ok(g, "condition over two lines\\n");
}
""".splitlines()


def test_block_lines_only():
    inside = todo_lines(SRC)
    assert {4, 7, 8, 9}.issubset(inside)
    assert 5 not in inside and 11 not in inside and 13 not in inside


def test_todo_if_is_its_own_bucket():
    inside = todo_lines(SRC, TODO_IF_RE)
    assert inside == {13, 16}
    log = (
        "[APITEST:x_winetest.exe t] x.c:13: Test failed: f\n"
        "[APITEST:x_winetest.exe t] x.c:5: Test failed: b\n"
    )
    import classify_failures as cf

    cf.Source.get = lambda self, suite, name: (SRC, todo_lines(SRC), inside)
    ((todo, todo_if, real, _),) = classify(log).values()
    assert (todo, todo_if, real) == (0, 1, 1)
