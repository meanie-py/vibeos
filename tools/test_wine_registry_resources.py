"""A module built with -DWINE_REGISTER_DLL registers from its OWN resources, so it must have some.

With WINE_REGISTER_DLL, rpcproxy.h makes DllRegisterServer `__wine_register_resources(hProxyDll)`,
which replays the module's WINE_REGISTRY resources. Those come either from an .rc line
(`N WINE_REGISTRY foo.rgs`) or from `add_idl_reg_scripts(...)` in CMakeLists.txt, which
generates them from the IDL. Lose both and DllRegisterServer writes NOTHING and returns S_OK:
no error anywhere, only a registry that silently lacks the module's classes and interfaces.

That happened to ole32 (2026-09-25): upstream 7675017f60d moved its registration from .rgs
resources to add_idl_reg_scripts, and the upstream sync kept the .rc deletion but lost the
CMake line. Every Interface\\{iid}\\ProxyStubClsid32 was gone, so no interface could be
marshalled across apartments (right-click > Properties showed an error box).
"""

from __future__ import annotations

import subprocess
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent


def modules_registering_from_resources() -> list[Path]:
    out = subprocess.run(
        ["git", "grep", "-l", "WINE_REGISTER_DLL", "--", "*CMakeLists.txt"],
        cwd=REPO,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    return [REPO / p for p in out.split()]


def has_registry_resources(cmakelists: Path) -> bool:
    if "add_idl_reg_scripts(" in cmakelists.read_text(errors="replace"):
        return True
    return any(
        "WINE_REGISTRY" in rc.read_text(errors="replace") for rc in cmakelists.parent.glob("*.rc")
    )


def test_every_wine_register_dll_module_has_registry_resources():
    mods = modules_registering_from_resources()
    assert mods, "no -DWINE_REGISTER_DLL modules found: the git grep itself is broken"
    missing = [str(m.parent.relative_to(REPO)) for m in mods if not has_registry_resources(m)]
    assert not missing, (
        "these modules register from resources (-DWINE_REGISTER_DLL) but have neither an "
        "add_idl_reg_scripts() line nor a WINE_REGISTRY .rc entry, so DllRegisterServer "
        f"writes nothing and returns S_OK: {missing}"
    )


def test_the_check_would_have_caught_ole32(tmp_path):
    """The control: ole32 as the sync left it (define, no rc resource, no CMake line)."""
    d = tmp_path / "ole32"
    d.mkdir()
    (d / "CMakeLists.txt").write_text(
        "add_definitions(-DWINE_REGISTER_DLL)\nset_module_type(ole32 win32dll)\n"
    )
    (d / "ole32res.rc").write_text("CURSOR_NODROP CURSOR nodrop.cur\n")
    assert not has_registry_resources(d / "CMakeLists.txt")
    (d / "CMakeLists.txt").write_text(
        "add_definitions(-DWINE_REGISTER_DLL)\nadd_idl_reg_scripts(ole32 registry dcom.idl)\n"
    )
    assert has_registry_resources(d / "CMakeLists.txt")
