use ntdll_sys::nt::*;
use core::ffi::{c_void, c_char};
use crate::types::*;

extern "C" {
}

// ---- file-scope type preamble (translated) ----
type cab_UBYTE = u8;
const CAB_BLOCKMAX: u32 = 32768;
const CAB_INPUTMAX: u32 = 32768 + 6144;
type cab_UWORD = u16;
type cab_ULONG = u32;
type cab_LONG = i32;
type cab_off_t = u32;

#[repr(C)]
struct list {
    next: *mut list,
    prev: *mut list,
}

pub const FCI_INT_MAGIC: u32 = 0xfcfcfc05;

#[repr(C)]
pub struct CFHEADER {
    pub signature: [cab_UBYTE; 4],
    pub reserved1: cab_ULONG,
    pub cbCabinet: cab_ULONG,
    pub reserved2: cab_ULONG,
    pub coffFiles: cab_ULONG,
    pub reserved3: cab_ULONG,
    pub versionMinor: cab_UBYTE,
    pub versionMajor: cab_UBYTE,
    pub cFolders: cab_UWORD,
    pub cFiles: cab_UWORD,
    pub flags: cab_UWORD,
    pub setID: cab_UWORD,
    pub iCabinet: cab_UWORD,
}

#[repr(C)]
pub struct CFFOLDER {
    pub coffCabStart: cab_ULONG,
    pub cCFData: cab_UWORD,
    pub typeCompress: cab_UWORD,
}

#[repr(C)]
pub struct CFFILE {
    pub cbFile: cab_ULONG,
    pub uoffFolderStart: cab_ULONG,
    pub iFolder: cab_UWORD,
    pub date: cab_UWORD,
    pub time: cab_UWORD,
    pub attribs: cab_UWORD,
}

#[repr(C)]
pub struct CFDATA {
    pub csum: cab_ULONG,
    pub cbData: cab_UWORD,
    pub cbUncomp: cab_UWORD,
}

#[repr(C)]
pub struct temp_file {
    pub handle: INT_PTR,
    pub name: [i8; CB_MAX_FILENAME as usize],
}

#[repr(C)]
pub struct folder {
    pub entry: list,
    pub files_list: list,
    pub blocks_list: list,
    pub data: temp_file,
    pub data_start: cab_ULONG,
    pub data_count: cab_UWORD,
    pub compression: TCOMP,
}

#[repr(C)]
pub struct file {
    pub entry: list,
    pub size: cab_ULONG,
    pub offset: cab_ULONG,
    pub folder: cab_UWORD,
    pub date: cab_UWORD,
    pub time: cab_UWORD,
    pub attribs: cab_UWORD,
    pub name: [i8; 1],
}

#[repr(C)]
pub struct data_block {
    pub entry: list,
    pub compressed: cab_UWORD,
    pub uncompressed: cab_UWORD,
}

#[repr(C)]
pub struct FCI_Int {
    pub magic: u32,
    pub perf: PERF,
    pub fileplaced: PFNFCIFILEPLACED,
    pub alloc: PFNFCIALLOC,
    pub free: PFNFCIFREE,
    pub open: PFNFCIOPEN,
    pub read: PFNFCIREAD,
    pub write: PFNFCIWRITE,
    pub close: PFNFCICLOSE,
    pub seek: PFNFCISEEK,
    pub delete: PFNFCIDELETE,
    pub gettemp: PFNFCIGETTEMPFILE,
    pub ccab: CCAB,
    pub pccab: PCCAB,
    pub fPrevCab: BOOL,
    pub fNextCab: BOOL,
    pub fSplitFolder: BOOL,
    pub statusFolderCopied: cab_ULONG,
    pub statusFolderTotal: cab_ULONG,
    pub fGetNextCabInVain: BOOL,
    pub pv: *mut core::ffi::c_void,
    pub szPrevCab: [i8; CB_MAX_CABINET_NAME],
    pub szPrevDisk: [i8; CB_MAX_DISK_NAME],
    pub data_in: [u8; CAB_BLOCKMAX as usize],
    pub data_out: [u8; 2 * CAB_BLOCKMAX as usize],
    pub cdata_in: cab_UWORD,
    pub cCompressedBytesInFolder: ULONG,
    pub cFolders: cab_UWORD,
    pub cFiles: cab_UWORD,
    pub cDataBlocks: cab_ULONG,
    pub cbFileRemainder: cab_ULONG,
    pub data: temp_file,
    pub fNewPrevious: BOOL,
    pub estimatedCabinetSize: cab_ULONG,
    pub folders_list: list,
    pub files_list: list,
    pub blocks_list: list,
    pub folders_size: cab_ULONG,
    pub files_size: cab_ULONG,
    pub placed_files_size: cab_ULONG,
    pub pending_data_size: cab_ULONG,
    pub folders_data_size: cab_ULONG,
    pub compression: TCOMP,
    pub compress: Option<unsafe extern "C" fn(*mut FCI_Int) -> cab_UWORD>,
}

// ---- consts / statics / type aliases (deduped) ----
const CB_MAX_CABINET_NAME: usize = 256;
const CB_MAX_DISK_NAME: usize = 256;
const statusFolder: u32 = 1;
const CB_MAX_FILENAME: i32 = 256;
const Z_DEFAULT_COMPRESSION: i32 = -1;
const Z_DEFLATED: i32 = 8;
const Z_DEFAULT_STRATEGY: i32 = 0;
const Z_FINISH: i32 = 4;
const Z_OK: i32 = 0;
// Declarations for unknown external symbols used in this function
// Declare callback type for status (best-effort signature matching the call)
type PFNFCISTATUS = unsafe extern "system" fn(
    status_type: u32,
    copied: u32,
    total: u32,
    pv: *mut core::ffi::c_void,
) -> i32;
// Constants not provided in context
const SEEK_SET: i32 = 0;
const SEEK_CUR: i32 = 1;
const statusFile: i32 = 0;
const cffileCONTINUED_FROM_PREV: u16 = 0xFFFD;
const cffileCONTINUED_TO_NEXT: u16 = 0xFFFE;
const cffileCONTINUED_PREV_AND_NEXT: u16 = 0xFFFF;
const CB_MAX_CAB_PATH: usize = 256;
const cfheadPREV_CABINET: u32 = 0x0001;
const cfheadNEXT_CABINET: u32 = 0x0002;
const cfheadRESERVE_PRESENT: u32 = 0x0004;
const _O_RDWR: i32 = 0x0002;
const _O_CREAT: i32 = 0x0100;
const _O_TRUNC: i32 = 0x0200;
const _O_BINARY: i32 = 0x8000;
const _S_IREAD: i32 = 0x0100;
const _S_IWRITE: i32 = 0x0080;
const statusCabinet: u32 = 0x0003;
// TCOMP (USHORT), like the field it is compared with and stored into. It was u32 and nobody
// noticed because its only use sat behind a cfg that was never set.
const tcompTYPE_MSZIP: TCOMP = 0x00000001;
const tcompTYPE_NONE: u32 = 0x00000000;
// Unknown externs
// Compress function pointers (defined elsewhere in the file)

extern "C" {
    fn strlen(s: *const core::ffi::c_char) -> usize;
    fn list_remove(entry: *mut list);
    fn list_init(list: *mut list);
    fn list_add_tail(head: *mut core::ffi::c_void, entry: *mut core::ffi::c_void);
    // NB: the C source calls the `deflateInit2(...)` MACRO, which expands to
    // deflateInit2_(..., ZLIB_VERSION, sizeof(z_stream)). The original
    // transliteration declared a 6-arg `deflateInit2` symbol that does NOT
    // exist in zlib (no real fn — it's a macro), so it stayed UNRESOLVED —
    // tolerated only because the link is -shared and nothing exercises the
    // compress path at boot. Declare the real exported entry point and expand
    // the macro at the call site so MSZIP compression actually works (zlib-rs).
    fn deflateInit2_(strm: *mut z_stream, level: i32, method: i32, windowBits: i32, memLevel: i32, strategy: i32, version: *const core::ffi::c_char, stream_size: i32) -> i32;
    fn deflate(strm: *mut z_stream, flush: i32) -> i32;
    fn deflateEnd(strm: *mut z_stream) -> i32;
    fn zlibVersion() -> *const core::ffi::c_char;
    fn list_add_before(pos: *mut list, entry: *mut list);
    fn memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
    fn memset(ptr: *mut core::ffi::c_void, val: i32, n: usize) -> *mut core::ffi::c_void;
    fn strcpy(dst: *mut i8, src: *const i8) -> *mut i8;
    fn strcat(dst: *mut i8, src: *const i8) -> *mut i8;
}

extern "system" {
    fn SetLastError(code: u32);
    fn lstrcpynA(dst: *mut i8, src: *const i8, max_len: i32) -> *mut i8;
    fn list_empty(head: *const list) -> BOOL;
}

unsafe fn set_error(fci: *mut FCI_Int, oper: i32, err: i32) {
    (*(*fci).perf).erfOper = oper;
    (*(*fci).perf).erfType = err;
    (*(*fci).perf).fError = 1;
    if err != 0 {
        SetLastError(err as u32);
    }
}

unsafe fn get_fci_ptr(hfci: *mut FCI_Int) -> *mut FCI_Int {
    let fci = hfci as *mut FCI_Int;
    if fci.is_null() || (*fci).magic != 0xfcfcfc05u32 /* FCI_INT_MAGIC */ {
        SetLastError(ERROR_INVALID_HANDLE);
        return core::ptr::null_mut();
    }
    fci
}

unsafe fn get_header_size(fci: *mut FCI_Int) -> u32 {
    let mut ret: u32 = core::mem::size_of::<CFHEADER>() as u32 + (*fci).ccab.cbReserveCFHeader as u32;

    if (*fci).ccab.cbReserveCFHeader != 0 || (*fci).ccab.cbReserveCFFolder != 0 || (*fci).ccab.cbReserveCFData != 0 {
        ret += 4;
    }

    if (*fci).fPrevCab != 0 {
        ret += (strlen((*fci).szPrevCab.as_ptr()) as u32) + 1 + (strlen((*fci).szPrevDisk.as_ptr()) as u32) + 1;
    }

    if (*fci).fNextCab != 0 {
        ret += (strlen((*(*fci).pccab).szCab.as_ptr()) as u32) + 1 + (strlen((*(*fci).pccab).szDisk.as_ptr()) as u32) + 1;
    }

    ret
}

unsafe fn free_file(fci: *mut FCI_Int, file: *mut file) {
    list_remove(core::ptr::addr_of_mut!((*file).entry));
    ((*fci).free)(file as *mut core::ffi::c_void);
}

unsafe fn free_data_block(fci: *mut FCI_Int, block: *mut data_block) {
    list_remove(core::ptr::addr_of_mut!((*block).entry));
    ((*fci).free)(block as *mut core::ffi::c_void);
}

unsafe fn fci_get_checksum(pv: *const core::ffi::c_void, cb: u32, seed: u32) -> u32 {
    let mut csum: u32;
    let mut ul: u32;
    let mut cUlong: i32;
    let mut pb: *const u8;

    csum = seed;
    cUlong = (cb / 4) as i32;
    pb = pv as *const u8;

    while cUlong > 0 {
        cUlong -= 1;
        let b0 = *pb;
        pb = pb.add(1);
        let b1 = *pb;
        pb = pb.add(1);
        let b2 = *pb;
        pb = pb.add(1);
        let b3 = *pb;
        pb = pb.add(1);
        ul = (b0 as u32) | ((b1 as u32) << 8) | ((b2 as u32) << 16) | ((b3 as u32) << 24);
        csum ^= ul;
    }

    ul = 0;
    match cb % 4 {
        3 => {
            ul |= (*pb as u32) << 16;
            pb = pb.add(1);
            /* fall through */
            ul |= (*pb as u32) << 8;
            pb = pb.add(1);
            /* fall through */
            ul |= *pb as u32;
        }
        2 => {
            ul |= (*pb as u32) << 8;
            pb = pb.add(1);
            /* fall through */
            ul |= *pb as u32;
        }
        1 => {
            ul |= *pb as u32;
        }
        _ => {}
    }
    csum ^= ul;
    return csum;
}

unsafe extern "C" fn compress_NONE(fci: *mut FCI_Int) -> cab_UWORD {
    memcpy(
        core::ptr::addr_of_mut!((*fci).data_out) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*fci).data_in) as *const core::ffi::c_void,
        (*fci).cdata_in as usize,
    );
    (*fci).cdata_in
}

unsafe extern "C" fn zalloc(opaque: *mut core::ffi::c_void, items: u32, size: u32) -> *mut core::ffi::c_void {
    let fci = opaque as *mut FCI_Int;
    ((*fci).alloc)(items.wrapping_mul(size) as u32)
}

unsafe extern "C" fn zfree(opaque: *mut core::ffi::c_void, ptr: *mut core::ffi::c_void) {
    let fci = opaque as *mut FCI_Int;
    ((*fci).free)(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn FCICreate(
    perf: *mut ERF,
    pfnfiledest: PFNFCIFILEPLACED,
    pfnalloc: PFNFCIALLOC,
    pfnfree: PFNFCIFREE,
    pfnopen: PFNFCIOPEN,
    pfnread: PFNFCIREAD,
    pfnwrite: PFNFCIWRITE,
    pfnclose: PFNFCICLOSE,
    pfnseek: PFNFCISEEK,
    pfndelete: PFNFCIDELETE,
    pfnfcigtf: PFNFCIGETTEMPFILE,
    pccab: *const CCAB,
    pv: *mut core::ffi::c_void,
) -> *mut core::ffi::c_void {
    let p_fci_internal: *mut FCI_Int;

    if perf.is_null() {
        SetLastError(ERROR_BAD_ARGUMENTS as u32);
        return core::ptr::null_mut();
    }
    if (pfnalloc as *const core::ffi::c_void).is_null()
        || (pfnfree as *const core::ffi::c_void).is_null()
        || (pfnopen as *const core::ffi::c_void).is_null()
        || (pfnread as *const core::ffi::c_void).is_null()
        || (pfnwrite as *const core::ffi::c_void).is_null()
        || (pfnclose as *const core::ffi::c_void).is_null()
        || (pfnseek as *const core::ffi::c_void).is_null()
        || (pfndelete as *const core::ffi::c_void).is_null()
        || (pfnfcigtf as *const core::ffi::c_void).is_null()
        || pccab.is_null()
    {
        (*perf).erfOper = FCIERR_NONE;
        (*perf).erfType = ERROR_BAD_ARGUMENTS as i32;
        (*perf).fError = 1;
        SetLastError(ERROR_BAD_ARGUMENTS as u32);
        return core::ptr::null_mut();
    }

    p_fci_internal = pfnalloc(core::mem::size_of::<FCI_Int>() as u32) as *mut FCI_Int;
    if p_fci_internal.is_null() {
        (*perf).erfOper = FCIERR_ALLOC_FAIL;
        (*perf).erfType = ERROR_NOT_ENOUGH_MEMORY as i32;
        (*perf).fError = 1;
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    core::ptr::write_bytes(p_fci_internal as *mut u8, 0, core::mem::size_of::<FCI_Int>());
    (*p_fci_internal).magic = FCI_INT_MAGIC;
    (*p_fci_internal).perf = perf;
    (*p_fci_internal).fileplaced = pfnfiledest;
    (*p_fci_internal).alloc = pfnalloc;
    (*p_fci_internal).free = pfnfree;
    (*p_fci_internal).open = pfnopen;
    (*p_fci_internal).read = pfnread;
    (*p_fci_internal).write = pfnwrite;
    (*p_fci_internal).close = pfnclose;
    (*p_fci_internal).seek = pfnseek;
    (*p_fci_internal).delete = pfndelete;
    (*p_fci_internal).gettemp = pfnfcigtf;
    (*p_fci_internal).ccab = core::ptr::read(pccab);
    (*p_fci_internal).pccab = pccab;
    (*p_fci_internal).pv = pv;
    (*p_fci_internal).data.handle = -1;
    (*p_fci_internal).compress = Some(compress_NONE);

    list_init(core::ptr::addr_of_mut!((*p_fci_internal).folders_list));
    list_init(core::ptr::addr_of_mut!((*p_fci_internal).files_list));
    list_init(core::ptr::addr_of_mut!((*p_fci_internal).blocks_list));

    core::ptr::copy_nonoverlapping(
        (*pccab).szCab.as_ptr() as *const u8,
        (*p_fci_internal).szPrevCab.as_mut_ptr() as *mut u8,
        CB_MAX_CABINET_NAME,
    );
    core::ptr::copy_nonoverlapping(
        (*pccab).szDisk.as_ptr() as *const u8,
        (*p_fci_internal).szPrevDisk.as_mut_ptr() as *mut u8,
        CB_MAX_DISK_NAME,
    );

    p_fci_internal as *mut core::ffi::c_void
}

unsafe extern "C" fn create_temp_file(fci: *mut FCI_Int, file: *mut temp_file) -> i32 {
    let mut err: i32 = 0;
    if ((*fci).gettemp)((*file).name.as_mut_ptr(), CB_MAX_FILENAME, (*fci).pv) == 0 {
        set_error(fci, FCIERR_TEMP_FILE, ERROR_FUNCTION_FAILED as i32);
        return 0;
    }
    let handle = ((*fci).open)(
        (*file).name.as_mut_ptr(),
        0x8502, // _O_RDWR | _O_CREAT | _O_EXCL | _O_BINARY
        0x0180, // _S_IREAD | _S_IWRITE
        &raw mut err,
        (*fci).pv,
    );
    if handle == -1isize {
        set_error(fci, FCIERR_TEMP_FILE, err);
        return 0;
    }
    (*file).handle = handle as INT_PTR;
    1
}

unsafe fn close_temp_file(fci: *mut FCI_Int, file: *mut temp_file) -> i32 {
    let mut err: i32 = 0;

    if (*file).handle == -1 {
        return 1;
    }
    if ((*fci).close)((*file).handle as isize, &raw mut err, (*fci).pv) == -1 {
        set_error(fci, FCIERR_TEMP_FILE, err);
        return 0;
    }
    (*file).handle = -1;
    if ((*fci).delete)((*file).name.as_mut_ptr(), &raw mut err, (*fci).pv) == -1 {
        set_error(fci, FCIERR_TEMP_FILE, err);
        return 0;
    }
    return 1;
}

unsafe fn add_file(fci: *mut FCI_Int, filename: *const i8) -> *mut file {
    extern "system" {
    }
    unsafe extern "C" {
        fn strcpy(dest: *mut i8, src: *const i8) -> *mut i8;
        fn strlen(s: *const i8) -> usize;
    }
    // list_add_tail is the cdecl wine_list shim (_list_add_tail); set_error is
    // the module-local `unsafe fn` below — declaring it extern "system" here
    // produced the undefined _set_error@12, so it is dropped.
    extern "C" {
        fn list_add_tail(list: *mut list, entry: *mut list);
    }

    let size = core::mem::offset_of!(file, name) + (strlen(filename) as usize) + 1;
    let alloc_fn = (*fci).alloc;
    let file_ptr: *mut file = alloc_fn(size as u32) as *mut file;

    if file_ptr.is_null() {
        set_error(fci, FCIERR_ALLOC_FAIL as i32, ERROR_NOT_ENOUGH_MEMORY as i32);
        return core::ptr::null_mut();
    }
    (*file_ptr).size = 0;
    (*file_ptr).offset = (*fci).cDataBlocks * CAB_BLOCKMAX + (*fci).cdata_in as u32;
    (*file_ptr).folder = (*fci).cFolders;
    (*file_ptr).date = 0;
    (*file_ptr).time = 0;
    (*file_ptr).attribs = 0;
    strcpy((*file_ptr).name.as_mut_ptr(), filename);
    list_add_tail(
        core::ptr::addr_of_mut!((*fci).files_list),
        core::ptr::addr_of_mut!((*file_ptr).entry),
    );
    (*fci).files_size += core::mem::size_of::<CFFILE>() as u32 + strlen(filename) as u32 + 1;
    file_ptr
}

#[allow(non_snake_case)]
unsafe fn copy_file(fci: *mut FCI_Int, orig: *const file) -> *mut file
{
    let name_offset = core::mem::offset_of!(file, name);
    let len = strlen((*orig).name.as_ptr());
    let size = name_offset + len + 1;
    let file = ((*fci).alloc)(size as u32) as *mut file;
    if file.is_null()
    {
        set_error(fci, FCIERR_ALLOC_FAIL, ERROR_NOT_ENOUGH_MEMORY as i32);
        return core::ptr::null_mut();
    }
    memcpy(file as *mut core::ffi::c_void, orig as *const core::ffi::c_void, size as usize);
    return file;
}

unsafe fn add_folder(fci: *mut FCI_Int) -> *mut folder {
    let folder: *mut folder = ((*fci).alloc)(core::mem::size_of::<folder>() as u32) as *mut folder;

    if folder.is_null() {
        set_error(fci, FCIERR_ALLOC_FAIL as i32, ERROR_NOT_ENOUGH_MEMORY as i32);
        return core::ptr::null_mut();
    }

    (*folder).data.handle = -1;
    (*folder).data_start = (*fci).folders_data_size;
    (*folder).data_count = 0;
    (*folder).compression = (*fci).compression;

    list_init(core::ptr::addr_of_mut!((*folder).files_list));
    list_init(core::ptr::addr_of_mut!((*folder).blocks_list));
    list_add_tail(
        core::ptr::addr_of_mut!((*fci).folders_list) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*folder).entry) as *mut core::ffi::c_void,
    );

    (*fci).folders_size = (*fci).folders_size.wrapping_add(
        (core::mem::size_of::<CFFOLDER>() + (*fci).ccab.cbReserveCFFolder as usize) as cab_ULONG,
    );
    (*fci).cFolders += 1;

    return folder;
}

unsafe fn write_folders(
    fci: *mut FCI_Int,
    handle: INT_PTR,
    header_size: cab_ULONG,
    _status_callback: PFNFCISTATUS,
) -> i32 {
    let cffolder: *mut CFFOLDER = (*fci).data_out.as_mut_ptr() as *mut CFFOLDER;
    let folder_size = core::mem::size_of::<CFFOLDER>() as cab_ULONG
        + (*fci).ccab.cbReserveCFFolder;
    // memset(cffolder, 0, folder_size)
    core::ptr::write_bytes(cffolder as *mut u8, 0, folder_size as usize);

    let head: *const list = &(*fci).folders_list;
    let mut next: *const list = (*head).next;
    while next != head {
        let folder_ptr = (next as *mut u8)
            .offset(-(core::mem::offset_of!(folder, entry) as isize)) as *mut folder;

        (*cffolder).coffCabStart = fci_endian_ulong((*folder_ptr).data_start + header_size);
        (*cffolder).cCFData = fci_endian_uword((*folder_ptr).data_count);
        (*cffolder).typeCompress = fci_endian_uword((*folder_ptr).compression as cab_UWORD);

        let mut err: i32 = 0;
        let written = ((*fci).write)(
            handle as isize,
            cffolder as *const core::ffi::c_void,
            folder_size as u32,
            &mut err as *mut i32,
            (*fci).pv,
        );
        if written != folder_size as u32 {
            set_error(fci, FCIERR_CAB_FILE as i32, err);
            return 0;
        }

        next = (*folder_ptr).entry.next;
    }
    1
}

unsafe fn write_files(
    fci: *mut FCI_Int,
    handle: INT_PTR,
    status_callback: PFNFCISTATUS,
) -> BOOL {
    let cffile = (*fci).data_out.as_mut_ptr() as *mut CFFILE;
    let mut folder_entry_ptr: *mut list = (*fci).folders_list.next;
    while folder_entry_ptr != &raw mut (*fci).folders_list as *mut list {
        let folder = folder_entry_ptr.byte_sub(core::mem::offset_of!(folder, entry)) as *mut folder;
        let mut file_entry_ptr: *mut list = (*folder).files_list.next;
        while file_entry_ptr != &raw mut (*folder).files_list as *mut list {
            let file = file_entry_ptr.byte_sub(core::mem::offset_of!(file, entry)) as *mut file;
            // cffile fields
            (*cffile).cbFile = fci_endian_ulong((*file).size);
            (*cffile).uoffFolderStart = fci_endian_ulong((*file).offset);
            (*cffile).iFolder = fci_endian_uword((*file).folder);
            (*cffile).date = fci_endian_uword((*file).date);
            (*cffile).time = fci_endian_uword((*file).time);
            (*cffile).attribs = fci_endian_uword((*file).attribs);
            // copy file name
            lstrcpynA(
                cffile.add(1) as *mut i8,
                (*file).name.as_ptr(),
                CB_MAX_FILENAME,
            );
            // file_size for write
            let name_len = strlen(cffile.add(1) as *const i8) as u32;
            let file_size = core::mem::size_of::<CFFILE>() as u32 + name_len + 1;
            let mut err: i32 = 0;
            let bytes_written = ((*fci).write)(
                handle as isize,
                cffile as *mut core::ffi::c_void,
                file_size,
                &raw mut err,
                (*fci).pv,
            );
            if bytes_written as u32 != file_size {
                set_error(fci, FCIERR_CAB_FILE as i32, err);
                return 0;
            }
            if (*fci).fSplitFolder == 0 {
                (*fci).statusFolderCopied = 0;
                (*fci).statusFolderTotal =
                    (*fci).folders_data_size + (*fci).placed_files_size;
            }
            (*fci).statusFolderCopied += file_size;
            let cb_status = (status_callback)(
                statusFolder,
                (*fci).statusFolderCopied,
                (*fci).statusFolderTotal,
                (*fci).pv,
            );
            if cb_status == -1 {
                set_error(fci, FCIERR_USER_ABORT, 0);
                return 0;
            }
            file_entry_ptr = (*file_entry_ptr).next;
        }
        folder_entry_ptr = (*folder_entry_ptr).next;
    }
    return 1;
}

unsafe extern "C" fn compress_MSZIP(fci: *mut FCI_Int) -> u16 {
    let mut stream: z_stream = core::mem::zeroed();
    stream.zalloc = Some(zalloc);
    stream.zfree = Some(zfree);
    stream.opaque = fci as *mut core::ffi::c_void;
    if deflateInit2_(&raw mut stream, Z_DEFAULT_COMPRESSION, Z_DEFLATED, -15, 8, Z_DEFAULT_STRATEGY, zlibVersion(), core::mem::size_of::<z_stream>() as i32) != Z_OK {
        set_error(fci, FCIERR_ALLOC_FAIL, ERROR_NOT_ENOUGH_MEMORY as i32);
        return 0;
    }
    stream.next_in = (*fci).data_in.as_ptr() as *const u8;
    stream.avail_in = (*fci).cdata_in as u32;
    stream.next_out = (*fci).data_out.as_mut_ptr().add(2);
    stream.avail_out = (core::mem::size_of::<[u8; 2 * 32768]>() - 2) as u32;
    // insert the signature
    (*fci).data_out[0] = b'C';
    (*fci).data_out[1] = b'K';
    deflate(&raw mut stream, Z_FINISH);
    deflateEnd(&raw mut stream);
    (stream.total_out as u32 + 2) as u16
}

unsafe fn write_data_blocks(
    fci: *mut FCI_Int,
    handle: INT_PTR,
    status_callback: PFNFCISTATUS,
) -> i32 {
    // Wine C passes a local `int err` to the seek/read/write callbacks' err
    // out-param; a prior translation passed &fci->pv cast to *mut i32, letting
    // callback error writes corrupt the low bytes of the user context pointer.
    let mut cb_err: i32 = 0;
    let header_size: u16 = (core::mem::size_of::<CFDATA>() as u16) + (*fci).ccab.cbReserveCFData as u16;
    let cfdata = (*fci).data_out.as_mut_ptr() as *mut CFDATA;
    // memset(cfdata, 0, header_size)
    core::ptr::write_bytes(cfdata as *mut u8, 0, header_size as usize);
    let data = (cfdata as *mut u8).add(header_size as usize) as *mut core::ffi::c_void;

    // Outer loop: LIST_FOR_EACH_ENTRY(folder, &fci->folders_list, struct folder, entry)
    let mut current = (*fci).folders_list.next;
    while current != core::ptr::addr_of!((*fci).folders_list) as *mut list {
        let folder = (current as *const u8).sub(core::mem::offset_of!(folder, entry)) as *mut folder;

        // Seek in folder's temp data file
        if ((*fci).seek)((*folder).data.handle as isize, 0, SEEK_SET, &mut cb_err, (*fci).pv) != 0 {
            set_error(fci, FCIERR_CAB_FILE as i32, (*fci).pv as i32); // FCIERR_CAB_FILE usage; assume value exists
            return 0;
        }

        // Inner loop: LIST_FOR_EACH_ENTRY(block, &folder->blocks_list, struct data_block, entry)
        let mut inner_current = (*folder).blocks_list.next;
        while inner_current != core::ptr::addr_of!((*folder).blocks_list) as *mut list {
            let block = (inner_current as *const u8).sub(core::mem::offset_of!(data_block, entry)) as *mut data_block;

            let len: i32 = ((*fci).read)(
                (*folder).data.handle as isize,
                data,
                (*block).compressed as u32,
                &mut cb_err,
                (*fci).pv,
            ) as i32;
            if len != (*block).compressed as i32 {
                return 0;
            }

            (*cfdata).cbData = fci_endian_uword((*block).compressed);
            (*cfdata).cbUncomp = fci_endian_uword((*block).uncompressed);
            (*cfdata).csum = fci_endian_ulong(
                fci_get_checksum(
                    core::ptr::addr_of!((*cfdata).cbData) as *const core::ffi::c_void,
                    header_size as u32 - core::mem::offset_of!(CFDATA, cbData) as u32,
                    fci_get_checksum(data, len as u32, 0),
                ),
            );

            (*fci).statusFolderCopied += len as u32;
            let len_plus_header = len + header_size as i32;
            if ((*fci).write)(
                handle as isize,
                (*fci).data_out.as_ptr() as *mut core::ffi::c_void,
                len_plus_header as u32,
                &mut cb_err,
                (*fci).pv,
            ) != len_plus_header as u32
            {
                set_error(fci, FCIERR_CAB_FILE as i32, (*fci).pv as i32);
                return 0;
            }

            // Callback statusFolder (assumed value 0)
            if status_callback(
                0, // statusFolder
                (*fci).statusFolderCopied,
                (*fci).statusFolderTotal,
                (*fci).pv,
            ) == -1
            {
                set_error(fci, FCIERR_USER_ABORT, 0);
                return 0;
            }

            inner_current = (*inner_current).next;
        }

        current = (*current).next;
    }

    return 1;
}

unsafe fn add_data_block(fci: *mut FCI_Int, status_callback: PFNFCISTATUS) -> i32 {
    let mut err: i32 = 0;
    let block: *mut data_block;

    if (*fci).cdata_in == 0 {
        return 1;
    }

    if (*fci).data.handle == -1 && create_temp_file(fci, core::ptr::addr_of_mut!((*fci).data)) == 0 {
        return 0;
    }

    let alloc_size = core::mem::size_of::<data_block>();
    block = ((*fci).alloc)(alloc_size as u32) as *mut data_block;
    if block.is_null() {
        set_error(fci, FCIERR_ALLOC_FAIL, ERROR_NOT_ENOUGH_MEMORY as i32);
        return 0;
    }

    (*block).uncompressed = (*fci).cdata_in;
    (*block).compressed = ((*fci).compress.unwrap())(fci);

    let write_result = ((*fci).write)((*fci).data.handle as isize, ((*fci).data_out).as_mut_ptr() as *const core::ffi::c_void, (*block).compressed as u32, core::ptr::addr_of_mut!(err), (*fci).pv);
    if write_result != (*block).compressed as u32 {
        set_error(fci, FCIERR_TEMP_FILE, err);
        ((*fci).free)(block as *mut core::ffi::c_void);
        return 0;
    }

    (*fci).cdata_in = 0;
    (*fci).pending_data_size += core::mem::size_of::<CFDATA>() as u32 + (*fci).ccab.cbReserveCFData as u32 + (*block).compressed as u32;
    (*fci).cCompressedBytesInFolder += (*block).compressed as u32;
    (*fci).cDataBlocks += 1;
    list_add_tail(core::ptr::addr_of_mut!((*fci).blocks_list) as *mut core::ffi::c_void, core::ptr::addr_of_mut!((*block).entry) as *mut core::ffi::c_void);

    let callback_result = status_callback(statusFile as u32, (*block).compressed as u32, (*block).uncompressed as u32, (*fci).pv);
    if callback_result == -1 {
        set_error(fci, FCIERR_USER_ABORT, 0);
        return 0;
    }
    return 1;
}

unsafe fn free_folder(fci: *mut FCI_Int, folder: *mut folder) {
    let head_files = core::ptr::addr_of_mut!((*folder).files_list);
    let mut file: *mut file;
    let mut file_next: *mut file;
    let mut cursor = (*head_files).next;
    while cursor != head_files {
        file = (cursor as *mut u8).sub(core::mem::offset_of!(file, entry)) as *mut file;
        file_next = ((*cursor).next as *mut u8).sub(core::mem::offset_of!(file, entry)) as *mut file;
        free_file(fci, file);
        cursor = (*cursor).next;
    }

    let head_blocks = core::ptr::addr_of_mut!((*folder).blocks_list);
    let mut block: *mut data_block;
    let mut block_next: *mut data_block;
    cursor = (*head_blocks).next;
    while cursor != head_blocks {
        block = (cursor as *mut u8).sub(core::mem::offset_of!(data_block, entry)) as *mut data_block;
        block_next = ((*cursor).next as *mut u8).sub(core::mem::offset_of!(data_block, entry)) as *mut data_block;
        free_data_block(fci, block);
        cursor = (*cursor).next;
    }

    close_temp_file(fci, core::ptr::addr_of_mut!((*folder).data));
    list_remove(core::ptr::addr_of_mut!((*folder).entry));
    ((*fci).free)(folder as *mut core::ffi::c_void);
}

unsafe extern "C" fn copy_data_blocks(
    fci: *mut FCI_Int,
    handle: INT_PTR,
    start_pos: cab_ULONG,
    temp: *mut temp_file,
    status_callback: unsafe extern "system" fn(u32, cab_ULONG, cab_ULONG, *mut core::ffi::c_void) -> i32,
) -> i32 {
    let mut err: i32 = 0;
    const statusFolder: u32 = 1;

    if ((*fci).seek)(handle as isize, start_pos as i32, SEEK_SET, core::ptr::addr_of_mut!(err), (*fci).pv) as isize
        != start_pos as isize
    {
        set_error(fci, FCIERR_TEMP_FILE, err);
        return 0; // FALSE
    }
    if create_temp_file(fci, temp) == 0 {
        return 0;
    }

    let head: *mut list = core::ptr::addr_of_mut!((*fci).blocks_list);
    let mut entry: *mut list = (*head).next;
    while entry != head {
        let block: *mut data_block = (entry as *mut u8)
            .sub(core::mem::offset_of!(data_block, entry))
            as *mut data_block;

        if ((*fci).read)(
            handle as isize,
            &raw mut (*fci).data_out as *mut u8 as *mut core::ffi::c_void,
            (*block).compressed as u32,
            core::ptr::addr_of_mut!(err),
            (*fci).pv,
        ) as isize
            != (*block).compressed as isize
        {
            close_temp_file(fci, temp);
            set_error(fci, FCIERR_TEMP_FILE, err);
            return 0;
        }
        if ((*fci).write)(
            (*temp).handle as isize,
            &raw mut (*fci).data_out as *mut u8 as *const core::ffi::c_void,
            (*block).compressed as u32,
            core::ptr::addr_of_mut!(err),
            (*fci).pv,
        ) as isize
            != (*block).compressed as isize
        {
            close_temp_file(fci, temp);
            set_error(fci, FCIERR_TEMP_FILE, err);
            return 0;
        }
        (*fci).pending_data_size += core::mem::size_of::<CFDATA>() as cab_ULONG
            + (*fci).ccab.cbReserveCFData as cab_ULONG
            + (*block).compressed as cab_ULONG;
        (*fci).statusFolderCopied += (*block).compressed as cab_ULONG;

        let ret = status_callback(
            statusFolder,
            (*fci).statusFolderCopied,
            (*fci).statusFolderTotal,
            (*fci).pv,
        );
        if ret == -1 {
            close_temp_file(fci, temp);
            set_error(fci, FCIERR_USER_ABORT, 0);
            return 0;
        }

        entry = (*entry).next;
    }
    return 1; // TRUE
}

unsafe fn add_files_to_folder(
    fci: *mut FCI_Int,
    folder: *mut folder,
    payload: u32,
) -> i32 {
    let mut size_of_files: u32 = 0;
    let mut size_of_files_prev: u32;
    let mut cb_file_remainder: u32 = 0;
    let mut file: *mut file;
    let mut next: *mut file;

    // LIST_FOR_EACH_ENTRY_SAFE simulation
    file = ((*fci).files_list.next as *const u8)
        .sub(core::mem::offset_of!(file, entry)) as *mut file;
    next = (((*file).entry).next as *const u8)
        .sub(core::mem::offset_of!(file, entry)) as *mut file;
    while core::ptr::addr_of!((*file).entry) as *const list
        != &raw const (*fci).files_list
    {
        let size = core::mem::size_of::<CFFILE>() as u32
            + strlen((*file).name.as_ptr() as *const core::ffi::c_char) as u32
            + 1;

        // fnfilfnfildest: placed file on cabinet
        ((*fci).fileplaced)(
            core::ptr::addr_of_mut!((*fci).ccab),
            (*file).name.as_mut_ptr(),
            (*file).size as i32,
            if (*file).folder == cffileCONTINUED_FROM_PREV { 1 } else { 0 },
            (*fci).pv,
        );

        size_of_files_prev = size_of_files;
        // set complete size of all processed files
        if (*file).folder == cffileCONTINUED_FROM_PREV && (*fci).cbFileRemainder != 0 {
            size_of_files += (*fci).cbFileRemainder;
            (*fci).cbFileRemainder = 0;
        } else {
            size_of_files += (*file).size;
        }

        // check if spanned file fits into this cabinet folder
        if size_of_files > payload {
            if (*file).folder == cffileCONTINUED_FROM_PREV {
                (*file).folder = cffileCONTINUED_PREV_AND_NEXT;
            } else {
                (*file).folder = cffileCONTINUED_TO_NEXT;
            }
        }

        list_remove(core::ptr::addr_of_mut!((*file).entry));
        list_add_tail(
            core::ptr::addr_of_mut!((*folder).files_list) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*file).entry) as *mut core::ffi::c_void,
        );
        (*fci).placed_files_size += size;
        (*fci).cFiles += 1;

        // This is only true for files which will be written into the
        // next cabinet of the spanning folder
        if size_of_files > payload {
            // add a copy back onto the list
            file = copy_file(fci, file);
            if file.is_null() {
                return 0;
            }
            list_add_before(
                core::ptr::addr_of_mut!((*next).entry) as *mut list,
                core::ptr::addr_of_mut!((*file).entry) as *mut list,
            );

            // Files which data will be partially written into the current cabinet
            if (*file).folder == cffileCONTINUED_PREV_AND_NEXT
                || (*file).folder == cffileCONTINUED_TO_NEXT
            {
                if size_of_files_prev <= payload {
                    // The size of the uncompressed data of a spanning file in a
                    // spanning data
                    cb_file_remainder = size_of_files - payload;
                }
                (*file).folder = cffileCONTINUED_FROM_PREV;
            } else {
                (*file).folder = 0;
            }
        } else {
            (*fci).files_size -= size;
        }

        // Advance pointers (LIST_FOR_EACH_ENTRY_SAFE remainder)
        file = next;
        next = (((*next).entry).next as *const u8)
            .sub(core::mem::offset_of!(file, entry)) as *mut file;
    }

    (*fci).cbFileRemainder = cb_file_remainder;
    return 1;
}

unsafe fn add_file_data(
    fci: *mut FCI_Int,
    sourcefile: *mut i8,
    filename: *mut i8,
    execute: i32,
    get_open_info: PFNFCIGETOPENINFO,
    status_callback: PFNFCISTATUS,
) -> i32 {
    let mut err: i32 = 0;
    let mut len: u32 = 0;
    let handle: isize;
    let file: *mut file;

    file = add_file(fci, filename);
    if file.is_null() {
        return 0;
    }

    handle = get_open_info(
        sourcefile,
        core::ptr::addr_of_mut!((*file).date),
        core::ptr::addr_of_mut!((*file).time),
        core::ptr::addr_of_mut!((*file).attribs),
        &mut err,
        (*fci).pv,
    );
    if handle == -1 {
        free_file(fci, file);
        set_error(fci, FCIERR_OPEN_SRC, err);
        return 0;
    }

    const _A_EXEC: u16 = 0x40; // _A_EXEC
    if execute != 0 {
        (*file).attribs |= _A_EXEC;
    }

    const CAB_BLOCKMAX: usize = 32768;

    loop {
        len = ((*fci).read)(
            handle as isize,
            (*fci).data_in.as_mut_ptr().add((*fci).cdata_in as usize) as *mut core::ffi::c_void,
            (CAB_BLOCKMAX - (*fci).cdata_in as usize) as u32,
            &mut err,
            (*fci).pv,
        );
        if len == 0 {
            break;
        }
        if len == (-1i32 as u32) {
            set_error(fci, FCIERR_READ_SRC, err);
            return 0;
        }
        (*file).size += len as u32;
        (*fci).cdata_in += len as u16;
        if (*fci).cdata_in as usize == CAB_BLOCKMAX && add_data_block(fci, status_callback as PFNFCISTATUS) == 0 {
            return 0;
        }
    }
    ((*fci).close)(handle, &mut err, (*fci).pv);
    return 1;
}

unsafe fn reset_cabinet(fci: *mut FCI_Int) {
    let head = &mut (*fci).folders_list as *mut list;
    let mut entry = (*head).next;
    if entry != head {
        let mut folder: *mut folder = (entry as usize - core::mem::offset_of!(folder, entry)) as *mut folder;
        let mut next_entry = (*entry).next;
        let mut folder_next: *mut folder = (next_entry as usize - core::mem::offset_of!(folder, entry)) as *mut folder;
        while &(*folder).entry as *const list != head as *const list {
            free_folder(fci, folder);
            folder = folder_next;
            let entry_ptr = &mut (*folder).entry as *mut list;
            next_entry = (*entry_ptr).next;
            folder_next = (next_entry as usize - core::mem::offset_of!(folder, entry)) as *mut folder;
        }
    }
    (*fci).cFolders = 0;
    (*fci).cFiles = 0;
    (*fci).folders_size = 0;
    (*fci).placed_files_size = 0;
    (*fci).folders_data_size = 0;
}

#[no_mangle]
pub unsafe extern "C" fn FCIDestroy(hfci: HFCI) -> i32 {
    let p_fci_internal = get_fci_ptr(hfci as *mut FCI_Int);
    if p_fci_internal.is_null() {
        return 0;
    }
    (*p_fci_internal).magic = 0;

    // folders list
    let head = &raw mut (*p_fci_internal).folders_list;
    let mut entry_ptr = (*head).next;
    while !entry_ptr.is_null() && (entry_ptr as *const _) != (head as *const _) {
        let folder = (entry_ptr as *mut u8).sub(core::mem::offset_of!(folder, entry)) as *mut folder;
        let next_entry = (*entry_ptr).next;
        free_folder(p_fci_internal, folder);
        entry_ptr = next_entry;
    }

    // files list
    let head = &raw mut (*p_fci_internal).files_list;
    let mut entry_ptr = (*head).next;
    while !entry_ptr.is_null() && (entry_ptr as *const _) != (head as *const _) {
        let file = (entry_ptr as *mut u8).sub(core::mem::offset_of!(file, entry)) as *mut file;
        let next_entry = (*entry_ptr).next;
        free_file(p_fci_internal, file);
        entry_ptr = next_entry;
    }

    // blocks list
    let head = &raw mut (*p_fci_internal).blocks_list;
    let mut entry_ptr = (*head).next;
    while !entry_ptr.is_null() && (entry_ptr as *const _) != (head as *const _) {
        let block = (entry_ptr as *mut u8).sub(core::mem::offset_of!(data_block, entry)) as *mut data_block;
        let next_entry = (*entry_ptr).next;
        free_data_block(p_fci_internal, block);
        entry_ptr = next_entry;
    }

    close_temp_file(p_fci_internal, &raw mut (*p_fci_internal).data);

    ((*p_fci_internal).free)(hfci);

    1
}

unsafe fn add_data_to_folder(
    fci: *mut FCI_Int,
    folder: *mut folder,
    payload: *mut cab_ULONG,
    status_callback: PFNFCISTATUS,
) -> BOOL {
    let mut split_block: BOOL = 0;
    let mut current_size: cab_ULONG;
    let mut start_pos: cab_ULONG = 0;

    *payload = 0;
    current_size = get_header_size(fci) + (*fci).folders_size +
                   (*fci).files_size + (*fci).placed_files_size + (*fci).folders_data_size;

    /* move the temp file into the folder structure */
    (*folder).data = core::ptr::read(&(*fci).data);
    (*fci).data.handle = -1;
    (*fci).pending_data_size = 0;

    let list_head: *mut list = core::ptr::addr_of_mut!((*fci).blocks_list);
    let mut entry_ptr: *mut list = (*list_head).next;
    while entry_ptr != list_head {
        let mut block = (entry_ptr as *mut u8)
            .sub(core::mem::offset_of!(data_block, entry))
            as *mut data_block;
        let next_entry_ptr = (*entry_ptr).next;
        let mut next: *mut data_block = if next_entry_ptr != list_head {
            (next_entry_ptr as *mut u8)
                .sub(core::mem::offset_of!(data_block, entry))
                as *mut data_block
        } else {
            core::ptr::null_mut::<data_block>()
        };

        /* No more CFDATA fits into the cabinet under construction */
        if (*fci).fNextCab != 0
            && (*fci).ccab.cb
                <= (core::mem::size_of::<CFDATA>() as cab_ULONG)
                    + (*fci).ccab.cbReserveCFData
                    + current_size
                    + (core::mem::size_of::<CFFOLDER>() as cab_ULONG)
                    + (*fci).ccab.cbReserveCFFolder
        {
            break;
        }

        let new = ((*fci).alloc)(core::mem::size_of::<data_block>() as u32) as *mut data_block;
        if new.is_null() {
            set_error(fci, FCIERR_ALLOC_FAIL, ERROR_NOT_ENOUGH_MEMORY as i32);
            return 0;
        }

        /* Is cabinet with new CFDATA too large? Then data block has to be split */
        if (*fci).fNextCab != 0
            && ((*fci).ccab.cb
                < (core::mem::size_of::<CFDATA>() as cab_ULONG)
                    + (*fci).ccab.cbReserveCFData
                    + (*block).compressed as u32
                    + current_size
                    + (core::mem::size_of::<CFFOLDER>() as cab_ULONG)
                    + (*fci).ccab.cbReserveCFFolder)
        {
            (*new).compressed = ((*fci).ccab.cb
                - (core::mem::size_of::<CFDATA>() as cab_ULONG
                    + (*fci).ccab.cbReserveCFData
                    + current_size
                    + (core::mem::size_of::<CFFOLDER>() as cab_ULONG)
                    + (*fci).ccab.cbReserveCFFolder)) as u16;
            (*new).uncompressed = 0;
            (*block).compressed -= (*new).compressed;
            split_block = 1;
        } else {
            (*new).compressed = (*block).compressed;
            (*new).uncompressed = (*block).uncompressed;
        }

        start_pos += (*new).compressed as u32;
        current_size += (core::mem::size_of::<CFDATA>() as cab_ULONG)
            + (*fci).ccab.cbReserveCFData
            + (*new).compressed as u32;
        (*fci).folders_data_size += (core::mem::size_of::<CFDATA>() as cab_ULONG)
            + (*fci).ccab.cbReserveCFData
            + (*new).compressed as u32;
        (*fci).statusFolderCopied += (*new).compressed as u32;
        *payload += (*new).uncompressed as u32;

        list_add_tail(
            core::ptr::addr_of_mut!((*folder).blocks_list) as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*new).entry) as *mut core::ffi::c_void,
        );
        (*folder).data_count += 1;

        /* report status with pfnfcis about copied size of folder */
        let ret = status_callback(
            statusFolder,
            (*fci).statusFolderCopied,
            (*fci).statusFolderTotal,
            (*fci).pv,
        );
        if ret == -1 {
            set_error(fci, FCIERR_USER_ABORT, 0);
            return 0;
        }

        if split_block != 0 {
            break;
        }
        free_data_block(fci, block);
        (*fci).cDataBlocks -= 1;

        entry_ptr = next_entry_ptr;
    }

    if list_empty(core::ptr::addr_of!((*fci).blocks_list)) != 0 {
        return 1;
    }
    copy_data_blocks(
        fci,
        (*folder).data.handle,
        start_pos,
        core::ptr::addr_of_mut!((*fci).data),
        status_callback,
    )
}

unsafe fn fci_endian_ulong(x: u32) -> u32 { x }
unsafe fn fci_endian_uword(x: u16) -> u16 { x }

unsafe fn write_cabinet(fci: *mut FCI_Int, status_callback: PFNFCISTATUS) -> i32 {
    let mut filename: [i8; CB_MAX_CAB_PATH + CB_MAX_CABINET_NAME] = [0; CB_MAX_CAB_PATH + CB_MAX_CABINET_NAME];
    let mut err: i32 = 0;
    let mut ptr: *mut i8;
    let mut handle: isize;
    let cfheader: *mut CFHEADER = (*fci).data_out.as_mut_ptr() as *mut CFHEADER;
    let mut flags: u16 = 0;
    let header_size: u32 = get_header_size(fci);
    let total_size: u32 = header_size + (*fci).folders_size + (*fci).placed_files_size + (*fci).folders_data_size;

    debug_assert!(header_size as usize <= core::mem::size_of_val(&(*fci).data_out));
    memset(cfheader as *mut core::ffi::c_void, 0, header_size as usize);

    if (*fci).fPrevCab != 0 { flags |= cfheadPREV_CABINET as u16; }
    if (*fci).fNextCab != 0 { flags |= cfheadNEXT_CABINET as u16; }
    if (*fci).ccab.cbReserveCFHeader != 0 || (*fci).ccab.cbReserveCFFolder != 0 || (*fci).ccab.cbReserveCFData != 0 {
        flags |= cfheadRESERVE_PRESENT as u16;
    }

    memcpy(
        (*cfheader).signature.as_mut_ptr() as *mut core::ffi::c_void,
        "!CAB\0".as_ptr() as *const core::ffi::c_void,
        4,
    );
    (*cfheader).cbCabinet = fci_endian_ulong(total_size);
    (*cfheader).coffFiles = fci_endian_ulong(header_size + (*fci).folders_size);
    (*cfheader).versionMinor = 3;
    (*cfheader).versionMajor = 1;
    (*cfheader).cFolders = fci_endian_uword((*fci).cFolders);
    (*cfheader).cFiles = fci_endian_uword((*fci).cFiles);
    (*cfheader).flags = fci_endian_uword(flags);
    (*cfheader).setID = fci_endian_uword((*fci).ccab.setID);
    (*cfheader).iCabinet = fci_endian_uword((*fci).ccab.iCab as u16);
    ptr = cfheader.add(1) as *mut i8;

    if (flags & cfheadRESERVE_PRESENT as u16) != 0 {
        let reserve: *mut Reserve = ptr as *mut Reserve;
        (*reserve).cbCFHeader = fci_endian_uword((*fci).ccab.cbReserveCFHeader as u16);
        (*reserve).cbCFFolder = (*fci).ccab.cbReserveCFFolder as u8;
        (*reserve).cbCFData = (*fci).ccab.cbReserveCFData as u8;
        ptr = reserve.add(1) as *mut i8;
    }
    ptr = ptr.offset((*fci).ccab.cbReserveCFHeader as isize);

    if (flags & cfheadPREV_CABINET as u16) != 0 {
        strcpy(ptr, (*fci).szPrevCab.as_ptr() as *const i8);
        ptr = ptr.offset(strlen(ptr) as isize + 1);
        strcpy(ptr, (*fci).szPrevDisk.as_ptr() as *const i8);
        ptr = ptr.offset(strlen(ptr) as isize + 1);
    }

    if (flags & cfheadNEXT_CABINET as u16) != 0 {
        strcpy(ptr, (*(*fci).pccab).szCab.as_ptr() as *const i8);
        ptr = ptr.offset(strlen(ptr) as isize + 1);
        strcpy(ptr, (*(*fci).pccab).szDisk.as_ptr() as *const i8);
        ptr = ptr.offset(strlen(ptr) as isize + 1);
    }

    debug_assert!(ptr as isize - cfheader as isize == header_size as isize);

    strcpy(filename.as_mut_ptr(), (*fci).ccab.szCabPath.as_ptr() as *const i8);
    strcat(filename.as_mut_ptr(), (*fci).ccab.szCab.as_ptr() as *const i8);

    handle = ((*fci).open)(
        filename.as_mut_ptr(),
        _O_RDWR | _O_CREAT | _O_TRUNC | _O_BINARY,
        _S_IREAD | _S_IWRITE,
        core::ptr::addr_of_mut!(err),
        (*fci).pv,
    );
    if handle == -1 {
        set_error(fci, FCIERR_CAB_FILE as i32, err);
        return 0;
    }

    let result: i32 = 'failed: {
        if ((*fci).write)(handle, cfheader as *const core::ffi::c_void, header_size, core::ptr::addr_of_mut!(err), (*fci).pv) != header_size {
            set_error(fci, FCIERR_CAB_FILE as i32, err);
            break 'failed 0;
        }

        let mut header_size = header_size + (*fci).placed_files_size + (*fci).folders_size;
        if write_folders(fci, handle as INT_PTR, header_size, status_callback) == 0 { break 'failed 0; }
        if write_files(fci, handle as INT_PTR, status_callback) == 0 { break 'failed 0; }
        if write_data_blocks(fci, handle as INT_PTR, status_callback) == 0 { break 'failed 0; }

        if ((*fci).seek)(handle, 0, SEEK_SET, core::ptr::addr_of_mut!(err), (*fci).pv) != 0 {
            set_error(fci, FCIERR_CAB_FILE as i32, err);
            break 'failed 0;
        }
        memcpy(
            (*cfheader).signature.as_mut_ptr() as *mut core::ffi::c_void,
            "MSCF\0".as_ptr() as *const core::ffi::c_void,
            4,
        );
        if ((*fci).write)(handle, (*cfheader).signature.as_ptr() as *const core::ffi::c_void, 4, core::ptr::addr_of_mut!(err), (*fci).pv) != 4
        {
            set_error(fci, FCIERR_CAB_FILE as i32, err);
            break 'failed 0;
        }

        // success path
        break 'failed 1;
    };
    ((*fci).close)(handle, core::ptr::addr_of_mut!(err), (*fci).pv);
    if result != 0 {
        reset_cabinet(fci);
        status_callback(statusCabinet, (*fci).estimatedCabinetSize, total_size, (*fci).pv);
    } else {
        ((*fci).delete)(filename.as_mut_ptr(), core::ptr::addr_of_mut!(err), (*fci).pv);
    }
    result
}

unsafe fn fci_flush_folder(
    p_fci_internal: *mut FCI_Int,
    fGetNextCab: i32,
    pfnfcignc: PFNFCIGETNEXTCABINET,
    pfnfcis: PFNFCISTATUS,
) -> i32 {
    let mut payload: u32 = 0;
    let mut read_result: u32;
    let folder: *mut folder;

    // pfnfcignc/pfnfcis are non-nullable fn pointers (canonical FCI callback
    // typedefs); the C null-check is statically impossible here.

    if (*p_fci_internal).fGetNextCabInVain != 0 && (*p_fci_internal).fNextCab != 0 {
        set_error(p_fci_internal, 0 /* FCIERR_NONE */, ERROR_GEN_FAILURE as i32);
        return 0;
    }

    if (*p_fci_internal).files_size == 0 {
        if (*p_fci_internal).pending_data_size != 0 {
            set_error(p_fci_internal, 0 /* FCIERR_NONE */, ERROR_GEN_FAILURE as i32);
            return 0;
        }
        return 1;
    }

    if (*p_fci_internal).fSplitFolder != 0 && (*p_fci_internal).placed_files_size != 0 {
        return 1;
    }

    (*p_fci_internal).fSplitFolder = 0;

    if add_data_block(p_fci_internal, pfnfcis) == 0 {
        return 0;
    }

    (*p_fci_internal).cDataBlocks = 0;

    (*p_fci_internal).statusFolderTotal = get_header_size(p_fci_internal)
        + core::mem::size_of::<CFFOLDER>() as u32
        + (*p_fci_internal).ccab.cbReserveCFFolder
        + (*p_fci_internal).placed_files_size
        + (*p_fci_internal).folders_data_size
        + (*p_fci_internal).files_size
        + (*p_fci_internal).pending_data_size
        + (*p_fci_internal).folders_size;
    (*p_fci_internal).statusFolderCopied = 0;

    let callback = pfnfcis;
    if callback(
        0x00000002, /* statusFolder */
        (*p_fci_internal).statusFolderCopied,
        (*p_fci_internal).statusFolderTotal,
        (*p_fci_internal).pv,
    ) == -1
    {
        set_error(p_fci_internal, 0 /* FCIERR_NONE */, 0);
        return 0;
    }

    read_result = get_header_size(p_fci_internal)
        + (*p_fci_internal).folders_data_size
        + (*p_fci_internal).placed_files_size
        + (*p_fci_internal).folders_size;

    if (*p_fci_internal).files_size != 0 {
        read_result +=
            core::mem::size_of::<CFFOLDER>() as u32 + (*p_fci_internal).ccab.cbReserveCFFolder;
    }

    if (*p_fci_internal).fGetNextCabInVain == 0
        && (*p_fci_internal).fNextCab == 0
        && (((*p_fci_internal).ccab.cb
            < read_result
                + (*p_fci_internal).pending_data_size
                + (*p_fci_internal).files_size
                + 256 /* CB_MAX_CABINET_NAME */
                + 256 /* CB_MAX_DISK_NAME */)
            || fGetNextCab != 0)
    {
        (*((*p_fci_internal).pccab as *mut CCAB)).iCab += 1;
        (*p_fci_internal).estimatedCabinetSize = (*p_fci_internal).statusFolderTotal;
        if (pfnfcignc)(
            (*p_fci_internal).pccab,
            (*p_fci_internal).estimatedCabinetSize,
            (*p_fci_internal).pv,
        ) == 0 {
            set_error(p_fci_internal, 0 /* FCIERR_NONE */, ERROR_FUNCTION_FAILED as i32);
            return 0;
        }
        (*p_fci_internal).fGetNextCabInVain = 1;
    }

    if ((*p_fci_internal).fGetNextCabInVain != 0 || (*p_fci_internal).fNextCab != 0)
        && (((*p_fci_internal).ccab.cb
            < read_result
                + (*p_fci_internal).pending_data_size
                + (*p_fci_internal).files_size
                + strlen((*(*p_fci_internal).pccab).szCab.as_ptr()) as u32
                + 1
                + strlen((*(*p_fci_internal).pccab).szDisk.as_ptr()) as u32
                + 1)
            || fGetNextCab != 0)
    {
        (*p_fci_internal).fGetNextCabInVain = 0;
        (*p_fci_internal).fNextCab = 1;

        if (*p_fci_internal).ccab.cb
            <= (*p_fci_internal).files_size
                + read_result
                + strlen((*(*p_fci_internal).pccab).szCab.as_ptr()) as u32
                + 1
                + strlen((*(*p_fci_internal).pccab).szDisk.as_ptr()) as u32
                + 1
        {
            return 0;
        }

        (*p_fci_internal).fSplitFolder = 1;
    } else {
        if (*p_fci_internal).fNextCab != 0 {
            set_error(p_fci_internal, 0 /* FCIERR_NONE */, ERROR_GEN_FAILURE as i32);
            return 0;
        }
    }

    folder = add_folder(p_fci_internal);
    if folder.is_null() {
        return 0;
    }
    if add_data_to_folder(p_fci_internal, folder, core::ptr::addr_of_mut!(payload), pfnfcis) == 0 {
        return 0;
    }
    if add_files_to_folder(p_fci_internal, folder, payload) == 0 {
        return 0;
    }

    (*p_fci_internal).cDataBlocks = 0;
    (*p_fci_internal).cCompressedBytesInFolder = 0;

    return 1;
}

unsafe fn fci_flush_cabinet(
    p_fci_internal: *mut FCI_Int,
    fGetNextCab: i32,
    pfnfcignc: PFNFCIGETNEXTCABINET,
    pfnfcis: PFNFCISTATUS,
) -> i32 {
    let mut read_result: u32 = 0;
    let mut returntrue: i32 = 0;

    /* TODO test if fci_flush_cabinet really aborts if there was no FCIAddFile */

    /* when FCIFlushCabinet was or FCIAddFile hasn't been called */
    if (*p_fci_internal).files_size == 0 && fGetNextCab != 0 {
        returntrue = 1;
    }

    if fci_flush_folder(p_fci_internal, fGetNextCab, pfnfcignc, pfnfcis) == 0 {
        /* TODO set error */
        return 0;
    }

    if returntrue != 0 {
        return 1;
    }

    if ((*p_fci_internal).fSplitFolder != 0 && (*p_fci_internal).fNextCab == 0)
        || ((*p_fci_internal).folders_size == 0
            && ((*p_fci_internal).files_size != 0
                || (*p_fci_internal).placed_files_size != 0))
    {
        /* error */
        set_error(p_fci_internal, FCIERR_NONE, ERROR_GEN_FAILURE as i32);
        return 0;
    }

    /* create the cabinet */
    if write_cabinet(p_fci_internal, pfnfcis) == 0 {
        return 0;
    }

    (*p_fci_internal).fPrevCab = 1;
    /* The sections szPrevCab and szPrevDisk are not being updated, because */
    /* MS CABINET.DLL always puts the first cabinet name and disk into them */

    if (*p_fci_internal).fNextCab != 0 {
        (*p_fci_internal).fNextCab = 0;

        if (*p_fci_internal).files_size == 0 && (*p_fci_internal).pending_data_size != 0 {
            /* THIS CAN NEVER HAPPEN */
            /* set error code */
            set_error(p_fci_internal, FCIERR_NONE, ERROR_GEN_FAILURE as i32);
            return 0;
        }

        if (*p_fci_internal).fNewPrevious != 0 {
            memcpy(
                (*p_fci_internal).szPrevCab.as_mut_ptr() as *mut core::ffi::c_void,
                (*p_fci_internal).ccab.szCab.as_ptr() as *const core::ffi::c_void,
                CB_MAX_CABINET_NAME,
            );
            memcpy(
                (*p_fci_internal).szPrevDisk.as_mut_ptr() as *mut core::ffi::c_void,
                (*p_fci_internal).ccab.szDisk.as_ptr() as *const core::ffi::c_void,
                CB_MAX_DISK_NAME,
            );
            (*p_fci_internal).fNewPrevious = 0;
        }

        (*p_fci_internal).ccab = core::ptr::read((*p_fci_internal).pccab);

        /* REUSE the variable read_result */
        read_result = get_header_size(p_fci_internal);
        if (*p_fci_internal).files_size != 0 {
            read_result += (*p_fci_internal).ccab.cbReserveCFFolder as u32;
        }
        read_result += (*p_fci_internal).pending_data_size
            + (*p_fci_internal).files_size
            + (*p_fci_internal).folders_data_size
            + (*p_fci_internal).placed_files_size
            + (*p_fci_internal).folders_size
            + core::mem::size_of::<CFFOLDER>() as u32;

        /* too much data for the maximum size of a cabinet */
        if (*p_fci_internal).fGetNextCabInVain == 0
            && (*p_fci_internal).ccab.cb < read_result
        {
            return fci_flush_cabinet(p_fci_internal, 0, pfnfcignc, pfnfcis);
        }

        /* Might be too much data for the maximum size of a cabinet.*/
        /* When any further data will be added later, it might not */
        /* be possible to flush the cabinet, because there might */
        /* not be enough space to store the name of the following */
        /* cabinet and name of the corresponding disk. */
        /* So take care of this and get the name of the next cabinet */
        if (*p_fci_internal).fGetNextCabInVain == 0
            && ((*p_fci_internal).ccab.cb
                < read_result + CB_MAX_CABINET_NAME as u32 + CB_MAX_DISK_NAME as u32)
        {
            /* increment cabinet index */
            (*((*p_fci_internal).pccab as *mut CCAB)).iCab += 1;
            /* get name of next cabinet */
            (*p_fci_internal).estimatedCabinetSize = (*p_fci_internal).statusFolderTotal;
            if pfnfcignc(
                (*p_fci_internal).pccab,
                (*p_fci_internal).estimatedCabinetSize, /* estimated size of cab */
                (*p_fci_internal).pv,
            ) == 0
            {
                /* error handling */
                set_error(p_fci_internal, FCIERR_NONE, ERROR_FUNCTION_FAILED as i32);
                return 0;
            }
            /* Skip a few lines of code. This is caught by the next if. */
            (*p_fci_internal).fGetNextCabInVain = 1;
        }

        /* too much data for cabinet */
        if (*p_fci_internal).fGetNextCabInVain != 0
            && ((*p_fci_internal).ccab.cb
                < read_result
                    + strlen((*p_fci_internal).ccab.szCab.as_ptr() as *const i8) as u32
                    + 1
                    + strlen((*p_fci_internal).ccab.szDisk.as_ptr() as *const i8) as u32
                    + 1)
        {
            (*p_fci_internal).fGetNextCabInVain = 0;
            (*p_fci_internal).fNextCab = 1;
            return fci_flush_cabinet(p_fci_internal, 0, pfnfcignc, pfnfcis);
        }

        /* if the FolderThreshold has been reached flush the folder automatically */
        if (*p_fci_internal).cCompressedBytesInFolder >= (*p_fci_internal).ccab.cbFolderThresh {
            return fci_flush_folder(p_fci_internal, 0, pfnfcignc, pfnfcis);
        }

        if (*p_fci_internal).files_size > 0 {
            if fci_flush_folder(p_fci_internal, 0, pfnfcignc, pfnfcis) == 0 {
                return 0;
            }
            (*p_fci_internal).fNewPrevious = 1;
        }
    } else {
        (*p_fci_internal).fNewPrevious = 0;
        if (*p_fci_internal).files_size > 0 || (*p_fci_internal).pending_data_size != 0 {
            /* THIS MAY NEVER HAPPEN */
            /* set error structures */
            set_error(p_fci_internal, FCIERR_NONE, ERROR_GEN_FAILURE as i32);
            return 0;
        }
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "C" fn FCIFlushFolder(
    hfci: HFCI,
    pfnfcignc: PFNFCIGETNEXTCABINET,
    pfnfcis: PFNFCISTATUS,
) -> BOOL {
    let p_fci_internal = get_fci_ptr(hfci as *mut FCI_Int);
    if p_fci_internal.is_null() {
        return 0;
    }
    fci_flush_folder(p_fci_internal, 0, pfnfcignc, pfnfcis)
}

/* FCIFlushCabinet (CABINET.13) — public export wrapper (dropped in the
 * DeepSeek translation; the internal fci_flush_cabinet helper was kept but the
 * exported wrapper was not). Faithful hand-port of fci.c:1675. */
#[no_mangle]
pub unsafe extern "C" fn FCIFlushCabinet(
    hfci: HFCI,
    fGetNextCab: BOOL,
    pfnfcignc: PFNFCIGETNEXTCABINET,
    pfnfcis: PFNFCISTATUS,
) -> BOOL {
    let p_fci_internal = get_fci_ptr(hfci as *mut FCI_Int);
    if p_fci_internal.is_null() {
        return 0;
    }
    if fci_flush_cabinet(p_fci_internal, fGetNextCab, pfnfcignc, pfnfcis) == 0 {
        return 0;
    }
    while (*p_fci_internal).files_size > 0 || (*p_fci_internal).placed_files_size > 0 {
        if fci_flush_cabinet(p_fci_internal, fGetNextCab, pfnfcignc, pfnfcis) == 0 {
            return 0;
        }
    }
    1
}

#[no_mangle]
pub unsafe extern "C" fn FCIAddFile(
    hfci: HFCI,
    pszSourceFile: *mut i8,
    pszFileName: *mut i8,
    fExecute: BOOL,
    pfnfcignc: PFNFCIGETNEXTCABINET,
    pfnfcis: PFNFCISTATUS,
    pfnfcigoi: PFNFCIGETOPENINFO,
    typeCompress: TCOMP,
) -> BOOL {
    let mut read_result: u32;
    let p_fci_internal = get_fci_ptr(hfci as *mut FCI_Int);

    if p_fci_internal.is_null() {
        return 0;
    }

    if pszSourceFile.is_null()
        || pszFileName.is_null()
        || (pfnfcignc as *const ()).is_null()
        || (pfnfcis as *const ()).is_null()
        || (pfnfcigoi as *const ()).is_null()
        || strlen(pszFileName) >= CB_MAX_FILENAME as usize
    {
        set_error(
            p_fci_internal,
            FCIERR_NONE,
            ERROR_BAD_ARGUMENTS as i32,
        );
        return 0;
    }

    if typeCompress != (*p_fci_internal).compression {
        if FCIFlushFolder(hfci, pfnfcignc, pfnfcis) == 0 {
            return 0;
        }
        match typeCompress {
            // No cfg gate (upstream 4681b3e473a dropped HAVE_ZLIB): nothing ever set the
            // `#[cfg(HAVE_ZLIB)]` this arm carried, so every cabinet we built was stored, not
            // deflated, while the C build had compression on.
            tcompTYPE_MSZIP => {
                (*p_fci_internal).compression = tcompTYPE_MSZIP;
                (*p_fci_internal).compress = Some(compress_MSZIP);
            }
            _ => {
                winedebug_rust::FIXME!("compression %x not supported, defaulting to none\n", typeCompress as u32);
                (*p_fci_internal).compression = tcompTYPE_NONE as TCOMP;
                (*p_fci_internal).compress = Some(compress_NONE);
            }
        }
    }

    /* TODO check if pszSourceFile??? */

    if (*p_fci_internal).fGetNextCabInVain != 0 && (*p_fci_internal).fNextCab != 0 {
        /* internal error */
        set_error(
            p_fci_internal,
            FCIERR_NONE,
            ERROR_GEN_FAILURE as i32,
        );
        return 0;
    }

    if (*p_fci_internal).fNextCab != 0 {
        /* internal error */
        set_error(
            p_fci_internal,
            FCIERR_NONE,
            ERROR_GEN_FAILURE as i32,
        );
        return 0;
    }

    /* REUSE the variable read_result */
    read_result = get_header_size(p_fci_internal) + (*p_fci_internal).ccab.cbReserveCFFolder;

    read_result += core::mem::size_of::<CFFILE>() as u32
        + (strlen(pszFileName) as u32) + 1
        + (*p_fci_internal).files_size
        + (*p_fci_internal).folders_data_size
        + (*p_fci_internal).placed_files_size
        + (*p_fci_internal).folders_size
        + core::mem::size_of::<CFFOLDER>() as u32; /* size of new CFFolder entry */

    /* Might be too much data for the maximum size of a cabinet. */
    /* When any further data will be added later, it might not */
    /* be possible to flush the cabinet, because there might */
    /* not be enough space to store the name of the following */
    /* cabinet and name of the corresponding disk. */
    /* So take care of this and get the name of the next cabinet */
    if (*p_fci_internal).fGetNextCabInVain == 0
        && (*p_fci_internal).fNextCab == 0
        && (*p_fci_internal).ccab.cb < read_result + CB_MAX_CABINET_NAME as u32 + CB_MAX_DISK_NAME as u32
    {
        /* increment cabinet index */
        (*((*p_fci_internal).pccab as *mut CCAB)).iCab += 1;
        /* get name of next cabinet */
        (*p_fci_internal).estimatedCabinetSize = (*p_fci_internal).statusFolderTotal;
        if pfnfcignc(
            (*p_fci_internal).pccab,
            (*p_fci_internal).estimatedCabinetSize, /* estimated size of cab */
            (*p_fci_internal).pv,
        ) == 0
        {
            /* error handling */
            set_error(
                p_fci_internal,
                FCIERR_NONE,
                ERROR_FUNCTION_FAILED as i32,
            );
            return 0;
        }
        /* Skip a few lines of code. This is caught by the next if. */
        (*p_fci_internal).fGetNextCabInVain = 1;
    }

    if (*p_fci_internal).fGetNextCabInVain != 0 && (*p_fci_internal).fNextCab != 0 {
        /* THIS CAN NEVER HAPPEN */
        /* set error code */
        set_error(p_fci_internal, FCIERR_NONE, ERROR_GEN_FAILURE as i32);
        return 0;
    }

    /* too much data for cabinet */
    if (*p_fci_internal).fGetNextCabInVain != 0
        && ((*p_fci_internal).ccab.cb
            < read_result
                + strlen((*(*p_fci_internal).pccab).szCab.as_ptr() as *const i8) as u32
                + 1
                + strlen((*(*p_fci_internal).pccab).szDisk.as_ptr() as *const i8) as u32
                + 1)
    {
        (*p_fci_internal).fGetNextCabInVain = 0;
        (*p_fci_internal).fNextCab = 1;
        if fci_flush_cabinet(p_fci_internal, 0, pfnfcignc, pfnfcis) == 0 {
            return 0;
        }
    }

    if (*p_fci_internal).fNextCab != 0 {
        /* THIS MAY NEVER HAPPEN */
        /* set error code */
        set_error(p_fci_internal, FCIERR_NONE, ERROR_GEN_FAILURE as i32);
        return 0;
    }

    if add_file_data(p_fci_internal, pszSourceFile, pszFileName, fExecute, pfnfcigoi, pfnfcis) == 0 {
        return 0;
    }

    /* REUSE the variable read_result */
    read_result = get_header_size(p_fci_internal) + (*p_fci_internal).ccab.cbReserveCFFolder;
    read_result += (*p_fci_internal).pending_data_size
        + (*p_fci_internal).files_size
        + (*p_fci_internal).folders_data_size
        + (*p_fci_internal).placed_files_size
        + (*p_fci_internal).folders_size
        + core::mem::size_of::<CFFOLDER>() as u32; /* set size of new CFFolder entry */

    /* too much data for the maximum size of a cabinet */
    /* (ignoring the unflushed data block) */
    if (*p_fci_internal).fGetNextCabInVain == 0
        && (*p_fci_internal).fNextCab == 0
        && (*p_fci_internal).ccab.cb < read_result
    {
        return fci_flush_cabinet(p_fci_internal, 0, pfnfcignc, pfnfcis);
    }

    /* Might be too much data for the maximum size of a cabinet.*/
    /* When any further data will be added later, it might not */
    /* be possible to flush the cabinet, because there might */
    /* not be enough space to store the name of the following */
    /* cabinet and name of the corresponding disk. */
    /* So take care of this and get the name of the next cabinet */
    /* (ignoring the unflushed data block) */
    if (*p_fci_internal).fGetNextCabInVain == 0
        && (*p_fci_internal).fNextCab == 0
        && ((*p_fci_internal).ccab.cb
            < read_result + CB_MAX_CABINET_NAME as u32 + CB_MAX_DISK_NAME as u32)
    {
        /* increment cabinet index */
        (*((*p_fci_internal).pccab as *mut CCAB)).iCab += 1;
        /* get name of next cabinet */
        (*p_fci_internal).estimatedCabinetSize = (*p_fci_internal).statusFolderTotal;
        if pfnfcignc(
            (*p_fci_internal).pccab,
            (*p_fci_internal).estimatedCabinetSize, /* estimated size of cab */
            (*p_fci_internal).pv,
        ) == 0
        {
            /* error handling */
            set_error(p_fci_internal, FCIERR_NONE, ERROR_FUNCTION_FAILED as i32);
            return 0;
        }
        /* Skip a few lines of code. This is caught by the next if. */
        (*p_fci_internal).fGetNextCabInVain = 1;
    }

    if (*p_fci_internal).fGetNextCabInVain != 0 && (*p_fci_internal).fNextCab != 0 {
        /* THIS CAN NEVER HAPPEN */
        set_error(p_fci_internal, FCIERR_NONE, ERROR_GEN_FAILURE as i32);
        return 0;
    }

    /* too much data for cabinet */
    if ((*p_fci_internal).fGetNextCabInVain != 0 || (*p_fci_internal).fNextCab != 0)
        && ((*p_fci_internal).ccab.cb
            < read_result
                + strlen((*(*p_fci_internal).pccab).szCab.as_ptr() as *const i8) as u32
                + 1
                + strlen((*(*p_fci_internal).pccab).szDisk.as_ptr() as *const i8) as u32
                + 1)
    {
        (*p_fci_internal).fGetNextCabInVain = 0;
        (*p_fci_internal).fNextCab = 1;
        return fci_flush_cabinet(p_fci_internal, 0, pfnfcignc, pfnfcis);
    }

    if (*p_fci_internal).fNextCab != 0 {
        /* THIS MAY NEVER HAPPEN */
        /* set error code */
        set_error(p_fci_internal, FCIERR_NONE, ERROR_GEN_FAILURE as i32);
        return 0;
    }

    /* if the FolderThreshold has been reached flush the folder automatically */
    if (*p_fci_internal).cCompressedBytesInFolder >= (*p_fci_internal).ccab.cbFolderThresh {
        return FCIFlushFolder(hfci, pfnfcignc, pfnfcis);
    }

    return 1;
}
