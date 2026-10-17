list(APPEND _keyboard_layouts
    kbd101
    kbd101a
    kbd103
    kbd106
    kbd106n
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
    kbdfo
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
    kbdnec95
    kbdnecat
    kbdnecnt
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

# Every layout's tables are Rust (sdk/lib/rust/kbdlayouts-rust, generated for $0 by
# `python -m rustconv.kbd_layouts` from the C, which stays on disk, uncompiled), and
# cargo links all 95 DLLs from one FAMILY cell: sdk/lib/rust/kbdlayouts-cargo, one bin
# per layout. Each layout's .spec exports `<layout>_KbdLayerDescriptor` from the shared
# body under the real name, and the linker takes only that layout's objects from it.
# Each layout's .rc is compiled by the engine with cmake's windres rule (Linker::rc).

# The layouts' C compile environment, kept for `python -m rustconv.kbd_layouts`: it ports a
# layout from clang's AST of its .c, which no target compiles any more, so it borrows this
# TU's command (the directory's flags, as the old per-layout targets had). Never linked.
add_library(kbdlayouts_flags OBJECT EXCLUDE_FROM_ALL rust_cohab_stub.c)
add_dependencies(kbdlayouts_flags xdk)

add_cargo_module(kbdlayouts kbdlayouts-cargo .dll
    KIND win32dll
    BINS ${_keyboard_layouts}
    BODY_CRATE kbdlayouts-rust
    DESTINATION reactos/system32
    EXTRA_DEPENDS xdk psdk)
