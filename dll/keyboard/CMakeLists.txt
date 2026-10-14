
list(APPEND _keyboard_layouts
    kbd101
    kbd101a
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
# The resources are the one thing cmake still compiles, in this OBJECT library whose
# windres commands the cargo links replay.
set(_keyboard_rcs)
foreach(_keyboard_layout ${_keyboard_layouts})
    list(APPEND _keyboard_rcs ${_keyboard_layout}/${_keyboard_layout}.rc)
endforeach()
add_library(kbdlayouts_res OBJECT ${_keyboard_rcs})
add_dependencies(kbdlayouts_res xdk)

add_cargo_module(kbdlayouts kbdlayouts-cargo .dll
    KIND win32dll
    BINS ${_keyboard_layouts}
    BODY_CRATE kbdlayouts-rust
    DESTINATION reactos/system32
    EXTRA_DEPENDS kbdlayouts_res)
