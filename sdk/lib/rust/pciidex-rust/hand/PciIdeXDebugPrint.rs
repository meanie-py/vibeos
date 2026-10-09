// HAND-PORTED (miniport.c PciIdeXDebugPrint): a C-variadic definition is `c_variadic`, which
// stable Rust does not have. The exported variadic symbol is a 3-line trampoline in
// drivers/storage/ide/pciidex/rust_varargs_shim.c; it hands this fixed-arity twin a POINTER
// to its va_list, and the level filter -- the only policy -- stays here. vDbgPrintEx takes the
// va_list by value, so the shim's vibeos_vDbgPrintEx dereferences the pointer for it.
extern "C" {
    fn vibeos_vDbgPrintEx(ComponentId: ULONG, Level: ULONG, Format: PCCH, ap: *mut c_void) -> ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn PciIdeXDebugPrint_v(
    mut DebugPrintLevel: ULONG,
    DebugMessage: PCCHAR,
    ap: *mut c_void,
) {
    /* Check if we can print anything */
    if DebugPrintLevel <= PciIdeDebug {
        DebugPrintLevel = 0;
    }
    vibeos_vDbgPrintEx(DPFLTR_PCIIDE_ID as ULONG, DebugPrintLevel, DebugMessage as PCCH, ap);
}
