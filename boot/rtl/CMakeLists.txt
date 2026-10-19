# blrtl, the boot loaders' RTL, is the blrtl-rust crate (sdk/lib/rust/blrtl-rust): the
# kernel's RTL rlibs plus nlsboot's port, linked by freeldr/rosload/uefildr as a cargo
# sibling, its asm assembled in-cargo from blrtl.cmake's list. No loader links this C
# target, so it only fed compile_commands.json, where scanners and coverage counts took
# its TUs for shipping code (the crt.cmake precedent, 2026-10-16). Pruned 2026-10-18.
#include(blrtl.cmake)
