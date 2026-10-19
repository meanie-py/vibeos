"""Native widl client guards preserve the legacy wrappers and refusal cases."""

import subprocess
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
FIXTURES = REPO / "tools/vibeos-codegen/tests/bindings"


def generate(tmp_path, header):
    source = tmp_path / "x_c.h"
    source.write_text(header)
    return subprocess.run(
        [
            str(REPO / "tools/codegen"),
            "rpc-guards",
            "--header",
            str(source),
            "--prelude",
            "prelude.h",
            "--out-c",
            str(tmp_path / "x_guard.c"),
            "--out-h",
            str(tmp_path / "x_guard.h"),
        ],
        capture_output=True,
        text=True,
        check=False,
    )


def test_client_server_arrays_void_and_return_values_match_legacy(tmp_path):
    result = generate(tmp_path, (FIXTURES / "x_c.h").read_text())
    assert result.returncode == 0, result.stderr
    for suffix in ("c", "h"):
        assert (tmp_path / f"x_guard.{suffix}").read_bytes() == (
            FIXTURES / f"x_guard.{suffix}"
        ).read_bytes()
    header = (tmp_path / "x_guard.h").read_text()
    source = (tmp_path / "x_guard.c").read_text()
    assert "s_remote_GetProperty_guarded" in header
    assert "ept_entry_t *entries" in header
    assert "a->guard_ret = remote_GetLanguage(a->hinst);" in source
    assert "    ept_insert(a->h, a->num_ents, a->entries, a->replace, a->status);" in source
    assert "struct PNP_GetVersion_args { DWORD guard_ret; };" in source
    assert "RtlSehTryExcept(" in header


@pytest.mark.parametrize("header", ["int foo(void);", "void __cdecl f(\n void (*cb)(int));"])
def test_rejects_missing_stubs_and_function_pointer_parameters_without_writes(tmp_path, header):
    old = tmp_path / "x_guard.c"
    old.write_text("keep")
    result = generate(tmp_path, header)
    assert result.returncode != 0
    assert old.read_text() == "keep"
    assert not (tmp_path / "x_guard.h").exists()
