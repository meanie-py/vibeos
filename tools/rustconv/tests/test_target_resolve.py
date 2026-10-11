"""`--target <name>` must PROVE its mapping against the tree, or raise.

The class this guards: `make_user_dll_target` used to hardcode `<name>-rust` +
`ntdll-sys`. For msvcrt both were wrong — the crate is `msvcrt-crt-rust`, and it does
not depend on ntdll-sys at all, so every `ntdll_sys::` path the model emitted was
unresolvable. Per rustconv's own `crate_bindgen_files` contract an unresolved name is
not a no-op: the model invents a forward declaration that compiles, links, and is
silently wrong (for msvcrt, a guessed `threadmbcinfo` whose `mbctype[257]` every
predicate indexes).
"""

from __future__ import annotations

import pytest

from rustconv.config import (
    REACTOS_ROOT,
    RUST_DIR,
    TargetResolveError,
    _detect_sys_umbrella,
    make_user_dll_target,
)


def _dll(name: str):
    return REACTOS_ROOT / "dll" / "win32" / name


# Every pre-existing standalone user DLL follows the <name>-rust + ntdll-sys convention
# this used to hardcode. Deriving from the tree must return their historical values
# UNCHANGED — the whole point is that generalizing costs no other lane anything.
@pytest.mark.parametrize("name", ["ws2_32", "iphlpapi", "netapi32", "dnsapi", "wkssvc"])
def test_conventional_targets_are_unchanged(name):
    if not (RUST_DIR / f"{name}-rust").is_dir():
        pytest.skip(f"{name}-rust not in this tree")
    t = make_user_dll_target(name, _dll(name))
    assert t.output_crate == RUST_DIR / f"{name}-rust"
    assert t.sys_crate == "ntdll_sys"
    assert t.sys_umbrella == "nt"
    assert t.bindgen_files["amd64"] == RUST_DIR / "ntdll-sys" / "src" / "bindgen_amd64.rs"


def test_unconventional_crate_raises_and_names_a_real_flag():
    """msvcrt's crate is `msvcrt-crt-rust`. Refuse to guess, and name the override."""
    if not (RUST_DIR / "msvcrt-crt-rust").is_dir():
        pytest.skip("msvcrt-crt-rust not in this tree")
    with pytest.raises(TargetResolveError) as e:
        make_user_dll_target("msvcrt", _dll("msvcrt"))
    msg = str(e.value)
    assert "msvcrt-crt-rust" in msg
    # The remedy must be a flag that EXISTS — a fail-loud naming a fictional flag is
    # worse than none (profile_derive shipped exactly that bug: "Pass --usermode/--kernel").
    assert "--output-crate" in msg


def test_override_derives_substrate_from_the_crates_own_cargo_toml():
    if not (RUST_DIR / "msvcrt-crt-rust").is_dir():
        pytest.skip("msvcrt-crt-rust not in this tree")
    t = make_user_dll_target("msvcrt", _dll("msvcrt"), output_crate="msvcrt-crt-rust")
    assert t.output_crate == RUST_DIR / "msvcrt-crt-rust"
    # msvcrt-crt-rust declares msvcrt-sys and NOT ntdll-sys.
    assert t.sys_crate == "msvcrt_sys"
    assert t.bindgen_files["amd64"] == RUST_DIR / "msvcrt-sys" / "src" / "bindgen_amd64.rs"
    # msvcrt-sys re-exports its bindgen FLAT: "" (no umbrella segment), not None
    # (which would mean "kernel target" and skip user-mode postprocessing entirely).
    assert t.sys_umbrella == ""


def test_unknown_override_crate_raises():
    with pytest.raises(TargetResolveError, match="no such directory"):
        make_user_dll_target("msvcrt", _dll("msvcrt"), output_crate="does-not-exist-rust")


def test_unknown_sys_crate_raises():
    if not (RUST_DIR / "msvcrt-crt-rust").is_dir():
        pytest.skip("msvcrt-crt-rust not in this tree")
    with pytest.raises(TargetResolveError, match="no such crate"):
        make_user_dll_target(
            "msvcrt", _dll("msvcrt"), output_crate="msvcrt-crt-rust", sys_crate="nope-sys"
        )


def test_umbrella_detection_both_shapes(tmp_path):
    """Both in-tree substrate shapes must be recognized, and neither guessed."""

    def sys_crate(body: str):
        d = tmp_path / f"x{abs(hash(body))}-sys"
        (d / "src").mkdir(parents=True)
        (d / "src" / "lib.rs").write_text(body)
        return d

    # ntdll-sys shape: one umbrella module.
    assert _detect_sys_umbrella(sys_crate("mod bindgen_amd64;\npub mod nt {\n}\n"), "x") == "nt"
    # msvcrt-sys shape: flat root re-export.
    assert (
        _detect_sys_umbrella(sys_crate("mod bindgen_amd64;\npub use bindgen_amd64::*;\n"), "x")
        == ""
    )
    # Ambiguous / neither: raise rather than pick.
    with pytest.raises(TargetResolveError):
        _detect_sys_umbrella(sys_crate("mod bindgen_amd64;\n"), "x")
    with pytest.raises(TargetResolveError):
        _detect_sys_umbrella(sys_crate("pub use bindgen_amd64::*;\npub mod nt {\n}\n"), "x")
