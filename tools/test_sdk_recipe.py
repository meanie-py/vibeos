import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import sdk_recipe as s

W = "cd /tmp/vibeos/sdk/include/psdk && /tmp/vibeos/build-amd64/host-tools/bin/widl"


def test_only_generator_commands_are_kept():
    assert s._keep(f"{W} -I/tmp/vibeos/sdk/include -h -o /tmp/vibeos/build-amd64/x.h x.idl")
    assert s._keep(
        "cd /tmp/vibeos/build-amd64/a && /usr/bin/flex -o/tmp/vibeos/build-amd64/a/m.c m.l"
    )
    assert s._keep("cd /b && /usr/bin/x86_64-w64-mingw32-windmc -u -h /b/ m.mc")
    # The engine builds its own import libs and compiles; cmake's ExternalProject is not ours.
    assert not s._keep("cd /b && /tmp/vibeos/build-amd64/host-tools/bin/spec2def --implib x.spec")
    assert not s._keep('"/tmp/vibeos/.cache/host-jobs/host-jobs" --kind compile -- clang -c x.c')
    assert not s._keep("cd /b && /usr/bin/cmake -E env python3 x.py")


def test_paths_are_written_against_the_roots():
    line = f"{W} -o /tmp/vibeos/build-amd64/sdk/include/psdk/x.h x.idl"
    assert s._encode(line, "amd64") == (
        "cd {REPO}/sdk/include/psdk && {TOOLS}/widl -o {BUILD}/sdk/include/psdk/x.h x.idl"
    )


def test_the_shared_widl_run_is_named_once(monkeypatch):
    run = " ".join(f"-I{x}" for x in "abcde")
    lines = [f"cd d{i} && {{TOOLS}}/widl {run} -o o{i}.h i{i}.idl" for i in range(5)]
    lines.append("cd d && /usr/bin/flex -o x.c x.l")
    monkeypatch.setattr(s, "_tool", lambda ln: "widl" if "widl" in ln else "flex")
    out = s._factor(lines)
    assert out[0].startswith("{WIDL} = ") and run in out[0]
    assert all("{WIDL}" in ln for ln in out[1:6]) and "{WIDL}" not in out[6]
    assert [ln.replace("{WIDL}", out[0][len("{WIDL} = ") :]) for ln in out[1:]] == lines


def test_a_replay_makes_the_directories_ninja_would(tmp_path, monkeypatch):
    gen, repo = tmp_path / "gen", tmp_path / "repo"
    paths = {False: tmp_path / "sdk.cmds", True: tmp_path / "host.cmds"}
    paths[True].write_text("touch sub/dir/tool.o\n")
    paths[False].write_text("cd {BUILD}/sdk/include/psdk && touch {BUILD}/a/b/c.h\n")
    monkeypatch.setattr(s, "recipe_path", lambda arch, host=False, repo=None: paths[host])
    s.run("amd64", gen, repo)
    assert (gen / "host-tools/bin/sub/dir/tool.o").exists()
    assert (gen / "a/b/c.h").exists()


def test_the_host_jobs_wrapper_is_not_part_of_a_command():
    line = '"/tmp/vibeos/.cache/host-jobs/host-jobs" --kind compile -- /usr/bin/cc -c x.c && : '
    assert s._unwrap(line) == "/usr/bin/cc -c x.c && : "
