
add_subdirectory(csr)
# NTVDM / MVDM (16-bit DOS/Win virtual DOS machine) dropped: pre-Vista
# platform support. Demand-only (launched only when a 16-bit binary runs),
# never on the boot/desktop path. 64-bit Windows has no NTVDM. The kernel-side
# VME init (KeI386VdmInitialize) stays — it's in ntoskrnl-rust and harmless.
add_subdirectory(win)
