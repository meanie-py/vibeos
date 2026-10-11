from rustconv.catalog import Symbol, build_catalog
from rustconv.config import TARGETS
from rustconv.function_prompt import _format_import_context
from rustconv.resolver import ImportGroup, ResolveResult
from rustconv.stitcher import _generate_imports_from_resolve


def test_kernel_private_island_paths_agree_in_prompt_and_assembly():
    resolved = ResolveResult(
        imports={"types": ImportGroup("types", [Symbol("PDISPATCH_CONTEXT", "type", "types")])}
    )
    target = TARGETS["sysaudio"]
    prompt = _format_import_context(resolved, target)
    assert "From `crate::types`" in prompt
    assert "ntkrnl_sys::types" not in prompt
    assert "use crate::types::*;" in _generate_imports_from_resolve(resolved, target)


def test_irp_inline_bodies_outrank_foreign_declarations():
    catalog = build_catalog("amd64", TARGETS["sysaudio"])
    for name in (
        "IoGetCurrentIrpStackLocation",
        "IoSkipCurrentIrpStackLocation",
        "IoGetNextIrpStackLocation",
    ):
        symbol = catalog.lookup(name)
        assert symbol is not None
        assert symbol.module == "inline::tier1"
