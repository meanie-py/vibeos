"""tools/merge_drivers.py: the structural merges for gen_driver_types.sh and the census."""

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import merge_drivers as md

HEAD = '#!/usr/bin/env bash\nDRIVER="$1"\ncase "$DRIVER" in\n'
TAIL = '  *) echo "unknown driver: $DRIVER" >&2; exit 2 ;;\nesac\necho done\n'


def arm(name: str, body: str = "X=1") -> str:
    return f"  # {name}'s arm\n  {name})\n    {body}\n    ;;\n"


def table(*arms: str) -> str:
    return HEAD + "".join(arms) + TAIL


def test_two_lanes_each_adding_an_arm_both_keep_theirs():
    # The conflict every lane merge hit: both appended an arm before `*)`.
    base = table(arm("a"))
    ours = table(arm("a"), arm("mine"))
    theirs = table(arm("a"), arm("yours"))
    out, clean = md.merge_case_arms(base, ours, theirs)
    assert clean
    assert out == table(arm("a"), arm("mine"), arm("yours"))
    assert out.index("  *)") > out.index("  yours)"), "an arm after the default never runs"


def test_an_arm_edited_on_one_side_takes_the_edit():
    base = table(arm("a"), arm("b"))
    ours = table(arm("a", "X=2"), arm("b"), arm("new"))
    theirs = table(arm("a"), arm("b", "Y=3"))
    out, clean = md.merge_case_arms(base, ours, theirs)
    assert clean and out == table(arm("a", "X=2"), arm("b", "Y=3"), arm("new"))


def test_an_arm_both_sides_edited_differently_is_a_visible_conflict():
    base = table(arm("a"))
    out, clean = md.merge_case_arms(base, table(arm("a", "X=2")), table(arm("a", "X=3")))
    assert not clean
    assert "<<<<<<<" in out and "X=2" in out and "X=3" in out


def test_a_deleted_arm_stays_deleted_when_the_other_side_left_it_alone():
    base = table(arm("a"), arm("gone"))
    out, clean = md.merge_case_arms(base, table(arm("a")), table(arm("a", "X=9"), arm("gone")))
    assert clean and out == table(arm("a", "X=9"))


def test_a_file_without_the_table_falls_back_to_the_text_merge():
    out, clean = md.merge_case_arms("a\nb\nc\nd\n", "A\nb\nc\nd\n", "a\nb\nc\nD\n")
    assert clean and out == "A\nb\nc\nD\n"


def test_census_rows_merge_as_a_map_and_stay_sorted():
    row = "{}\t1\t# {}.c  # replaced by Rust\n".format
    base = row("a/CMakeLists.txt", "x")
    ours = base + row("m/CMakeLists.txt", "mine")
    theirs = row("b/CMakeLists.txt", "theirs") + base
    out, clean = md.merge_census(base, ours, theirs)
    assert clean
    assert out == (
        row("a/CMakeLists.txt", "x")
        + row("b/CMakeLists.txt", "theirs")
        + row("m/CMakeLists.txt", "mine")
    )


def test_census_count_changed_both_ways_conflicts():
    base = "a/CMakeLists.txt\t1\t# x.c\n"
    out, clean = md.merge_census(
        base, base.replace("\t1\t", "\t2\t"), base.replace("\t1\t", "\t3\t")
    )
    assert not clean and "<<<<<<<" in out


@pytest.mark.skipif(shutil.which("git") is None, reason="needs git")
def test_git_runs_the_driver_on_a_real_merge(tmp_path: Path):
    """End to end, as install-git-hooks.sh configures it: the relative script path must
    resolve from where git runs the driver."""
    repo = tmp_path / "r"
    (repo / "tools" / "rustconv").mkdir(parents=True)
    shutil.copy(Path(md.__file__), repo / "tools" / "merge_drivers.py")

    def git(*args: str) -> str:
        return subprocess.run(
            ["git", "-C", str(repo), *args], check=True, capture_output=True, text=True
        ).stdout

    git("init", "-q", "-b", "main")
    git("config", "user.email", "t@example.com")
    git("config", "user.name", "t")
    git(
        "config",
        "merge.vibeos-case-arms.driver",
        "python3 tools/merge_drivers.py case-arms %O %A %B",
    )
    (repo / ".gitattributes").write_text(
        "tools/rustconv/gen_driver_types.sh merge=vibeos-case-arms\n"
    )
    script = repo / "tools" / "rustconv" / "gen_driver_types.sh"
    script.write_text(table(arm("a")))
    git("add", "-A")
    git("commit", "-qm", "base")
    git("checkout", "-qb", "lane")
    script.write_text(table(arm("a"), arm("mine")))
    git("commit", "-qam", "mine")
    git("checkout", "-q", "main")
    script.write_text(table(arm("a"), arm("yours")))
    git("commit", "-qam", "yours")
    git("merge", "-q", "--no-edit", "lane")
    assert script.read_text() == table(arm("a"), arm("yours"), arm("mine"))


MEMBERS_HEAD = "[workspace]\nmembers = [\n"
MEMBERS_TAIL = ']\nresolver = "2"\n\n[profile.release]\nopt-level = 2\n'


def members(*names: str) -> str:
    return MEMBERS_HEAD + "".join(f'    "{n}",\n' for n in names) + MEMBERS_TAIL


def test_two_lanes_each_adding_a_crate_both_keep_theirs():
    base = members("a-rust", "z-rust")
    ours = members("a-rust", "mine-rust", "z-rust")
    theirs = members("a-rust", "yours-rust", "z-rust")
    out, clean = md.merge_cargo_members(base, ours, theirs)
    assert clean
    assert out == members("a-rust", "mine-rust", "yours-rust", "z-rust")


def test_a_comment_block_travels_with_its_member_and_the_rest_merges_as_text():
    base = members("a-rust")
    ours = base.replace('    "a-rust",\n', '    "a-rust",\n    # why b\n    "b-rust",\n')
    theirs = members("a-rust", "c-rust").replace("opt-level = 2", "opt-level = 3")
    out, clean = md.merge_cargo_members(base, ours, theirs)
    assert clean
    assert '    # why b\n    "b-rust",\n    "c-rust",\n' in out
    assert "opt-level = 3" in out


def test_a_member_removed_on_one_side_stays_removed():
    base = members("a-rust", "gone-rust", "z-rust")
    out, clean = md.merge_cargo_members(
        base, members("a-rust", "z-rust"), members("a-rust", "gone-rust", "z-rust", "new-rust")
    )
    assert clean and out == members("a-rust", "z-rust", "new-rust")


def test_an_unrecognised_members_list_falls_back_to_the_text_merge():
    weird = '[workspace]\nmembers = ["a", "b"]\n'
    out, clean = md.merge_cargo_members(weird, weird.replace('"b"', '"c"'), weird)
    assert clean and out == weird.replace('"b"', '"c"')


def test_a_crate_listed_twice_still_merges_by_crate():
    # aclui-rust sat in members twice; refusing such a file sent 23 of 174 merges to text.
    base = members("a-rust", "dup-rust", "dup-rust")
    ours = members("a-rust", "dup-rust", "dup-rust", "mine-rust")
    theirs = members("a-rust", "dup-rust", "dup-rust", "yours-rust")
    out, clean = md.merge_cargo_members(base, ours, theirs)
    assert clean
    assert out == members("a-rust", "dup-rust", "dup-rust", "mine-rust", "yours-rust")
