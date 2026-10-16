"""c_oracle: a flipped module's pre-flip C as an opt-in `<target>_c`, for runtime diffs."""

import pytest

from rustconv import c_oracle as co

_CML = """\
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/fmifs)

list(APPEND SOURCE
    # active.c  # replaced by Rust (sdk/lib/rust/x-rust/src/x/active.rs)
    # help.c  # replaced by Rust (sdk/lib/rust/x-rust/src/x/help.rs)
    rust_cohab_stub.c
    x.h)

rust_staticlib_variant(x_rust x-rust "" TRUE)
add_executable(x ${SOURCE} x.rc)
add_dependencies(x rust_x_rust)
target_link_libraries(x ${x_rust_LIB})
add_pch(x x.h SOURCE)
if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(x PRIVATE "-Wno-foo")
endif()
if(ARCH STREQUAL "i386")
    add_cd_file(TARGET x DESTINATION reactos/system32 FOR all)
endif()
set_module_type(x win32cui UNICODE)
target_link_libraries(x conutils ${PSEH_LIB})
add_importlibs(x msvcrt kernel32 ntdll)
set_wine_module_FIXME(x)
add_library(x_static STATIC helper.c)
add_executable(user EXCLUDE_FROM_ALL user.c)
target_link_libraries(user x)
add_cd_file(TARGET x DESTINATION reactos/system32 FOR all)
"""


def test_the_oracle_is_the_pre_flip_c_with_the_rust_dropped():
    out = co.generate(_CML)
    # The comment's own `)` must not end the source list early.
    assert "set(SOURCE_C_ORACLE\n    active.c\n    help.c\n    x.h\n)" in out
    assert "add_executable(x_c EXCLUDE_FROM_ALL ${SOURCE_C_ORACLE} x.rc)" in out
    assert "set_module_type(x_c win32cui UNICODE)" in out
    assert "target_link_libraries(x_c conutils ${PSEH_LIB})" in out
    assert "add_importlibs(x_c msvcrt kernel32 ntdll)" in out
    # A command naming the target alone configures it (Wine's defines, ARRAY_SIZE).
    assert "set_wine_module_fixme(x_c)" in out  # CMake names are case-insensitive
    for gone in ("rust", "add_pch", "add_cd_file", "rust_cohab_stub.c"):
        assert gone not in out.split("never shipped.")[1]
    # A conditional that configured the target keeps its condition; one left empty goes.
    assert 'if(CMAKE_C_COMPILER_ID STREQUAL "Clang")\ntarget_compile_options(x_c' in out
    assert "ARCH" not in out
    # A target that links the flipped one gets a C twin too, and keeps its own flag once.
    assert "add_executable(user_c EXCLUDE_FROM_ALL user.c)" in out
    assert "target_link_libraries(user_c x_c)" in out
    # Unrelated targets are not cloned.
    assert "x_static" not in out


def test_an_unflipped_module_has_no_oracle():
    with pytest.raises(co.OracleError, match="nothing flipped"):
        co.generate("add_executable(x x.c)\n")


def test_write_includes_once_and_check_sees_staleness(tmp_path):
    (tmp_path / "CMakeLists.txt").write_text(_CML)
    assert co.write(tmp_path) is False
    assert co.write(tmp_path) is True  # current now
    assert (tmp_path / "CMakeLists.txt").read_text().count("include(c_oracle.cmake)") == 1
    (tmp_path / "CMakeLists.txt").write_text(
        (tmp_path / "CMakeLists.txt").read_text().replace("conutils", "conutils2")
    )
    assert co.write(tmp_path, check=True) is False


def test_an_inline_flipped_executable_gets_its_oracle():
    # `port flip` writes a single-list module's TUs inside add_executable (find, and every
    # batch C/D1 tool); only list(APPEND ..) was understood, so none could be oracled.
    cml = (
        'rust_staticlib_variant(find_rust find-rust "" TRUE)\n'
        "add_executable(find\n"
        "    # find.c  # replaced by Rust (sdk/lib/rust/find-rust/src/find/find.rs)\n"
        "    rust_cohab_stub.c\n"
        "    find.rc)\n"
        "add_dependencies(find rust_find_rust)\n"
        "target_link_libraries(find ${find_rust_LIB})\n"
        "set_module_type(find win32cui UNICODE)\n"
        "add_importlibs(find msvcrt kernel32)\n"
    )
    out = co.generate(cml)
    assert "add_executable(find_c EXCLUDE_FROM_ALL find.c find.rc)" in out
    assert "rust" not in out.split("never shipped.")[1].lower()


def test_the_varargs_shim_stays_out_of_the_oracle():
    # netsh: the flip's rust_varargs_shim.c defines PrintError etc. as trampolines into the
    # Rust; in the C oracle they collide with netsh.c's own definitions.
    cml = (
        "list(APPEND SOURCE\n"
        "    # netsh.c  # replaced by Rust (x)\n"
        "    rust_varargs_shim.c\n"
        "    rust_cohab_stub.c)\n"
        "add_executable(netsh ${SOURCE})\n"
    )
    assert "rust_" not in co.generate(cml)


def test_an_oracle_output_name_is_suffixed_too():
    # cmd_help's OUTPUT_NAME "help": the copied property made two rules for help.exe.
    cml = (
        "add_executable(cmd_help\n    # help.c  # replaced by Rust (x)\n    rust_cohab_stub.c)\n"
        'set_target_properties(cmd_help PROPERTIES OUTPUT_NAME "help")\n'
    )
    assert 'set_target_properties(cmd_help_c PROPERTIES OUTPUT_NAME "help_c")' in co.generate(cml)


def test_a_win32_executable_keeps_its_keyword_ahead_of_the_sources():
    from rustconv.c_oracle import generate

    cml = (
        "add_executable(testgui WIN32\n"
        "    # testgui.c  # replaced by Rust (sdk/lib/rust/testgui-rust/src/testgui/testgui.rs)\n"
        "    rust_cohab_stub.c\n    )\n"
        "set_module_type(testgui win32gui UNICODE)\n"
    )
    out = generate(cml)
    assert "add_executable(testgui_c WIN32 EXCLUDE_FROM_ALL testgui.c)" in out, out
