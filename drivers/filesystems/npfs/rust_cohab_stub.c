/* Cohabitation placeholder TU for the npfs kernelmodedriver MODULE.
 *
 * The Named Pipe filesystem (DriverEntry + the IRP dispatch + fast-I/O paths + the VCB/DCB/
 * FCB/CCB node graph, NP_DATA_QUEUE engine, pipe state machine, wait queue and client
 * security contexts across the 20 .c files cleanup/close/create/datasup/fileinfo/fileobsup/
 * flushbuf/fsctrl/main/prefxsup/read/readsup/secursup/seinfo/statesup/strucsup/volinfo/
 * waitsup/write/writesup) is translated to Rust (sdk/lib/rust/npfs-rust, linked as
 * libnpfs_rust.a). The 20 .c bodies are removed from the add_library SOURCES and kept on
 * disk per the never-delete rule. DriverEntry + the whole driver come from the linked Rust
 * staticlib; the Io/Ke/Ex/Mm/Ob/Se/FsRtl/Rtl entry points stay ntoskrnl imports
 * (add_importlibs(npfs ntoskrnl hal)). PSEH2 is dead under USE_DUMMY_PSEH, so the Rust
 * translation contains no SEH and the MODULE no longer links the PSEH static lib. npfs has
 * no GUID data TU, so this stub is the only C source the MODULE needs. npfs is NOT
 * boot-loaded (no 'x' in txtsetup.sif), so the bar is link-clean x3 + no smoke regression.
 */
typedef int npfs_rust_cohab_placeholder;
