"""Tree consensus must not keep a copy the prelude now defines, or answer a macro with a fn."""

from __future__ import annotations

from pathlib import Path

from rustconv import prelude_resolve
from rustconv.port import _macro_only_names, _prelude_definitions_outside_consensus


def test_extern_declaration_is_not_a_definition(tmp_path: Path):
    # shsvcs/netlogon: consensus copied appwiz's extern DECLARATION of the default channel;
    # the wine-channel block then defined it, and the pair was E0428 on every run.
    prelude = tmp_path / "prelude.rs"
    prelude.write_text(
        f"{prelude_resolve.BEGIN}\n"
        'extern "C" {\n    pub static __wine_dbch___default: *mut __wine_debug_channel;\n}\n'
        "pub const IN_BLOCK: u32 = 1;\n"
        f"{prelude_resolve.END}\n"
        'extern "C" {\n    pub static OnlyDeclared: u32;\n}\n'
        "pub static mut __wine_dbch___default: *mut __wine_debug_channel =\n"
        "    unsafe { &raw mut __wine_dbch_shsvcs };\n"
        "pub type HAND = u32;\n"
    )
    defined = _prelude_definitions_outside_consensus(prelude)
    assert {"__wine_dbch___default", "HAND"} <= defined
    assert "OnlyDeclared" not in defined and "IN_BLOCK" not in defined


def test_macro_only_names(tmp_path: Path):
    # srvsvc: `FIXME!(..)` got mlang's stray `fn FIXME` from consensus.
    (tmp_path / "a.rs").write_text('FIXME!("x"); ERR!("y"); ERR(1); f(1);\n')
    assert _macro_only_names(tmp_path) == {"FIXME"}
