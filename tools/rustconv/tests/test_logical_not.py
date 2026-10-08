from rustconv import logical_not as ln
from rustconv import pointer_depth as pd


def _decls(c):
    return {"Q": pd.c_declarations("Q", [], c, set())}


RUST = (
    'unsafe extern "C" fn Q(b: u32, m: u32) {\n'
    "    if !(b & (1 << i)) != 0 {\n        x();\n    }\n"
    "    if (b & !(m)) != 0 {\n        y();\n    }\n"
    "}\n"
)


def test_a_c_logical_not_read_as_bitwise_is_restored():
    # pciidex's QBR: C `if (!(ChannelBitmap & (1 << i))) continue;` came back as
    # `!(..) != 0` -- true for every channel, so no IDE channel was ever enumerated.
    c = "{\n    if (!(b & (1 << i))) continue;\n    if (b & ~m) y();\n}\n"
    fixed, notes, refused = ln.repair(RUST, _decls(c))
    assert "if (b & (1 << i)) == 0 {" in fixed
    assert "(b & !(m)) != 0" in fixed  # a mask after `&` is not a site
    assert notes == ["Q: 1"] and refused == []


def test_a_function_whose_c_uses_tilde_on_that_operand_is_refused():
    c = "{\n    if (~(b & (1 << i))) x();\n}\n"
    fixed, notes, refused = ln.repair(RUST, _decls(c))
    assert fixed == RUST and notes == [] and refused == ["Q"]
