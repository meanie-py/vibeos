"""sdk_macros' preprocessor probe: one file name on every arch."""

from __future__ import annotations

import shutil
from pathlib import Path

import pytest

from rustconv import sdk_macros

CC = shutil.which("clang") or shutil.which("gcc")


@pytest.mark.skipif(CC is None, reason="needs a host C preprocessor")
def test_file_in_an_expansion_is_the_same_on_every_probe(tmp_path: Path):
    """Each arch's probe lives in its own temp dir. Without `#line`, a macro expanding
    `__FILE__` came back with a different path per arch and was refused as "expands
    differently per arch" instead of for the string literal it is."""
    src = tmp_path / "m.c"
    src.write_text("#define WHERE(x) (__FILE__, x)\n")
    probe = "__RCV_BEGIN_WHERE__ WHERE(__rcv_p0__) __RCV_END__"
    one = sdk_macros._preprocess([str(CC), str(src)], src, probe)
    two = sdk_macros._preprocess([str(CC), str(src)], src, probe)
    assert one == two
    assert '"rcv-sdkmacro-probe.c"' in one and "rcv-sdkmacro-" not in one.replace(
        '"rcv-sdkmacro-probe.c"', ""
    )
