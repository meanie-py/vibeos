"""link_pulls: which modules link an archive, and what lld's --verbose log says it loaded."""

from __future__ import annotations

import importlib.util
from pathlib import Path

_spec = importlib.util.spec_from_file_location(
    "link_pulls", Path(__file__).resolve().parent / "link_pulls.py"
)
assert _spec and _spec.loader
lp = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(lp)


def test_linking_reads_link_libraries_and_flags_by_basename(tmp_path):
    (tmp_path / "build.ninja").write_text(
        "build dll/a.dll: LINKER a.obj\n"
        "  LINK_LIBRARIES = sdk/lib/crt/libcrtmath.a dll/libother.a\n"
        "build dll/b.sys: LINKER b.obj\n"
        "  LINK_FLAGS = -nostdlib /abs/sdk/lib/crt/libcrtmath.a\n"
        "build dll/c.dll: LINKER c.obj\n"
        "  LINK_LIBRARIES = sdk/lib/crt/libcrtmath_extra.a\n"
        "build sdk/lib/crt/libcrtmath.a: AR x.obj\n"
    )
    assert lp.linking(tmp_path, "libcrtmath.a") == ["dll/a.dll", "dll/b.sys"]


def test_loaded_lines_name_archive_members_and_thin_archive_objects():
    log = (
        "ld.lld: Loaded libgdi32_rust.a(gdi32_rust-1.cgu.25.rcgu.o) for DllMain\n"
        "ld.lld: Loaded sdk/lib/crt/CMakeFiles/atan2.dir/math/libm_sse2/atan2.c.obj for atan2\n"
        "ld.lld: warning: something unrelated\n"
    )
    assert lp.LOADED.findall(log) == [
        ("libgdi32_rust.a(gdi32_rust-1.cgu.25.rcgu.o)", "DllMain"),
        ("sdk/lib/crt/CMakeFiles/atan2.dir/math/libm_sse2/atan2.c.obj", "atan2"),
    ]
