"""The `--local-raii NAME=PATH` hand-verified lowerings.

These files are spliced VERBATIM into an assembled module and their API is sealed into every
affected worker prompt, so a method that quietly vanishes from one is not a style difference —
it is a contract the workers were told about and the code no longer provides.

The repo holds a lowering for `CDataObjectHIDA` in two directories, and they drifted. The one
that says "THE canonical copy" in its own header was extracted from a flipped module's output
rather than from the sibling file, so it silently lost `DestroyCIDA` — a method a previous
commit had added ON PURPOSE, with a comment explaining that cabview's folder.cpp calls it
directly and shell32-rust's prelude re-exports it. Nothing read both files, so nothing noticed;
which copy you get depends only on which path the operator happens to type on the command line.
"""

import re
from collections import defaultdict
from pathlib import Path

RUSTCONV = Path(__file__).resolve().parents[1]
# Every directory that holds --local-raii lowering sources. A new one must be listed here, or
# it drifts from these unobserved exactly the way data/local_raii drifted from provided_raii.
LOWERING_DIRS = (RUSTCONV / "provided_raii", RUSTCONV / "data" / "local_raii")

_METHOD = re.compile(r"^\s*(?:pub\s+)?(?:unsafe\s+)?fn\s+([A-Za-z_]\w*)", re.MULTILINE)


def _lowerings_by_name() -> dict[str, list[Path]]:
    found: dict[str, list[Path]] = defaultdict(list)
    for directory in LOWERING_DIRS:
        for path in sorted(directory.glob("*.rs")) if directory.is_dir() else ():
            found[path.stem].append(path)
    return found


def test_a_lowering_kept_in_two_places_offers_the_same_api():
    """Byte-equality is the wrong bar; the sealed API surface is the right one.

    The copies legitimately differ in header prose and in a few `.cast()` spellings that were
    proven against the island of whichever TU each was extracted from. What may NOT differ is
    the set of callable methods, because that is what the worker prompts were handed as fixed.
    """
    duplicated = {n: p for n, p in _lowerings_by_name().items() if len(p) > 1}
    assert duplicated, "expected at least one lowering kept in two places; update LOWERING_DIRS"
    for name, paths in sorted(duplicated.items()):
        surfaces = {p: set(_METHOD.findall(p.read_text(encoding="utf-8"))) for p in paths}
        first, *rest = paths
        for other in rest:
            missing = surfaces[first] ^ surfaces[other]
            assert not missing, (
                f"{name}: the two lowerings disagree on {sorted(missing)}.\n"
                f"  {first}\n  {other}\n"
                "A --local-raii lowering is a sealed contract spliced verbatim; a method present "
                "in one copy and absent from the other means whichever path is typed on the "
                "command line decides whether the generated module links."
            )


def test_destroy_cida_is_callable_in_every_copy():
    """The specific method that went missing, named so a re-drop fails by name.

    shell32-rust's prelude re-exports DestroyCIDA and move_to_menu.rs reaches it through
    CDataObjectHIDA_DestroyCIDA_RustBridge, so a lowering that only inlines it into Drop
    generates a module that links short.
    """
    copies = _lowerings_by_name()["CDataObjectHIDA"]
    assert copies, "CDataObjectHIDA lowering not found"
    for path in copies:
        assert "DestroyCIDA" in set(_METHOD.findall(path.read_text(encoding="utf-8"))), path
