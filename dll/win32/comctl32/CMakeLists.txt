
add_definitions(
    -D_WINE
    -D__ROS_LONG64__
    -D_COMCTL32_)

remove_definitions(-D_WIN32_WINNT=0x502 -DWINVER=0x502)
add_definitions(-D_WIN32_WINNT=0x600 -DWINVER=0x600)

spec2def(comctl32.dll comctl32.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # animate.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/animate.rs)
    # button.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/button.rs)
    # combo.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/combo.rs)
    # comboex.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/comboex.rs)
    # comctl32undoc.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/comctl32undoc.rs)
    # commctrl.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/commctrl.rs)
    # datetime.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/datetime.rs)
    # dpa.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/dpa.rs)
    # draglist.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/draglist.rs)
    # dsa.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/dsa.rs)
    # edit.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/edit.rs)
    # flatsb.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/flatsb.rs)
    # header.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/header.rs)
    # hotkey.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/hotkey.rs)
    # imagelist.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/imagelist.rs)
    # ipaddress.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/ipaddress.rs)
    # listbox.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/listbox.rs)
    # listview.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/listview.rs)
    # monthcal.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/monthcal.rs)
    # nativefont.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/nativefont.rs)
    # pager.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/pager.rs)
    # progress.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/progress.rs)
    # propsheet.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/propsheet.rs)
    # rebar.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/rebar.rs)
    # smoothscroll.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/smoothscroll.rs)
    # static.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/static.rs)
    # status.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/status.rs)
    # string.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/string.rs)
    # syslink.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/syslink.rs)
    # tab.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/tab.rs)
    # taskdialog.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/taskdialog.rs)
    # theme_scrollbar.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/theme_scrollbar.rs)
    # theming.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/theming.rs)
    # toolbar.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/toolbar.rs)
    # tooltips.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/tooltips.rs)
    # trackbar.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/trackbar.rs)
    # treeview.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/treeview.rs)
    # updown.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/updown.rs)
    )

list(APPEND PCH_SKIP_SOURCE
rust_cohab_stub.c
    # stubs.c  # replaced by Rust (sdk/lib/rust/comctl32-rust/src/comctl32/stubs.rs)
    )

# Every translation unit is Rust now (comctl32-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(comctl32_rust comctl32-rust "" TRUE)
add_library(comctl32 MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    rsrc.rc
    ${CMAKE_CURRENT_BINARY_DIR}/comctl32.def)
add_dependencies(comctl32 rust_comctl32_rust)
target_link_libraries(comctl32 ${comctl32_rust_LIB})

set_module_type(comctl32 win32dll UNICODE)
target_link_libraries(comctl32 uuid wine ${PSEH_LIB} oldnames)
add_delay_importlibs(comctl32 winmm uxtheme)
add_importlibs(comctl32 user32 gdi32 advapi32 usp10 imm32 msvcrt kernel32 ntdll)
add_pch(comctl32 precomp.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET comctl32 DESTINATION reactos/system32 FOR all)

add_cd_file(TARGET comctl32 DESTINATION reactos/winsxs/${WINARCH}_microsoft.windows.common-controls_6595b64144ccf1df_5.82.2600.2982_none_deadbeef FOR all)
add_cd_file(TARGET comctl32 DESTINATION reactos/winsxs/${WINARCH}_microsoft.windows.common-controls_6595b64144ccf1df_6.0.2600.2982_none_deadbeef FOR all)
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/${WINARCH}_microsoft.windows.common-controls_6595b64144ccf1df_5.82.2600.2982_none_deadbeef.manifest DESTINATION reactos/winsxs/manifests FOR all)
add_cd_file(FILE ${CMAKE_CURRENT_SOURCE_DIR}/${WINARCH}_microsoft.windows.common-controls_6595b64144ccf1df_6.0.2600.2982_none_deadbeef.manifest DESTINATION reactos/winsxs/manifests FOR all)
set_wine_module_FIXME(comctl32) # CORE-5743: No ARRAY_SIZE macro
