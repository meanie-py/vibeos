"""Deep-marshaller output and compiler failure paths retain the Python contracts."""

import json
import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
FIXTURES = REPO / "tools/vibeos-codegen/tests/marshal"


def run(root, *args, request=None):
    return subprocess.run(
        [str(REPO / "tools/codegen"), "--root", str(root), "marshal", *map(str, args)],
        input=None if request is None else json.dumps(request),
        capture_output=True,
        text=True,
        check=False,
        timeout=180,
    )


def fixture(root):
    data = json.loads((FIXTURES / "fixture.json").read_text())
    tool = root / ".build-tools/llvm-mingw/bin/clang"
    tool.parent.mkdir(parents=True)
    dumps = {}
    for width in [32, 64]:
        lines = []
        for rec in data[f"db{width}"].values():
            lines += ["*** Dumping AST Record Layout", f"0 | {rec['name']}"]
            for f in rec["fields"]:
                lines.append(f"{f['offset']} | " + "  " * f["depth"] + f"{f['ctype']} {f['name']}")
            lines.append(f"  | [sizeof={rec['size']}, align={rec['align']}]")
        dumps[str(width)] = "\n".join(lines) + "\n"
    tool.write_text(
        "#!/usr/bin/env python3\nimport sys,json\ndata="
        + repr(dumps)
        + '\nprint(data["32" if any("i686" in a for a in sys.argv) else "64"],end="")\n'
    )
    tool.chmod(0o755)
    for arch, triple in [("i386", "i686-w64-mingw32"), ("amd64", "x86_64-w64-mingw32")]:
        db = root / f"build-{arch}/compile_commands.json"
        db.parent.mkdir()
        db.write_text(
            json.dumps(
                [
                    {"file": suffix, "arguments": ["clang", f"--target={triple}"]}
                    for suffix in ["user32_layout_probe.c", "gdi32/wine/rust_cohab_stub.c"]
                ]
            )
        )

    def write(name, text):
        p = root / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)

    write("win32ss/w32ksvc32.h", "".join(f"SVC_({n[2:]}, 0)\n" for n in data["manual"]))
    write("win32ss/win32u/win32u.spec", "".join(f"@ stdcall {n}()\n" for n in data["linkable"]))
    write("tools/wow64gen/manual-win32k.txt", "NtHand\n")
    write("tools/wow64gen/automarshal-win32k.txt", "\n".join(data["manual"]) + "\n")
    header = "\n".join(
        p["ret"]
        + " NTAPI "
        + p["name"]
        + "("
        + (
            ", ".join(
                ("_Inout_ " if a["sal"] == "inout" else "") + a["ctype"] + " " + a["name"]
                for a in p["params"]
            )
            or "void"
        )
        + ");"
        for p in data["protos"].values()
    )
    write("headers.h", header)
    return data, tool


def test_frozen_pure_renderer_outputs_and_counts():
    data = json.loads((FIXTURES / "fixture.json").read_text())
    req = {
        "op": "generate",
        "db32": data["db32"],
        "db64": data["db64"],
        "protos": data["protos"],
        "manual": data["manual"],
        "linkable": data["linkable"],
        "hand_written": data["hand"],
        "export_spec": "exports-input.txt",
    }
    result = run(REPO, "--api-json", request=req)
    assert result.returncode == 0, result.stderr
    got = json.loads(result.stdout)
    assert got["c"] == (FIXTURES / "oracle-gen.txt").read_text()
    assert (got["thunks"], got["struct_pairs"], len(got["skipped"])) == (2, 2, 5)
    result = run(
        REPO,
        "--api-json",
        request={"op": "generate_callback", "db32": data["db32"], "db64": data["db64"]},
    )
    assert result.returncode == 0, result.stderr
    got = json.loads(result.stdout)
    assert got["c"] == (FIXTURES / "oracle-out.txt").read_text()
    assert got["header"] == (FIXTURES / "oracle-out-header.txt").read_text()


def test_cli_probes_writes_checks_and_preserves_existing_outputs_on_failure(tmp_path):
    _data, tool = fixture(tmp_path)
    source = tmp_path / "out.c"
    header = tmp_path / "out.h"
    args = ["gen-callback", "--build-dir", "build-amd64", "--out", source, "--out-header", header]
    result = run(tmp_path, *args)
    assert result.returncode == 0, result.stderr
    assert source.read_text() == (FIXTURES / "oracle-out.txt").read_text()
    assert header.read_text() == (FIXTURES / "oracle-out-header.txt").read_text()
    assert run(tmp_path, *args, "--check").returncode == 0
    source.write_text("keep c")
    header.write_text("keep h")
    assert run(tmp_path, *args, "--check").returncode == 1
    tool.write_text('#!/bin/sh\necho "error: compiler failed" >&2\nexit 7\n')
    result = run(tmp_path, *args)
    assert result.returncode != 0 and "compiler failed" in result.stderr
    assert source.read_text() == "keep c" and header.read_text() == "keep h"


def test_foreign_compile_database_and_ignored_options_are_refused(tmp_path):
    fixture(tmp_path)
    db = tmp_path / "build-i386/compile_commands.json"
    foreign = tmp_path.parent / (tmp_path.name + "-foreign.json")
    db.replace(foreign)
    db.symlink_to(foreign)
    result = run(tmp_path, "gen-callback", "--build-dir", "build-amd64")
    assert result.returncode != 0 and "escapes checkout" in result.stderr
    assert (
        run(tmp_path, "--api-json", "--out", "ignored", request={"op": "metadata"}).returncode != 0
    )
    assert run(tmp_path, "report", "--out", "ignored").returncode != 0


def test_real_compiler_rejects_wrong_pointer_width(tmp_path):
    fixture(tmp_path)
    tool = tmp_path / ".build-tools/llvm-mingw/bin/clang"
    tool.unlink()
    tool.symlink_to(REPO / ".build-tools/llvm-mingw/bin/clang")
    inc = tmp_path / "include"
    inc.mkdir()
    (inc / "user32.h").write_text("struct Sample { void *pointer; };\n")
    (inc / "precomp.h").write_text("#include <user32.h>\n")
    for arch in ["i386", "amd64"]:
        db = tmp_path / f"build-{arch}/compile_commands.json"
        rows = json.loads(db.read_text())
        for row in rows:
            row["arguments"] = ["clang", "--target=i686-w64-mingw32", "-I" + str(inc)]
        db.write_text(json.dumps(rows))
    result = run(tmp_path, "gen-callback", "--build-dir", "build-amd64")
    assert result.returncode != 0 and "wrong pointer width" in result.stderr


def test_quoted_compile_paths_and_argument_values_reach_real_compiler(tmp_path):
    import shlex

    fixture(tmp_path)
    tool = tmp_path / ".build-tools/llvm-mingw/bin/clang"
    tool.unlink()
    tool.symlink_to(REPO / ".build-tools/llvm-mingw/bin/clang")
    inc = tmp_path / "headers with spaces"
    inc.mkdir()
    (inc / "sample.h").write_text("struct Sample { void *pointer; };\n")
    args = [
        "clang",
        "--target=i686-w64-mingw32",
        "-I" + str(inc),
        "-DTEST=7",
        "-c",
        "user32_layout_probe.c",
        "-o",
        "ignored.obj",
    ]
    db = tmp_path / "build-i386/compile_commands.json"
    db.write_text(json.dumps([{"file": "user32_layout_probe.c", "command": shlex.join(args)}]))
    result = run(
        tmp_path,
        "--api-json",
        request={"op": "extract_flags", "build": str(db.parent), "suffix": "user32_layout_probe.c"},
    )
    assert result.returncode == 0, result.stderr
    flags = json.loads(result.stdout)
    assert flags == args[1:4]
    result = run(
        tmp_path,
        "--api-json",
        request={
            "op": "dump_layouts",
            "flags": flags,
            "tu": '#include <sample.h>\n_Static_assert(TEST == 7, "define missing");\n',
        },
    )
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout)["struct Sample"]["size"] == 4


def test_empty_layout_output_is_a_hard_failure(tmp_path):
    _data, tool = fixture(tmp_path)
    tool.write_text("#!/bin/sh\nexit 0\n")
    result = run(tmp_path, "gen-callback", "--build-dir", "build-amd64")
    assert result.returncode != 0 and "empty" in result.stderr
