use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::*;

use ntkrnl_sys::ddk::{
    PsGetCurrentThread,
    _KGUARDED_MUTEX, PKGUARDED_MUTEX,
    MM_AVL_TABLE, PMM_AVL_TABLE, PMMADDRESS_NODE, PMMVAD, PMMVAD_LONG,
    PETHREAD, PSECTION, KPROCESSOR_MODE,
    MiCheckForConflictingNode as ddk_MiCheckForConflictingNode,
    MiCheckSecuredVad as ddk_MiCheckSecuredVad,
    MiFindEmptyAddressRangeDownBasedTree as ddk_MiFindEmptyAddressRangeDownBasedTree,
    MiFindEmptyAddressRangeDownTree as ddk_MiFindEmptyAddressRangeDownTree,
    MiFindEmptyAddressRangeInTree as ddk_MiFindEmptyAddressRangeInTree,
    MiGetNextNode as ddk_MiGetNextNode,
    MiGetPreviousNode as ddk_MiGetPreviousNode,
    MiInsertBasedSection as ddk_MiInsertBasedSection,
    MiInsertNode as ddk_MiInsertNode,
    MiInsertVad as ddk_MiInsertVad,
    MiInsertVadEx as ddk_MiInsertVadEx,
    MiLocateAddress as ddk_MiLocateAddress,
    MiLocateVad as ddk_MiLocateVad,
    MiRemoveNode as ddk_MiRemoveNode,
    MiLockProcessWorkingSetUnsafe, MiUnlockProcessWorkingSetUnsafe,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    static mut MiRosKernelVadRoot: MM_AVL_TABLE;
    static mut MmSectionBasedRoot: MM_AVL_TABLE;
    static mut MmSectionBasedMutex: KGUARDED_MUTEX;
    static mut PsIdleProcess: PEPROCESS;
    fn RtlpFindAvlTableNodeOrParent( Table: PMM_AVL_TABLE, Vpn: PVOID, NodeOrParent: *mut PMMADDRESS_NODE, ) -> TABLE_SEARCH_RESULT;
    fn RtlpInsertAvlTreeNode( Table: PMM_AVL_TABLE, NewNode: PMMADDRESS_NODE, Parent: PMMADDRESS_NODE, Result: TABLE_SEARCH_RESULT, );
    fn RtlpDeleteAvlTreeNode(Table: PMM_AVL_TABLE, Node: PMMADDRESS_NODE);
    fn RtlLeftChildAvl(Node: PMMADDRESS_NODE) -> PMMADDRESS_NODE;
    fn RtlRightChildAvl(Node: PMMADDRESS_NODE) -> PMMADDRESS_NODE;
    fn RtlParentAvl(Node: PMMADDRESS_NODE) -> PMMADDRESS_NODE;
    fn RtlIsRightChildAvl(Node: PMMADDRESS_NODE) -> BOOLEAN;
    fn RtlIsLeftChildAvl(Node: PMMADDRESS_NODE) -> BOOLEAN;
    fn MI_WS_OWNER(Process: PEPROCESS) -> BOOLEAN;
}


type TABLE_SEARCH_RESULT = i32;
const TableEmptyTree: TABLE_SEARCH_RESULT = 0;
const TableFoundNode: TABLE_SEARCH_RESULT = 1;
const TableInsertAsLeft: TABLE_SEARCH_RESULT = 2;
const TableInsertAsRight: TABLE_SEARCH_RESULT = 3;

#[no_mangle]
pub static mut MmReadWrite: [i8; 32] = [
    MM_NO_ACCESS_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8,
    MM_READ_ONLY_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
    MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
    MM_NO_ACCESS_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8,
    MM_READ_ONLY_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
    MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
    MM_NO_ACCESS_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8,
    MM_READ_ONLY_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
    MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
    MM_NO_ACCESS_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8,
    MM_READ_ONLY_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
    MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
];

#[no_mangle]
pub unsafe extern "system" fn MiLocateVad(Table: PMM_AVL_TABLE, VirtualAddress: PVOID) -> PMMVAD {
    let VirtualAddress = opaque_ptr(VirtualAddress);
    let mut found_vad: PMMVAD;
    let vpn: ULONG_PTR;
    let mut search_result: TABLE_SEARCH_RESULT;

    found_vad = (*Table).NodeHint as PMMVAD;
    if found_vad.is_null() {
        return core::ptr::null_mut();
    }

    vpn = VirtualAddress as ULONG_PTR >> PAGE_SHIFT;
    if (vpn >= ((*found_vad).StartingVpn as ULONG_PTR)) && (vpn <= ((*found_vad).EndingVpn as ULONG_PTR)) {
        return found_vad;
    }

    search_result = RtlpFindAvlTableNodeOrParent(
        Table,
        vpn as PVOID,
        &mut found_vad as *mut PMMVAD as *mut PMMADDRESS_NODE,
    );
    if search_result != TableFoundNode {
        return core::ptr::null_mut();
    }

    (*Table).NodeHint = found_vad as PVOID;
    return found_vad;
}

#[no_mangle]
pub unsafe extern "system" fn MiLocateAddress(VirtualAddress: PVOID) -> PMMVAD {
    let VirtualAddress = opaque_ptr(VirtualAddress);
    let table: PMM_AVL_TABLE = &mut (*(PsGetCurrentProcess() as *mut ntkrnl_sys::ddk::_EPROCESS)).VadRoot;
    return MiLocateVad(table, VirtualAddress);
}

#[no_mangle]
pub unsafe extern "system" fn MiCheckForConflictingNode(
    StartVpn: ULONG_PTR,
    EndVpn: ULONG_PTR,
    Table: PMM_AVL_TABLE,
    NodeOrParent: *mut PMMADDRESS_NODE,
) -> TABLE_SEARCH_RESULT {
    let NodeOrParent = opaque_ptr(NodeOrParent);
    let mut parent_node: PMMADDRESS_NODE = core::ptr::null_mut();
    let mut current_node: PMMADDRESS_NODE;

    if (*Table).NumberGenericTableElements() == 0 {
        return TableEmptyTree;
    }

    current_node = RtlRightChildAvl(&mut (*Table).BalancedRoot as PMMADDRESS_NODE);
    while !current_node.is_null() {
        parent_node = current_node;

        if StartVpn > ((*current_node).EndingVpn as ULONG_PTR) {
            current_node = RtlRightChildAvl(current_node);
        } else if EndVpn < ((*current_node).StartingVpn as ULONG_PTR) {
            current_node = RtlLeftChildAvl(current_node);
        } else {
            *NodeOrParent = parent_node;
            return TableFoundNode;
        }
    }

    *NodeOrParent = parent_node;
    if StartVpn > ((*parent_node).EndingVpn as ULONG_PTR) {
        return TableInsertAsRight;
    } else {
        return TableInsertAsLeft;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiInsertNode(
    Table: PMM_AVL_TABLE,
    NewNode: PMMADDRESS_NODE,
    Parent: PMMADDRESS_NODE,
    Result: TABLE_SEARCH_RESULT,
) {
    RtlpInsertAvlTreeNode(Table, NewNode, Parent, Result);
}

#[no_mangle]
pub unsafe extern "system" fn MiInsertVad(Vad: PMMVAD, VadRoot: PMM_AVL_TABLE) {
    let Vad = opaque_ptr(Vad);
    let mut result: TABLE_SEARCH_RESULT;
    let mut parent: PMMADDRESS_NODE = core::ptr::null_mut();

    (*VadRoot).NodeHint = Vad as PVOID;

    result = RtlpFindAvlTableNodeOrParent(VadRoot, ((*Vad).StartingVpn as ULONG_PTR) as PVOID, &mut parent);

    MiInsertNode(VadRoot, Vad as PMMADDRESS_NODE, parent, result);
}

#[no_mangle]
pub unsafe extern "system" fn MiInsertVadEx(
    Vad: PMMVAD,
    BaseAddress: *mut ULONG_PTR,
    ViewSize: SIZE_T,
    HighestAddress: ULONG_PTR,
    Alignment: ULONG_PTR,
    AllocationType: ULONG,
) -> NTSTATUS {
    let Vad = opaque_ptr(Vad);
    let BaseAddress = opaque_ptr(BaseAddress);
    let mut starting_address: ULONG_PTR = 0;
    let mut ending_address: ULONG_PTR = 0;
    let current_process: PEPROCESS;
    let current_thread: PETHREAD;
    let mut result: TABLE_SEARCH_RESULT;
    let mut parent: PMMADDRESS_NODE = core::ptr::null_mut();
    let mut view_size = ViewSize;

    view_size = ALIGN_UP_BY(view_size, PAGE_SIZE);

    current_process = PsGetCurrentProcess();

    KeAcquireGuardedMutex(core::ptr::addr_of_mut!((*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).AddressCreationLock).cast::<KGUARDED_MUTEX>());
    if (*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.VmDeleted() != 0 {
        KeReleaseGuardedMutex(core::ptr::addr_of_mut!((*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).AddressCreationLock).cast::<KGUARDED_MUTEX>());
        return STATUS_PROCESS_IS_TERMINATING;
    }

    if *BaseAddress == 0 {
        let mut highest_address = HighestAddress;
        if highest_address > MM_HIGHEST_VAD_ADDRESS as ULONG_PTR {
            highest_address = MM_HIGHEST_VAD_ADDRESS as ULONG_PTR;
        }

        if ((AllocationType & MEM_TOP_DOWN as ULONG) != 0) || ((*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.VmTopDown() != 0) {
            result = MiFindEmptyAddressRangeDownTree(
                view_size,
                highest_address,
                Alignment,
                &mut (*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).VadRoot,
                &mut starting_address,
                &mut parent,
            );
        } else {
            result = MiFindEmptyAddressRangeInTree(
                view_size,
                Alignment,
                &mut (*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).VadRoot,
                &mut parent,
                &mut starting_address,
            );
        }

        ending_address = starting_address + view_size - 1;

        if (result == TableFoundNode) || (ending_address > highest_address) {
            KeReleaseGuardedMutex(core::ptr::addr_of_mut!((*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).AddressCreationLock).cast::<KGUARDED_MUTEX>());
            return STATUS_NO_MEMORY;
        }
    } else {
        starting_address = ALIGN_DOWN_BY(*BaseAddress, Alignment);
        ending_address = starting_address + view_size - 1;

        result = MiCheckForConflictingNode(
            starting_address >> PAGE_SHIFT,
            ending_address >> PAGE_SHIFT,
            &mut (*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).VadRoot,
            &mut parent,
        );
        if result == TableFoundNode {
            KeReleaseGuardedMutex(core::ptr::addr_of_mut!((*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).AddressCreationLock).cast::<KGUARDED_MUTEX>());
            return STATUS_CONFLICTING_ADDRESSES;
        }
    }

    (*Vad).StartingVpn = (starting_address >> PAGE_SHIFT) as _;
    (*Vad).EndingVpn = (ending_address >> PAGE_SHIFT) as _;

    if ((*Vad).u.VadFlags.PrivateMemory() != 0 && (*Vad).u.VadFlags.MemCommit() != 0) ||
        ((*Vad).u.VadFlags.PrivateMemory() == 0 && ((*Vad).u.VadFlags.Protection() as ULONG & PAGE_WRITECOPY) != 0)
    {
        (*Vad).u.VadFlags.set_CommitCharge((view_size / PAGE_SIZE) as _);
    }

    if (*Vad).u2.VadFlags2.OneSecured() != 0 {
        (*(Vad as PMMVAD_LONG)).u3.Secured.StartVpn = starting_address as _;
        (*(Vad as PMMVAD_LONG)).u3.Secured.EndVpn = ending_address as _;
    }

    current_thread = PsGetCurrentThread();
    MiLockProcessWorkingSetUnsafe(current_process as ntkrnl_sys::ddk::PEPROCESS, current_thread as ntkrnl_sys::ddk::PETHREAD);

    (*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).VadRoot.NodeHint = Vad as PVOID;
    MiInsertNode(&mut (*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).VadRoot, Vad as PMMADDRESS_NODE, parent, result);

    MiUnlockProcessWorkingSetUnsafe(current_process as ntkrnl_sys::ddk::PEPROCESS, current_thread as ntkrnl_sys::ddk::PETHREAD);

    (*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).VirtualSize += view_size as ntkrnl_sys::ddk::SIZE_T;
    if (*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).VirtualSize > (*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).PeakVirtualSize {
        (*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).PeakVirtualSize = (*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).VirtualSize;
    }

    KeReleaseGuardedMutex(core::ptr::addr_of_mut!((*(current_process as *mut ntkrnl_sys::ddk::_EPROCESS)).AddressCreationLock).cast::<KGUARDED_MUTEX>());

    *BaseAddress = starting_address;
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn MiInsertBasedSection(Section: PSECTION) {
    let mut result: TABLE_SEARCH_RESULT;
    let mut parent: PMMADDRESS_NODE = core::ptr::null_mut();

    result = RtlpFindAvlTableNodeOrParent(
        &mut MmSectionBasedRoot,
        ((*Section).Address.StartingVpn as usize) as PVOID,
        &mut parent,
    );
    MiInsertNode(&mut MmSectionBasedRoot, &mut (*Section).Address as PMMADDRESS_NODE, parent, result);
}

#[no_mangle]
pub unsafe extern "system" fn MiRemoveNode(Node: PMMADDRESS_NODE, Table: PMM_AVL_TABLE) {
    RtlpDeleteAvlTreeNode(Table, Node);

    { let _ngte = (*Table).NumberGenericTableElements(); (*Table).set_NumberGenericTableElements(_ngte - 1); }

    if (*Table).NodeHint == Node as PVOID {
        if (*Table).NumberGenericTableElements() == 0 {
            (*Table).NodeHint = core::ptr::null_mut();
        } else {
            (*Table).NodeHint = (*Table).BalancedRoot.RightChild as PVOID;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn MiGetPreviousNode(Node: PMMADDRESS_NODE) -> PMMADDRESS_NODE {
    let mut parent: PMMADDRESS_NODE;
    let mut node = Node;

    let left_child = RtlLeftChildAvl(node);
    if !left_child.is_null() {
        node = left_child;
        loop {
            let right_child = RtlRightChildAvl(node);
            if right_child.is_null() {
                break;
            }
            node = right_child;
        }
        return node;
    }

    parent = RtlParentAvl(node);
    while parent != node {
        if RtlIsRightChildAvl(node) != 0 {
            if parent == RtlParentAvl(parent) {
                parent = core::ptr::null_mut();
            }
            return parent;
        }

        node = parent;
        parent = RtlParentAvl(node);
    }

    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn MiGetNextNode(Node: PMMADDRESS_NODE) -> PMMADDRESS_NODE {
    let mut parent: PMMADDRESS_NODE;
    let mut node = Node;

    let right_child = RtlRightChildAvl(node);
    if !right_child.is_null() {
        node = right_child;
        loop {
            let left_child = RtlLeftChildAvl(node);
            if left_child.is_null() {
                break;
            }
            node = left_child;
        }
        return node;
    }

    parent = RtlParentAvl(node);
    while parent != node {
        if RtlIsLeftChildAvl(node) != 0 {
            return parent;
        }

        node = parent;
        parent = RtlParentAvl(node);
    }

    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn MiFindEmptyAddressRangeInTree(
    Length: SIZE_T,
    Alignment: ULONG_PTR,
    Table: PMM_AVL_TABLE,
    PreviousVad: *mut PMMADDRESS_NODE,
    Base: *mut ULONG_PTR,
) -> TABLE_SEARCH_RESULT {
    let PreviousVad = opaque_ptr(PreviousVad);
    let Base = opaque_ptr(Base);
    let mut node: PMMADDRESS_NODE;
    let mut previous_node: PMMADDRESS_NODE;
    let page_count: ULONG_PTR;
    let alignment_vpn: ULONG_PTR;
    let mut low_vpn: ULONG_PTR;
    let mut highest_vpn: ULONG_PTR;

    page_count = BYTES_TO_PAGES(Length);
    alignment_vpn = Alignment >> PAGE_SHIFT;
    low_vpn = ALIGN_UP_BY(MM_LOWEST_USER_ADDRESS as ULONG_PTR >> PAGE_SHIFT, alignment_vpn);

    if (*Table).Unused() == 1 {
        low_vpn = ALIGN_UP_BY(MmSystemRangeStart as ULONG_PTR >> PAGE_SHIFT, alignment_vpn);
    }

    if (*Table).NumberGenericTableElements() == 0 {
        *Base = low_vpn << PAGE_SHIFT;
        return TableEmptyTree;
    }

    node = RtlRightChildAvl(&mut (*Table).BalancedRoot as PMMADDRESS_NODE);
    loop {
        let left_child = RtlLeftChildAvl(node);
        if left_child.is_null() {
            break;
        }
        node = left_child;
    }

    previous_node = core::ptr::null_mut();
    while !node.is_null() {
        if ((*node).StartingVpn as ULONG_PTR) >= low_vpn + page_count {
            *Base = low_vpn << PAGE_SHIFT;

            if RtlLeftChildAvl(node).is_null() {
                *PreviousVad = node;
                return TableInsertAsLeft;
            } else {
                *PreviousVad = previous_node;
                return TableInsertAsRight;
            }
        }

        if ((*node).EndingVpn as ULONG_PTR) >= low_vpn {
            low_vpn = ALIGN_UP_BY(((*node).EndingVpn as ULONG_PTR) + 1, alignment_vpn);
        }

        previous_node = node;
        node = MiGetNextNode(node);
    }

    highest_vpn = ((MM_HIGHEST_VAD_ADDRESS as ULONG_PTR) + 1) / PAGE_SIZE;

    if (*Table).Unused() == 1 {
        highest_vpn = ALIGN_UP_BY(((-1isize) as ULONG_PTR) >> PAGE_SHIFT, alignment_vpn);
    }

    if highest_vpn >= low_vpn + page_count {
        *PreviousVad = previous_node;
        *Base = low_vpn << PAGE_SHIFT;
        return TableInsertAsRight;
    }

    return TableFoundNode;
}

#[no_mangle]
pub unsafe extern "system" fn MiFindEmptyAddressRangeDownTree(
    Length: SIZE_T,
    BoundaryAddress: ULONG_PTR,
    Alignment: ULONG_PTR,
    Table: PMM_AVL_TABLE,
    Base: *mut ULONG_PTR,
    Parent: *mut PMMADDRESS_NODE,
) -> TABLE_SEARCH_RESULT {
    let Base = opaque_ptr(Base);
    let Parent = opaque_ptr(Parent);
    let mut node: PMMADDRESS_NODE;
    let mut old_node: PMMADDRESS_NODE = core::ptr::null_mut();
    let mut child: PMMADDRESS_NODE;
    let mut low_vpn: ULONG_PTR;
    let mut high_vpn: ULONG_PTR;
    let alignment_vpn: ULONG_PTR;
    let page_count: PFN_NUMBER;
    let mut length = Length;

    length = ROUND_TO_PAGES(length);
    page_count = length >> PAGE_SHIFT;
    alignment_vpn = Alignment / PAGE_SIZE;

    if (*Table).Unused() == 1 {
        low_vpn = ALIGN_UP_BY(MmSystemRangeStart as ULONG_PTR >> PAGE_SHIFT, alignment_vpn);
    } else {
        low_vpn = ALIGN_UP_BY(MM_LOWEST_USER_ADDRESS as ULONG_PTR, Alignment);
    }

    if (low_vpn + length) > (BoundaryAddress + 1) {
        return TableFoundNode;
    }

    if (*Table).NumberGenericTableElements() == 0 {
        *Base = ALIGN_DOWN_BY(BoundaryAddress + 1 - length, Alignment);
        return TableEmptyTree;
    }

    high_vpn = (BoundaryAddress + 1) >> PAGE_SHIFT;

    node = RtlRightChildAvl(&mut (*Table).BalancedRoot as PMMADDRESS_NODE);
    loop {
        if ((*node).EndingVpn as ULONG_PTR) >= high_vpn {
            break;
        }
        child = RtlRightChildAvl(node);
        if child.is_null() {
            break;
        }
        node = child;
    }

    while !node.is_null() {
        low_vpn = ALIGN_UP_BY(((*node).EndingVpn as ULONG_PTR) + 1, alignment_vpn);

        if (high_vpn > low_vpn) && ((high_vpn - low_vpn) >= page_count) {
            low_vpn = ALIGN_DOWN_BY(high_vpn - page_count, alignment_vpn);
            *Base = low_vpn << PAGE_SHIFT;

            if RtlRightChildAvl(node).is_null() {
                *Parent = node;
                return TableInsertAsRight;
            } else {
                *Parent = old_node;
                return TableInsertAsLeft;
            }
        }

        if ((*node).StartingVpn as ULONG_PTR) < high_vpn {
            high_vpn = ((*node).StartingVpn as ULONG_PTR);
        }

        old_node = node;
        node = MiGetPreviousNode(node);
    }

    low_vpn = ALIGN_UP_BY(MI_LOWEST_VAD_ADDRESS as ULONG_PTR, Alignment) / PAGE_SIZE;
    if (high_vpn > low_vpn) && ((high_vpn - low_vpn) >= page_count) {
        low_vpn = ALIGN_DOWN_BY(high_vpn - page_count, Alignment >> PAGE_SHIFT);
        *Base = low_vpn << PAGE_SHIFT;
        *Parent = old_node;
        return TableInsertAsLeft;
    }

    *Base = 0;
    *Parent = core::ptr::null_mut();
    return TableFoundNode;
}

#[no_mangle]
pub unsafe extern "system" fn MiFindEmptyAddressRangeDownBasedTree(
    Length: SIZE_T,
    BoundaryAddress: ULONG_PTR,
    Alignment: ULONG_PTR,
    Table: PMM_AVL_TABLE,
    Base: *mut ULONG_PTR,
) -> NTSTATUS {
    let Base = opaque_ptr(Base);
    let mut node: PMMADDRESS_NODE;
    let mut lowest_node: PMMADDRESS_NODE;
    let mut low_vpn: ULONG_PTR;
    let best_vpn: ULONG_PTR;
    let mut length = Length;

    length = ROUND_TO_PAGES(length);
    if (BoundaryAddress + 1) < length {
        return STATUS_NO_MEMORY;
    }

    best_vpn = ROUND_DOWN(BoundaryAddress + 1 - length, Alignment);
    if (*Table).NumberGenericTableElements() == 0 {
        *Base = best_vpn;
        return STATUS_SUCCESS;
    }

    node = ((*Table).BalancedRoot.RightChild as PMMADDRESS_NODE) as PMMADDRESS_NODE;
    loop {
        let right_child = RtlRightChildAvl(node);
        if right_child.is_null() {
            break;
        }
        node = right_child;
    }

    low_vpn = ROUND_UP(((*node).EndingVpn as ULONG_PTR) + 1, Alignment);
    if (low_vpn < BoundaryAddress) && (length <= (BoundaryAddress - low_vpn)) {
        #[cfg(not(feature = "ntddi_vista"))]
        {
            *Base = ROUND_DOWN(BoundaryAddress - length, Alignment);
        }
        #[cfg(feature = "ntddi_vista")]
        {
            *Base = best_vpn;
        }
        return STATUS_SUCCESS;
    }

    loop {
        lowest_node = MiGetPreviousNode(node);
        if lowest_node.is_null() {
            break;
        }

        low_vpn = ROUND_UP(((*lowest_node).EndingVpn as ULONG_PTR) + 1, Alignment);
        if (((*lowest_node).EndingVpn as ULONG_PTR) < best_vpn) &&
            (low_vpn < ((*node).StartingVpn as ULONG_PTR)) &&
            (length <= (((*node).StartingVpn as ULONG_PTR) - low_vpn))
        {
            if BoundaryAddress < ((*node).StartingVpn as ULONG_PTR) {
                *Base = best_vpn;
                return STATUS_SUCCESS;
            } else {
                *Base = ROUND_DOWN(((*node).StartingVpn as ULONG_PTR) - length, Alignment);
                return STATUS_SUCCESS;
            }
        }

        node = lowest_node;
    }

    if (((*node).StartingVpn as ULONG_PTR) > MI_LOWEST_VAD_ADDRESS as ULONG_PTR) &&
        ((((*node).StartingVpn as ULONG_PTR) - MI_LOWEST_VAD_ADDRESS as ULONG_PTR) >= length)
    {
        if BoundaryAddress < ((*node).StartingVpn as ULONG_PTR) {
            *Base = best_vpn;
            return STATUS_SUCCESS;
        }

        *Base = ROUND_DOWN(((*node).StartingVpn as ULONG_PTR) - length, Alignment);
        return STATUS_SUCCESS;
    }

    return STATUS_NO_MEMORY;
}

#[no_mangle]
pub unsafe extern "system" fn MiCheckSecuredVad(
    Vad: PMMVAD,
    Base: PVOID,
    Size: SIZE_T,
    ProtectionMask: ULONG,
) -> NTSTATUS {
    let Vad = opaque_ptr(Vad);
    let Base = opaque_ptr(Base);
    let start_address: ULONG_PTR;
    let end_address: ULONG_PTR;
    let mut protection_mask = ProtectionMask;

    start_address = Base as ULONG_PTR;
    end_address = start_address + Size - 1;

    if protection_mask < MM_DELETE_CHECK {
        if ((*Vad).u.VadFlags.NoChange() == 1) &&
            ((*Vad).u2.VadFlags2.SecNoChange() == 1) &&
            ((*Vad).u.VadFlags.Protection() != protection_mask as _)
        {
            return STATUS_INVALID_PAGE_PROTECTION;
        }
    } else {
        protection_mask = 0;
    }

    if (*Vad).u2.VadFlags2.OneSecured() != 0 {
        if (start_address <= (*(Vad as PMMVAD_LONG)).u3.Secured.EndVpn as ULONG_PTR) &&
            (end_address >= (*(Vad as PMMVAD_LONG)).u3.Secured.StartVpn as ULONG_PTR)
        {
            if (protection_mask & MM_DECOMMIT as ULONG) != 0 {
                return STATUS_INVALID_PAGE_PROTECTION;
            }

            if MmReadWrite[protection_mask as usize] < (MM_READ_WRITE_ALLOWED as i8) {
                return STATUS_INVALID_PAGE_PROTECTION;
            }
        }
    }

    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/vadnode.c
//  * PURPOSE:         ARM Memory Manager VAD Node Algorithms
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  *                  Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define MODULE_INVOLVED_IN_ARM3
// #include <mm/ARM3/miarm.h>
//
// /* Include Mm version of AVL support */
// #include "miavl.h"
// #include <sdk/lib/rtl/avlsupp.c>
//
// /* GLOBALS ********************************************************************/
//
// CHAR MmReadWrite[32] =
// {
//     MM_NO_ACCESS_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8,
//     MM_READ_ONLY_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
//     MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
//
//     MM_NO_ACCESS_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8,
//     MM_READ_ONLY_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
//     MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
//
//     MM_NO_ACCESS_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8,
//     MM_READ_ONLY_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
//     MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
//
//     MM_NO_ACCESS_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8, MM_READ_ONLY_ALLOWED as i8,
//     MM_READ_ONLY_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
//     MM_READ_WRITE_ALLOWED as i8, MM_READ_WRITE_ALLOWED as i8,
// };
//
// /* FUNCTIONS ******************************************************************/
//
// extern MM_AVL_TABLE MiRosKernelVadRoot;
//
// #if DBG
//
// static
// VOID
// MiDbgAssertIsLockedForRead(_In_ PMM_AVL_TABLE Table)
// {
//     if (Table == &MmSectionBasedRoot)
//     {
//         /* Need to hold MmSectionBasedMutex */
//         ASSERT(MmSectionBasedMutex.Owner == KeGetCurrentThread());
//     }
//     else if (Table == &MiRosKernelVadRoot)
//     {
//         /* Need to hold either the system working-set lock or
//            the idle process' AddressCreationLock */
//         ASSERT(PsGetCurrentThread()->OwnsSystemWorkingSetExclusive ||
//                PsGetCurrentThread()->OwnsSystemWorkingSetShared ||
//                (PsIdleProcess->AddressCreationLock.Owner == KeGetCurrentThread()));
//     }
//     else
//     {
//         /* Need to hold either the process working-set lock or
//            the current process' AddressCreationLock */
//         PEPROCESS Process = CONTAINING_RECORD(Table, EPROCESS, VadRoot);
//         ASSERT(MI_WS_OWNER(Process) ||
//                (Process->AddressCreationLock.Owner == KeGetCurrentThread()));
//     }
// }
//
// static
// VOID
// MiDbgAssertIsLockedForWrite(_In_ PMM_AVL_TABLE Table)
// {
//     if (Table == &MmSectionBasedRoot)
//     {
//         /* Need to hold MmSectionBasedMutex */
//         ASSERT(MmSectionBasedMutex.Owner == KeGetCurrentThread());
//     }
//     else if (Table == &MiRosKernelVadRoot)
//     {
//         /* Need to hold both the system working-set lock exclusive and
//            the idle process' AddressCreationLock */
//         ASSERT(PsGetCurrentThread()->OwnsSystemWorkingSetExclusive);
//         ASSERT(PsIdleProcess->AddressCreationLock.Owner == KeGetCurrentThread());
//     }
//     else
//     {
//         /* Need to hold both the process working-set lock exclusive and
//            the current process' AddressCreationLock */
//         PEPROCESS Process = CONTAINING_RECORD(Table, EPROCESS, VadRoot);
//         ASSERT(Process == PsGetCurrentProcess());
//         ASSERT(PsGetCurrentThread()->OwnsProcessWorkingSetExclusive);
//         ASSERT(Process->AddressCreationLock.Owner == KeGetCurrentThread());
//     }
// }
//
// #define ASSERT_LOCKED_FOR_READ(Table) MiDbgAssertIsLockedForRead(Table)
// #define ASSERT_LOCKED_FOR_WRITE(Table) MiDbgAssertIsLockedForWrite(Table)
//
// #else // DBG
//
// #define ASSERT_LOCKED_FOR_READ(Table)
// #define ASSERT_LOCKED_FOR_WRITE(Table)
//
// #endif // DBG
//
// PMMVAD
// NTAPI
// MiLocateVad(_In_ PMM_AVL_TABLE Table, _In_ PVOID VirtualAddress)
// {
//     PMMVAD FoundVad;
//     ULONG_PTR Vpn;
//     TABLE_SEARCH_RESULT SearchResult;
//
//     ASSERT_LOCKED_FOR_READ(Table);
//
//     /* Start with the the hint */
//     FoundVad = (PMMVAD)Table->NodeHint;
//     if (!FoundVad) return NULL;
//
//     /* Check if this VPN is in the hint, if so, use it */
//     Vpn = (ULONG_PTR)VirtualAddress >> PAGE_SHIFT;
//     if ((Vpn >= FoundVad->StartingVpn) && (Vpn <= FoundVad->EndingVpn)) return FoundVad;
//
//     /* VAD hint didn't work, go look for it */
//     SearchResult = RtlpFindAvlTableNodeOrParent(Table,
//                                                 (PVOID)Vpn,
//                                                 (PMMADDRESS_NODE*)&FoundVad);
//     if (SearchResult != TableFoundNode) return NULL;
//
//     /* We found it, update the hint */
//     ASSERT(FoundVad != NULL);
//     ASSERT((Vpn >= FoundVad->StartingVpn) && (Vpn <= FoundVad->EndingVpn));
//
//     /* We allow this (atomic) update without exclusive lock, because it's a hint only */
//     Table->NodeHint = FoundVad;
//     return FoundVad;
// }
//
// PMMVAD
// NTAPI
// MiLocateAddress(_In_ PVOID VirtualAddress)
// {
//     PMM_AVL_TABLE Table = &PsGetCurrentProcess()->VadRoot;
//     return MiLocateVad(Table, VirtualAddress);
// }
//
// TABLE_SEARCH_RESULT
// NTAPI
// MiCheckForConflictingNode(IN ULONG_PTR StartVpn,
//                           IN ULONG_PTR EndVpn,
//                           IN PMM_AVL_TABLE Table,
//                           OUT PMMADDRESS_NODE *NodeOrParent)
// {
//     PMMADDRESS_NODE ParentNode, CurrentNode;
//
//     ASSERT_LOCKED_FOR_READ(Table);
//
//     /* If the tree is empty, there is no conflict */
//     if (Table->NumberGenericTableElements == 0) return TableEmptyTree;
//
//     /* Start looping from the root node */
//     CurrentNode = RtlRightChildAvl(&Table->BalancedRoot);
//     ASSERT(CurrentNode != NULL);
//     while (CurrentNode)
//     {
//         ParentNode = CurrentNode;
//
//         /* This address comes after */
//         if (StartVpn > CurrentNode->EndingVpn)
//         {
//             /* Keep searching on the right */
//             CurrentNode = RtlRightChildAvl(CurrentNode);
//         }
//         else if (EndVpn < CurrentNode->StartingVpn)
//         {
//             /* This address ends before the node starts, search on the left */
//             CurrentNode = RtlLeftChildAvl(CurrentNode);
//         }
//         else
//         {
//             /* This address is part of this node, return it */
//             *NodeOrParent = ParentNode;
//             return TableFoundNode;
//         }
//     }
//
//     /* There is no more child, save the current node as parent */
//     *NodeOrParent = ParentNode;
//     if (StartVpn > ParentNode->EndingVpn)
//     {
//         return TableInsertAsRight;
//     }
//     else
//     {
//         return TableInsertAsLeft;
//     }
// }
//
// VOID
// NTAPI
// MiInsertNode(IN PMM_AVL_TABLE Table,
//              IN PMMADDRESS_NODE NewNode,
//              IN PMMADDRESS_NODE Parent,
//              IN TABLE_SEARCH_RESULT Result)
// {
//     ASSERT_LOCKED_FOR_WRITE(Table);
//
//     /* Insert it into the tree */
//     RtlpInsertAvlTreeNode(Table, NewNode, Parent, Result);
// }
//
// VOID
// NTAPI
// MiInsertVad(IN PMMVAD Vad,
//             IN PMM_AVL_TABLE VadRoot)
// {
//     TABLE_SEARCH_RESULT Result;
//     PMMADDRESS_NODE Parent = NULL;
//
//     ASSERT_LOCKED_FOR_WRITE(VadRoot);
//
//     /* Validate the VAD and set it as the current hint */
//     ASSERT(Vad->EndingVpn >= Vad->StartingVpn);
//     VadRoot->NodeHint = Vad;
//
//     /* Find the parent VAD and where this child should be inserted */
//     Result = RtlpFindAvlTableNodeOrParent(VadRoot, (PVOID)Vad->StartingVpn, &Parent);
//     ASSERT(Result != TableFoundNode);
//     ASSERT((Parent != NULL) || (Result == TableEmptyTree));
//
//     /* Do the actual insert operation */
//     MiInsertNode(VadRoot, (PVOID)Vad, Parent, Result);
// }
//
// NTSTATUS
// NTAPI
// MiInsertVadEx(
//     _Inout_ PMMVAD Vad,
//     _In_ ULONG_PTR *BaseAddress,
//     _In_ SIZE_T ViewSize,
//     _In_ ULONG_PTR HighestAddress,
//     _In_ ULONG_PTR Alignment,
//     _In_ ULONG AllocationType)
// {
//     ULONG_PTR StartingAddress, EndingAddress;
//     PEPROCESS CurrentProcess;
//     PETHREAD CurrentThread;
//     TABLE_SEARCH_RESULT Result;
//     PMMADDRESS_NODE Parent;
//
//     /* Align the view size to pages */
//     ViewSize = ALIGN_UP_BY(ViewSize, PAGE_SIZE);
//
//     /* Get the current process */
//     CurrentProcess = PsGetCurrentProcess();
//
//     /* Acquire the address creation lock and make sure the process is alive */
//     KeAcquireGuardedMutex(&CurrentProcess->AddressCreationLock);
//     if (CurrentProcess->VmDeleted)
//     {
//         KeReleaseGuardedMutex(&CurrentProcess->AddressCreationLock);
//         DPRINT1("The process is dying\n");
//         return STATUS_PROCESS_IS_TERMINATING;
//     }
//
//     /* Did the caller specify an address? */
//     if (*BaseAddress == 0)
//     {
//         /* Make sure HighestAddress is not too large */
//         HighestAddress = min(HighestAddress, (ULONG_PTR)MM_HIGHEST_VAD_ADDRESS);
//
//         /* Which way should we search? */
//         if ((AllocationType & MEM_TOP_DOWN) || CurrentProcess->VmTopDown)
//         {
//             /* Find an address top-down */
//             Result = MiFindEmptyAddressRangeDownTree(ViewSize,
//                                                      HighestAddress,
//                                                      Alignment,
//                                                      &CurrentProcess->VadRoot,
//                                                      &StartingAddress,
//                                                      &Parent);
//         }
//         else
//         {
//             /* Find an address bottom-up */
//             Result = MiFindEmptyAddressRangeInTree(ViewSize,
//                                                    Alignment,
//                                                    &CurrentProcess->VadRoot,
//                                                    &Parent,
//                                                    &StartingAddress);
//         }
//
//         /* Get the ending address, which is the last piece we need for the VAD */
//         EndingAddress = StartingAddress + ViewSize - 1;
//
//         /* Check if we found a suitable location */
//         if ((Result == TableFoundNode) || (EndingAddress > HighestAddress))
//         {
//             DPRINT1("Not enough free space to insert this VAD node!\n");
//             KeReleaseGuardedMutex(&CurrentProcess->AddressCreationLock);
//             return STATUS_NO_MEMORY;
//         }
//
//         ASSERT(StartingAddress != 0);
//         ASSERT(StartingAddress < (ULONG_PTR)HighestAddress);
//         ASSERT(EndingAddress > StartingAddress);
//     }
//     else
//     {
//         /* Calculate the starting and ending address */
//         StartingAddress = ALIGN_DOWN_BY(*BaseAddress, Alignment);
//         EndingAddress = StartingAddress + ViewSize - 1;
//
//         /* Make sure it doesn't conflict with an existing allocation */
//         Result = MiCheckForConflictingNode(StartingAddress >> PAGE_SHIFT,
//                                            EndingAddress >> PAGE_SHIFT,
//                                            &CurrentProcess->VadRoot,
//                                            &Parent);
//         if (Result == TableFoundNode)
//         {
//             DPRINT("Given address conflicts with existing node\n");
//             KeReleaseGuardedMutex(&CurrentProcess->AddressCreationLock);
//             return STATUS_CONFLICTING_ADDRESSES;
//         }
//     }
//
//     /* Now set the VAD address */
//     Vad->StartingVpn = StartingAddress >> PAGE_SHIFT;
//     Vad->EndingVpn = EndingAddress >> PAGE_SHIFT;
//
//     /* Check if we already need to charge for the pages */
//     if ((Vad->u.VadFlags.PrivateMemory() && Vad->u.VadFlags.MemCommit()) ||
//         (!Vad->u.VadFlags.PrivateMemory() &&
//          (Vad->u.VadFlags.Protection() & PAGE_WRITECOPY)))
//     {
//         /* Set the commit charge */
//         Vad->u.VadFlags.CommitCharge() = ViewSize / PAGE_SIZE;
//     }
//
//     /* Check if the VAD is to be secured */
//     if (Vad->u2.VadFlags2.OneSecured())
//     {
//         /* This *must* be a long VAD! */
//         ASSERT(Vad->u2.VadFlags2.LongVad);
//
//         /* Yeah this is retarded, I didn't invent it! */
//         ((PMMVAD_LONG)Vad)->u3.Secured.StartVpn = StartingAddress;
//         ((PMMVAD_LONG)Vad)->u3.Secured.EndVpn = EndingAddress;
//     }
//
//     /* Lock the working set */
//     CurrentThread = PsGetCurrentThread();
//     MiLockProcessWorkingSetUnsafe(CurrentProcess, CurrentThread);
//
//     /* Insert the VAD */
//     CurrentProcess->VadRoot.NodeHint = Vad;
//     MiInsertNode(&CurrentProcess->VadRoot, (PVOID)Vad, Parent, Result);
//
//     /* Release the working set */
//     MiUnlockProcessWorkingSetUnsafe(CurrentProcess, CurrentThread);
//
//     /* Update the process' virtual size, and peak virtual size */
//     CurrentProcess->VirtualSize += ViewSize;
//     if (CurrentProcess->VirtualSize > CurrentProcess->PeakVirtualSize)
//     {
//         CurrentProcess->PeakVirtualSize = CurrentProcess->VirtualSize;
//     }
//
//     /* Unlock the address space */
//     KeReleaseGuardedMutex(&CurrentProcess->AddressCreationLock);
//
//     *BaseAddress = StartingAddress;
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// MiInsertBasedSection(IN PSECTION Section)
// {
//     TABLE_SEARCH_RESULT Result;
//     PMMADDRESS_NODE Parent = NULL;
//     ASSERT(Section->Address.EndingVpn >= Section->Address.StartingVpn);
//
//     ASSERT_LOCKED_FOR_WRITE(&MmSectionBasedRoot);
//
//     /* Find the parent VAD and where this child should be inserted */
//     Result = RtlpFindAvlTableNodeOrParent(&MmSectionBasedRoot, (PVOID)Section->Address.StartingVpn, &Parent);
//     ASSERT(Result != TableFoundNode);
//     ASSERT((Parent != NULL) || (Result == TableEmptyTree));
//     MiInsertNode(&MmSectionBasedRoot, &Section->Address, Parent, Result);
// }
//
// VOID
// NTAPI
// MiRemoveNode(IN PMMADDRESS_NODE Node,
//              IN PMM_AVL_TABLE Table)
// {
//     ASSERT_LOCKED_FOR_WRITE(Table);
//
//     /* Call the AVL code */
//     RtlpDeleteAvlTreeNode(Table, Node);
//
//     /* Decrease element count */
//     Table->NumberGenericTableElements--;
//
//     /* Check if this node was the hint */
//     if (Table->NodeHint == Node)
//     {
//         /* Get a new hint, unless we're empty now, in which case nothing */
//         if (!Table->NumberGenericTableElements) Table->NodeHint = NULL;
//         else Table->NodeHint = Table->BalancedRoot.RightChild;
//     }
// }
//
// PMMADDRESS_NODE
// NTAPI
// MiGetPreviousNode(IN PMMADDRESS_NODE Node)
// {
//     PMMADDRESS_NODE Parent;
//
//     /* Get the left child */
//     if (RtlLeftChildAvl(Node))
//     {
//         /* Get right-most child */
//         Node = RtlLeftChildAvl(Node);
//         while (RtlRightChildAvl(Node)) Node = RtlRightChildAvl(Node);
//         return Node;
//     }
//
//     Parent = RtlParentAvl(Node);
//     ASSERT(Parent != NULL);
//     while (Parent != Node)
//     {
//         /* The parent should be a right child, return the real predecessor */
//         if (RtlIsRightChildAvl(Node))
//         {
//             /* Return it unless it's the root */
//             if (Parent == RtlParentAvl(Parent)) Parent = NULL;
//             return Parent;
//         }
//
//         /* Keep lopping until we find our parent */
//         Node = Parent;
//         Parent = RtlParentAvl(Node);
//     }
//
//     /* Nothing found */
//     return NULL;
// }
//
// PMMADDRESS_NODE
// NTAPI
// MiGetNextNode(IN PMMADDRESS_NODE Node)
// {
//     PMMADDRESS_NODE Parent;
//
//     /* Get the right child */
//     if (RtlRightChildAvl(Node))
//     {
//         /* Get left-most child */
//         Node = RtlRightChildAvl(Node);
//         while (RtlLeftChildAvl(Node)) Node = RtlLeftChildAvl(Node);
//         return Node;
//     }
//
//     Parent = RtlParentAvl(Node);
//     ASSERT(Parent != NULL);
//     while (Parent != Node)
//     {
//         /* The parent should be a left child, return the real predecessor */
//         if (RtlIsLeftChildAvl(Node))
//         {
//             /* Return it */
//             return Parent;
//         }
//
//         /* Keep lopping until we find our parent */
//         Node = Parent;
//         Parent = RtlParentAvl(Node);
//     }
//
//     /* Nothing found */
//     return NULL;
// }
//
// TABLE_SEARCH_RESULT
// NTAPI
// MiFindEmptyAddressRangeInTree(IN SIZE_T Length,
//                               IN ULONG_PTR Alignment,
//                               IN PMM_AVL_TABLE Table,
//                               OUT PMMADDRESS_NODE *PreviousVad,
//                               OUT PULONG_PTR Base)
// {
//     PMMADDRESS_NODE Node, PreviousNode;
//     ULONG_PTR PageCount, AlignmentVpn, LowVpn, HighestVpn;
//     ASSERT(Length != 0);
//
//     ASSERT_LOCKED_FOR_READ(Table);
//
//     /* Calculate page numbers for the length, alignment, and starting address */
//     PageCount = BYTES_TO_PAGES(Length);
//     AlignmentVpn = Alignment >> PAGE_SHIFT;
//     LowVpn = ALIGN_UP_BY((ULONG_PTR)MM_LOWEST_USER_ADDRESS >> PAGE_SHIFT, AlignmentVpn);
//
//     /* Check for kernel mode table (memory areas) */
//     if (Table->Unused == 1)
//     {
//         LowVpn = ALIGN_UP_BY((ULONG_PTR)MmSystemRangeStart >> PAGE_SHIFT, AlignmentVpn);
//     }
//
//     /* Check if the table is empty */
//     if (Table->NumberGenericTableElements == 0)
//     {
//         /* Tree is empty, the candidate address is already the best one */
//         *Base = LowVpn << PAGE_SHIFT;
//         return TableEmptyTree;
//     }
//
//     /* Otherwise, follow the leftmost child of the right root node's child */
//     Node = RtlRightChildAvl(&Table->BalancedRoot);
//     while (RtlLeftChildAvl(Node)) Node = RtlLeftChildAvl(Node);
//
//     /* Start a search to find a gap */
//     PreviousNode = NULL;
//     while (Node != NULL)
//     {
//         /* Check if the gap below the current node is suitable */
//         if (Node->StartingVpn >= LowVpn + PageCount)
//         {
//             /* There is enough space to add our node */
//             *Base = LowVpn << PAGE_SHIFT;
//
//             /* Can we use the current node as parent? */
//             if (RtlLeftChildAvl(Node) == NULL)
//             {
//                 /* Node has no left child, so use it as parent */
//                 *PreviousVad = Node;
//                 return TableInsertAsLeft;
//             }
//             else
//             {
//                 /* Node has a left child, this means that the previous node is
//                    the right-most child of it's left child and can be used as
//                    the parent. In case we use the space before the left-most
//                    node, it's left child must be NULL. */
//                 ASSERT(PreviousNode != NULL);
//                 ASSERT(RtlRightChildAvl(PreviousNode) == NULL);
//                 *PreviousVad = PreviousNode;
//                 return TableInsertAsRight;
//             }
//         }
//
//         /* The next candidate is above the current node */
//         if (Node->EndingVpn >= LowVpn)
//             LowVpn = ALIGN_UP_BY(Node->EndingVpn + 1, AlignmentVpn);
//
//         /* Remember the current node and go to the next node */
//         PreviousNode = Node;
//         Node = MiGetNextNode(Node);
//     }
//
//     /* We're up to the highest VAD, will this allocation fit above it? */
//     HighestVpn = ((ULONG_PTR)MM_HIGHEST_VAD_ADDRESS + 1) / PAGE_SIZE;
//
//     /* Check for kernel mode table (memory areas) */
//     if (Table->Unused == 1)
//     {
//         HighestVpn = ALIGN_UP_BY((ULONG_PTR)(LONG_PTR)-1 >> PAGE_SHIFT, AlignmentVpn);
//     }
//
//     if (HighestVpn >= LowVpn + PageCount)
//     {
//         /* Yes! Use this VAD to store the allocation */
//         *PreviousVad = PreviousNode;
//         *Base = LowVpn << PAGE_SHIFT;
//         return TableInsertAsRight;
//     }
//
//     /* Nyet, there's no free address space for this allocation, so we'll fail */
//     return TableFoundNode;
// }
//
// TABLE_SEARCH_RESULT
// NTAPI
// MiFindEmptyAddressRangeDownTree(IN SIZE_T Length,
//                                 IN ULONG_PTR BoundaryAddress,
//                                 IN ULONG_PTR Alignment,
//                                 IN PMM_AVL_TABLE Table,
//                                 OUT PULONG_PTR Base,
//                                 OUT PMMADDRESS_NODE *Parent)
// {
//     PMMADDRESS_NODE Node, OldNode = NULL, Child;
//     ULONG_PTR LowVpn, HighVpn, AlignmentVpn;
//     PFN_NUMBER PageCount;
//
//     ASSERT_LOCKED_FOR_READ(Table);
//
//     /* Sanity checks */
//     ASSERT(BoundaryAddress);
//     ASSERT(BoundaryAddress <= ((ULONG_PTR)MM_HIGHEST_VAD_ADDRESS));
//     ASSERT((Alignment & (PAGE_SIZE - 1)) == 0);
//
//     /* Calculate page numbers for the length and alignment */
//     Length = ROUND_TO_PAGES(Length);
//     PageCount = Length >> PAGE_SHIFT;
//     AlignmentVpn = Alignment / PAGE_SIZE;
//
//     /* Check for kernel mode table (memory areas) */
//     if (Table->Unused == 1)
//     {
//         LowVpn = ALIGN_UP_BY((ULONG_PTR)MmSystemRangeStart >> PAGE_SHIFT, AlignmentVpn);
//     }
//     else
//     {
//         LowVpn = ALIGN_UP_BY((ULONG_PTR)MM_LOWEST_USER_ADDRESS, Alignment);
//     }
//
//     /* Check if there is enough space below the boundary */
//     if ((LowVpn + Length) > (BoundaryAddress + 1))
//     {
//         return TableFoundNode;
//     }
//
//     /* Check if the table is empty */
//     if (Table->NumberGenericTableElements == 0)
//     {
//         /* Tree is empty, the candidate address is already the best one */
//         *Base = ALIGN_DOWN_BY(BoundaryAddress + 1 - Length, Alignment);
//         return TableEmptyTree;
//     }
//
//     /* Calculate the initial upper margin */
//     HighVpn = (BoundaryAddress + 1) >> PAGE_SHIFT;
//
//     /* Starting from the root, follow the right children until we found a node
//        that ends above the boundary */
//     Node = RtlRightChildAvl(&Table->BalancedRoot);
//     while ((Node->EndingVpn < HighVpn) &&
//            ((Child = RtlRightChildAvl(Node)) != NULL)) Node = Child;
//
//     /* Now loop the Vad nodes */
//     while (Node)
//     {
//         /* Calculate the lower margin */
//         LowVpn = ALIGN_UP_BY(Node->EndingVpn + 1, AlignmentVpn);
//
//         /* Check if the current bounds are suitable */
//         if ((HighVpn > LowVpn) && ((HighVpn - LowVpn) >= PageCount))
//         {
//             /* There is enough space to add our node */
//             LowVpn = ALIGN_DOWN_BY(HighVpn - PageCount, AlignmentVpn);
//             *Base = LowVpn << PAGE_SHIFT;
//
//             /* Can we use the current node as parent? */
//             if (!RtlRightChildAvl(Node))
//             {
//                 /* Node has no right child, so use it as parent */
//                 *Parent = Node;
//                 return TableInsertAsRight;
//             }
//             else
//             {
//                 /* Node has a right child. This means we must have already
//                    moved one node left from the right-most node we started
//                    with, thus we already have an OldNode! */
//                 ASSERT(OldNode != NULL);
//
//                 /* The node we had before is the most left grandchild of
//                    that right child, use it as parent. */
//                 ASSERT(RtlLeftChildAvl(OldNode) == NULL);
//                 *Parent = OldNode;
//                 return TableInsertAsLeft;
//             }
//         }
//
//         /* Update the upper margin if necessary */
//         if (Node->StartingVpn < HighVpn) HighVpn = Node->StartingVpn;
//
//         /* Remember the current node and go to the previous node */
//         OldNode = Node;
//         Node = MiGetPreviousNode(Node);
//     }
//
//     /* Check if there's enough space before the lowest Vad */
//     LowVpn = ALIGN_UP_BY((ULONG_PTR)MI_LOWEST_VAD_ADDRESS, Alignment) / PAGE_SIZE;
//     if ((HighVpn > LowVpn) && ((HighVpn - LowVpn) >= PageCount))
//     {
//         /* There is enough space to add our address */
//         LowVpn = ALIGN_DOWN_BY(HighVpn - PageCount, Alignment >> PAGE_SHIFT);
//         *Base = LowVpn << PAGE_SHIFT;
//         *Parent = OldNode;
//         return TableInsertAsLeft;
//     }
//
//     /* No address space left at all */
//     *Base = 0;
//     *Parent = NULL;
//     return TableFoundNode;
// }
//
// NTSTATUS
// NTAPI
// MiFindEmptyAddressRangeDownBasedTree(IN SIZE_T Length,
//                                      IN ULONG_PTR BoundaryAddress,
//                                      IN ULONG_PTR Alignment,
//                                      IN PMM_AVL_TABLE Table,
//                                      OUT PULONG_PTR Base)
// {
//     PMMADDRESS_NODE Node, LowestNode;
//     ULONG_PTR LowVpn, BestVpn;
//
//     ASSERT_LOCKED_FOR_READ(Table);
//
//     /* Sanity checks */
//     ASSERT(Table == &MmSectionBasedRoot);
//     ASSERT(BoundaryAddress);
//     ASSERT(BoundaryAddress <= ((ULONG_PTR)MM_HIGHEST_VAD_ADDRESS + 1));
//
//     /* Compute page length, make sure the boundary address is valid */
//     Length = ROUND_TO_PAGES(Length);
//     if ((BoundaryAddress + 1) < Length) return STATUS_NO_MEMORY;
//
//     /* Check if the table is empty */
//     BestVpn = ROUND_DOWN(BoundaryAddress + 1 - Length, Alignment);
//     if (Table->NumberGenericTableElements == 0)
//     {
//         /* Tree is empty, the candidate address is already the best one */
//         *Base = BestVpn;
//         return STATUS_SUCCESS;
//     }
//
//     /* Go to the right-most node which should be the biggest address */
//     Node = Table->BalancedRoot.RightChild;
//     while (RtlRightChildAvl(Node)) Node = RtlRightChildAvl(Node);
//
//     /* Check if we can fit in here */
//     LowVpn = ROUND_UP(Node->EndingVpn + 1, Alignment);
//     if ((LowVpn < BoundaryAddress) && (Length <= (BoundaryAddress - LowVpn)))
//     {
// #if (NTDDI_VERSION >= NTDDI_VISTA)
//         /* Return the address. */
//         *Base = BestVpn;
// #else
//         /* Note: this is a compatibility hack that mimics a bug in the 2k3
//            kernel. It will can waste up to Alignment bytes of memory above
//            the allocation. This bug was fixed in Windows Vista */
//         *Base = ROUND_DOWN(BoundaryAddress - Length, Alignment);
// #endif
//         return STATUS_SUCCESS;
//     }
//
//     /* Now loop the Vad nodes */
//     do
//     {
//         /* Break out if we've reached the last node */
//         LowestNode = MiGetPreviousNode(Node);
//         if (!LowestNode) break;
//
//         /* Check if this node could contain the requested address */
//         LowVpn = ROUND_UP(LowestNode->EndingVpn + 1, Alignment);
//         if ((LowestNode->EndingVpn < BestVpn) &&
//             (LowVpn < Node->StartingVpn) &&
//             (Length <= (Node->StartingVpn - LowVpn)))
//         {
//             /* Check if we need to take BoundaryAddress into account */
//             if (BoundaryAddress < Node->StartingVpn)
//             {
//                 /* Return the optimal VPN address */
//                 *Base = BestVpn;
//                 return STATUS_SUCCESS;
//             }
//             else
//             {
//                 /* The upper margin is given by the Node's starting address */
//                 *Base = ROUND_DOWN(Node->StartingVpn - Length, Alignment);
//                 return STATUS_SUCCESS;
//             }
//         }
//
//         /* Move to the next node */
//         Node = LowestNode;
//     } while (TRUE);
//
//     /* Check if there's enough space before the lowest Vad */
//     if ((Node->StartingVpn > (ULONG_PTR)MI_LOWEST_VAD_ADDRESS) &&
//         ((Node->StartingVpn - (ULONG_PTR)MI_LOWEST_VAD_ADDRESS) >= Length))
//     {
//         /* Check if it fits in perfectly */
//         if (BoundaryAddress < Node->StartingVpn)
//         {
//             /* Return the optimal VPN address */
//             *Base = BestVpn;
//             return STATUS_SUCCESS;
//         }
//
//         /* Return an aligned base address within this node */
//         *Base = ROUND_DOWN(Node->StartingVpn - Length, Alignment);
//         return STATUS_SUCCESS;
//     }
//
//     /* No address space left at all */
//     return STATUS_NO_MEMORY;
// }
//
// NTSTATUS
// NTAPI
// MiCheckSecuredVad(IN PMMVAD Vad,
//                   IN PVOID Base,
//                   IN SIZE_T Size,
//                   IN ULONG ProtectionMask)
// {
//     ULONG_PTR StartAddress, EndAddress;
//
//     /* Compute start and end address */
//     StartAddress = (ULONG_PTR)Base;
//     EndAddress = StartAddress + Size - 1;
//
//     /* Are we deleting/unmapping, or changing? */
//     if (ProtectionMask < MM_DELETE_CHECK)
//     {
//         /* Changing... are we allowed to do so? */
//         if ((Vad->u.VadFlags.NoChange() == 1) &&
//             (Vad->u2.VadFlags2.SecNoChange() == 1) &&
//             (Vad->u.VadFlags.Protection() != ProtectionMask))
//         {
//             /* Nope, bail out */
//             DPRINT1("Trying to mess with a no-change VAD!\n");
//             return STATUS_INVALID_PAGE_PROTECTION;
//         }
//     }
//     else
//     {
//         /* This is allowed */
//         ProtectionMask = 0;
//     }
//
//     /* ARM3 doesn't support this yet */
//     ASSERT(Vad->u2.VadFlags2.MultipleSecured == 0);
//
//     /* Is this a one-secured VAD, like a TEB or PEB? */
//     if (Vad->u2.VadFlags2.OneSecured())
//     {
//         /* Is this allocation being described by the VAD? */
//         if ((StartAddress <= ((PMMVAD_LONG)Vad)->u3.Secured.EndVpn) &&
//             (EndAddress >= ((PMMVAD_LONG)Vad)->u3.Secured.StartVpn))
//         {
//             /* Guard page? */
//             if (ProtectionMask & MM_DECOMMIT)
//             {
//                 DPRINT1("Not allowed to change protection on guard page!\n");
//                 return STATUS_INVALID_PAGE_PROTECTION;
//             }
//
//             /* ARM3 doesn't have read-only VADs yet */
//             ASSERT(Vad->u2.VadFlags2.ReadOnly == 0);
//
//             /* Check if read-write protections are allowed */
//             if (MmReadWrite[ProtectionMask] < MM_READ_WRITE_ALLOWED as i8)
//             {
//                 DPRINT1("Invalid protection mask for RW access!\n");
//                 return STATUS_INVALID_PAGE_PROTECTION;
//             }
//         }
//     }
//
//     /* All good, allow the change */
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
