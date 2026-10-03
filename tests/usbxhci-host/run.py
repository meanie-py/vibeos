"""Build and run the shipping miniport regressions on the Windows host from WSL.

The NT bindgen island requires Windows' LLP64 ABI. Discover the installed MSVC
and SDK import libraries and use the repo's lld-link; never rewrite the bindings
to make them compile as Linux LP64 types.
"""

from __future__ import annotations

import os
import subprocess
from pathlib import Path


def newest(pattern: str) -> Path:
    matches = sorted(Path("/mnt/c").glob(pattern))
    if not matches:
        raise SystemExit(f"Missing Windows host test dependency: /mnt/c/{pattern}")
    return matches[-1]


def main() -> int:
    here = Path(__file__).resolve().parent
    repo = here.parent.parent
    linker = repo / ".build-tools/llvm-mingw/bin/lld-link"
    if not linker.is_file():
        raise SystemExit(f"Missing Windows linker: {linker}")
    vc = newest("Program Files/Microsoft Visual Studio/*/*/VC/Tools/MSVC/*/lib/x64")
    sdk = newest("Program Files (x86)/Windows Kits/10/Lib/*/um/x64").parent.parent
    env = os.environ.copy()
    env["LIB"] = ";".join(str(p) for p in (vc, sdk / "ucrt/x64", sdk / "um/x64"))
    env["CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER"] = str(linker)
    return subprocess.run(
        [
            "cargo",
            "test",
            "--manifest-path",
            str(here / "Cargo.toml"),
            "--target",
            "x86_64-pc-windows-msvc",
        ],
        cwd=repo,
        env=env,
        check=False,
    ).returncode


if __name__ == "__main__":
    raise SystemExit(main())
