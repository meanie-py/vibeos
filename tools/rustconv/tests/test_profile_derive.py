"""Tests for profile_derive — the bindgen-island profile deriver.

Every case here is one of the two bug classes the module exists to prevent, or a false
positive it produced while being calibrated against the real tree:

* **§9.112** an incomplete island: a constant a ``switch`` dispatches on that the island
  will not carry becomes a silent catch-all ``match`` arm. Detection cases below assert the
  LOUD finding; the ``#ifndef __REACTOS__`` case asserts we do NOT cry wolf on dead code
  (a false ``DLL_WINE_PREATTACH`` alarm on the real ``snmpapi`` is what motivated it).
* **§9.108** mirroring each TU's real include set instead of a master header, and splitting
  into ``TYPES_STEM`` islands only for a pair that genuinely collides.

Fixtures are synthetic trees under ``tmp_path`` — no dependency on real repo paths, so the
tests keep working in a worktree with no build dirs.
"""

from __future__ import annotations

import re
from pathlib import Path

import pytest

from rustconv.profile_derive import (
    DerivedProfile,
    Finding,
    ProfileDeriveError,
    _interface_include_index,
    build_include_env,
    derive,
    emit_profile,
    emit_report,
    find_conflicts,
    find_switch_constants,
    header_symbols,
    is_shim_tu,
    parse_cmakelists,
    resolve_sys_crate,
    scaffold_crate,
    scan_includes,
    scan_preamble,
    to_json,
    uncomment_flipped_source_lists,
    walk_includes,
)

# ---------------------------------------------------------------------------
# fixture builder
# ---------------------------------------------------------------------------

CMAKE_USERMODE = """
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/idl)
spec2def(widget.dll widget.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    main.c
    helper.c
    ${CMAKE_CURRENT_BINARY_DIR}/widget_c.c)

add_library(widget MODULE ${SOURCE} widget.rc)
set_module_type(widget win32dll UNICODE)
add_importlibs(widget msvcrt kernel32 ntdll)
"""


def _tree(tmp_path: Path) -> Path:
    """A minimal fake ReactOS tree: sdk include roots + one module dir."""
    for d in ("sdk/include", "sdk/include/psdk", "sdk/include/ddk", "sdk/include/reactos/idl"):
        (tmp_path / d).mkdir(parents=True, exist_ok=True)
    (tmp_path / "dll/win32/widget").mkdir(parents=True, exist_ok=True)
    return tmp_path


def _mod(tmp_path: Path) -> Path:
    return tmp_path / "dll/win32/widget"


def _write(path: Path, text: str) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text)
    return path


def _basic_module(tmp_path: Path, *, main_extra: str = "", cmake: str = CMAKE_USERMODE) -> Path:
    """widget.dll: main.c + helper.c compiled, orphan.c dead on disk.

    Include chain: main.c -> "precomp.h" -> <widget_defs.h> (psdk) -> <widget_base.h> (psdk)
    """
    root = _tree(tmp_path)
    mod = _mod(tmp_path)
    _write(mod / "CMakeLists.txt", cmake)
    _write(
        root / "sdk/include/psdk/widget_base.h",
        "#ifndef _WIDGET_BASE_H\n#define _WIDGET_BASE_H\ntypedef struct _WBASE { int x; } WBASE;\n"
        "#define WIDGET_BASE_OK 1\n#endif\n",
    )
    _write(
        root / "sdk/include/psdk/widget_defs.h",
        "#ifndef _WIDGET_DEFS_H\n#define _WIDGET_DEFS_H\n#include <widget_base.h>\n"
        "#define WIDGET_MODE_FAST  1\n#define WIDGET_MODE_SLOW  2\n"
        "#define WIDGET_MASK (WIDGET_MODE_FAST | WIDGET_MODE_SLOW)\n"
        "#define WIDGET_CALC(x) ((x) + 1)\n#endif\n",
    )
    _write(mod / "precomp.h", "#include <widget_defs.h>\ntypedef struct _WPRIV { int y; } WPRIV;\n")
    _write(
        mod / "main.c",
        '#define WIN32_NO_STATUS\n#include <stdarg.h>\n#include "precomp.h"\n\n'
        "int dispatch(int mode)\n{\n"
        "    switch (mode)\n    {\n"
        "        case WIDGET_MODE_FAST: return 1;\n"
        "        case WIDGET_MODE_SLOW: return 2;\n"
        "        default: return 0;\n    }\n}\n" + main_extra,
    )
    _write(mod / "helper.c", '#include "precomp.h"\nint helper(void) { return 0; }\n')
    _write(mod / "orphan.c", "int never_compiled(void) { return 1; }\n")
    return root


# ---------------------------------------------------------------------------
# CMakeLists: which TUs are ACTUALLY compiled
# ---------------------------------------------------------------------------


def test_dead_source_is_excluded_and_reported(tmp_path):
    """orphan.c exists on disk but is not in the source list — paying to translate it is
    money spent on a file nobody links (it happened last batch)."""
    root = _basic_module(tmp_path)
    m = parse_cmakelists(_mod(tmp_path) / "CMakeLists.txt", root)
    assert [p.name for p in m.compiled_tus] == ["main.c", "helper.c"]
    assert [p.name for p in m.dead_sources] == ["orphan.c"]
    assert any(f.code == "DEAD_SOURCE" for f in m.findings)


CMAKE_STATIC_SIBLING = """\
list(APPEND SOURCE helper.c orphan_is_live.c)
add_library(widget_static STATIC ${SOURCE})
add_library(widget MODULE main.c widget.rc)
target_link_libraries(widget widget_static)
set_module_type(widget nativedll)
"""


def test_sibling_target_sources_are_live_not_dead(tmp_path):
    """setuplib's shape: the .c live in a STATIC sibling linked into the module. Judged
    against the module alone they read as dead, and DEAD_SOURCE says do not pay for them."""
    root = _basic_module(tmp_path, cmake=CMAKE_STATIC_SIBLING)
    _write(_mod(tmp_path) / "orphan_is_live.c", "int live(void) { return 1; }\n")
    m = parse_cmakelists(_mod(tmp_path) / "CMakeLists.txt", root, target="widget")
    assert [p.name for p in m.dead_sources] == ["orphan.c"]
    sib = [f for f in m.findings if f.code == "SIBLING_TARGET"]
    assert len(sib) == 1 and "--target widget_static" in sib[0].detail
    assert "orphan_is_live.c" in sib[0].detail and "helper.c" in sib[0].detail


def test_static_library_inherits_its_consumers_mode(tmp_path):
    root = _basic_module(tmp_path, cmake=CMAKE_STATIC_SIBLING)
    m = parse_cmakelists(_mod(tmp_path) / "CMakeLists.txt", root, target="widget_static")
    assert m.is_usermode
    assert any(f.code == "MODULE_TYPE_INHERITED" for f in m.findings)
    # one consumer: the crate belongs to it (widget-rust, `widget)` arm), not widget_static
    assert m.crate_stem == "widget"


def test_generated_sources_are_kept_separate_from_translatable_ones(tmp_path):
    root = _basic_module(tmp_path)
    m = parse_cmakelists(_mod(tmp_path) / "CMakeLists.txt", root)
    assert m.generated_sources == ["widget_c.c"]  # MIDL product: stays C, never translated
    assert all("widget_c.c" not in p.name for p in m.compiled_tus)


def test_module_type_and_unicode_come_from_set_module_type(tmp_path):
    root = _basic_module(tmp_path)
    m = parse_cmakelists(_mod(tmp_path) / "CMakeLists.txt", root)
    assert m.module_type == "win32dll"
    assert m.is_usermode is True
    assert m.unicode is True


def test_kernel_module_type_flips_usermode(tmp_path):
    cmake = (
        "add_definitions(-DNDIS50_MINIPORT)\n"
        "list(APPEND SOURCE main.c helper.c)\n"
        "add_library(widget MODULE ${SOURCE})\n"
        "set_module_type(widget kernelmodedriver)\n"
    )
    root = _basic_module(tmp_path, cmake=cmake)
    m = parse_cmakelists(_mod(tmp_path) / "CMakeLists.txt", root)
    assert m.is_usermode is False
    assert "-DNDIS50_MINIPORT" in m.definitions


def test_target_compile_definitions_are_picked_up(tmp_path):
    """`disk` carries its whole Win8 version profile as target_compile_definitions (no -D,
    per target). Missing it derives the wrong version-gated struct layouts."""
    cmake = (
        "list(APPEND SOURCE main.c helper.c)\n"
        "add_library(widget MODULE ${SOURCE})\n"
        "target_compile_definitions(widget PUBLIC DEBUG_USE_KDPRINT _WIN32_WINNT=0x602)\n"
        "set_module_type(widget kernelmodedriver)\n"
    )
    root = _basic_module(tmp_path, cmake=cmake)
    dp = derive(_mod(tmp_path), root)
    assert "-DDEBUG_USE_KDPRINT" in dp.ver_defs
    assert "-D_WIN32_WINNT=0x602" in dp.ver_defs
    # WINVER tracks _WIN32_WINNT when the module bumps one and not the other
    assert "-DWINVER=0x602" in dp.ver_defs
    assert "-DNTDDI_VERSION=0x06020000" in dp.ver_defs


def test_arch_conditional_sources_resolve_against_the_requested_arch(tmp_path):
    cmake = (
        "list(APPEND SOURCE main.c)\n"
        'if(ARCH STREQUAL "i386")\n    list(APPEND SOURCE helper.c)\nendif()\n'
        "add_library(widget MODULE ${SOURCE})\nset_module_type(widget win32dll)\n"
    )
    root = _basic_module(tmp_path, cmake=cmake)
    i386 = parse_cmakelists(_mod(tmp_path) / "CMakeLists.txt", root, arch="i386")
    amd64 = parse_cmakelists(_mod(tmp_path) / "CMakeLists.txt", root, arch="amd64")
    assert [p.name for p in i386.compiled_tus] == ["main.c", "helper.c"]
    assert [p.name for p in amd64.compiled_tus] == ["main.c"]


def test_ambiguous_target_fails_loudly(tmp_path):
    cmake = (
        "list(APPEND SOURCE main.c)\n"
        "add_library(alpha MODULE ${SOURCE})\nset_module_type(alpha win32dll)\n"
        "add_library(beta MODULE ${SOURCE})\nset_module_type(beta win32dll)\n"
    )
    root = _basic_module(tmp_path, cmake=cmake)
    with pytest.raises(ProfileDeriveError, match="pass --target"):
        parse_cmakelists(_mod(tmp_path) / "CMakeLists.txt", root)


def test_missing_cmakelists_fails_loudly(tmp_path):
    with pytest.raises(ProfileDeriveError, match=re.escape("no CMakeLists.txt")):
        parse_cmakelists(tmp_path / "nope/CMakeLists.txt", tmp_path)


# ---------------------------------------------------------------------------
# include closure
# ---------------------------------------------------------------------------


def test_include_closure_follows_the_nested_chain(tmp_path):
    root = _basic_module(tmp_path)
    m = parse_cmakelists(_mod(tmp_path) / "CMakeLists.txt", root)
    env = build_include_env(root, m)
    closure = walk_includes(m.compiled_tus, env)
    names = {r.name for r in closure.headers.values()}
    assert {"precomp.h", "widget_defs.h", "widget_base.h"} <= names
    depths = {r.name: r.depth for r in closure.headers.values()}
    assert depths["precomp.h"] == 1
    assert depths["widget_defs.h"] == 2
    assert depths["widget_base.h"] == 3
    kinds = {r.name: r.kind for r in closure.headers.values()}
    assert kinds["precomp.h"] == "module"
    assert kinds["widget_defs.h"] == "sdk"


def test_quoted_include_prefers_the_includers_own_directory(tmp_path):
    """`#include "shared.h"` next to the TU must NOT resolve to the same-named SDK header."""
    root = _basic_module(tmp_path)
    _write(root / "sdk/include/psdk/shared.h", "#define WHICH 2\n")
    _write(_mod(tmp_path) / "shared.h", "#define WHICH 1\n")
    _write(_mod(tmp_path) / "helper.c", '#include "shared.h"\nint helper(void){return WHICH;}\n')
    m = parse_cmakelists(_mod(tmp_path) / "CMakeLists.txt", root)
    env = build_include_env(root, m)
    closure = walk_includes([_mod(tmp_path) / "helper.c"], env)
    hit = next(p for p, r in closure.headers.items() if r.name == "shared.h")
    assert hit == (_mod(tmp_path) / "shared.h").resolve()


def test_compile_db_include_order_outranks_the_fixed_list(tmp_path):
    """A wine module's `-I sdk/include/wine` comes before psdk, so `<commctrl.h>` is wine's."""
    root = _basic_module(tmp_path)
    _write(root / "sdk/include/psdk/commctrl.h", "#define WHICH 2\n")
    _write(root / "sdk/include/wine/commctrl.h", "#define WHICH 1\n")
    _write(_mod(tmp_path) / "helper.c", "#include <commctrl.h>\nint helper(void){return WHICH;}\n")
    m = parse_cmakelists(_mod(tmp_path) / "CMakeLists.txt", root)
    env = build_include_env(
        root, m, compile_dirs=(root / "sdk/include/wine", root / "sdk/include/psdk")
    )
    closure = walk_includes([_mod(tmp_path) / "helper.c"], env)
    hit = next(p for p, r in closure.headers.items() if r.name == "commctrl.h")
    assert hit == (root / "sdk/include/wine/commctrl.h").resolve()


def test_unresolved_include_raises_loudly(tmp_path):
    """Never emit a partial allowlist that looks complete (AGENTS.md: fail loud)."""
    root = _basic_module(tmp_path)
    _write(
        _mod(tmp_path) / "helper.c", "#include <nowhere_at_all.h>\nint helper(void){return 0;}\n"
    )
    with pytest.raises(ProfileDeriveError) as exc:
        derive(_mod(tmp_path), root)
    assert "nowhere_at_all.h" in str(exc.value)
    assert "--allow-unresolved" in str(exc.value)


def test_unresolved_include_can_be_demoted_to_a_finding(tmp_path):
    root = _basic_module(tmp_path)
    _write(
        _mod(tmp_path) / "helper.c", "#include <nowhere_at_all.h>\nint helper(void){return 0;}\n"
    )
    dp = derive(_mod(tmp_path), root, strict_unresolved=False)
    codes = [(f.severity, f.code) for f in dp.findings]
    assert ("error", "UNRESOLVED_INCLUDE") in codes
    assert any("nowhere_at_all.h" in u.name for u in dp.closure.unresolved)


def test_unresolved_include_inside_a_conditional_does_not_block(tmp_path):
    """`#ifdef _MAC #include <rpcmac.h>` is a dead platform arm, not a broken search path."""
    root = _basic_module(tmp_path)
    _write(
        _mod(tmp_path) / "helper.c",
        '#include "precomp.h"\n#ifdef _SOME_OTHER_PLATFORM\n#include <mac_only.h>\n#endif\n'
        "int helper(void){return 0;}\n",
    )
    dp = derive(_mod(tmp_path), root)  # strict: must NOT raise
    assert any(u.name == "mac_only.h" and u.conditional for u in dp.closure.unresolved)
    assert not any(f.code == "UNRESOLVED_INCLUDE" and f.severity == "error" for f in dp.findings)


def test_if_zero_block_is_skipped_but_its_else_arm_is_not():
    incs = scan_includes("#if 0\n#include <dead.h>\n#else\n#include <live.h>\n#endif\n")
    assert [i.name for i in incs] == ["live.h"]


def test_ifndef_reactos_is_dead_and_its_else_arm_is_live():
    incs = scan_includes(
        "#ifndef __REACTOS__\n#include <wine.h>\n#else\n#include <ros.h>\n#endif\n"
    )
    assert [i.name for i in incs] == ["ros.h"]


def test_include_guard_does_not_mark_everything_conditional():
    text = "#ifndef _X_H\n#define _X_H\n#include <real.h>\n#endif\n"
    (inc,) = scan_includes(text)
    assert inc.conditional is False


def test_preamble_is_mirrored_verbatim_including_its_defines():
    """§9.108: a TU's own prologue is a guaranteed-legal wrapper; a master header is not."""
    lines, conditional = scan_preamble(
        "/* c */\n#define WIN32_NO_STATUS\n#include <windef.h>\n"
        '#include "precomp.h"\n\nint f(void) { return 0; }\n#include <late.h>\n'
    )
    assert lines == ["#define WIN32_NO_STATUS", "#include <windef.h>", '#include "precomp.h"']
    assert conditional is False


# ---------------------------------------------------------------------------
# §9.112 — constants a switch dispatches on
# ---------------------------------------------------------------------------


def test_switch_constants_are_found_via_tree_sitter(tmp_path):
    root = _basic_module(tmp_path)
    consts = find_switch_constants([_mod(tmp_path) / "main.c"], root)
    assert {c.ident for c in consts} == {"WIDGET_MODE_FAST", "WIDGET_MODE_SLOW"}


def test_case_label_in_a_dead_reactos_branch_is_not_audited(tmp_path):
    """The false `DLL_WINE_PREATTACH` alarm: wine's arm is inside `#ifndef __REACTOS__`."""
    root = _basic_module(tmp_path)
    _write(
        _mod(tmp_path) / "helper.c",
        '#include "precomp.h"\nint h(int r){ switch (r) {\n'
        "#ifndef __REACTOS__\n    case WINE_ONLY_REASON: return -1;\n#endif\n"
        "    case WIDGET_MODE_FAST: return 1;\n} return 0; }\n",
    )
    consts = find_switch_constants([_mod(tmp_path) / "helper.c"], root)
    assert "WINE_ONLY_REASON" not in {c.ident for c in consts}
    assert "WIDGET_MODE_FAST" in {c.ident for c in consts}


def test_a_macro_call_in_a_case_label_audits_its_arguments_not_its_name(tmp_path):
    """inetcpl: `case MAKEWPARAM(IDOK, BN_CLICKED):` -- MAKEWPARAM is no constant."""
    root = _basic_module(tmp_path)
    _write(
        _mod(tmp_path) / "cmd.c",
        '#include "precomp.h"\nint c(int w){ switch (w) {\n'
        "    case MAKEWPARAM(WIDGET_MODE_FAST, WIDGET_MODE_SLOW): return 1;\n} return 0; }\n",
    )
    consts = find_switch_constants([_mod(tmp_path) / "cmd.c"], root)
    assert {c.ident for c in consts} == {"WIDGET_MODE_FAST", "WIDGET_MODE_SLOW"}


def test_case_labels_hidden_behind_a_function_like_macro_are_audited(tmp_path):
    """msvcrt's errno.c: all 39 arms are `ERR_CASE(x)` / `ERR_MAPS(x, y)`, no literal `case`.

    The AST walk sees unexpanded source, so before this these audited as ZERO constants —
    a clean bill of health for a file with 39 unaudited arms, which is precisely the silent
    miss the tree-sitter-only rule was chosen to avoid.
    """
    root = _basic_module(tmp_path)
    _write(
        _mod(tmp_path) / "helper.c",
        '#include "precomp.h"\n'
        "void s(int e){ int *p; switch (e) {\n"
        "#define ERR_CASE(oserr) case oserr:\n"
        "#define ERR_MAPS(oserr, crterr) case oserr: *p = crterr; break\n"
        "    ERR_CASE(ERROR_ACCESS_DENIED)\n"
        "    ERR_MAPS(ERROR_LOCK_VIOLATION, WIDGET_EACCES);\n"
        "    default: break;\n} }\n",
    )
    idents = {c.ident for c in find_switch_constants([_mod(tmp_path) / "helper.c"], root)}
    assert "ERROR_ACCESS_DENIED" in idents
    assert "ERROR_LOCK_VIOLATION" in idents
    # the SECOND parameter of ERR_MAPS is the assigned value, not an arm
    assert "WIDGET_EACCES" not in idents
    # and the macro's own parameter names are not constants
    assert "oserr" not in idents
    assert "crterr" not in idents


def test_case_macro_with_a_parenthesised_argument_is_audited(tmp_path):
    """secur32's `_x(x) case (x): ...` and dnsapi's `X(x) case (x): return #x;`."""
    root = _basic_module(tmp_path)
    _write(
        _mod(tmp_path) / "helper.c",
        '#include "precomp.h"\n'
        "const char *n(int t){ switch (t) {\n"
        "#define X(x)    case (x): return #x;\n"
        "    X(WIDGET_TYPE_A)\n"
        "    X(WIDGET_TYPE_B)\n"
        "} return 0; }\n",
    )
    idents = {c.ident for c in find_switch_constants([_mod(tmp_path) / "helper.c"], root)}
    assert {"WIDGET_TYPE_A", "WIDGET_TYPE_B"} <= idents
    assert "x" not in idents


def test_multiline_case_macro_is_audited(tmp_path):
    """gdi32's `META_UNIMP(x)` spreads its body over backslash-continued lines."""
    root = _basic_module(tmp_path)
    _write(
        _mod(tmp_path) / "helper.c",
        '#include "precomp.h"\n'
        "void p(int r){ switch (r) {\n"
        "#define META_UNIMP(x) case x: \\\n"
        'FIXME("record type "#x" not implemented.\\n"); \\\n'
        "break;\n"
        "    META_UNIMP(META_DRAWTEXT)\n"
        "} }\n",
    )
    idents = {c.ident for c in find_switch_constants([_mod(tmp_path) / "helper.c"], root)}
    assert "META_DRAWTEXT" in idents


def test_function_like_macro_without_a_case_body_contributes_nothing(tmp_path):
    """Only macros that actually GENERATE an arm count — not every function-like macro."""
    root = _basic_module(tmp_path)
    _write(
        _mod(tmp_path) / "helper.c",
        '#include "precomp.h"\n'
        "#define MAX(a, b) ((a) > (b) ? (a) : (b))\n"
        "int h(int r){ return MAX(WIDGET_LIMIT, r); }\n",
    )
    idents = {c.ident for c in find_switch_constants([_mod(tmp_path) / "helper.c"], root)}
    assert "WIDGET_LIMIT" not in idents


def test_reachable_const_owner_is_force_added_to_the_allowlist(tmp_path):
    """The §9.112 prophylactic: allowlist the header owning any switch constant."""
    root = _basic_module(tmp_path)
    dp = derive(_mod(tmp_path), root)
    owners = {o.ident: o for o in dp.const_owners}
    assert owners["WIDGET_MODE_FAST"].owner is not None
    assert owners["WIDGET_MODE_FAST"].owner.name == "widget_defs.h"
    assert owners["WIDGET_MODE_FAST"].allowlisted is True
    assert any("widget_defs" in rx for rx in dp.allow_files)


def test_unreachable_switch_constant_is_a_loud_finding(tmp_path):
    """No header in the closure defines it -> the match arm silently matches EVERYTHING."""
    root = _basic_module(tmp_path)
    # WIDGET_MODE_TURBO lives in a header the module never includes
    _write(root / "sdk/include/psdk/widget_extra.h", "#define WIDGET_MODE_TURBO 3\n")
    _write(
        _mod(tmp_path) / "helper.c",
        '#include "precomp.h"\nint h(int m){ switch (m) { case WIDGET_MODE_TURBO: return 3; } '
        "return 0; }\n",
    )
    dp = derive(_mod(tmp_path), root)
    bad = [f for f in dp.findings if f.code == "SWITCH_CONST_UNOWNED"]
    assert len(bad) == 1
    assert "WIDGET_MODE_TURBO" in bad[0].detail
    assert bad[0].severity == "error"
    assert dp.errors, "an unowned switch constant must make the run non-zero"


def test_function_like_macro_in_a_case_label_is_flagged_not_blocking(tmp_path):
    """bindgen emits no constant for a function-like macro, so allowlisting is not enough --
    but a call-shaped pattern cannot become a silent catch-all (rustc rejects it), so it
    warns rather than blocking the module."""
    root = _basic_module(tmp_path)
    _write(
        _mod(tmp_path) / "helper.c",
        '#include "precomp.h"\nint h(int m){ switch (m) { case WIDGET_CALC: return 1; } '
        "return 0; }\n",
    )
    dp = derive(_mod(tmp_path), root)
    hits = [f for f in dp.findings if f.code == "SWITCH_CONST_NOT_BINDGENABLE"]
    assert hits and all(f.severity == "warn" for f in hits)


def test_sys_crate_consts_carries_bindgen_enum_members_by_bare_name(tmp_path):
    """bindgen names an enumerator `<type>_<member>`; the bare member the C switches on is
    still carried (port resolve aliases it), so it must not read as missing."""
    from rustconv.profile_derive import sys_crate_consts

    src = tmp_path / "sdk/lib/rust/ntkrnl-sys/src"
    src.mkdir(parents=True)
    (src / "bindgen_i386.rs").write_text(
        "pub const _DEVICE_USAGE_NOTIFICATION_TYPE_DeviceUsageTypeHibernation:\n"
        "    _DEVICE_USAGE_NOTIFICATION_TYPE = 2;\n"
        "pub const PLAIN: u32 = 1;\n"
    )
    got = sys_crate_consts(tmp_path, usermode=False, arch="i386")
    assert got["DeviceUsageTypeHibernation"] == "bindgen_i386"
    assert got["PLAIN"] == "bindgen_i386"


def test_macro_expression_const_is_flagged_for_verification(tmp_path):
    root = _basic_module(tmp_path)
    _write(
        _mod(tmp_path) / "helper.c",
        '#include "precomp.h"\nint h(int m){ switch (m) { case WIDGET_MASK: return 1; } '
        "return 0; }\n",
    )
    dp = derive(_mod(tmp_path), root)
    assert any(f.code == "SWITCH_CONST_EXPRESSION" for f in dp.findings)


def test_tu_local_const_needs_no_island_change(tmp_path):
    root = _basic_module(tmp_path)
    _write(
        _mod(tmp_path) / "helper.c",
        '#include "precomp.h"\n#define LOCAL_MODE 7\n'
        "int h(int m){ switch (m) { case LOCAL_MODE: return 7; } return 0; }\n",
    )
    dp = derive(_mod(tmp_path), root)
    owner = next(o for o in dp.const_owners if o.ident == "LOCAL_MODE")
    assert owner.owner_kind == "tu-local"
    assert any(f.code == "SWITCH_CONST_TU_LOCAL" for f in dp.findings)
    assert not dp.errors


def test_header_symbols_sees_a_comment_wrapped_enum_member(tmp_path):
    # setuplib's fsutil.h, verbatim shape: the member was reported UNOWNED.
    h = tmp_path / "fsutil.h"
    h.write_text(
        "typedef enum _FSVOLNOTIFY\n{\n    FSVOLNOTIFY_CHECKERROR,\n"
        "    /**/ChangeSystemPartition/**/ // FIXME: Deprecate!\n} FSVOLNOTIFY;\n"
    )
    syms = header_symbols(h, frozenset({"ChangeSystemPartition"}))
    assert syms["ChangeSystemPartition"][0] == "enum"


def test_header_symbols_classifies_macro_kinds(tmp_path):
    h = _write(
        tmp_path / "h.h",
        "#define LIT 0x10\n#define EXPR (LIT | 1)\n#define FN(x) ((x)+1)\n"
        "enum _E { E_ONE = 1, E_TWO };\n",
    )
    syms = header_symbols(h)
    assert syms["LIT"][0] == "literal"
    assert syms["EXPR"][0] == "expression"
    assert syms["FN"][0] == "function-like"
    assert syms["E_ONE"][0] == "enum"
    # the targeted (single-alternation) fast path must agree with the full scan
    assert header_symbols(h, frozenset({"LIT", "FN"})).keys() == {"LIT", "FN"}


def test_header_symbols_owns_a_cxx_const(tmp_path):
    # browseui's internettoolbar.h: a legal C++ case label that neither the macro nor the enum
    # scan saw, so the module was blocked on a false SWITCH_CONST_UNOWNED.
    h = _write(tmp_path / "itb.h", "static const int gViewsCommandID = FCIDM_SHVIEW_AUTOARRANGE;\n")
    assert header_symbols(h, frozenset({"gViewsCommandID"}))["gViewsCommandID"] == (
        "cxx-const",
        "FCIDM_SHVIEW_AUTOARRANGE",
    )
    assert header_symbols(h)["gViewsCommandID"][0] == "cxx-const"


def test_targeted_scan_is_not_fooled_by_a_prefix_identifier(tmp_path):
    """Regression, found on the real afd/tdihelpers closure: alternation is first-match-wins,
    so `TDI_ADDRESS_TYPE_IP` swallowed the `#define TDI_ADDRESS_TYPE_IPX` line and IPX was
    reported UNOWNED — a FALSE §9.112 alarm on a constant defined two lines from its sibling.
    """
    h = _write(
        tmp_path / "tdi.h",
        "#define TDI_ADDRESS_TYPE_IP                 2\n"
        "#define TDI_ADDRESS_TYPE_IPX                6\n",
    )
    syms = header_symbols(h, frozenset({"TDI_ADDRESS_TYPE_IP", "TDI_ADDRESS_TYPE_IPX"}))
    assert syms["TDI_ADDRESS_TYPE_IP"] == ("literal", "2")
    assert syms["TDI_ADDRESS_TYPE_IPX"] == ("literal", "6")


# ---------------------------------------------------------------------------
# §9.108 — genuine conflicts get separate islands, co-reachable ones do not
# ---------------------------------------------------------------------------


def test_anonymous_aggregate_macro_is_not_a_type_definition(tmp_path):
    """Regression, found on the real wldap32 closure: `} DUMMYUNIONNAME;` closes an inline
    union FIELD. Counting it as a type made every header with an anonymous union "conflict"
    with every other one and proposed a bogus 3-island split."""
    root = _basic_module(tmp_path)
    body = (
        "typedef struct _THING {\n    union {\n        int a;\n        long b;\n"
        "    } DUMMYUNIONNAME;\n} THING;\n"
    )
    _write(root / "sdk/include/psdk/one.h", body)
    _write(
        root / "sdk/include/psdk/two.h",
        body.replace("_THING", "_OTHER").replace("} THING", "} OTHER"),
    )
    _write(_mod(tmp_path) / "main.c", "#include <one.h>\nint main_fn(void){return 0;}\n")
    _write(_mod(tmp_path) / "helper.c", "#include <two.h>\nint helper(void){return 0;}\n")
    dp = derive(_mod(tmp_path), root)
    assert len(dp.islands) == 1
    assert not any(f.code == "HEADER_CONFLICT" for f in dp.findings)


def test_last_enum_member_without_a_trailing_comma_is_still_found(tmp_path):
    """Regression, found on the real samlib closure: ntsam.h's `DomainDisplayServer` is the
    last enumerator, so its `}` is on the NEXT line — a same-line-only separator match
    reported it UNOWNED, i.e. a false §9.112 alarm."""
    h = _write(
        tmp_path / "ntsam.h",
        "typedef enum _DDI\n{\n    DomainDisplayUser = 1,\n    DomainDisplayGroup,\n"
        "    DomainDisplayServer\n} DDI, *PDDI;\n",
    )
    syms = header_symbols(h, frozenset({"DomainDisplayServer", "DomainDisplayGroup"}))
    assert syms["DomainDisplayServer"][0] == "enum"
    assert syms["DomainDisplayGroup"][0] == "enum"


def test_genuine_conflict_proposes_a_types_stem_split(tmp_path):
    """Two MIDL-ish headers redeclaring the same struct, and no TU includes both."""
    root = _basic_module(tmp_path)
    _write(
        root / "sdk/include/psdk/alpha_c.h",
        "typedef struct _SERVER_INFO_100 { int a; } SERVER_INFO_100;\n",
    )
    _write(
        root / "sdk/include/psdk/beta_c.h",
        "typedef struct _SERVER_INFO_100 { long b; } SERVER_INFO_100;\n",
    )
    _write(_mod(tmp_path) / "main.c", "#include <alpha_c.h>\nint main_fn(void){return 0;}\n")
    _write(_mod(tmp_path) / "helper.c", "#include <beta_c.h>\nint helper(void){return 0;}\n")
    dp = derive(_mod(tmp_path), root)
    assert len(dp.islands) == 2
    assert {i.stem for i in dp.islands} == {"types", "types_1"}
    assert any(f.code == "HEADER_CONFLICT" for f in dp.findings)
    assert any(f.code == "TYPES_STEM_SPLIT" for f in dp.findings)
    text = emit_profile(dp)
    assert "TYPES_STEM=types_1" in text
    # each island gets its OWN wrapper: emitting the UNION would rebuild the exact
    # redefinition wall the split exists to avoid (§9.108)
    wrappers = ["\n".join(i.wrapper_inc) for i in dp.islands]
    assert sum("alpha_c.h" in w for w in wrappers) == 1
    assert sum("beta_c.h" in w for w in wrappers) == 1
    assert not any("alpha_c.h" in w and "beta_c.h" in w for w in wrappers)
    assert 'case "$DRIVER" in' in text and "widget-types_1)" in text
    assert "OUT_CRATE_DIR=" in text  # both islands land in the one crate (netapi32 shape)


def test_co_reachable_headers_are_not_a_conflict(tmp_path):
    """If one .c includes both and compiles, the pair is compatible by construction (§9.108)."""
    root = _basic_module(tmp_path)
    _write(
        root / "sdk/include/psdk/a.h",
        "#ifndef _T\n#define _T\ntypedef struct _T { int a; } T;\n#endif\n",
    )
    _write(
        root / "sdk/include/psdk/b.h",
        "#ifndef _T\n#define _T\ntypedef struct _T { int a; } T;\n#endif\n",
    )
    _write(
        _mod(tmp_path) / "main.c", "#include <a.h>\n#include <b.h>\nint main_fn(void){return 0;}\n"
    )
    _write(_mod(tmp_path) / "helper.c", "#include <b.h>\nint helper(void){return 0;}\n")
    m = parse_cmakelists(_mod(tmp_path) / "CMakeLists.txt", root)
    env = build_include_env(root, m)
    closure = walk_includes(m.compiled_tus, env)
    assert find_conflicts(closure, sorted(closure.headers)) == []


# ---------------------------------------------------------------------------
# emission
# ---------------------------------------------------------------------------


def test_emitted_profile_uses_gen_driver_types_vocabulary(tmp_path):
    root = _basic_module(tmp_path)
    dp = derive(_mod(tmp_path), root)
    text = emit_profile(dp)
    assert "  widget)" in text
    assert "USERMODE=1" in text
    assert "BLOCKLIST_FUNCS=1" in text
    assert 'DRIVER_DIR="${SRC}/dll/win32/widget"' in text
    assert "WRAPPER_INC=$'" in text
    assert "#define WIN32_NO_STATUS" in text  # the TU's prologue, mirrored verbatim
    assert "ALLOW_FILES=(" in text and text.rstrip().endswith(";;")
    assert "--allowlist-file '.*widget[/\\\\].*\\.h'" in text
    assert "-DUNICODE" in text and "-D_UNICODE" in text  # from set_module_type(... UNICODE)
    assert "-DNTDDI_VERSION=" in text


def test_report_and_json_carry_the_evidence(tmp_path):
    root = _basic_module(tmp_path)
    dp = derive(_mod(tmp_path), root)
    report = emit_report(dp)
    assert "orphan.c" in report  # dead source is visible
    assert "switch-constant audit" in report
    assert "precomp.h" in report
    blob = to_json(dp)
    assert blob["module"] == "widget"
    assert blob["usermode"] is True
    assert "dll/win32/widget/orphan.c" in blob["dead_sources"]
    assert any(h["name"] == "widget_base.h" for h in blob["closure"])
    assert blob["switch_constants"][0]["owner"].endswith("widget_defs.h")


def test_no_build_dir_is_reported_not_hidden(tmp_path):
    root = _basic_module(tmp_path)
    dp = derive(_mod(tmp_path), root)
    codes = {f.code for f in dp.findings}
    assert "NO_BUILD_DIR" in codes  # generated MIDL headers cannot be resolved
    assert "NO_COMPILE_DB" in codes  # VER_DEFS are heuristic, and the report says so


# ---------------------------------------------------------------------------
# crate scaffold
# ---------------------------------------------------------------------------


def test_scaffold_writes_the_crate_skeleton(tmp_path):
    root = _basic_module(tmp_path)
    dp = derive(_mod(tmp_path), root)
    out = tmp_path / "widget-rust"
    written = scaffold_crate(dp, out)
    assert {p.relative_to(out).as_posix() for p in written} >= {
        "Cargo.toml",
        "src/lib.rs",
        "src/prelude.rs",
        "src/types.rs",
        "src/types_i386.rs",
        "src/types_amd64.rs",
        "src/types_arm64.rs",
    }
    cargo = (out / "Cargo.toml").read_text()
    assert 'name = "widget-rust"' in cargo
    assert 'features = ["user"]' in cargo and "ntdll-sys" in cargo
    lib = (out / "src/lib.rs").read_text()
    assert "pub mod main;" in lib and "pub mod helper;" in lib
    assert "#[panic_handler]" in lib and "rust_eh_personality" in lib
    # an ungenerated island must FAIL the build loudly, never compile as an empty module
    assert "compile_error!" in (out / "src/types_i386.rs").read_text()


def _cpp_module(tmp_path):
    """widget.dll with one C++ TU: the island must parse as C++, the crate gets atlclasses-rust."""
    root = _basic_module(tmp_path, cmake=CMAKE_USERMODE.replace("helper.c", "helper.cpp"))
    _write(_mod(tmp_path) / "helper.cpp", "int helper(void){return 0;}\n")
    return root


def test_cpp_module_island_parses_as_cpp(tmp_path):
    root = _cpp_module(tmp_path)
    assert "LANG_ARGS=( -x c++ -std=gnu++17 )" in emit_profile(derive(_mod(tmp_path), root))
    # and a pure-C module does not get it
    assert "LANG_ARGS" not in emit_profile(derive(_mod(tmp_path), _basic_module(tmp_path)))


def test_cpp_module_scaffold_wires_atl_rust(tmp_path):
    root = _cpp_module(tmp_path)
    out = tmp_path / "widget-rust"
    scaffold_crate(derive(_mod(tmp_path), root), out)
    assert 'atlclasses-rust = { path = "../atlclasses-rust" }' in (out / "Cargo.toml").read_text()
    prelude = (out / "src/prelude.rs").read_text()
    assert "pub use atl_rust::{atl_format," in prelude and "CStringW, POSITION}" in prelude


def test_scaffold_refuses_to_clobber(tmp_path):
    root = _basic_module(tmp_path)
    dp = derive(_mod(tmp_path), root)
    out = tmp_path / "widget-rust"
    scaffold_crate(dp, out)
    with pytest.raises(ProfileDeriveError, match="refusing to overwrite"):
        scaffold_crate(dp, out)


def test_scaffold_never_replaces_a_real_island_with_a_placeholder(tmp_path):
    root = _basic_module(tmp_path)
    dp = derive(_mod(tmp_path), root)
    out = tmp_path / "widget-rust"
    (out / "src").mkdir(parents=True)
    real = out / "src/types_i386.rs"
    real.write_text("// a REAL gen_driver_types.sh island\npub struct WPRIV { pub y: i32 }\n")
    scaffold_crate(dp, out)
    assert "REAL gen_driver_types.sh island" in real.read_text()


def test_scaffold_runs_after_gen_driver_types(tmp_path):
    """The two steps must be order-insensitive.

    gen_driver_types.sh writes src/types.rs (the per-arch dispatch stub) as well as the
    islands. It used to abort the whole scaffold — over a stub whose content is identical
    either way — so a module whose island was generated first could not be scaffolded at all.
    """
    root = _basic_module(tmp_path)
    dp = derive(_mod(tmp_path), root)
    out = tmp_path / "widget-rust"
    (out / "src").mkdir(parents=True)
    dispatch = out / "src/types.rs"
    dispatch.write_text('#[path = "types_i386.rs"] mod types_arch;\n')
    for arch in ("i386", "amd64", "arm64"):
        (out / f"src/types_{arch}.rs").write_text(f"// REAL island {arch}\n")

    kept: list[Path] = []
    written = scaffold_crate(dp, out, kept=kept)

    # the crate skeleton still gets written...
    assert {p.relative_to(out).as_posix() for p in written} >= {
        "Cargo.toml",
        "src/lib.rs",
        "src/prelude.rs",
    }
    # ...and every gen_driver_types.sh output survives byte-for-byte, reported not silent
    assert dispatch.read_text() == '#[path = "types_i386.rs"] mod types_arch;\n'
    assert "REAL island arm64" in (out / "src/types_arm64.rs").read_text()
    assert {p.relative_to(out).as_posix() for p in kept} == {
        "src/types.rs",
        "src/types_i386.rs",
        "src/types_amd64.rs",
        "src/types_arm64.rs",
    }


def test_a_refused_scaffold_writes_nothing_at_all(tmp_path):
    """Atomicity: a refusal must not leave a partial crate behind.

    It used to write Cargo.toml, lib.rs and prelude.rs and only THEN hit the conflict, so the
    retry failed on Cargo.toml — the tool wedged against its own output, reporting it as a
    pre-existing file the caller had to resolve.
    """
    root = _basic_module(tmp_path)
    dp = derive(_mod(tmp_path), root)
    out = tmp_path / "widget-rust"
    (out / "src").mkdir(parents=True)
    # a conflict on the LAST-planned kind of file, so anything non-atomic writes the earlier ones
    (out / "src/lib.rs").write_text("// pre-existing\n")

    with pytest.raises(ProfileDeriveError, match="refusing to overwrite"):
        scaffold_crate(dp, out)

    survivors = {p.relative_to(out).as_posix() for p in out.rglob("*") if p.is_file()}
    assert survivors == {"src/lib.rs"}
    assert not (out / "Cargo.toml").exists()


def test_scaffold_still_refuses_to_clobber_hand_written_files(tmp_path):
    """The keep-existing carve-out is ONLY for gen_driver_types.sh's files."""
    root = _basic_module(tmp_path)
    dp = derive(_mod(tmp_path), root)
    out = tmp_path / "widget-rust"
    (out / "src").mkdir(parents=True)
    (out / "src/prelude.rs").write_text("// hand-written substrate, do not lose\n")
    with pytest.raises(ProfileDeriveError, match="refusing to overwrite"):
        scaffold_crate(dp, out)
    assert "hand-written substrate" in (out / "src/prelude.rs").read_text()


def test_kernel_scaffold_depends_on_ntkrnl_sys(tmp_path):
    cmake = (
        "list(APPEND SOURCE main.c helper.c)\n"
        "add_library(widget MODULE ${SOURCE})\nset_module_type(widget kernelmodedriver)\n"
    )
    root = _basic_module(tmp_path, cmake=cmake)
    dp = derive(_mod(tmp_path), root)
    out = tmp_path / "widget-rust"
    scaffold_crate(dp, out)
    cargo = (out / "Cargo.toml").read_text()
    assert 'features = ["kernel"]' in cargo and "ntkrnl-sys" in cargo
    assert "USERMODE=0" in emit_profile(dp)
    # A kernel DPRINT1 prints too: win32macros' (DbgPrint is cdecl in ntoskrnl as in
    # ntdll), never a local no-op that drops every diagnostic.
    assert "win32macros-rust" in cargo
    assert "macro_rules! DPRINT1" not in (out / "src/prelude.rs").read_text()


# --------------------------------------------------------------------------- #
# "no in-tree .c" has two causes; and a fail-loud must name a real flag
# --------------------------------------------------------------------------- #


def test_no_c_points_at_the_sibling_target_that_owns_the_sources(tmp_path):
    """A cohab/static-lib module is NOT a Rust-flipped one, and must not be called one.

    msvcrt is the real case: the `msvcrt` MODULE target compiles only generated stubs +
    .rc, while `msvcrt_static` owns all 32 .c. Telling that user "an already-Rust-flipped
    module looks exactly like this, go read git history" sends them the wrong way — and
    is doubly wrong for a whole-archive cohab module, whose .c stay compiled forever.
    """
    cmake = (
        "list(APPEND SOURCE main.c helper.c)\n"
        "add_library(widget_static ${SOURCE})\n"
        "add_library(widget MODULE widget.rc)\n"
        "set_module_type(widget win32dll)\n"
    )
    root = _basic_module(tmp_path, cmake=cmake)
    with pytest.raises(ProfileDeriveError) as e:
        derive(_mod(tmp_path), root, target="widget")
    msg = str(e.value)
    assert "widget_static" in msg
    assert "--target widget_static" in msg
    assert "already-Rust-flipped" not in msg


def test_usermode_override_exists_for_a_target_with_no_set_module_type(tmp_path):
    """The error used to name `--usermode/--kernel`, which the argparser did not define.

    A fail-loud whose remedy is fictional is worse than no message. A static library
    shared between DLLs never calls set_module_type(), and it is exactly that target
    which owns the C in a cohab module — so the flag has to exist.
    """
    cmake = "list(APPEND SOURCE main.c helper.c)\nadd_library(widget_static ${SOURCE})\n"
    root = _basic_module(tmp_path, cmake=cmake)

    with pytest.raises(ProfileDeriveError, match="--usermode or --kernel"):
        emit_profile(derive(_mod(tmp_path), root, target="widget_static"))

    assert "USERMODE=1" in emit_profile(
        derive(_mod(tmp_path), root, target="widget_static", usermode=True)
    )
    assert "USERMODE=0" in emit_profile(
        derive(_mod(tmp_path), root, target="widget_static", usermode=False)
    )


# ---------------------------------------------------------------------------
# The generator's include set is not our search path
# ---------------------------------------------------------------------------


class TestGeneratorBaseIncludes:
    """An -I we resolve through must be EMITTED unless gen_driver_types.sh already
    passes it. Conflating the two produced a profile that could not generate:
    wldap32 includes "config.h", which lives only in sdk/include/wine, and the
    derived profile omitted that -I because our own search list mentions it —
    bindgen then died with `'config.h' file not found`."""

    def _db(self, tmp_path: Path, inc_dirs: list[str]) -> None:
        import json

        build = tmp_path / "build-i386"
        build.mkdir(exist_ok=True)
        incs = " ".join(f"-I/tmp/vibeos/{d}" for d in inc_dirs)
        build.joinpath("compile_commands.json").write_text(
            json.dumps(
                [
                    {
                        "directory": "/tmp/vibeos/build-i386",
                        "file": f"/tmp/vibeos/dll/win32/widget/{stem}.c",
                        "command": f"clang {incs} -c /tmp/vibeos/dll/win32/widget/{stem}.c",
                    }
                    for stem in ("main", "helper")
                ]
            )
        )

    def test_a_dir_the_generator_lacks_is_emitted(self, tmp_path: Path):
        root = _basic_module(tmp_path)
        (root / "sdk/include/wine").mkdir(parents=True, exist_ok=True)
        # wine BEFORE sdk/include, exactly as wldap32's real command orders it
        self._db(tmp_path, ["dll/win32/widget", "sdk/include/wine", "sdk/include"])
        dp = derive(_mod(tmp_path), root)
        assert any("sdk/include/wine" in e for e in dp.extra_inc), dp.extra_inc
        assert not any("sdk/include/wine" in e for e in dp.late_inc)

    def test_the_same_dir_ranked_late_goes_to_late_inc(self, tmp_path: Path):
        """samlib's ordering: hoisting wine ahead of the SDK resolves <winioctl.h> to
        the wine copy and collides with ndk/iotypes.h."""
        root = _basic_module(tmp_path)
        (root / "sdk/include/wine").mkdir(parents=True, exist_ok=True)
        self._db(tmp_path, ["dll/win32/widget", "sdk/include", "sdk/include/wine"])
        dp = derive(_mod(tmp_path), root)
        assert any("sdk/include/wine" in e for e in dp.late_inc), dp.late_inc
        assert not any("sdk/include/wine" in e for e in dp.extra_inc)

    def test_an_isystem_dir_is_read_like_an_I_dir(self, tmp_path: Path):
        """CMake spells an INTERFACE library's include dirs `-isystem <dir>`. Reading only
        `-I` lost browseui's atl_classes (sdk/lib/atl) and derive could not find atlstr.h."""
        import json

        from rustconv.profile_derive import load_compile_db

        root = _basic_module(tmp_path)
        (root / "sdk/lib/atl").mkdir(parents=True, exist_ok=True)
        build = tmp_path / "build-i386"
        build.mkdir(exist_ok=True)
        build.joinpath("compile_commands.json").write_text(
            json.dumps(
                [
                    {
                        "directory": "/tmp/vibeos/build-i386",
                        "file": f"/tmp/vibeos/dll/win32/widget/{stem}.c",
                        "command": "clang -I/tmp/vibeos/dll/win32/widget -isystem "
                        "/tmp/vibeos/sdk/lib/atl -isystem/tmp/vibeos/sdk/include "
                        f"-c /tmp/vibeos/dll/win32/widget/{stem}.c",
                    }
                    for stem in ("main", "helper")
                ]
            )
        )
        module = parse_cmakelists(_mod(tmp_path) / "CMakeLists.txt", root)
        db = load_compile_db(root, module, "i386")
        assert db is not None
        incs = [str(p) for p in db.include_dirs]
        assert any(p.endswith("sdk/lib/atl") for p in incs), incs
        assert any(p.endswith("sdk/include") for p in incs), incs

    def test_a_dir_the_generator_already_passes_is_not_re_emitted(self, tmp_path: Path):
        root = _basic_module(tmp_path)
        self._db(tmp_path, ["dll/win32/widget", "sdk/include", "sdk/include/psdk"])
        dp = derive(_mod(tmp_path), root)
        both = dp.extra_inc + dp.late_inc
        assert not any(e.endswith('/sdk/include"') for e in both), both
        assert not any("sdk/include/psdk" in e for e in both), both

    def test_the_build_dir_sdk_includes_never_bake_an_arch(self, tmp_path: Path):
        """`${BUILD_DIR}/sdk/...` is per-arch in the generator. Emitting the observed
        `build-i386/sdk/...` would grade all three arches against i386's generated
        headers — the cross-arch leak AGENTS.md forbids."""
        root = _basic_module(tmp_path)
        (root / "build-i386/sdk/include/reactos/mc").mkdir(parents=True, exist_ok=True)
        self._db(
            tmp_path,
            ["dll/win32/widget", "sdk/include", "build-i386/sdk/include/reactos/mc"],
        )
        dp = derive(_mod(tmp_path), root)
        both = dp.extra_inc + dp.late_inc
        assert not any("build-i386/sdk/" in e for e in both), both


# ---------------------------------------------------------------------------
# The wrapper must mirror what the TU ACTUALLY compiles with
# ---------------------------------------------------------------------------


class TestPrologueFidelity:
    def test_a_config_h_undef_kills_the_guarded_include(self, tmp_path: Path):
        """`#ifdef HAVE_LDAP_H / #include <ldap.h> / #endif` used to drop the guard and
        KEEP the include, promoting a conditional include to an unconditional one.
        ReactOS does not vendor OpenLDAP, so bindgen then died on a missing header."""
        undefs = frozenset({"HAVE_LDAP_H"})
        src = (
            '#include "config.h"\n'
            "#ifdef HAVE_LDAP_H\n"
            "#include <ldap.h>\n"
            "#endif\n"
            '#include "windef.h"\n'
            "int f(void) { return 0; }\n"
        )
        lines, conditional = scan_preamble(src, undefs)
        assert "#include <ldap.h>" not in lines
        assert lines == ['#include "config.h"', '#include "windef.h"']
        assert not conditional  # decided, so nothing for the caller to review

    def test_the_live_arm_of_a_decided_guard_is_kept(self, tmp_path: Path):
        undefs = frozenset({"HAVE_LDAP_H"})
        src = '#ifdef HAVE_LDAP_H\n#include <ldap.h>\n#else\n#include "fallback.h"\n#endif\n'
        lines, _ = scan_preamble(src, undefs)
        assert lines == ['#include "fallback.h"']

    def test_an_undecidable_guard_still_keeps_both_arms_and_warns(self, tmp_path: Path):
        src = "#ifdef SOME_UNKNOWN\n#include <a.h>\n#else\n#include <b.h>\n#endif\n"
        lines, conditional = scan_preamble(src, frozenset())
        assert lines == ["#include <a.h>", "#include <b.h>"]
        assert conditional

    def test_a_pch_skipped_tu_does_not_get_the_pch_in_its_wrapper(self, tmp_path: Path):
        """wldap32's ber.c is add_pch()'s SKIP argument precisely because precomp.h's
        headers conflict with its own — injecting it recreates the conflict."""
        cmake = (
            "list(APPEND SOURCE main.c helper.c)\n"
            "list(APPEND PCH_SKIP_SOURCE helper.c)\n"
            "add_library(widget MODULE ${SOURCE} ${PCH_SKIP_SOURCE})\n"
            "set_module_type(widget win32dll)\n"
            'add_pch(widget precomp.h "${PCH_SKIP_SOURCE}")\n'
        )
        root = _basic_module(tmp_path, cmake=cmake)
        dp = derive(_mod(tmp_path), root)
        assert (_mod(tmp_path) / "helper.c").resolve() in dp.module.pch_skip
        assert (_mod(tmp_path) / "main.c").resolve() not in dp.module.pch_skip


class TestMostlyDeadBodies:
    """A pre-PAYMENT check, and it exists because skipping it cost real money.

    wldap32 looks like an 8,625-line LDAP client. In this tree every entry point
    compiles to `return WLDAP32_LDAP_NOT_SUPPORTED;` — 181 `#ifdef HAVE_LDAP` blocks
    across 19 of its 20 TUs are disabled because ReactOS does not vendor OpenLDAP.
    Translating it bought 162 functions of the path the compiler throws away.
    """

    def _tree_with_config(self, tmp_path: Path, body: str):
        root = _tree(tmp_path)
        (root / "sdk/include/wine").mkdir(parents=True, exist_ok=True)
        (root / "sdk/include/wine/config.h").write_text("/* #undef HAVE_LDAP */\n")
        mod = _mod(tmp_path)
        _write(mod / "CMakeLists.txt", CMAKE_USERMODE)
        _write(mod / "precomp.h", "typedef struct _WPRIV { int y; } WPRIV;\n")
        _write(mod / "main.c", '#include "precomp.h"\n' + body)
        _write(mod / "helper.c", '#include "precomp.h"\nint helper(void) { return 0; }\n')
        from rustconv.profile_derive import _wine_config_undefs

        _wine_config_undefs.cache_clear()
        return root

    def test_a_module_that_is_mostly_a_disabled_ifdef_is_flagged(self, tmp_path: Path):
        dead = "\n".join(f"    int x{i} = {i};" for i in range(60))
        body = f"int go(void) {{\n    int ret = 0;\n#ifdef HAVE_LDAP\n{dead}\n#endif\n    return ret;\n}}\n"
        root = self._tree_with_config(tmp_path, body)
        dp = derive(_mod(tmp_path), root)
        hits = [f for f in dp.findings if f.code == "MOSTLY_DEAD_BODIES"]
        assert hits, [f.code for f in dp.findings]
        assert "HAVE_LDAP" in hits[0].detail

    def test_an_ordinary_module_is_not_flagged(self, tmp_path: Path):
        live = "\n".join(f"    int x{i} = {i};" for i in range(60))
        body = f"int go(void) {{\n{live}\n    return 0;\n}}\n"
        root = self._tree_with_config(tmp_path, body)
        dp = derive(_mod(tmp_path), root)
        assert not [f for f in dp.findings if f.code == "MOSTLY_DEAD_BODIES"]

    def test_the_LIVE_arm_of_an_ifndef_is_not_counted_dead(self, tmp_path: Path):
        """`#ifndef HAVE_LDAP` over an undefined macro is the arm that IS compiled."""
        live = "\n".join(f"    int x{i} = {i};" for i in range(60))
        body = f"int go(void) {{\n#ifndef HAVE_LDAP\n{live}\n#endif\n    return 0;\n}}\n"
        root = self._tree_with_config(tmp_path, body)
        dp = derive(_mod(tmp_path), root)
        assert not [f for f in dp.findings if f.code == "MOSTLY_DEAD_BODIES"]


class TestIntegerGatedDeadCode:
    """The OTHER decidable gate: a module that pins a macro to a value with `-D`.

    wine's `/* #undef HAVE_LDAP */` was the only class the audit knew, so msvcrt sailed
    straight through it. msvcrt compiles with `-D_MSVCR_VER=0`, and concurrency.c puts all
    3,839 of its lines inside `#if _MSVCR_VER >= 100` while handler4.c puts 775 inside
    `defined(__x86_64__) && _MSVCR_VER>=140`. Both build to a 396-byte object with ZERO
    defined symbols, and both looked like prime translation targets.

    They are also why the finding is PER-TU: together they are 10.6% of msvcrt's compiled
    lines, under the 15% module-wide threshold, so the module-wide warning alone stayed
    silent on 4,614 lines of guaranteed-dead code.
    """

    def _tree_with_pinned_ver(self, tmp_path: Path, body: str, *, value: str = "0"):
        root = _tree(tmp_path)
        (root / "sdk/include/wine").mkdir(parents=True, exist_ok=True)
        (root / "sdk/include/wine/config.h").write_text("/* #undef HAVE_LDAP */\n")
        mod = _mod(tmp_path)
        _write(
            mod / "CMakeLists.txt",
            CMAKE_USERMODE + f"target_compile_definitions(widget PRIVATE _MSVCR_VER={value})\n",
        )
        _write(mod / "precomp.h", "typedef struct _WPRIV { int y; } WPRIV;\n")
        _write(mod / "main.c", '#include "precomp.h"\n' + body)
        _write(mod / "helper.c", '#include "precomp.h"\nint helper(void) { return 0; }\n')
        from rustconv.profile_derive import _wine_config_undefs

        _wine_config_undefs.cache_clear()
        return root

    def _dead_tu(self, tmp_path: Path, body: str, **kw):
        root = self._tree_with_pinned_ver(tmp_path, body, **kw)
        dp = derive(_mod(tmp_path), root)
        return [f for f in dp.findings if f.code == "DEAD_TU"]

    def test_a_wholly_version_gated_tu_is_flagged(self, tmp_path: Path):
        """concurrency.c's shape: the entire file inside one `#if _MSVCR_VER >= 100`."""
        dead = "\n".join(f"int fn{i}(void) {{ return {i}; }}" for i in range(60))
        hits = self._dead_tu(tmp_path, f"#if _MSVCR_VER >= 100\n{dead}\n#endif\n")
        assert hits, "a 100%-dead TU must be reported"
        assert "main.c" in hits[0].detail
        assert hits[0].severity == "error", "it is BLOCKING — it is about to cost money"

    def test_one_false_conjunct_kills_the_block_despite_an_unknown_neighbour(self, tmp_path: Path):
        """handler4.c's shape. We never learn whether `__x86_64__` is defined, and short
        circuit means we do not need to: `_MSVCR_VER>=140` is false on every arch."""
        dead = "\n".join(f"int fn{i}(void) {{ return {i}; }}" for i in range(60))
        hits = self._dead_tu(
            tmp_path, f"#if defined(__x86_64__) && _MSVCR_VER>=140\n{dead}\n#endif\n"
        )
        assert hits, "a decidably-false conjunct settles the whole `&&`"

    def test_a_satisfied_gate_is_not_flagged(self, tmp_path: Path):
        """The mirror image, and the one that would hurt: reporting LIVE code as dead
        sends someone to delete a body the compiler really is building."""
        live = "\n".join(f"int fn{i}(void) {{ return {i}; }}" for i in range(60))
        assert not self._dead_tu(tmp_path, f"#if _MSVCR_VER == 0\n{live}\n#endif\n")

    def test_the_else_arm_of_a_false_gate_is_live(self, tmp_path: Path):
        live = "\n".join(f"int fn{i}(void) {{ return {i}; }}" for i in range(60))
        body = f"#if _MSVCR_VER >= 100\nint dead_one(void);\n#else\n{live}\n#endif\n"
        assert not self._dead_tu(tmp_path, body)

    def test_an_unpinned_macro_decides_nothing(self, tmp_path: Path):
        """No `-D` for it, so its value is unknown and the block must stay UNKNOWN --
        never assumed dead."""
        code = "\n".join(f"int fn{i}(void) {{ return {i}; }}" for i in range(60))
        assert not self._dead_tu(tmp_path, f"#if _SOME_OTHER_VER >= 100\n{code}\n#endif\n")


class TestIntGuardEvaluator:
    """Unit-level truth table for `_eval_int_guard`. None means UNDECIDABLE, and the
    distinction from False is the whole safety property: False deletes code."""

    @staticmethod
    def _ev(expr: str, **pinned):
        from rustconv.profile_derive import _eval_int_guard

        return _eval_int_guard(expr, pinned)

    @pytest.mark.parametrize(
        "expr,expected",
        [
            ("_MSVCR_VER >= 100", False),
            ("_MSVCR_VER == 0", True),
            ("_MSVCR_VER != 0", False),
            ("_MSVCR_VER < 100", True),
            ("_MSVCR_VER > 100", False),
            ("_MSVCR_VER <= 0", True),
            ("(_MSVCR_VER >= 100)", False),
            ("_MSVCR_VER>=100", False),
        ],
    )
    def test_single_comparisons(self, expr, expected):
        assert self._ev(expr, _MSVCR_VER=0) is expected

    def test_hex_and_suffixed_literals(self):
        assert self._ev("_WIN32_WINNT >= 0x0601", _WIN32_WINNT=0x502) is False
        assert self._ev("_MSVCR_VER >= 100L", _MSVCR_VER=0) is False

    def test_and_chain_needs_every_conjunct_true(self):
        assert self._ev("_MSVCR_VER>=70 && _MSVCR_VER<=71", _MSVCR_VER=0) is False
        assert self._ev("_MSVCR_VER>=0 && _MSVCR_VER<=71", _MSVCR_VER=0) is True

    def test_or_chain_needs_one_true(self):
        assert self._ev("_MSVCR_VER>=140 || _MSVCR_VER==0", _MSVCR_VER=0) is True
        assert self._ev("_MSVCR_VER>=140 || _MSVCR_VER==99", _MSVCR_VER=0) is False

    def test_an_or_with_an_unknown_term_is_undecidable_not_false(self):
        """`#if _MSVCR_VER>=140 || defined(__UCRTSUPPORT__)` -- except.c really has this."""
        assert self._ev("_MSVCR_VER>=140 || defined(__UCRTSUPPORT__)", _MSVCR_VER=0) is None

    def test_mixed_precedence_is_refused(self):
        """Getting C's `&&`/`||` precedence subtly wrong would mark live code dead, and no
        real gate needs it badly enough to take that risk."""
        assert self._ev("A>1 && B>1 || C>1", A=0, B=0, C=9) is None

    def test_an_unpinned_macro_is_unknown(self):
        assert self._ev("_MSVCR_VER >= 100", SOMETHING_ELSE=1) is None


# --- an already-flipped module must refuse, not report empty sections ----------------


class TestFlippedModule:
    """A cohab flip comments the real `.c` out and leaves `rust_*.c` compiled, so the TU list
    is not EMPTY — it is one shim with no switch statements in it. Every audit then printed an
    empty section that read as a pass. sysdm shipped that way while carrying the exact defect
    the case-label audit exists to catch."""

    def test_a_cohab_scaffolding_tu_is_not_a_translation_surface(self):
        for name in (
            "rust_varargs_shim.c",
            "rust_scanf_shim.c",
            "rust_strsafe_shim.c",
            "rust_cohab_fat.c",
            "rust_layout_assert.c",
        ):
            assert is_shim_tu(Path("dll/cpl/sysdm") / name), name

    def test_a_real_tu_is_not_mistaken_for_scaffolding(self):
        for name in ("sysdm.c", "environment.c", "rustle.c", "trust.c"):
            assert not is_shim_tu(Path("dll/cpl/sysdm") / name), name

    def test_a_commented_out_source_list_is_restored(self):
        text = (
            "# list(APPEND SOURCE\n"
            "#     advanced.c\n"
            "#     environment.c\n"
            "#     precomp.h)\n"
            "\n"
            "list(APPEND SOURCE\n"
            "    rust_varargs_shim.c\n"
            "    precomp.h)\n"
        )
        out, restored = uncomment_flipped_source_lists(text)
        assert restored == ["advanced.c", "environment.c", "precomp.h"]
        assert "#" not in out
        assert out.count("list(APPEND SOURCE") == 2

    def test_a_commented_out_NON_source_block_is_left_commented(self):
        """Somebody disabled `# set(FOO bar)` on purpose; resurrecting it is not our business."""
        text = "# set(WITH_DEBUG TRUE)\n# set(OPTIMIZE off)\nlist(APPEND SOURCE a.c)\n"
        out, restored = uncomment_flipped_source_lists(text)
        assert restored == [] and out == text.rstrip("\n")

    def test_an_unbalanced_commented_block_is_left_alone(self):
        """Prose that happens to open a paren must not swallow the rest of the file."""
        text = "# list(APPEND SOURCE\n#     advanced.c\n\nlist(APPEND SOURCE b.c)\n"
        _out, restored = uncomment_flipped_source_lists(text)
        assert restored == []

    def test_a_comment_that_merely_mentions_sources_is_not_a_list(self):
        text = "# these used to be here: advanced.c environment.c\nlist(APPEND SOURCE a.c)\n"
        _out, restored = uncomment_flipped_source_lists(text)
        assert restored == []

    def test_a_shim_only_module_refuses_instead_of_auditing_nothing(self, tmp_path):
        flipped = CMAKE_USERMODE.replace(
            "list(APPEND SOURCE\n    main.c\n    helper.c\n",
            "# list(APPEND SOURCE\n#     main.c\n#     helper.c)\n\nlist(APPEND SOURCE\n"
            "    rust_varargs_shim.c\n",
        )
        root = _basic_module(tmp_path, cmake=flipped)
        _write(_mod(tmp_path) / "rust_varargs_shim.c", "#include <stdarg.h>\nvoid shim(void) {}\n")
        with pytest.raises(ProfileDeriveError, match="ONLY cohab scaffolding"):
            derive(_mod(tmp_path), root, use_compile_db=False)

    def test_flipped_sources_restores_the_translation_surface_it_refused_on(self, tmp_path):
        """The point of refusing: the pre-flip surface is still IN the CMakeLists, so a flipped
        module can be re-audited without going to git. sysdm goes 1 TU / 0 switch constants →
        13 TUs / 77 this way."""
        flipped = CMAKE_USERMODE.replace(
            "list(APPEND SOURCE\n    main.c\n    helper.c\n",
            "# list(APPEND SOURCE\n#     main.c\n#     helper.c)\n\nlist(APPEND SOURCE\n"
            "    rust_varargs_shim.c\n",
        )
        root = _basic_module(tmp_path, cmake=flipped)
        _write(_mod(tmp_path) / "rust_varargs_shim.c", "#include <stdarg.h>\nvoid shim(void) {}\n")
        dp = derive(_mod(tmp_path), root, use_compile_db=False, flipped_sources=True)
        assert {p.name for p in dp.module.compiled_tus} >= {"main.c", "helper.c"}
        assert {s.ident for s in dp.switch_consts} == {"WIDGET_MODE_FAST", "WIDGET_MODE_SLOW"}
        # And it says so, because the audit now describes C the build does not compile.
        assert any(f.code == "FLIPPED_SOURCES_RESTORED" for f in dp.module.findings)


class TestBlocklistFuncsEvidence:
    """The measurement REPORTS. It does not recommend, and the calibration is why."""

    def test_the_survey_separates_declared_prototypes_from_undeclared_calls(self, tmp_path):
        root = _basic_module(
            tmp_path,
            main_extra=(
                "int use(void)\n{\n"
                "    WidgetOpen(0);\n"  # declared by a closure header -> a prototype
                "    SamrConnect(0);\n"  # declared nowhere the closure walked
                "    return helper();\n"  # defined in this module -> not external
                "}\n"
            ),
        )
        _write(
            tmp_path / "sdk/include/psdk/widget_base.h",
            "#ifndef _WIDGET_BASE_H\n#define _WIDGET_BASE_H\n"
            "typedef struct _WBASE { int x; } WBASE;\n#define WIDGET_BASE_OK 1\n"
            "int WidgetOpen(int);\n#endif\n",
        )
        dp = derive(_mod(tmp_path), root, use_compile_db=False)
        assert dp.prototype_survey is not None
        assert "WidgetOpen" in dp.prototype_survey.prototypes
        assert "SamrConnect" in dp.prototype_survey.undeclared
        assert "helper" not in dp.prototype_survey.prototypes

    def test_the_crt_and_the_macro_vocabulary_are_not_a_hand_writing_burden(self, tmp_path):
        """A Rust crate gets `wcslen` from its own substrate and `ARRAYSIZE` expands inline.
        Leaving them in moved tcpsvcs a whole band (66 -> ~44)."""
        root = _basic_module(
            tmp_path,
            main_extra="int use(void) { return wcslen(0) + ARRAYSIZE(0) + NT_SUCCESS(0); }\n",
        )
        dp = derive(_mod(tmp_path), root, use_compile_db=False)
        both = set(dp.prototype_survey.prototypes) | set(dp.prototype_survey.undeclared)
        assert both.isdisjoint({"wcslen", "ARRAYSIZE", "NT_SUCCESS"})

    def test_the_survey_never_picks_the_flag(self, tmp_path):
        """samlib measures 60 declared prototypes and chose 0; tcpsvcs measures 47 and kept 1.
        A 13-unit gap on ten samples, whose boundary pair is exactly the pair whose reasoning
        was never about the count — so there is deliberately no recommendation field."""
        root = _basic_module(tmp_path)
        dp = derive(_mod(tmp_path), root, use_compile_db=False)
        assert not hasattr(dp.prototype_survey, "recommendation")
        assert "does NOT settle BLOCKLIST_FUNCS" in dp.prototype_survey.summary
        # And the emitted profile still says 1, with the measurement beside it, never in it.
        profile = emit_profile(dp)
        assert "    BLOCKLIST_FUNCS=1" in profile
        assert "BLOCKLIST_FUNCS=0" not in profile
        assert "# BLOCKLIST_FUNCS, MEASURED:" in profile


# ---------------------------------------------------------------------------
# A per-file ERROR must not block the whole module
# ---------------------------------------------------------------------------
#
# `derive_source` is MODULE-scoped, so every finding it produces reaches the
# caller no matter which .c is being translated. A DEAD_TU finding says "do not
# pay to translate THIS TU" about one specific file — but the translate gate
# blocked on `derived.errors` wholesale, so msvcrt's two permanently-dead TUs
# (handler4.c at 97%, concurrency.c at 98%) made EVERY other file in msvcrt
# unpayable, with no waiver flag anywhere. Both findings are correct; blocking
# the wrong files on them was not.


def _result_with(findings):
    """A minimal stand-in carrying just the finding-partition API under test."""

    class _R:
        def __init__(self, fs):
            self.findings = list(fs)

        errors = DerivedProfile.errors
        blocking_errors = DerivedProfile.blocking_errors
        deferred_errors = DerivedProfile.deferred_errors

    return _R(findings)


def test_per_file_error_blocks_only_its_own_file():
    dead = Finding("error", "DEAD_TU", "concurrency.c is 98% dead", subject="msvcrt/concurrency.c")
    res = _result_with([dead])

    # Translating the dead TU itself: still blocked. This is the half that must
    # never regress — it is the finding doing its actual job.
    assert res.blocking_errors("msvcrt/concurrency.c") == [dead]
    assert res.deferred_errors("msvcrt/concurrency.c") == []

    # Translating a different TU in the same module: not blocked...
    assert res.blocking_errors("msvcrt/console.c") == []
    # ...but still REPORTED. A finding that stops blocking must not also stop
    # being visible, or the module quietly loses the warning entirely.
    assert res.deferred_errors("msvcrt/console.c") == [dead]


def test_module_wide_error_blocks_every_file():
    """`subject is None` is the module-wide case and keeps blocking everything."""
    wide = Finding("error", "SWITCH_CONST_IN_BASE_HEADER", "_PC_24 is not in the island")
    res = _result_with([wide])
    for target in ("msvcrt/console.c", "msvcrt/concurrency.c", "msvcrt/math.c"):
        assert res.blocking_errors(target) == [wide]
        assert res.deferred_errors(target) == []


def test_no_file_argument_keeps_whole_module_meaning():
    """A bare report (no file selected) still sees every error."""
    dead = Finding("error", "DEAD_TU", "x", subject="m/a.c")
    wide = Finding("error", "OTHER", "y")
    res = _result_with([dead, wide])
    assert res.blocking_errors() == [dead, wide]


def test_resolve_sys_crate_strips_every_target_role_suffix(tmp_path: Path) -> None:
    """A CMake TARGET carries a role suffix the -sys crate does not.

    msvcrt's `.c` are split across `msvcrt_shared` and `msvcrt_static`, and BOTH import
    `msvcrt-sys`. Resolving to `ntdll-sys` instead makes every constant look absent, and
    those findings are BLOCKING (§9.112) — so the gate that exists to catch silent
    catch-all `match` arms becomes one that cannot be satisfied. The first fix here
    special-cased `_static` only, which left `--target msvcrt_shared` still emitting six
    blocking findings for constants that were present in `msvcrt-sys` and correct.
    A partial fix was indistinguishable from no fix, so pin every shape.
    """
    rust = tmp_path / "sdk/lib/rust"
    for crate in ("msvcrt-sys", "ntdll-sys", "ntkrnl-sys"):
        (rust / crate / "src").mkdir(parents=True)

    for target in ("msvcrt", "msvcrt_shared", "msvcrt_static", "msvcrt_a_b_c"):
        assert resolve_sys_crate(tmp_path, target, usermode=True) == "msvcrt-sys", target

    # A path, not just a bare name.
    assert resolve_sys_crate(tmp_path, "dll/win32/msvcrt", usermode=True) == "msvcrt-sys"

    # No own island -> the mode default, and the suffix walk must not invent a crate.
    assert resolve_sys_crate(tmp_path, "shell32_static", usermode=True) == "ntdll-sys"
    assert resolve_sys_crate(tmp_path, "portcls", usermode=False) == "ntkrnl-sys"


class TestCargoKernelTarget:
    """ntoskrnl links through `add_cargo_module(... KIND kernelexe)` and flips per entry."""

    def _tree(self, tmp_path):
        root = tmp_path
        (root / "ntoskrnl/ps").mkdir(parents=True)
        (root / "ntoskrnl/ps/job.c").write_text("int PspJob(void) { return 0; }\n")
        (root / "sdk/lib/cmlib").mkdir(parents=True)
        (root / "sdk/lib/cmlib/cmlib.h").write_text("/* */\n")
        (root / "sdk/lib/cmlib/CMakeLists.txt").write_text(
            "add_library(cmlib x.c)\n"
            "target_include_directories(cmlib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})\n"
        )
        (root / "ntoskrnl/ntos.cmake").write_text(
            "list(APPEND SOURCE\n"
            "    # ps/ — replaced by Rust\n"
            "    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/job.c\n"
            "    #${REACTOS_SOURCE_DIR}/ntoskrnl/ps/gone.c\n"
            "    )\n"
        )
        (root / "ntoskrnl/CMakeLists.txt").write_text(
            "include(ntos.cmake)\n"
            "add_cargo_module(ntoskrnl ntoskrnl-cargo ntoskrnl.exe\n"
            "    KIND kernelexe\n"
            "    EXTRA_DEPENDS cmlib rtl)\n"
        )
        return root

    def test_repo_rooted_flipped_entry_is_restored_only_if_on_disk(self, tmp_path):
        root = self._tree(tmp_path)
        text = (root / "ntoskrnl/ntos.cmake").read_text()
        _out, restored = uncomment_flipped_source_lists(text, root / "ntoskrnl", root)
        assert restored == ["${REACTOS_SOURCE_DIR}/ntoskrnl/ps/job.c"]

    def test_kernelexe_is_a_kernel_target_with_linked_interface_includes(self, tmp_path):
        root = self._tree(tmp_path)
        _interface_include_index.cache_clear()
        m = parse_cmakelists(
            root / "ntoskrnl/CMakeLists.txt", root, arch="amd64", flipped_sources=True
        )
        assert m.name == "ntoskrnl" and m.module_type == "kernel"
        assert [p.name for p in m.compiled_tus] == ["job.c"]
        assert (root / "sdk/lib/cmlib") in m.include_dirs


def test_a_backslash_include_resolves_like_the_compiler_does(tmp_path):
    """win32ss/gdi/dib/dib_new.c writes `#include "..\\diblib\\DibLib_interface.h"`."""
    from rustconv.profile_derive import Include, IncludeEnv, resolve_include

    (tmp_path / "dib").mkdir()
    (tmp_path / "diblib").mkdir()
    header = tmp_path / "diblib" / "DibLib_interface.h"
    header.write_text("/* */\n")
    inc = Include(
        spelling='"..\\\\diblib\\\\DibLib_interface.h"',
        name="..\\diblib\\DibLib_interface.h",
        angled=False,
        line=11,
    )
    env = IncludeEnv(repo_root=tmp_path, arch="amd64", module_dir=tmp_path, build_dir=None)
    assert resolve_include(inc, tmp_path / "dib" / "dib_new.c", env) == header.resolve()


def test_a_packing_push_in_a_prologue_never_reaches_the_wrapper():
    # partlist.c: its prologue ends in `#include <pshpack1.h>`; the matching poppack comes after
    # two typedefs. Mirrored into setuplib's merged wrapper it packed every later header.
    src = (
        '#include "precomp.h"\n#include "registry.h"\n#include <pshpack1.h>\n'
        "typedef struct _X { char a; int b; } X;\n#include <poppack.h>\n"
    )
    lines, _ = scan_preamble(src)
    assert lines == ['#include "precomp.h"', '#include "registry.h"']


def test_dummyunionname1_is_an_anonymous_member_not_a_type(tmp_path):
    from rustconv.profile_derive import header_type_defs

    h = tmp_path / "cryptuiapi.h"
    h.write_text("typedef struct _S {\n    union {\n        int a;\n    } DUMMYUNIONNAME1;\n} S;\n")
    # inetcpl was split into two islands over this name alone.
    assert header_type_defs(h) == {"_S", "S"}
