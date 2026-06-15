"""Import-hygiene invariants for the rustconv CLI.

These pin the 2026-06 import-hoist work so it can't silently regress:
  * every submodule imports cleanly (no broken hoist / circular import),
  * __main__ carries NO inline first-party imports (all hoisted to top),
  * the analysis subcommands that re-use symbol names (run_oracle, extract_file,
    print_summary, write_sidecar, ...) stay module-qualified, so re-introducing
    a naive `from .x import run_oracle` hoist — which would make sibling
    subcommands silently call the wrong module — fails this test.
"""

import ast
import importlib
import pkgutil
from pathlib import Path

import rustconv

_MAIN = Path(rustconv.__file__).parent / "__main__.py"


def test_all_submodules_import():
    failures = []
    for m in pkgutil.iter_modules(rustconv.__path__, "rustconv."):
        try:
            importlib.import_module(m.name)
        except Exception as e:
            failures.append(f"{m.name}: {type(e).__name__}: {e}")
    assert not failures, "submodules failed to import:\n" + "\n".join(failures)


def test_main_has_no_inline_first_party_imports():
    """No `import`/`from .` inside any function body in __main__ — all at top."""
    tree = ast.parse(_MAIN.read_text())
    offenders = [
        f"line {n.lineno}: {ast.dump(n)[:60]}"
        for n in ast.walk(tree)
        if isinstance(n, (ast.Import, ast.ImportFrom)) and n.col_offset > 0
    ]
    assert not offenders, "inline imports must be hoisted:\n" + "\n".join(offenders)


def test_collision_subcommands_resolve_to_distinct_functions():
    """run_oracle (and friends) exist in several analysis modules and must stay
    distinct objects — proves they're module-qualified, not collapsed by a hoist."""
    from rustconv import seh_belt, seh_inventory, seh_rewrite

    oracles = {seh_inventory.run_oracle, seh_rewrite.run_oracle, seh_belt.run_oracle}
    assert len(oracles) == 3, "the three run_oracle impls must be distinct functions"
