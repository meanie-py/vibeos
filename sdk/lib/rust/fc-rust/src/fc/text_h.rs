use crate::fc::fc::Different;
use crate::fc::fc::NoDifference;
use crate::fc::fc::OutOfMemory;
use crate::fc::fc::PrintCaption;
use crate::fc::fc::PrintDots;
use crate::fc::fc::PrintEndOfDiff;
use crate::fc::fc::PrintLineA;
use crate::fc::fc::PrintLineW;
use crate::fc::fc::ResyncFailed;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// text.h is a TCHAR TEMPLATE: texta.c includes it with UNICODE undefined and textw.c with it
// defined, and the `NODE`/`PrintLine`/`TextCompare` macros pick the A or W name. Rust says
// that with a generic: every function below is generic over the char unit `T`, NODE_A/NODE_W
// are the one `Node<T>`, and texta.rs/textw.rs each export their instantiation under the C
// name fc.c calls (TextCompareA / TextCompareW). The bodies were bought once, through
// textw.c, and made generic by hand.

pub const TAB_WIDTH: i32 = 8;

pub const HASH_EOF: u32 = 0xFFFFFFFF;

pub const HASH_MASK: u32 = 0x7FFFFFFF;

// ntbasedef.h: MAKELCID(MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT), SORT_DEFAULT).
const LOCALE_USER_DEFAULT: LCID = 0x0400;

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn calloc(num: usize, size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
    fn abs(n: c_int) -> c_int;
}

extern "system" {
    fn CompareStringA(
        Locale: LCID,
        dwCmpFlags: DWORD,
        lpString1: LPCSTR,
        cchCount1: c_int,
        lpString2: LPCSTR,
        cchCount2: c_int,
    ) -> c_int;
    fn CompareStringW(
        Locale: LCID,
        dwCmpFlags: DWORD,
        lpString1: LPCWSTR,
        cchCount1: c_int,
        lpString2: LPCWSTR,
        cchCount2: c_int,
    ) -> c_int;
}

/// The TCHAR a text.h instantiation is compiled with: `CHAR` (texta.c) or `WCHAR` (textw.c).
pub trait TChar: Copy + PartialEq {
    fn from_ascii(b: u8) -> Self;
    fn is_nul(self) -> bool;
    /// One step of GetHash: `bIgnoreCase ? towupper(*psz) : *psz`, as C evaluates it for
    /// this char type.
    unsafe fn hash_unit(self, ignore_case: bool) -> DWORD;
    /// `CompareString(LOCALE_USER_DEFAULT, flags, a, -1, b, -1)`: the A or W API.
    unsafe fn compare_string(flags: DWORD, a: *const Self, b: *const Self) -> c_int;
    /// `PrintLine`: PrintLineA or PrintLineW.
    unsafe fn print_line(pFC: *const FILECOMPARE, lineno: DWORD, psz: *const Self);
}

impl TChar for WCHAR {
    fn from_ascii(b: u8) -> Self {
        b as WCHAR
    }
    fn is_nul(self) -> bool {
        self == 0
    }
    unsafe fn hash_unit(self, ignore_case: bool) -> DWORD {
        if ignore_case {
            towupper(self) as DWORD
        } else {
            self as DWORD
        }
    }
    unsafe fn compare_string(flags: DWORD, a: *const Self, b: *const Self) -> c_int {
        CompareStringW(LOCALE_USER_DEFAULT, flags, a, -1, b, -1)
    }
    unsafe fn print_line(pFC: *const FILECOMPARE, lineno: DWORD, psz: *const Self) {
        PrintLineW(pFC, lineno, psz)
    }
}

impl TChar for CHAR {
    fn from_ascii(b: u8) -> Self {
        b as CHAR
    }
    fn is_nul(self) -> bool {
        self == 0
    }
    unsafe fn hash_unit(self, ignore_case: bool) -> DWORD {
        // `char` is signed on every Windows target, so a byte >= 0x80 sign-extends: through
        // towupper's wint_t (unsigned short) parameter it is 0xFFxx, and as a plain `int`
        // operand of the ?: it is negative. The hash only has to agree between the two files,
        // but the A instantiation hashes exactly as the C does.
        let c = self as i8;
        if ignore_case {
            towupper(c as u16) as DWORD
        } else {
            c as c_int as DWORD
        }
    }
    unsafe fn compare_string(flags: DWORD, a: *const Self, b: *const Self) -> c_int {
        CompareStringA(LOCALE_USER_DEFAULT, flags, a, -1, b, -1)
    }
    unsafe fn print_line(pFC: *const FILECOMPARE, lineno: DWORD, psz: *const Self) {
        PrintLineA(pFC, lineno, psz)
    }
}

/// NODE_A / NODE_W (fc.h): identical but for the char type of the two strings.
#[repr(C)]
pub struct Node<T> {
    pub entry: list,
    pub pszLine: *mut T,
    pub pszComp: *mut T, // compressed
    pub lineno: DWORD,
    pub hash: DWORD,
}

/// LIST_ENTRY(ptr, NODE, entry). Wrapping, because the C applies it to a NULL `ptr` too
/// (TextCompare, after SkipIdentical ran off a list) and IsEOFNode then sees NULL.
#[inline]
fn node_of<T>(ptr: *mut list) -> *mut Node<T> {
    (ptr as *mut u8).wrapping_sub(core::mem::offset_of!(Node<T>, entry)) as *mut Node<T>
}

#[inline]
fn IS_SPACE<T: TChar>(ch: T) -> bool {
    ch == T::from_ascii(b' ') || ch == T::from_ascii(b'\t')
}

unsafe fn AllocLine<T: TChar>(pch: *const T, cch: DWORD) -> *mut T {
    let pszNew: *mut T =
        malloc((cch as usize).wrapping_add(1).wrapping_mul(core::mem::size_of::<T>())) as *mut T;
    if pszNew.is_null() {
        return core::ptr::null_mut();
    }
    memcpy(
        pszNew as *mut c_void,
        pch as *const c_void,
        (cch as usize).wrapping_mul(core::mem::size_of::<T>()),
    );
    *pszNew.add(cch as usize) = T::from_ascii(0);
    pszNew
}

unsafe fn AllocNode<T: TChar>(psz: *mut T, lineno: DWORD) -> *mut Node<T> {
    if psz.is_null() {
        return core::ptr::null_mut();
    }
    let node = calloc(1, core::mem::size_of::<Node<T>>()) as *mut Node<T>;
    if node.is_null() {
        free(psz as *mut c_void);
        return core::ptr::null_mut();
    }
    (*node).pszLine = psz;
    (*node).lineno = lineno;
    node
}

unsafe fn DeleteNode<T>(node: *mut Node<T>) {
    if !node.is_null() {
        free((*node).pszLine as *mut c_void);
        free((*node).pszComp as *mut c_void);
        free(node as *mut c_void);
    }
}

unsafe fn DeleteList<T>(list: *mut list) {
    loop {
        let ptr = list_head(list);
        if ptr.is_null() {
            break;
        }
        list_remove(ptr);
        DeleteNode(node_of::<T>(ptr));
    }
}

unsafe fn SkipSpace<T: TChar>(mut pch: *const T) -> *const T {
    while IS_SPACE(*pch) {
        pch = pch.add(1);
    }
    pch
}

unsafe fn FindLastNonSpace<T: TChar>(mut pch: *const T) -> *const T {
    let mut pchLast: *const T = core::ptr::null();
    while !(*pch).is_nul() {
        if !IS_SPACE(*pch) {
            pchLast = pch;
        }
        pch = pch.add(1);
    }
    pchLast
}

unsafe fn DeleteDuplicateSpaces<T: TChar>(psz: *mut T) {
    let mut pch0: *mut T = psz;
    let mut pch1: *mut T = psz;
    while !(*pch0).is_nul() {
        *pch1 = *pch0;
        pch1 = pch1.add(1);
        if IS_SPACE(*pch0) {
            loop {
                pch0 = pch0.add(1);
                if !IS_SPACE(*pch0) {
                    break;
                }
            }
            pch0 = pch0.sub(1);
        }
        pch0 = pch0.add(1);
    }
    *pch1 = T::from_ascii(0);
}

unsafe fn CompressSpace<T: TChar>(mut line: *const T) -> *mut T {
    line = SkipSpace(line);
    let pchLast = FindLastNonSpace(line);
    if pchLast.is_null() {
        return AllocLine::<T>(core::ptr::null(), 0);
    }
    let pszNew = AllocLine(line, (pchLast.offset_from(line) as DWORD).wrapping_add(1));
    if pszNew.is_null() {
        return core::ptr::null_mut();
    }
    DeleteDuplicateSpaces(pszNew);
    pszNew
}

unsafe fn ExpandTabLength<T: TChar>(line: *const T) -> INT {
    let mut pch = line;
    let mut cch: INT = 0;
    while !(*pch).is_nul() {
        if *pch == T::from_ascii(b'\t') {
            cch += TAB_WIDTH - (cch % TAB_WIDTH);
        } else {
            cch += 1;
        }
        pch = pch.add(1);
    }
    cch
}

unsafe fn ExpandTab<T: TChar>(line: *const T) -> *mut T {
    let mut spaces: INT;
    let cch: INT = ExpandTabLength(line);
    let mut ich: INT = 0;
    let pszNew = malloc(((cch + 1) as usize).wrapping_mul(core::mem::size_of::<T>())) as *mut T;
    if pszNew.is_null() {
        return core::ptr::null_mut();
    }
    let mut pch = line;
    while !(*pch).is_nul() {
        if *pch == T::from_ascii(b'\t') {
            spaces = TAB_WIDTH - (ich % TAB_WIDTH);
            // while (spaces-- > 0)
            loop {
                let before = spaces;
                spaces -= 1;
                if before <= 0 {
                    break;
                }
                *pszNew.offset(ich as isize) = T::from_ascii(b' ');
                ich += 1;
            }
        } else {
            *pszNew.offset(ich as isize) = *pch;
            ich += 1;
        }
        pch = pch.add(1);
    }
    *pszNew.offset(ich as isize) = T::from_ascii(0);
    pszNew
}

unsafe fn GetHash<T: TChar>(mut psz: *const T, bIgnoreCase: BOOL) -> DWORD {
    let mut ret: DWORD = 0xDEADFACE;
    while !(*psz).is_nul() {
        ret = ret.wrapping_add((*psz).hash_unit(bIgnoreCase != 0));
        ret <<= 2;
        psz = psz.add(1);
    }
    ret & HASH_MASK
}

unsafe fn AllocEOFNode<T: TChar>(lineno: DWORD) -> *mut Node<T> {
    let node = AllocNode(AllocLine::<T>(core::ptr::null(), 0), 0);
    if node.is_null() {
        return core::ptr::null_mut();
    }
    (*node).pszComp = AllocLine::<T>(core::ptr::null(), 0);
    if (*node).pszComp.is_null() {
        DeleteNode(node);
        return core::ptr::null_mut();
    }
    (*node).lineno = lineno;
    (*node).hash = HASH_EOF;
    node
}

unsafe fn IsEOFNode<T>(node: *const Node<T>) -> BOOL {
    (node.is_null() || (*node).hash == HASH_EOF) as BOOL
}

unsafe fn ConvertNode<T: TChar>(pFC: *const FILECOMPARE, node: *mut Node<T>) -> BOOL {
    if (*pFC).dwFlags & FLAG_T == 0 {
        let tmp = ExpandTab((*node).pszLine);
        if tmp.is_null() {
            return FALSE as BOOL;
        }
        free((*node).pszLine as *mut c_void);
        (*node).pszLine = tmp;
        if (*pFC).dwFlags & FLAG_W == 0 {
            (*node).hash = GetHash((*node).pszLine, ((*pFC).dwFlags & FLAG_C != 0) as BOOL);
        }
    }
    if (*pFC).dwFlags & FLAG_W != 0 {
        (*node).pszComp = CompressSpace((*node).pszLine);
        if (*node).pszComp.is_null() {
            return FALSE as BOOL;
        }
        (*node).hash = GetHash((*node).pszComp, ((*pFC).dwFlags & FLAG_C != 0) as BOOL);
    }
    TRUE as BOOL
}

unsafe fn CompareNode<T: TChar>(
    pFC: *const FILECOMPARE,
    node0: *const Node<T>,
    node1: *const Node<T>,
) -> FCRET {
    if (*node0).hash != (*node1).hash {
        return FCRET_DIFFERENT;
    }
    let compress = (*pFC).dwFlags & FLAG_W != 0;
    let psz0 = if compress { (*node0).pszComp } else { (*node0).pszLine };
    let psz1 = if compress { (*node1).pszComp } else { (*node1).pszLine };
    let dwCmpFlags: DWORD = if (*pFC).dwFlags & FLAG_C != 0 { NORM_IGNORECASE } else { 0 };
    let ret = T::compare_string(dwCmpFlags, psz0, psz1);
    if ret == CSTR_EQUAL as c_int {
        FCRET_IDENTICAL
    } else {
        FCRET_DIFFERENT
    }
}

unsafe fn FindNextLine<T: TChar>(pch: *const T, mut ich: DWORD, cch: DWORD, pich: LPDWORD) -> BOOL {
    while ich < cch {
        let c = *pch.add(ich as usize);
        if c == T::from_ascii(b'\n') || c.is_nul() {
            *pich = ich;
            return TRUE as BOOL;
        }
        ich = ich.wrapping_add(1);
    }
    *pich = cch;
    FALSE as BOOL
}

unsafe fn ParseLines<T: TChar>(
    pFC: *const FILECOMPARE,
    phMapping: *mut HANDLE,
    pib: *mut LARGE_INTEGER,
    pcb: *const LARGE_INTEGER,
    list: *mut list,
) -> FCRET {
    let mut lineno: DWORD = 1;
    // The C leaves ichNext uninitialised and reads it after the loop; with a view shorter
    // than one TCHAR (a 1-byte file compared /U) the loop never runs. 0 = nothing consumed.
    let mut ichNext: DWORD = 0;

    if (*phMapping).is_null() {
        return FCRET_FCRET_NO_MORE_DATA;
    }

    if (*pib).QuadPart >= (*pcb).QuadPart {
        CloseHandle(*phMapping);
        *phMapping = core::ptr::null_mut();
        return FCRET_FCRET_NO_MORE_DATA;
    }

    let cbView = core::cmp::min((*pcb).QuadPart - (*pib).QuadPart, MAX_VIEW_SIZE as LONGLONG)
        as DWORD;
    let psz = MapViewOfFile(
        *phMapping,
        FILE_MAP_READ,
        (*pib).u.HighPart as DWORD,
        (*pib).u.LowPart,
        cbView as SIZE_T,
    ) as *mut T;
    if psz.is_null() {
        return OutOfMemory();
    }

    let mut ich: DWORD = 0;
    let cch: DWORD = cbView / core::mem::size_of::<T>() as DWORD;
    let fLast = (*pib).QuadPart + cbView as LONGLONG >= (*pcb).QuadPart;
    while ich < cch
        && (FindNextLine(psz, ich, cch, &mut ichNext) != 0
            || (ichNext == cch && (fLast || ich == 0)))
    {
        let bCR = (ichNext > 0 && *psz.add(ichNext as usize - 1) == T::from_ascii(b'\r')) as DWORD;
        let cchNode = ichNext.wrapping_sub(ich).wrapping_sub(bCR);
        let pszLine = AllocLine(psz.add(ich as usize), cchNode);
        let node = AllocNode(pszLine, lineno);
        lineno = lineno.wrapping_add(1);
        if node.is_null() || ConvertNode(pFC, node) == 0 {
            DeleteNode(node);
            UnmapViewOfFile(psz as LPCVOID);
            return OutOfMemory();
        }
        list_add_tail(list, &mut (*node).entry);
        ich = ichNext.wrapping_add(1);
    }

    UnmapViewOfFile(psz as LPCVOID);

    // DIVERGENCE: the C advances by `ichNext * sizeof(WCHAR)`, but ichNext counts TCHARs
    // (`cch = cbView / sizeof(TCHAR)` above), so the ANSI instantiation skipped twice the
    // bytes it parsed. Only a file larger than MAX_VIEW_SIZE reaches a second view.
    (*pib).QuadPart += (ichNext as usize * core::mem::size_of::<T>()) as LONGLONG;

    if (*pib).QuadPart < (*pcb).QuadPart {
        return FCRET_IDENTICAL;
    }

    // append EOF node
    let node = AllocEOFNode::<T>(lineno);
    if node.is_null() {
        return OutOfMemory();
    }
    list_add_tail(list, &mut (*node).entry);

    FCRET_FCRET_NO_MORE_DATA
}

unsafe fn ShowDiff<T: TChar>(pFC: *mut FILECOMPARE, i: INT, mut begin: *mut list, end: *mut list) {
    let list: *mut list = &mut (*pFC).list[i as usize];
    let mut first: *mut list = core::ptr::null_mut();
    let mut last: *mut list = core::ptr::null_mut();
    PrintCaption((*pFC).file[i as usize]);
    if !begin.is_null() && !end.is_null() && !list_prev(list, begin).is_null() {
        begin = list_prev(list, begin);
    }
    while begin != end {
        let node = node_of::<T>(begin);
        if IsEOFNode(node) != 0 {
            break;
        }
        if first.is_null() {
            first = begin;
        }
        last = begin;
        if (*pFC).dwFlags & FLAG_A == 0 {
            T::print_line(pFC, (*node).lineno, (*node).pszLine);
        }
        begin = list_next(list, begin);
    }
    if (*pFC).dwFlags & FLAG_A != 0 && !first.is_null() {
        let node = node_of::<T>(first);
        T::print_line(pFC, (*node).lineno, (*node).pszLine);
        first = list_next(list, first);
        if first != last {
            if list_next(list, first) == last {
                let node = node_of::<T>(first);
                T::print_line(pFC, (*node).lineno, (*node).pszLine);
            } else {
                PrintDots();
            }
        }
        let node = node_of::<T>(last);
        T::print_line(pFC, (*node).lineno, (*node).pszLine);
    }
}

unsafe fn SkipIdentical<T: TChar>(
    pFC: *mut FILECOMPARE,
    pptr0: *mut *mut list,
    pptr1: *mut *mut list,
) {
    let mut ptr0 = *pptr0;
    let mut ptr1 = *pptr1;
    while !ptr0.is_null() && !ptr1.is_null() {
        let node0 = node_of::<T>(ptr0);
        let node1 = node_of::<T>(ptr1);
        if CompareNode(pFC, node0, node1) != FCRET_IDENTICAL {
            break;
        }
        ptr0 = list_next(&mut (*pFC).list[0], ptr0);
        ptr1 = list_next(&mut (*pFC).list[1], ptr1);
    }
    *pptr0 = ptr0;
    *pptr1 = ptr1;
}

unsafe fn SkipIdenticalN<T: TChar>(
    pFC: *mut FILECOMPARE,
    pptr0: *mut *mut list,
    pptr1: *mut *mut list,
    nnnn: DWORD,
    lineno0: DWORD,
    lineno1: DWORD,
) -> DWORD {
    let mut ptr0 = *pptr0;
    let mut ptr1 = *pptr1;
    let mut count: DWORD = 0;
    while !ptr0.is_null() && !ptr1.is_null() {
        let node0 = node_of::<T>(ptr0);
        let node1 = node_of::<T>(ptr1);
        if (*node0).lineno >= lineno0 {
            break;
        }
        if (*node1).lineno >= lineno1 {
            break;
        }
        if CompareNode(pFC, node0, node1) != FCRET_IDENTICAL {
            break;
        }
        ptr0 = list_next(&mut (*pFC).list[0], ptr0);
        ptr1 = list_next(&mut (*pFC).list[1], ptr1);
        count = count.wrapping_add(1);
        if count >= nnnn {
            break;
        }
    }
    *pptr0 = ptr0;
    *pptr1 = ptr1;
    count
}

unsafe fn ScanDiff<T: TChar>(
    pFC: *mut FILECOMPARE,
    pptr0: *mut *mut list,
    pptr1: *mut *mut list,
    lineno0: DWORD,
    lineno1: DWORD,
) -> FCRET {
    let mut ptr0 = *pptr0;
    let mut ptr1 = *pptr1;
    while !ptr0.is_null() && !ptr1.is_null() {
        let node0 = node_of::<T>(ptr0);
        let node1 = node_of::<T>(ptr1);
        if (*node0).lineno >= lineno0 {
            return FCRET_DIFFERENT;
        }
        if (*node1).lineno >= lineno1 {
            return FCRET_DIFFERENT;
        }
        let mut tmp0 = ptr0;
        let mut tmp1 = ptr1;
        // INT count = SkipIdenticalN(..., pFC->nnnn, ...): the DWORD result and the INT
        // parameter convert both ways, as C does.
        let count =
            SkipIdenticalN::<T>(pFC, &mut tmp0, &mut tmp1, (*pFC).nnnn as DWORD, lineno0, lineno1)
                as INT;
        if count >= (*pFC).nnnn {
            break;
        }
        if count > 0 {
            ptr0 = tmp0;
            ptr1 = tmp1;
        } else {
            ptr0 = list_next(&mut (*pFC).list[0], ptr0);
            ptr1 = list_next(&mut (*pFC).list[1], ptr1);
        }
    }
    *pptr0 = ptr0;
    *pptr1 = ptr1;
    FCRET_IDENTICAL
}

unsafe fn Resync<T: TChar>(
    pFC: *mut FILECOMPARE,
    pptr0: *mut *mut list,
    pptr1: *mut *mut list,
) -> FCRET {
    let mut save0: *mut list = core::ptr::null_mut();
    let mut save1: *mut list = core::ptr::null_mut();
    let list0: *mut list = &mut (*pFC).list[0];
    let list1: *mut list = &mut (*pFC).list[1];
    let mut min_penalty: INT = MAXLONG as INT;

    let node0 = node_of::<T>(*pptr0);
    let node1 = node_of::<T>(*pptr1);
    let lineno0 = (*node0).lineno.wrapping_add((*pFC).n as DWORD);
    let lineno1 = (*node1).lineno.wrapping_add((*pFC).n as DWORD);

    // ``If the files that you are comparing have more than pFC->n consecutive
    //   differing lines, FC cancels the comparison,,
    // ``If the number of matching lines in the files is less than pFC->nnnn,
    //   FC displays the matching lines as differences,,
    let mut ptr1 = list_next(list1, *pptr1);
    let mut i1: INT = 0;
    while !ptr1.is_null() {
        let node1 = node_of::<T>(ptr1);
        if (*node1).lineno >= lineno1 {
            break;
        }
        let mut ptr0 = list_next(list0, *pptr0);
        let mut i0: INT = 0;
        while !ptr0.is_null() {
            let node0 = node_of::<T>(ptr0);
            if (*node0).lineno >= lineno0 {
                break;
            }
            if CompareNode(pFC, node0, node1) == FCRET_IDENTICAL {
                let penalty = core::cmp::min(i0, i1) + abs(i1 - i0);
                if min_penalty > penalty {
                    min_penalty = penalty;
                    save0 = ptr0;
                    save1 = ptr1;
                }
            }
            ptr0 = list_next(list0, ptr0);
            i0 += 1;
        }
        ptr1 = list_next(list1, ptr1);
        i1 += 1;
    }

    if !save0.is_null() && !save1.is_null() {
        *pptr0 = save0;
        *pptr1 = save1;
        let ret = ScanDiff::<T>(pFC, &mut save0, &mut save1, lineno0, lineno1);
        if !save0.is_null() && !save1.is_null() {
            *pptr0 = save0;
            *pptr1 = save1;
        }
        return ret;
    }

    let mut ptr0 = *pptr0;
    while !ptr0.is_null() {
        if (*node_of::<T>(ptr0)).lineno == lineno0 {
            break;
        }
        ptr0 = list_next(list0, ptr0);
    }
    let mut ptr1 = *pptr1;
    while !ptr1.is_null() {
        if (*node_of::<T>(ptr1)).lineno == lineno1 {
            break;
        }
        ptr1 = list_next(list1, ptr1);
    }
    *pptr0 = ptr0;
    *pptr1 = ptr1;
    FCRET_DIFFERENT
}

unsafe fn Finalize<T: TChar>(
    pFC: *mut FILECOMPARE,
    ptr0: *mut list,
    ptr1: *mut list,
    fDifferent: BOOL,
) -> FCRET {
    if ptr0.is_null() && ptr1.is_null() {
        if fDifferent != 0 {
            return Different((*pFC).file[0], (*pFC).file[1]);
        }
        NoDifference()
    } else {
        ShowDiff::<T>(pFC, 0, ptr0, core::ptr::null_mut());
        ShowDiff::<T>(pFC, 1, ptr1, core::ptr::null_mut());
        PrintEndOfDiff();
        FCRET_DIFFERENT
    }
}

// FIXME: "cmd_apitest fc" has some failures.
pub unsafe fn TextCompare<T: TChar>(
    pFC: *mut FILECOMPARE,
    phMapping0: *mut HANDLE,
    pcb0: *const LARGE_INTEGER,
    phMapping1: *mut HANDLE,
    pcb1: *const LARGE_INTEGER,
) -> FCRET {
    let mut ret: FCRET;
    let mut ptr0: *mut list;
    let mut ptr1: *mut list;
    let mut fDifferent: BOOL = FALSE as BOOL;
    let mut ib0: LARGE_INTEGER = core::mem::zeroed();
    let mut ib1: LARGE_INTEGER = core::mem::zeroed();
    let list0: *mut list = &mut (*pFC).list[0];
    let list1: *mut list = &mut (*pFC).list[1];
    list_init(list0);
    list_init(list1);

    'cleanup: {
        // do { ... } while (ret0 != NO_MORE_DATA || ret1 != NO_MORE_DATA): the inner for(;;)
        // leaves only by `goto quit` or `goto cleanup`, so the condition is never reached and
        // the do runs once.
        let ret0 = ParseLines::<T>(pFC, phMapping0, &mut ib0, pcb0, list0);
        if ret0 == FCRET_INVALID {
            ret = ret0;
            break 'cleanup;
        }
        let ret1 = ParseLines::<T>(pFC, phMapping1, &mut ib1, pcb1, list1);
        if ret1 == FCRET_INVALID {
            ret = ret1;
            break 'cleanup;
        }

        ptr0 = list_head(list0);
        ptr1 = list_head(list1);
        loop {
            if ptr0.is_null() || ptr1.is_null() {
                break; // goto quit
            }

            // skip identical (sync'ed)
            SkipIdentical::<T>(pFC, &mut ptr0, &mut ptr1);
            if !ptr0.is_null() || !ptr1.is_null() {
                fDifferent = TRUE as BOOL;
            }
            if IsEOFNode(node_of::<T>(ptr0)) != 0 || IsEOFNode(node_of::<T>(ptr1)) != 0 {
                break; // goto quit
            }

            // try to resync
            let save0 = ptr0;
            let save1 = ptr1;
            ret = Resync::<T>(pFC, &mut ptr0, &mut ptr1);
            if ret == FCRET_INVALID {
                break 'cleanup;
            }
            if ret == FCRET_DIFFERENT {
                // resync failed
                ret = ResyncFailed();
                // show the difference
                ShowDiff::<T>(pFC, 0, save0, ptr0);
                ShowDiff::<T>(pFC, 1, save1, ptr1);
                PrintEndOfDiff();
                break 'cleanup;
            }

            // show the difference
            fDifferent = TRUE as BOOL;
            let next0 = if !ptr0.is_null() { list_next(list0, ptr0) } else { ptr0 };
            let next1 = if !ptr1.is_null() { list_next(list1, ptr1) } else { ptr1 };
            ShowDiff::<T>(pFC, 0, save0, if !next0.is_null() { next0 } else { ptr0 });
            ShowDiff::<T>(pFC, 1, save1, if !next1.is_null() { next1 } else { ptr1 });
            PrintEndOfDiff();

            // now resync'ed
        }
        // quit:
        ret = Finalize::<T>(pFC, ptr0, ptr1, fDifferent);
    }
    // cleanup:
    DeleteList::<T>(list0);
    DeleteList::<T>(list1);
    ret
}
