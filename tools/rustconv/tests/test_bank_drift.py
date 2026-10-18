"""bank_drift: a banked function whose C body hash no longer matches today's scan."""

import json

from rustconv import bank_drift as bd
from rustconv.scanner import scan_file

C = """int keep(int x)
{
    return x + 1;
}

int edited(int x)
{
    return x * 2;
}
"""


def _bank(tmp_path, root, name, hashes):
    bank = tmp_path / "bank" / name
    bank.mkdir(parents=True)
    (bank / "manifest.json").write_text(
        json.dumps({fn: {"file": f"{fn}.rs", "hash": h} for fn, h in hashes.items()})
    )
    return bank


def test_moved_and_gone_functions_are_reported(tmp_path):
    root = tmp_path / "tree"
    src = root / "dll" / "mod" / "mod.c"
    src.parent.mkdir(parents=True)
    src.write_text(C)
    bodies = {f.name: f.body_text for f in scan_file(src).functions}
    bank = _bank(
        tmp_path,
        root,
        "mod__mod__factseal_v1",
        {
            "keep": bd._hash(bodies["keep"]),
            "edited": bd._hash(bodies["edited"].replace("* 2", "* 3")),  # bought older text
            "deleted": "0" * 16,
        },
    )
    checked, drifts, unresolved = bd.audit([bank], root)
    assert checked == 3 and not unresolved
    assert {(d.function, d.state) for d in drifts} == {("edited", "moved"), ("deleted", "gone")}


def test_an_ambiguous_bank_name_is_reported_not_guessed(tmp_path):
    root = tmp_path / "tree"
    for top in ("dll/a/mod", "base/b/mod"):
        p = root / top / "x.c"
        p.parent.mkdir(parents=True)
        p.write_text(C)
    bank = _bank(tmp_path, root, "mod__x", {"keep": "0" * 16})
    checked, drifts, unresolved = bd.audit([bank], root)
    assert checked == 0 and not drifts
    assert next(iter(unresolved.values())).startswith("ambiguous")
