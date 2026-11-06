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


def test_todo_if_is_its_own_bucket(tmp_path, monkeypatch):
    inside = todo_lines(SRC, TODO_IF_RE)
    assert inside == {13, 16}
    log = (
        "[APITEST:x_winetest.exe t] x.c:13: Test failed: f\n"
        "[APITEST:x_winetest.exe t] x.c:5: Test failed: b\n"
    )
    import classify_failures as cf

    source = tmp_path / "modules/rostests/winetests/x/x.c"
    source.parent.mkdir(parents=True)
    source.write_text("\n".join(SRC))
    monkeypatch.setattr(cf, "REPO", tmp_path)
    ((todo, todo_if, real, _),) = classify(log).values()
    assert (todo, todo_if, real) == (0, 1, 1)


def test_native_classification_counts_repeats_and_ignores_missing_sources(tmp_path, monkeypatch):
    import classify_failures as cf

    p = tmp_path / "modules/rostests/winetests/x/x.c"
    p.parent.mkdir(parents=True)
    p.write_text("todo_wine {\n ok(a);\n}\ntodo_wine_if(a) ok(b);\nok(c);\n")
    monkeypatch.setattr(cf, "REPO", tmp_path)
    text = (
        "".join(f"[APITEST:x_winetest.exe t] x.c:{n}: Test failed: message\n" for n in [2, 4, 5, 5])
        + "[APITEST:missing_winetest.exe t] missing.c:1: Test failed: ignored\n"
    )
    result = cf.classify(text)
    assert list(result) == ["x:x.c"]
    todo, conditional, real, detail = result["x:x.c"]
    assert (todo, conditional, real) == (1, 1, 2)
    assert detail[(5, "message")] == 2
