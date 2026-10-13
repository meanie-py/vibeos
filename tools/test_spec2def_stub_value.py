"""spec2def's `-stub=VALUE`: the stub that logs its first call and returns VALUE.

Built from sdk/tools/spec2def/spec2def.c with the host compiler, so the test grades the
source in this tree, never a build dir's binary from some earlier commit.
"""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parent.parent
CC = shutil.which("cc") or shutil.which("gcc") or shutil.which("clang")

pytestmark = pytest.mark.skipif(CC is None, reason="no host C compiler")


@pytest.fixture(scope="module")
def spec2def(tmp_path_factory: pytest.TempPathFactory) -> Path:
    assert CC is not None
    out = tmp_path_factory.mktemp("spec2def") / "spec2def"
    src = REPO / "sdk/tools/spec2def/spec2def.c"
    subprocess.run([CC, "-O1", "-o", str(out), str(src)], check=True, capture_output=True)
    return out


def _run(spec2def: Path, tmp: Path, spec: str, arch: str, out: str) -> subprocess.CompletedProcess:
    (tmp / "t.spec").write_text(spec)
    return subprocess.run(
        [
            str(spec2def),
            "-n=t.dll",
            f"-a={arch}",
            f"-d={tmp / 't.def'}",
            f"-s={tmp / out}",
            str(tmp / "t.spec"),
        ],
        capture_output=True,
        text=True,
    )


def test_rust_stub_logs_once_and_returns_the_value(spec2def: Path, tmp_path: Path) -> None:
    spec = "@ stdcall -stub=E_NOTIMPL Hr(ptr long)\n@ stdcall -ret64 -stub=-1 Big(int64)\n"
    assert _run(spec2def, tmp_path, spec, "i386", "s.rs").returncode == 0
    rs = (tmp_path / "s.rs").read_text()
    assert "WARNING: calling stub t.dll!Hr(" in rs
    assert "LOGGED.swap(true" in rs
    assert "0x80004001u64 as i32" in rs
    assert '    unimplemented(b"' not in rs  # never raises (the header still defines it)
    assert "\n    -1\n" in rs
    # The argument list is kept: i386 stdcall pops its own arguments.
    assert "Hr=Hr@8" in (tmp_path / "t.def").read_text()


def test_c_stub_logs_once_and_returns_the_value(spec2def: Path, tmp_path: Path) -> None:
    spec = "@ stdcall -stub=STATUS_NOT_IMPLEMENTED Nt(ptr)\n"
    assert _run(spec2def, tmp_path, spec, "x86_64", "s.c").returncode == 0
    c = (tmp_path / "s.c").read_text()
    assert "static int logged;" in c
    assert "return (int)0xC0000002;" in c


def test_plain_stubs_name_their_module(spec2def: Path, tmp_path: Path) -> None:
    spec = "@ stub Raiser\n@ stdcall -stub Zero(ptr)\n"
    assert _run(spec2def, tmp_path, spec, "x86_64", "s.rs").returncode == 0
    rs = (tmp_path / "s.rs").read_text()
    assert "calling stub t.dll!Raiser(" in rs and "calling stub t.dll!Zero(" in rs


@pytest.mark.parametrize(
    "spec",
    [
        "@ stub -stub=E_NOTIMPL NoSignature\n",  # needs the argument list
        "@ stdcall -stub=E_WHATEVER Bad(ptr)\n",  # not a value spec2def knows
    ],
)
def test_refused(spec2def: Path, tmp_path: Path, spec: str) -> None:
    r = _run(spec2def, tmp_path, spec, "i386", "s.rs")
    assert r.returncode != 0 and "-stub=" in (r.stdout + r.stderr)
