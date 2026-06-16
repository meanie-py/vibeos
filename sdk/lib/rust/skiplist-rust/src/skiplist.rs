use ntdll_sys::nt::{PVOID, DWORD, BOOL};

// sdk/lib/skiplist/CMakeLists.txt builds skiplist16 with -DSKIPLIST_LEVELS=16.
pub const SKIPLIST_LEVELS: usize = 16;

// --- Type definitions (must match sdk/lib/skiplist/skiplist.h exactly) ---

pub type PSKIPLIST_ALLOCATE_ROUTINE = unsafe extern "system" fn(DWORD) -> PVOID;
pub type PSKIPLIST_COMPARE_ROUTINE = unsafe extern "system" fn(PVOID, PVOID) -> i32;
pub type PSKIPLIST_FREE_ROUTINE = unsafe extern "system" fn(PVOID);

#[repr(C)]
pub struct SKIPLIST_NODE {
    pub Distance: [DWORD; SKIPLIST_LEVELS],
    pub Next: [*mut SKIPLIST_NODE; SKIPLIST_LEVELS],
    pub Element: PVOID,
}

#[repr(C)]
pub struct SKIPLIST {
    pub Head: SKIPLIST_NODE,
    pub MaximumLevel: i8,       // CHAR (signed)
    pub NodeCount: DWORD,
    pub AllocateRoutine: PSKIPLIST_ALLOCATE_ROUTINE,
    pub CompareRoutine: PSKIPLIST_COMPARE_ROUTINE,
    pub FreeRoutine: PSKIPLIST_FREE_ROUTINE,
}

// ============================================================================
// Internal helpers
// ============================================================================

unsafe fn _GetRandomLevel() -> i8 {
    static mut dwRandom: u32 = 1;
    let mut dwLevel: u32 = 0;
    let dwShifted: u32;

    dwRandom = ((dwRandom as u64 * 48271u64) % 2147483647u64) as u32;
    dwShifted = dwRandom >> (31 - SKIPLIST_LEVELS as u32);

    if dwShifted != 0 {
        // BitScanForward sets dwLevel to the zero-based position of the
        // first set bit (from LSB to MSB).
        dwLevel = dwShifted.trailing_zeros();
    }

    return dwLevel as i8;
}

unsafe fn _InsertElementSkiplistWithInformation(
    Skiplist: *mut SKIPLIST,
    Element: PVOID,
    pUpdate: *mut *mut SKIPLIST_NODE,
    dwDistance: *mut DWORD,
) -> BOOL {
    let mut chNewLevel: i8;
    let mut i: i8;
    let mut pNode: *mut SKIPLIST_NODE;

    chNewLevel = _GetRandomLevel();

    if chNewLevel > (*Skiplist).MaximumLevel {
        let mut i = (*Skiplist).MaximumLevel + 1;
        while i <= chNewLevel {
            *pUpdate.add(i as usize) = core::ptr::addr_of_mut!((*Skiplist).Head);
            (*(*pUpdate.add(i as usize))).Distance[i as usize] = (*Skiplist).NodeCount + 1;
            i += 1;
        }
        (*Skiplist).MaximumLevel = chNewLevel;
    }

    pNode = ((*Skiplist).AllocateRoutine)(
        core::mem::size_of::<SKIPLIST_NODE>() as DWORD,
    ) as *mut SKIPLIST_NODE;
    if pNode.is_null() {
        return 0; // FALSE
    }

    (*pNode).Element = Element;

    let mut i = 0i8;
    while i <= chNewLevel {
        let node_ptr = *pUpdate.add(i as usize);
        (*pNode).Next[i as usize] = (*node_ptr).Next[i as usize];
        (*node_ptr).Next[i as usize] = pNode;
        (*pNode).Distance[i as usize] = *dwDistance.add(i as usize)
            + ((*node_ptr).Distance[i as usize] - *dwDistance.add(0));
        (*node_ptr).Distance[i as usize] =
            *dwDistance.add(0) + 1 - *dwDistance.add(i as usize);
        i += 1;
    }

    let mut i = chNewLevel + 1;
    while i <= (*Skiplist).MaximumLevel {
        let node_ptr = *pUpdate.add(i as usize);
        (*node_ptr).Distance[i as usize] += 1;
        i += 1;
    }

    (*Skiplist).NodeCount += 1;
    return 1; // TRUE
}

// ============================================================================
// Public API (WINAPI / stdcall — i386 needs _Name@N decoration)
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn DeleteElementSkiplist(
    Skiplist: *mut SKIPLIST,
    Element: PVOID,
) -> PVOID {
    let mut i: i8;
    let mut pLastComparedNode: *mut SKIPLIST_NODE = core::ptr::null_mut();
    let mut pNode: *mut SKIPLIST_NODE = core::ptr::addr_of_mut!((*Skiplist).Head);
    let mut pUpdate: [*mut SKIPLIST_NODE; SKIPLIST_LEVELS] =
        [core::ptr::null_mut(); SKIPLIST_LEVELS];
    let pReturnValue: PVOID;

    i = ((*Skiplist).MaximumLevel + 1) as i8;
    loop {
        i -= 1;
        if i < 0 {
            break;
        }
        while !(*pNode).Next[i as usize].is_null()
            && (*pNode).Next[i as usize] != pLastComparedNode
            && ((*Skiplist).CompareRoutine)((*(*pNode).Next[i as usize]).Element, Element) < 0
        {
            pNode = (*pNode).Next[i as usize];
        }
        pLastComparedNode = (*pNode).Next[i as usize];
        pUpdate[i as usize] = pNode;
    }
    pNode = (*pNode).Next[0];
    if pNode.is_null() || ((*Skiplist).CompareRoutine)((*pNode).Element, Element) != 0 {
        return core::ptr::null_mut();
    }
    i = 0;
    while i <= (*Skiplist).MaximumLevel as i8 && (*pUpdate[i as usize]).Next[i as usize] == pNode {
        (*pUpdate[i as usize]).Distance[i as usize] += (*pNode).Distance[i as usize] - 1;
        (*pUpdate[i as usize]).Next[i as usize] = (*pNode).Next[i as usize];
        i += 1;
    }
    while i <= (*Skiplist).MaximumLevel as i8 {
        (*pUpdate[i as usize]).Distance[i as usize] -= 1;
        i += 1;
    }
    pReturnValue = (*pNode).Element;
    ((*Skiplist).FreeRoutine)(pNode as PVOID);
    while (*Skiplist).MaximumLevel > 0
        && (*Skiplist).Head.Next[(*Skiplist).MaximumLevel as usize].is_null()
    {
        (*Skiplist).MaximumLevel -= 1;
    }
    (*Skiplist).NodeCount -= 1;
    pReturnValue
}

#[no_mangle]
pub unsafe extern "C" fn InitializeSkiplist(
    Skiplist: *mut SKIPLIST,
    AllocateRoutine: PSKIPLIST_ALLOCATE_ROUTINE,
    CompareRoutine: PSKIPLIST_COMPARE_ROUTINE,
    FreeRoutine: PSKIPLIST_FREE_ROUTINE,
) {
    // Store the routines.
    (*Skiplist).AllocateRoutine = AllocateRoutine;
    (*Skiplist).CompareRoutine = CompareRoutine;
    (*Skiplist).FreeRoutine = FreeRoutine;

    // Initialize the members and pointers.
    // The Distance array is only used when a node is non-NULL, so it
    // doesn't need initialization.
    (*Skiplist).MaximumLevel = 0;
    (*Skiplist).NodeCount = 0;
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*Skiplist).Head.Next).cast::<*mut SKIPLIST_NODE>(),
        0u8,
        SKIPLIST_LEVELS,
    );
}

#[no_mangle]
pub unsafe extern "C" fn InsertElementSkiplist(
    Skiplist: *mut SKIPLIST,
    Element: PVOID,
) -> BOOL {
    let mut i: i8;
    let mut dwDistance: [DWORD; SKIPLIST_LEVELS + 1] = [0; SKIPLIST_LEVELS + 1];
    let mut pLastComparedNode: *mut SKIPLIST_NODE = core::ptr::null_mut();
    let mut pNode: *mut SKIPLIST_NODE = core::ptr::addr_of_mut!((*Skiplist).Head);
    let mut pUpdate: [*mut SKIPLIST_NODE; SKIPLIST_LEVELS] =
        [core::ptr::null_mut(); SKIPLIST_LEVELS];

    i = ((*Skiplist).MaximumLevel as i8) + 1;
    while {
        i -= 1;
        i >= 0
    } {
        dwDistance[i as usize] = dwDistance[(i + 1) as usize];
        while !((*pNode).Next[i as usize]).is_null()
            && (*pNode).Next[i as usize] != pLastComparedNode
            && ((*Skiplist).CompareRoutine)((*(*pNode).Next[i as usize]).Element, Element) < 0
        {
            dwDistance[i as usize] += (*pNode).Distance[i as usize];
            pNode = (*pNode).Next[i as usize];
        }
        pLastComparedNode = (*pNode).Next[i as usize];
        pUpdate[i as usize] = pNode;
    }

    pNode = (*pNode).Next[0];
    if !pNode.is_null() && ((*Skiplist).CompareRoutine)((*pNode).Element, Element) == 0 {
        return 0; // FALSE
    }

    _InsertElementSkiplistWithInformation(
        Skiplist,
        Element,
        pUpdate.as_mut_ptr(),
        dwDistance.as_mut_ptr(),
    )
}

#[no_mangle]
pub unsafe extern "C" fn InsertTailElementSkiplist(
    Skiplist: *mut SKIPLIST,
    Element: PVOID,
) -> BOOL {
    let mut i: i8;
    let mut dwDistance: [DWORD; SKIPLIST_LEVELS + 1] = [0; SKIPLIST_LEVELS + 1];
    let mut pNode: *mut SKIPLIST_NODE = core::ptr::addr_of_mut!((*Skiplist).Head);
    let mut pUpdate: [*mut SKIPLIST_NODE; SKIPLIST_LEVELS] =
        [core::ptr::null_mut(); SKIPLIST_LEVELS];

    i = ((*Skiplist).MaximumLevel as i8) + 1;
    loop {
        i -= 1;
        if i < 0 {
            break;
        }
        dwDistance[i as usize] = dwDistance[(i + 1) as usize];
        while !(*pNode).Next[i as usize].is_null() {
            dwDistance[i as usize] = dwDistance[i as usize]
                .wrapping_add((*pNode).Distance[i as usize]);
            pNode = (*pNode).Next[i as usize];
        }
        pUpdate[i as usize] = pNode;
    }

    _InsertElementSkiplistWithInformation(
        Skiplist,
        Element,
        pUpdate.as_mut_ptr(),
        dwDistance.as_mut_ptr(),
    )
}

#[no_mangle]
pub unsafe extern "C" fn LookupElementSkiplist(
    Skiplist: *mut SKIPLIST,
    Element: PVOID,
    ElementIndex: *mut DWORD,
) -> PVOID {
    let mut i: i8;
    let mut dwIndex: DWORD = 0;
    let mut pLastComparedNode: *mut SKIPLIST_NODE = core::ptr::null_mut();
    let mut pNode: *mut SKIPLIST_NODE = core::ptr::addr_of_mut!((*Skiplist).Head);

    i = ((*Skiplist).MaximumLevel as i8) + 1;
    while {
        i -= 1;
        i >= 0
    } {
        loop {
            let next = (*pNode).Next[i as usize];
            if next.is_null() {
                break;
            }
            if next == pLastComparedNode {
                break;
            }
            if ((*Skiplist).CompareRoutine)((*next).Element, Element) >= 0 {
                break;
            }
            dwIndex = dwIndex.wrapping_add((*pNode).Distance[i as usize]);
            pNode = next;
        }
        pLastComparedNode = (*pNode).Next[i as usize];
    }

    pNode = (*pNode).Next[0];
    if pNode.is_null() || ((*Skiplist).CompareRoutine)((*pNode).Element, Element) != 0 {
        return core::ptr::null_mut();
    }

    if !ElementIndex.is_null() {
        core::ptr::write_volatile(ElementIndex, dwIndex);
    }

    (*pNode).Element
}

#[no_mangle]
pub unsafe extern "C" fn LookupNodeByIndexSkiplist(
    Skiplist: *mut SKIPLIST,
    ElementIndex: DWORD,
) -> *mut SKIPLIST_NODE {
    let mut i: i8;
    let mut dwIndex: DWORD = 0;
    let mut pNode: *mut SKIPLIST_NODE = core::ptr::addr_of_mut!((*Skiplist).Head);

    if ElementIndex >= (*Skiplist).NodeCount {
        return core::ptr::null_mut();
    }

    i = ((*Skiplist).MaximumLevel as i8) + 1;
    while {
        i -= 1;
        i >= 0
    } {
        while !((*pNode).Next[i as usize]).is_null()
            && dwIndex + (*pNode).Distance[i as usize] <= ElementIndex
        {
            dwIndex += (*pNode).Distance[i as usize];
            pNode = (*pNode).Next[i as usize];
        }
    }

    return (*pNode).Next[0];
}
