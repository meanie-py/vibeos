project(SHELL)

# THE WHOLE `recyclebin` STATIC LIBRARY IS RUST NOW — sdk/lib/rust/recyclebin-rust, built and
# linked as ${RECYCLEBIN_RUST_LIB} (see the root CMakeLists.txt block next to menuband-rust,
# and shell32's target_link_libraries).
#
# This is the first WHOLE-LIBRARY-TARGET swap rather than a per-TU cohab: these six sources
# were the entire library, so there is no surviving half to keep compiling and the `recyclebin`
# target itself goes away. The parent's `add_subdirectory(shellrecyclebin)` is commented out,
# so nothing below is processed; the list is kept, commented, because it is the record of what
# the library contained, and the .c/.cpp stay on disk inert like every other cohab flip.
#
# Capture set measured at the swap (defined by librecyclebin.a AND undefined in a live
# consumer — shell32's objects plus libshell32_rust.a): SEVEN symbols, all published by the
# Rust — CloseRecycleBinHandle, DeleteFileToRecycleBinW, EmptyRecycleBinW,
# EnumerateRecycleBinW, GetDefaultRecycleBin, GetRecycleBinPathFromDriveNumber,
# RemoveFromRecycleBinDatabase. Everything else the C++ library defined was C++-mangled class
# methods (_ZN11RecycleBin5...) and ATL guard variables that never left it.
#
# add_definitions(
#     -DUNICODE
#     -D_UNICODE)
#
# list(APPEND SOURCE
#     recyclebin.c                        -> recyclebin-rust: shellrecyclebin/recyclebin.rs
#     recyclebin_generic.cpp              -> RecycleBinGeneric.rs + recyclebin_generic_free.rs
#     recyclebin_generic_enumerator.cpp   -> RecycleBinGenericEnum.rs
#                                            + recyclebin_generic_enumerator_free.rs
#     recyclebin_v5.cpp                   -> RecycleBin5.rs + recyclebin_v5_free.rs
#     recyclebin_v5_enumerator.cpp        -> RecycleBin5Enum.rs + RecycleBin5File.rs
#                                            + recyclebin_v5_enumerator_free.rs
#     recyclebin_private.h)
#
# add_library(recyclebin ${SOURCE} guid.c)   # guid.c -> recyclebin-rust: shellrecyclebin/guid.rs
# target_link_libraries(recyclebin PRIVATE atl_classes)
# add_pch(recyclebin recyclebin_private.h SOURCE)
# add_dependencies(recyclebin psdk)
