"""vtbl_gen: a `#ifdef __REACTOS__` slot inside a vtbl initializer (msxml3's setType)."""

import pytest

from rustconv import vtbl_gen as v


def _inits(src: str) -> list[str]:
    text = v._resolve_reactos_conditionals(src).encode()
    tree = v._ts_parse(text)
    decl = tree.root_node.named_children[0]
    init = next(iter(decl.children_by_field_name("declarator"))).child_by_field_name("value")
    return [val for _, val in v._initializer_values(init, text)]


def test_reactos_slot_is_one_slot_and_line_numbers_hold():
    src = "static const IFooVtbl t = {\n  A,\n#ifdef __REACTOS__\n  B,\n#endif\n  C\n};\n"
    assert _inits(src) == ["A", "B", "C"]
    assert v._resolve_reactos_conditionals(src).count("\n") == src.count("\n")


def test_dead_arm_drops_and_undecidable_refuses():
    dead = "static const IFooVtbl t = {\n  A,\n#ifndef __REACTOS__\n  X,\n#else\n  B,\n#endif\n  C\n};\n"
    assert _inits(dead) == ["A", "B", "C"]
    other = "static const IFooVtbl t = {\n  A,\n#ifdef HAVE_FOO\n  X,\n#endif\n  C\n};\n"
    with pytest.raises(v.VtblArityError):
        _inits(other)
