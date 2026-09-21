
list(APPEND UCRT_STDLIB_SOURCES
    stdlib/abs.cpp
    stdlib/bsearch.cpp
    stdlib/bsearch_s.cpp
    stdlib/div.cpp
    stdlib/imaxabs.cpp
    stdlib/imaxdiv.cpp
    stdlib/labs.cpp
    stdlib/ldiv.cpp
    stdlib/lfind.cpp
    stdlib/lfind_s.cpp
    stdlib/llabs.cpp
    stdlib/lldiv.cpp
    stdlib/lsearch.cpp
    stdlib/lsearch_s.cpp
    stdlib/qsort.cpp
    stdlib/qsort_s.cpp
    stdlib/rand.cpp
    stdlib/rand_s.cpp
)

# rotl.cpp / rotr.cpp / byteswap.cpp and the clang-hacks.s alias shim were
# DELETED 2026-09-20: all 9 exports are Rust now (sdk/lib/rust/ucrt-rust,
# src/bits.rs), declared in tools/ucrt-takeover.yaml. The .s existed only
# because clang treats _rotl and friends as builtins, so the C could not define
# them and had to be renamed (-D_rotl=___rotl, also gone) with assembly jmps
# recreating the real symbols. Rust has no builtin to collide with. Side effect
# worth noting: the .s was i386/amd64 only while the renames were not, so arm64
# had no _rotl at all -- it does now.
