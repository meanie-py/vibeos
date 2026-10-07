"""Two modules each carrying their own ``spapisup/fileqsup.c`` must not share a bank."""

from __future__ import annotations

from rustconv import response_cache as rc


def test_a_second_file_with_the_same_dir_and_stem_gets_its_own_namespace(tmp_path, monkeypatch):
    monkeypatch.setattr(rc, "RESPONSES_DIR", tmp_path / "bank")
    root = tmp_path / "repo"
    lib = root / "base/setup/lib/spapisup/fileqsup.c"
    use = root / "base/setup/usetup/spapisup/fileqsup.c"
    for p in (lib, use):
        p.parent.mkdir(parents=True)
        p.write_text("void f(void) {}\n")
    sfx = "__factseal_v1"
    first = rc.bank_base("spapisup", lib, sfx, root)
    assert first == "spapisup__fileqsup"
    rc.claim_bank(first + sfx, lib, root)
    assert rc.bank_base("spapisup", lib, sfx, root) == first, "the owner keeps its key"
    assert rc.bank_base("spapisup", use, sfx, root) == "usetup__spapisup__fileqsup"


def test_an_unclaimed_existing_bank_stays_addressable(tmp_path, monkeypatch):
    monkeypatch.setattr(rc, "RESPONSES_DIR", tmp_path / "bank")
    (tmp_path / "bank" / "dir__x__s").mkdir(parents=True)
    c = tmp_path / "repo" / "m" / "dir" / "x.c"
    c.parent.mkdir(parents=True)
    c.write_text("")
    assert rc.bank_base("dir", c, "__s", tmp_path / "repo") == "dir__x"
