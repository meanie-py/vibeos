
spec2def(kdvbox.dll kdvm.spec)

# ============================================================================
# kdvbox.dll (VirtualBox KD transport) — cargo-driven final link
# ============================================================================
# kdvm.c / kdvbox.c (+ kdvbox_asm.S) are translated to Rust
# (sdk/lib/rust/kdvm-rust, kept on disk per the never-delete rule). The VBox
# backdoor port-I/O `KdVmExchange` (kdvbox_asm.S) is rendered as a cfg-gated
# in-crate inline-asm fn — no asm archive needed. The 8 kdvm.spec exports come
# from libkdvm_rust.a; the kdvbox.dll link is driven by cargo (kdvm-cargo's
# build.rs -> reactos-link) linking kdvm-rust + libcntpr (CRT) + memcmp + the
# ntoskrnl/hal import libs. Mirrors the kdrosdbg cohab. Only added under
# _WINKD_ && !GDB (see drivers/base/CMakeLists.txt). i386 + amd64.
#
#   kdvm.c kdvbox.c kdvbox_asm.S  -- replaced by Rust (kdvm-rust); kept on disk.
# DESTINATION: txtsetup.sif copies kdvbox.dll, so setup needs it on the CD.
add_cargo_kernel_dll(kdvbox kdvm-cargo kdvbox.dll BODY_CRATE kdvm-rust
    DESTINATION reactos/system32
    EXTRA_DEPENDS libcntpr memcmp)
