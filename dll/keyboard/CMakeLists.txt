
list(APPEND _keyboard_layouts
    kbd101
    kbd106
    kbda1
    kbda2
    kbda3
    kbdal
    kbdarme
    kbdarmw
    kbdaze
    kbdazel
    kbdbe
    kbdbga
    kbdbgt
    kbdblr
    kbdbr
    kbdbu
    kbdbur
    kbdcan
    kbdcmk
    kbdcr
    kbdcz
    kbdcz1
    kbdda
    kbddv
    kbdeo
    kbdest
    kbdfc
    kbdfi
    kbdfr
    kbdgeo
    kbdgerg
    kbdgneo
    kbdgr
    kbdgr1
    kbdgre1
    kbdgrist
    kbdhe
    kbdheb
    kbdhu
    kbdic
    kbdinasa
    kbdinben
    kbdindev
    kbdinguj
    kbdinmal
    kbdir
    kbdit
    kbdjpn
    kbdkaz
    kbdkor
    kbdla
    kbdlt1
    kbdlv
    kbdmac
    kbdne
    kbdnec
    kbdno
    kbdpl
    kbdpl1
    kbdpo
    kbdro
    kbdrost
    kbdru
    kbdru1
    kbdsf
    kbdsg
    kbdsl
    kbdsl1
    kbdsp
    kbdsw
    kbdtat
    kbdth0
    kbdth1
    kbdth2
    kbdth3
    kbdtuf
    kbdtuq
    kbduk
    kbdur
    kbdurs
    kbdus
    kbdusa
    kbdusl
    kbdusr
    kbdusx
    kbduzb
    kbdvntc
    kbdycc
    kbdycl)

# Every layout's tables are Rust now (sdk/lib/rust/kbdlayouts-rust, generated for $0 by
# `python -m rustconv.kbd_layouts` from the C, which stays on disk). Each DLL links the one
# archive; its .spec exports `<layout>_KbdLayerDescriptor` under the real name, and the linker
# takes only that layout's objects out of the archive.
rust_staticlib_variant(kbdlayouts_rust kbdlayouts-rust "" TRUE)

foreach(_keyboard_layout ${_keyboard_layouts})
    spec2def(${_keyboard_layout}.dll ${_keyboard_layout}/${_keyboard_layout}.spec)

    add_library(${_keyboard_layout} MODULE
        # ${_keyboard_layout}/${_keyboard_layout}.c  # replaced by Rust (kbdlayouts-rust/src/<layout>/)
        rust_cohab_stub.c
        ${_keyboard_layout}/${_keyboard_layout}.rc
        ${CMAKE_CURRENT_BINARY_DIR}/${_keyboard_layout}.def)
    add_dependencies(${_keyboard_layout} rust_kbdlayouts_rust)
    target_link_libraries(${_keyboard_layout} ${kbdlayouts_rust_LIB})

    set_entrypoint(${_keyboard_layout} 0)
    set_image_base(${_keyboard_layout} 0x5FFF0000)
    set_subsystem(${_keyboard_layout} native)

    if(USE_LLVM_MINGW)
        # LLD in MinGW PE mode does not support -T linker scripts; use /MERGE directives instead
        # Use SHELL: prefix to prevent CMake from splitting on commas in /SECTION:.data,ER
        target_link_options(${_keyboard_layout} PRIVATE
            "LINKER:/MERGE:.text=.data"
            "LINKER:/MERGE:.rdata=.data"
            "LINKER:/MERGE:.bss=.data"
            "LINKER:/MERGE:.edata=.data"
            "SHELL:-Xlinker /SECTION:.data,ER")
    else()
        # Use a custom linker script
        target_link_options(${_keyboard_layout} PRIVATE "-Wl,-T,${CMAKE_CURRENT_SOURCE_DIR}/kbdlayout.lds")
    endif()

    # dynamic analysis switches
    if(STACK_PROTECTOR)
        target_compile_options(${_keyboard_layout} PRIVATE -fno-stack-protector)
    endif()

    if(RUNTIME_CHECKS)
        target_link_libraries(${_keyboard_layout} runtmchk)
    endif()

    add_dependencies(${_keyboard_layout} xdk)
    add_cd_file(TARGET ${_keyboard_layout} DESTINATION reactos/system32 FOR all)
endforeach()
