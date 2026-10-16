use crate::prelude::*;
use ntkrnl_sys::ddk::{GUID, LPGUID};
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn MMixerPrintTopology(Topology: PTOPOLOGY) {
    let mut Index: ULONG = 0;
    let mut SubIndex: ULONG = 0;
    DPRINT!("Num Pins %lu NumNodes %lu\n", (*Topology).TopologyPinsCount, (*Topology).TopologyNodesCount);
    Index = 0;
    while Index < (*Topology).TopologyPinsCount {
        DPRINT!("PinId %lu NodesConnectedFromCount %lu NodesConnectedToCount %lu Visited %lu\n", (*(*Topology).TopologyPins.offset(Index as isize)).PinId,
            (*(*Topology).TopologyPins.offset(Index as isize)).NodesConnectedFromCount, (*(*Topology).TopologyPins.offset(Index as isize)).NodesConnectedToCount, (*(*Topology).TopologyPins.offset(Index as isize)).Visited);

        SubIndex = 0;
        while SubIndex < (*(*Topology).TopologyPins.offset(Index as isize)).NodesConnectedFromCount {
            DPRINT!("NodesConnectedFrom Index %lu NodeId %lu\n", SubIndex,
                (**(*(*Topology).TopologyPins.offset(Index as isize)).NodesConnectedFrom.offset(SubIndex as isize)).NodeIndex);
            SubIndex = SubIndex.wrapping_add(1);
        }

        SubIndex = 0;
        while SubIndex < (*(*Topology).TopologyPins.offset(Index as isize)).NodesConnectedToCount {
            DPRINT!("NodesConnectedTo Index %lu NodeId %lu\n", SubIndex,
                (**(*(*Topology).TopologyPins.offset(Index as isize)).NodesConnectedTo.offset(SubIndex as isize)).NodeIndex);
            SubIndex = SubIndex.wrapping_add(1);
        }
        Index = Index.wrapping_add(1);
    }

    Index = 0;
    while Index < (*Topology).TopologyNodesCount {
        DPRINT!("NodeId %lu NodesConnectedFromCount %lu NodesConnectedToCount %lu Visited %lu PinConnectedFromCount %lu PinConnectedToCount %lu\n", (*(*Topology).TopologyNodes.offset(Index as isize)).NodeIndex,
            (*(*Topology).TopologyNodes.offset(Index as isize)).NodeConnectedFromCount, (*(*Topology).TopologyNodes.offset(Index as isize)).NodeConnectedToCount, (*(*Topology).TopologyNodes.offset(Index as isize)).Visited,
            (*(*Topology).TopologyNodes.offset(Index as isize)).PinConnectedFromCount, (*(*Topology).TopologyNodes.offset(Index as isize)).PinConnectedToCount);
        Index = Index.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerAllocateTopology(
    mut MixerContext: PMIXER_CONTEXT,
    mut NodesCount: ULONG,
    mut PinCount: ULONG,
    mut OutTopology: *mut PTOPOLOGY,
) -> MIXER_STATUS {
    let mut Topology: PTOPOLOGY = ::core::ptr::null_mut::<TOPOLOGY>();

    /* allocate topology */
    Topology = (*MixerContext).Alloc.unwrap_unchecked()(
        ::core::mem::size_of::<TOPOLOGY>() as ULONG,
    ) as PTOPOLOGY;

    if Topology.is_null() {
        /* out of memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* allocate topology pins */
    (*Topology).TopologyPins = (*MixerContext).Alloc.unwrap_unchecked()(
        (::core::mem::size_of::<PIN>() as usize).wrapping_mul(PinCount as usize) as ULONG,
    ) as PPIN;

    if (*Topology).TopologyPins.is_null() {
        /* release memory */
        (*MixerContext).Free.unwrap_unchecked()(Topology as PVOID);

        /* out of memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* allocate topology nodes */
    if NodesCount != 0 {
        (*Topology).TopologyNodes = (*MixerContext).Alloc.unwrap_unchecked()(
            (::core::mem::size_of::<TOPOLOGY_NODE>() as usize).wrapping_mul(NodesCount as usize)
                as ULONG,
        ) as PTOPOLOGY_NODE;

        if (*Topology).TopologyNodes.is_null() {
            /* release memory */
            (*MixerContext).Free.unwrap_unchecked()(
                (*Topology).TopologyPins as PVOID,
            );
            (*MixerContext).Free.unwrap_unchecked()(Topology as PVOID);

            /* out of memory */
            return MM_STATUS_NO_MEMORY;
        }
    }

    /* initialize topology */
    (*Topology).TopologyPinsCount = PinCount;
    (*Topology).TopologyNodesCount = NodesCount;

    /* store result */
    *OutTopology = Topology;

    /* done */
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerResetTopologyVisitStatus(Topology: PTOPOLOGY) {
    let mut Index: ULONG;

    Index = 0;
    while Index < (*Topology).TopologyNodesCount {
        /* reset visited status */
        (*(*Topology).TopologyNodes.offset(Index as isize)).Visited = (FALSE as u32);
        Index = Index.wrapping_add(1);
    }

    Index = 0;
    while Index < (*Topology).TopologyPinsCount {
        /* reset visited status */
        (*(*Topology).TopologyPins.offset(Index as isize)).Visited = (FALSE as u32);
        Index = Index.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerInitializeTopologyNodes(
    mut MixerContext: PMIXER_CONTEXT,
    mut NodeTypes: PKSMULTIPLE_ITEM,
    mut Topology: PTOPOLOGY,
) {
    let mut Index: ULONG = 0;
    let mut Guids: LPGUID = core::ptr::null_mut();

    /* sanity check */
    ASSERT!((*Topology).TopologyNodesCount == (*NodeTypes).Count);

    /* get topology node types */
    Guids = NodeTypes.offset(1 as isize) as LPGUID;

    Index = 0 as ULONG;
    while Index < (*Topology).TopologyNodesCount {
        /* store node connection index */
        (*(*Topology).TopologyNodes.offset(Index as isize)).NodeIndex = Index;

        /* store topology node type */
        (*MixerContext).Copy.unwrap_unchecked()(
            core::ptr::addr_of_mut!((*(*Topology).TopologyNodes.offset(Index as isize)).NodeType) as PVOID,
            Guids.offset(Index as isize) as *mut GUID as PVOID,
            core::mem::size_of::<GUID>() as ULONG,
        );
        Index = Index.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerAddPinConnection(
    mut MixerContext: PMIXER_CONTEXT,
    mut Pin: PPIN,
    mut Node: PTOPOLOGY_NODE,
    mut bPinToNode: ULONG,
) -> MIXER_STATUS {
    let mut Count: ULONG = 0;
    let mut NewPinsIndex: PULONG = ::core::ptr::null_mut::<ULONG>();
    let mut OldPinsIndex: PULONG = ::core::ptr::null_mut::<ULONG>();
    let mut NewNodes: *mut PTOPOLOGY_NODE = ::core::ptr::null_mut::<PTOPOLOGY_NODE>();
    let mut OldNodes: *mut PTOPOLOGY_NODE = ::core::ptr::null_mut::<PTOPOLOGY_NODE>();

    if bPinToNode != 0 {
        /* get existing count */
        Count = (*Pin).NodesConnectedToCount;
        OldNodes = (*Pin).NodesConnectedTo;
    } else {
        /* get existing count */
        Count = (*Pin).NodesConnectedFromCount;
        OldNodes = (*Pin).NodesConnectedFrom;
    }

    /* allocate new nodes array */
    NewNodes = (*MixerContext).Alloc.unwrap_unchecked()(
        (::core::mem::size_of::<PTOPOLOGY_NODE>() as usize)
            .wrapping_mul(Count.wrapping_add(1) as usize) as ULONG,
    ) as *mut PTOPOLOGY_NODE;

    if NewNodes.is_null() {
        /* out of memory */
        return MM_STATUS_NO_MEMORY;
    }

    if Count != 0 {
        /* copy existing nodes */
        (*MixerContext).Copy.unwrap_unchecked()(
            NewNodes as PVOID,
            OldNodes as PVOID,
            (::core::mem::size_of::<PTOPOLOGY>() as usize).wrapping_mul(Count as usize) as ULONG,
        );

        /* release old nodes array */
        (*MixerContext).Free.unwrap_unchecked()(OldNodes as PVOID);
    }

    /* add new topology node */
    let ref mut fresh17 = *NewNodes.offset(Count as isize);
    *fresh17 = Node;

    if bPinToNode != 0 {
        /* replace old nodes array */
        (*Pin).NodesConnectedTo = NewNodes;

        /* increment nodes count */
        (*Pin).NodesConnectedToCount = (*Pin).NodesConnectedToCount.wrapping_add(1);

        /* now enlarge PinConnectedFromCount*/
        Count = (*Node).PinConnectedFromCount;

        /* connected pin count for node */
        OldPinsIndex = (*Node).PinConnectedFrom;
    } else {
        /* replace old nodes array */
        (*Pin).NodesConnectedFrom = NewNodes;

        /* increment nodes count */
        (*Pin).NodesConnectedFromCount = (*Pin).NodesConnectedFromCount.wrapping_add(1);

        /* now enlarge PinConnectedFromCount*/
        Count = (*Node).PinConnectedToCount;

        /* connected pin count for node */
        OldPinsIndex = (*Node).PinConnectedTo;
    }

    /* allocate pin connection index */
    NewPinsIndex = (*MixerContext).Alloc.unwrap_unchecked()(
        (::core::mem::size_of::<ULONG>() as usize)
            .wrapping_mul(Count.wrapping_add(1) as usize) as ULONG,
    ) as PULONG;

    if NewPinsIndex.is_null() {
        /* out of memory */
        return MM_STATUS_NO_MEMORY;
    }

    if Count != 0 {
        /* copy existing nodes */
        (*MixerContext).Copy.unwrap_unchecked()(
            NewPinsIndex as PVOID,
            OldPinsIndex as PVOID,
            (::core::mem::size_of::<ULONG>() as usize).wrapping_mul(Count as usize) as ULONG,
        );

        /* release old nodes array */
        (*MixerContext).Free.unwrap_unchecked()(OldPinsIndex as PVOID);
    }

    /* add new topology node */
    *NewPinsIndex.offset(Count as isize) = (*Pin).PinId as ULONG;

    if bPinToNode != 0 {
        /* replace old nodes array */
        (*Node).PinConnectedFrom = NewPinsIndex;

        /* increment pin count */
        (*Node).PinConnectedFromCount = (*Node).PinConnectedFromCount.wrapping_add(1);
    } else {
        /* replace old nodes array */
        (*Node).PinConnectedTo = NewPinsIndex;

        /* increment pin count */
        (*Node).PinConnectedToCount = (*Node).PinConnectedToCount.wrapping_add(1);
    }

    /* done */
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerHandleNodeToNodeConnection(
    mut MixerContext: PMIXER_CONTEXT,
    mut Connection: PKSTOPOLOGY_CONNECTION,
    mut Topology: PTOPOLOGY,
) -> MIXER_STATUS {
    let mut InNode: PTOPOLOGY_NODE = core::ptr::null_mut();
    let mut OutNode: PTOPOLOGY_NODE = core::ptr::null_mut();
    let mut NewNodes: *mut PTOPOLOGY_NODE = core::ptr::null_mut();
    let mut NewLogicalPinNodeConnectedFrom: PULONG = core::ptr::null_mut();
    let mut Count: ULONG = 0;
    let mut LogicalPinId: ULONG = 0;

    /* sanity checks */
    ASSERT!((*Topology).TopologyNodesCount > (*Connection).ToNode);
    ASSERT!((*Topology).TopologyNodesCount > (*Connection).FromNode);

    /* get node */
    InNode = (*Topology)
        .TopologyNodes
        .offset((*Connection).FromNode as isize) as *mut __TOPOLOGY_NODE__
        as PTOPOLOGY_NODE;
    OutNode = (*Topology)
        .TopologyNodes
        .offset((*Connection).ToNode as isize) as *mut __TOPOLOGY_NODE__
        as PTOPOLOGY_NODE;

    /* get logical pin node id */
    LogicalPinId = (*Connection).ToNodePin;

    /* get existing count */
    Count = (*OutNode).NodeConnectedFromCount;

    /* allocate new nodes array */
    NewNodes = (*MixerContext).Alloc.unwrap_unchecked()(
        ((core::mem::size_of::<PTOPOLOGY_NODE>() as usize)
            .wrapping_mul(Count.wrapping_add(1 as ULONG) as usize)) as ULONG,
    ) as *mut PTOPOLOGY_NODE;

    if NewNodes.is_null() {
        /* out of memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* allocate logical pin nodes array */
    NewLogicalPinNodeConnectedFrom = (*MixerContext).Alloc.unwrap_unchecked()(
        (Count.wrapping_add(1 as ULONG) as usize)
            .wrapping_mul(core::mem::size_of::<ULONG>() as usize) as ULONG,
    ) as PULONG;
    if NewLogicalPinNodeConnectedFrom.is_null() {
        /* out of memory */
        (*MixerContext).Free.unwrap_unchecked()(NewNodes as PVOID);
        return MM_STATUS_NO_MEMORY;
    }

    if Count != 0 {
        /* copy existing nodes */
        (*MixerContext).Copy.unwrap_unchecked()(
            NewNodes as PVOID,
            (*OutNode).NodeConnectedFrom as PVOID,
            (core::mem::size_of::<PTOPOLOGY>() as usize).wrapping_mul(Count as usize) as ULONG,
        );

        /* copy existing logical pin node array */
        (*MixerContext).Copy.unwrap_unchecked()(
            NewLogicalPinNodeConnectedFrom as PVOID,
            (*OutNode).LogicalPinNodeConnectedFrom as PVOID,
            (core::mem::size_of::<ULONG>() as usize).wrapping_mul(Count as usize) as ULONG,
        );

        /* release old nodes array */
        (*MixerContext).Free.unwrap_unchecked()(
            (*OutNode).NodeConnectedFrom as PVOID,
        );

        /* release old logical pin node array */
        (*MixerContext).Free.unwrap_unchecked()(
            (*OutNode).LogicalPinNodeConnectedFrom as PVOID,
        );
    }

    /* add new topology node */
    *NewNodes.offset((*OutNode).NodeConnectedFromCount as isize) = InNode;

    /* add logical node id */
    *NewLogicalPinNodeConnectedFrom.offset((*OutNode).NodeConnectedFromCount as isize) =
        LogicalPinId as ::core::ffi::c_ulong;

    /* replace old nodes array */
    (*OutNode).NodeConnectedFrom = NewNodes as *mut *mut __TOPOLOGY_NODE__;

    /* replace old logical pin node array */
    (*OutNode).LogicalPinNodeConnectedFrom = NewLogicalPinNodeConnectedFrom;

    /* increment nodes count */
    (*OutNode).NodeConnectedFromCount = (*OutNode).NodeConnectedFromCount.wrapping_add(1);

    /* get existing count */
    Count = (*InNode).NodeConnectedToCount;

    /* allocate new nodes array */
    NewNodes = (*MixerContext).Alloc.unwrap_unchecked()(
        ((core::mem::size_of::<PTOPOLOGY_NODE>() as usize)
            .wrapping_mul(Count.wrapping_add(1 as ULONG) as usize)) as ULONG,
    ) as *mut PTOPOLOGY_NODE;

    if NewNodes.is_null() {
        /* out of memory */
        return MM_STATUS_NO_MEMORY;
    }

    if Count != 0 {
        /* copy existing nodes */
        (*MixerContext).Copy.unwrap_unchecked()(
            NewNodes as PVOID,
            (*InNode).NodeConnectedTo as PVOID,
            (core::mem::size_of::<PTOPOLOGY>() as usize).wrapping_mul(Count as usize) as ULONG,
        );

        /* release old nodes array */
        (*MixerContext).Free.unwrap_unchecked()(
            (*InNode).NodeConnectedTo as PVOID,
        );
    }

    /* add new topology node */
    *NewNodes.offset((*InNode).NodeConnectedToCount as isize) = OutNode;

    /* replace old nodes array */
    (*InNode).NodeConnectedTo = NewNodes as *mut *mut __TOPOLOGY_NODE__;

    /* increment nodes count */
    (*InNode).NodeConnectedToCount = (*InNode).NodeConnectedToCount.wrapping_add(1);

    /* done */
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerAddPinToPinConnection(
    mut MixerContext: PMIXER_CONTEXT,
    mut InPin: PPIN,
    mut OutPin: PPIN,
) -> MIXER_STATUS {
    let mut Count: ULONG = 0;
    let mut NewPinsIndex: PULONG = ::core::ptr::null_mut::<::core::ffi::c_ulong>();
    Count = (*InPin).PinConnectedToCount;
    NewPinsIndex = (*MixerContext).Alloc.unwrap_unchecked()(
        (::core::mem::size_of::<ULONG>() as usize)
            .wrapping_mul(Count.wrapping_add(1 as ULONG) as usize) as ULONG,
    ) as PULONG;
    if NewPinsIndex.is_null() {
        return MM_STATUS_NO_MEMORY;
    }
    if Count != 0 {
        (*MixerContext).Copy.unwrap_unchecked()(
            NewPinsIndex as PVOID,
            (*InPin).PinConnectedTo as PVOID,
            (::core::mem::size_of::<ULONG>() as usize).wrapping_mul(Count as usize) as ULONG,
        );
        (*MixerContext).Free.unwrap_unchecked()((*InPin).PinConnectedTo as PVOID);
    }
    *NewPinsIndex.offset(Count as isize) = (*OutPin).PinId as ::core::ffi::c_ulong;
    (*InPin).PinConnectedTo = NewPinsIndex;
    (*InPin).PinConnectedToCount = (*InPin).PinConnectedToCount.wrapping_add(1);
    Count = (*OutPin).PinConnectedFromCount;
    NewPinsIndex = (*MixerContext).Alloc.unwrap_unchecked()(
        (::core::mem::size_of::<ULONG>() as usize)
            .wrapping_mul(Count.wrapping_add(1 as ULONG) as usize) as ULONG,
    ) as PULONG;
    if NewPinsIndex.is_null() {
        return MM_STATUS_NO_MEMORY;
    }
    if Count != 0 {
        (*MixerContext).Copy.unwrap_unchecked()(
            NewPinsIndex as PVOID,
            (*OutPin).PinConnectedFrom as PVOID,
            (::core::mem::size_of::<ULONG>() as usize).wrapping_mul(Count as usize) as ULONG,
        );
        (*MixerContext).Free.unwrap_unchecked()(
            (*OutPin).PinConnectedFrom as PVOID,
        );
    }
    *NewPinsIndex.offset(Count as isize) = (*InPin).PinId as ::core::ffi::c_ulong;
    (*OutPin).PinConnectedFrom = NewPinsIndex;
    (*OutPin).PinConnectedFromCount = (*OutPin).PinConnectedFromCount.wrapping_add(1);
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerHandleNodePinConnection(
    mut MixerContext: PMIXER_CONTEXT,
    mut Connection: PKSTOPOLOGY_CONNECTION,
    mut Topology: PTOPOLOGY,
) -> MIXER_STATUS {
    let mut Pin: PPIN = core::ptr::null_mut();
    let mut Node: PTOPOLOGY_NODE = core::ptr::null_mut();

    /* check type */
    if (*Connection).FromNode == KSFILTER_NODE &&
        (*Connection).ToNode == KSFILTER_NODE
    {
        /* Pin -> Pin direction */

        /* sanity checks */
        ASSERT!((*Topology).TopologyPinsCount > (*Connection).FromNodePin);
        ASSERT!((*Topology).TopologyPinsCount > (*Connection).ToNodePin);

        /* add connection */
        return MMixerAddPinToPinConnection(
            MixerContext,
            (*Topology).TopologyPins.offset((*Connection).FromNodePin as isize) as PPIN,
            (*Topology).TopologyPins.offset((*Connection).ToNodePin as isize) as PPIN,
        );
    } else if (*Connection).FromNode == KSFILTER_NODE {
        /* Pin -> Node direction */

        /* sanity checks */
        ASSERT!((*Topology).TopologyPinsCount > (*Connection).FromNodePin);
        ASSERT!((*Topology).TopologyNodesCount > (*Connection).ToNode);
        ASSERT!((*Connection).ToNode != KSFILTER_NODE);

        /* get pin */
        Pin = (*Topology).TopologyPins.offset((*Connection).FromNodePin as isize) as PPIN;

        /* get node */
        Node = (*Topology).TopologyNodes.offset((*Connection).ToNode as isize) as PTOPOLOGY_NODE;

        /* initialize pin */
        (*Pin).PinId = (*Connection).FromNodePin;

        /* mark as visited */
        (*Pin).Visited = (TRUE as u32);
        (*Node).Visited = (TRUE as u32);

        /* add connection */
        return MMixerAddPinConnection(MixerContext, Pin, Node, (TRUE) as ULONG);
    } else if (*Connection).ToNode == KSFILTER_NODE {
        /* Node -> Pin direction */

        /* sanity checks */
        ASSERT!((*Topology).TopologyPinsCount > (*Connection).ToNodePin);
        ASSERT!((*Topology).TopologyNodesCount > (*Connection).FromNode);
        ASSERT!((*Connection).FromNode != KSFILTER_NODE);

        /* get pin */
        Pin = (*Topology).TopologyPins.offset((*Connection).ToNodePin as isize) as PPIN;

        /* get node */
        Node = (*Topology).TopologyNodes.offset((*Connection).FromNode as isize) as PTOPOLOGY_NODE;

        /* initialize pin */
        (*Pin).PinId = (*Connection).ToNodePin;

        /* mark as visited */
        (*Pin).Visited = (TRUE as u32);
        (*Node).Visited = (TRUE as u32);

        /* add connection */
        return MMixerAddPinConnection(MixerContext, Pin, Node, (FALSE) as ULONG);
    }
    /* invalid call */
    ASSERT!(0);
    return MM_STATUS_INVALID_PARAMETER;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerExploreTopology(
    mut MixerContext: PMIXER_CONTEXT,
    mut NodeConnections: PKSMULTIPLE_ITEM,
    mut NodeTypes: PKSMULTIPLE_ITEM,
    mut Topology: PTOPOLOGY,
) -> MIXER_STATUS {
    let mut Index: ULONG;
    let mut Connection: PKSTOPOLOGY_CONNECTION;
    let mut Status: MIXER_STATUS = MM_STATUS_SUCCESS;

    /* sanity check */
    ASSERT!((*Topology).TopologyNodesCount == (*NodeTypes).Count);

    /* get node connections */
    Connection = NodeConnections.add(1 as usize) as PKSTOPOLOGY_CONNECTION;

    Index = 0;
    while Index < (*NodeConnections).Count {
        if (*Connection.add(Index as usize)).FromNode == KSFILTER_NODE
            || (*Connection.add(Index as usize)).ToNode == KSFILTER_NODE
        {
            /* handle connection from Pin -> Node / Node->Pin */
            Status = MMixerHandleNodePinConnection(
                MixerContext,
                Connection.add(Index as usize),
                Topology,
            );
        } else {
            /* handle connection from Node -> Node */
            Status = MMixerHandleNodeToNodeConnection(
                MixerContext,
                Connection.add(Index as usize),
                Topology,
            );
        }

        if Status != MM_STATUS_SUCCESS {
            /* failed to handle connection */
            return Status;
        }
        Index = Index.wrapping_add(1);
    }

    /* done */
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerAddPinIndexToArray(
    MixerContext: PMIXER_CONTEXT,
    PinId: ULONG,
    MaxPins: ULONG,
    OutPinCount: PULONG,
    OutPins: PULONG,
) {
    let mut Index: ULONG = 0 as ULONG;
    while Index < MaxPins {
        if *OutPins.offset(Index as isize) != MAXULONG as ULONG {
            if *OutPins.offset(Index as isize) > PinId {
                /* shift entries up */
                (*MixerContext).Copy.unwrap_unchecked()(
                    OutPins.offset(Index.wrapping_add(1 as ULONG) as isize)
                        as *mut ::core::ffi::c_ulong as PVOID,
                    OutPins.offset(Index as isize) as *mut ::core::ffi::c_ulong as PVOID,
                    (MaxPins.wrapping_sub(Index.wrapping_add(1 as ULONG)) as usize)
                        .wrapping_mul(::core::mem::size_of::<ULONG>() as usize)
                        as ULONG,
                );

                /* store pin id */
                *OutPins.offset(Index as isize) = PinId;

                /* increment pin count */
                *OutPinCount = (*OutPinCount).wrapping_add(1);

                /* done */
                return;
            }
        } else {
            /* store pin id */
            *OutPins.offset(Index as isize) = PinId;

            /* increment pin count */
            *OutPinCount = (*OutPinCount).wrapping_add(1);

            /* done */
            return;
        }
        Index = Index.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetUpOrDownStreamPins(
    MixerContext: PMIXER_CONTEXT,
    Topology: PTOPOLOGY,
    TopologyNode: PTOPOLOGY_NODE,
    bUpStream: ULONG,
    OutPinCount: PULONG,
    OutPins: PULONG,
) {
    let mut Index: ULONG = 0;
    let mut TopologyNodesCount: ULONG = 0;
    let mut PinsCount: ULONG = 0;
    let mut TopologyNodes: *mut PTOPOLOGY_NODE = core::ptr::null_mut::<PTOPOLOGY_NODE>();
    let mut Pins: PULONG = core::ptr::null_mut::<ULONG>();
    let mut Pin: PPIN = core::ptr::null_mut::<PIN>();

    /* sanity check */
    ASSERT!((*TopologyNode).Visited == (FALSE as u32));

    if bUpStream != 0 {
        /* use pins to which a node is attached to */
        PinsCount = (*TopologyNode).PinConnectedFromCount;
        Pins = (*TopologyNode).PinConnectedFrom;

        TopologyNodesCount = (*TopologyNode).NodeConnectedFromCount;
        TopologyNodes = (*TopologyNode).NodeConnectedFrom as *mut PTOPOLOGY_NODE;
    } else {
        /* use pins which are attached to a node */
        PinsCount = (*TopologyNode).PinConnectedToCount;
        Pins = (*TopologyNode).PinConnectedTo;

        TopologyNodesCount = (*TopologyNode).NodeConnectedToCount;
        TopologyNodes = (*TopologyNode).NodeConnectedTo as *mut PTOPOLOGY_NODE;
    }

    /* add all diretly connected pins */
    Index = 0;
    while Index < PinsCount {
        /* sanity check */
        ASSERT!(*Pins.offset(Index as isize) < (*Topology).TopologyPinsCount);

        /* get pin */
        Pin = (*Topology)
            .TopologyPins
            .offset(*Pins.offset(Index as isize) as isize) as *mut PIN as PPIN;

        /* pin should not have been visited */
        ASSERT!((*Pin).Visited == (FALSE as u32));
        ASSERT!(*Pins.offset(Index as isize) == (*Pin).PinId);

        /* FIXME support Pin -> Pin connections in iteration */
        if bUpStream != 0 {
            /* indicates a very broken topology Pin -> Pin -> Node <-... */
            ASSERT!((*Pin).PinConnectedFromCount == 0);
        } else {
            /* indicates a very broken topology -> Node -> Pin -> Pin */
            ASSERT!((*Pin).PinConnectedToCount == 0);
        }

        /* add them to pin array */
        MMixerAddPinIndexToArray(
            MixerContext,
            (*Pin).PinId,
            (*Topology).TopologyPinsCount,
            OutPinCount,
            OutPins,
        );

        /* mark pin as visited */
        (*Pin).Visited = (TRUE as u32);

        Index += 1;
    }

    /* mark node as visited */
    (*TopologyNode).Visited = (TRUE as u32);

    /* now visit all connected nodes */
    Index = 0;
    while Index < TopologyNodesCount {
        /* recursively visit them */
        MMixerGetUpOrDownStreamPins(
            MixerContext,
            Topology,
            *TopologyNodes.offset(Index as isize),
            bUpStream,
            OutPinCount,
            OutPins,
        );

        Index += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetNodeIndexFromGuid(
    Topology: PTOPOLOGY,
    NodeType: *const GUID,
) -> ULONG {
    let mut Index: ULONG;

    Index = 0;
    while Index < (*Topology).TopologyNodesCount {
        if IsEqualGUIDAligned(
            NodeType,
            core::ptr::addr_of!((*(*Topology).TopologyNodes.offset(Index as isize)).NodeType),
        ) != false
        {
            return Index;
        }
        Index = Index.wrapping_add(1);
    }

    return MAXULONG;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetAllUpOrDownstreamPinsFromNodeIndex(
    MixerContext: PMIXER_CONTEXT,
    Topology: PTOPOLOGY,
    NodeIndex: ULONG,
    bUpStream: ULONG,
    OutPinsCount: PULONG,
    OutPins: PULONG,
) {
    let mut TopologyNode: PTOPOLOGY_NODE;

    /* reset visited status */
    MMixerResetTopologyVisitStatus(Topology);

    /* sanity check */
    ASSERT!((*Topology).TopologyNodesCount > NodeIndex);

    /* get topology node */
    TopologyNode = (*Topology).TopologyNodes.add(NodeIndex as usize);

    /* now visit all upstream pins & nodes */
    MMixerGetUpOrDownStreamPins(MixerContext, Topology, TopologyNode, bUpStream, OutPinsCount, OutPins);
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetUpOrDownstreamNodes(
    MixerContext: PMIXER_CONTEXT,
    Topology: PTOPOLOGY,
    TopologyNode: PTOPOLOGY_NODE,
    bUpStream: ULONG,
    OutNodeCount: PULONG,
    OutNodes: PULONG,
) {
    let mut Index: ULONG;
    let mut TopologyNodesCount: ULONG;
    let mut Node: PTOPOLOGY_NODE;
    let mut TopologyNodes: *mut PTOPOLOGY_NODE;

    if bUpStream != 0 {
        /* use nodes to which a node is attached to */
        TopologyNodesCount = (*TopologyNode).NodeConnectedFromCount;
        TopologyNodes = (*TopologyNode).NodeConnectedFrom as *mut PTOPOLOGY_NODE;
    } else {
        /* use nodes which are attached to a node */
        TopologyNodesCount = (*TopologyNode).NodeConnectedToCount;
        TopologyNodes = (*TopologyNode).NodeConnectedTo as *mut PTOPOLOGY_NODE;
    }

    /* sanity check */
    ASSERT!((*TopologyNode).Visited == (FALSE as u32));

    /* add all connected nodes */
    Index = 0;
    while Index < TopologyNodesCount {
        /* get node */
        Node = *TopologyNodes.offset(Index as isize);

        /* node should not have been visited */
        ASSERT!((*Node).Visited == (FALSE as u32));

        /* mark node as visited */
        (*TopologyNode).Visited = (TRUE as u32);

        /* add them to node array */
        MMixerAddPinIndexToArray(
            MixerContext,
            (*Node).NodeIndex,
            (*Topology).TopologyNodesCount,
            OutNodeCount,
            OutNodes,
        );

        /* recursively visit them */
        MMixerGetUpOrDownstreamNodes(
            MixerContext,
            Topology,
            *TopologyNodes.offset(Index as isize),
            bUpStream,
            OutNodeCount,
            OutNodes,
        );

        Index = Index.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetAllUpOrDownstreamNodesFromNodeIndex(
    MixerContext: PMIXER_CONTEXT,
    Topology: PTOPOLOGY,
    NodeIndex: ULONG,
    bUpStream: ULONG,
    OutNodesCount: PULONG,
    OutNodes: PULONG,
) -> MIXER_STATUS {
    let mut TopologyNode: PTOPOLOGY_NODE;

    /* reset visited status */
    MMixerResetTopologyVisitStatus(Topology);

    /* sanity check */
    ASSERT!((*Topology).TopologyNodesCount > NodeIndex);

    /* get topology node */
    TopologyNode = (*Topology).TopologyNodes.offset(NodeIndex as isize) as *mut _ as PTOPOLOGY_NODE;

    /* now visit all upstream pins & nodes */
    MMixerGetUpOrDownstreamNodes(MixerContext, Topology, TopologyNode, bUpStream, OutNodesCount, OutNodes);

    /* done */
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetAllUpOrDownstreamPinsFromPinIndex(
    MixerContext: PMIXER_CONTEXT,
    Topology: PTOPOLOGY,
    PinIndex: ULONG,
    bUpStream: ULONG,
    OutPinsCount: PULONG,
    OutPins: PULONG,
) -> MIXER_STATUS {
    let mut Index: ULONG = 0;
    let mut TopologyNodesCount: ULONG = 0;
    let mut TopologyPinsCount: ULONG = 0;
    let mut Pin: PPIN = core::ptr::null_mut::<PIN>();
    let mut TopologyNodes: *mut PTOPOLOGY_NODE = core::ptr::null_mut::<PTOPOLOGY_NODE>();
    let mut TopologyPins: PULONG = core::ptr::null_mut::<ULONG>();

    /* get pin */
    Pin = (*Topology).TopologyPins.offset(PinIndex as isize) as *mut PIN as PPIN;

    if bUpStream != 0 {
        /* use nodes to which this pin is attached to */
        TopologyNodes = (*Pin).NodesConnectedFrom;
        TopologyNodesCount = (*Pin).NodesConnectedFromCount;

        /* use pins to which this pin is attached to */
        TopologyPins = (*Pin).PinConnectedFrom;
        TopologyPinsCount = (*Pin).PinConnectedFromCount;
    } else {
        /* use nodes which are attached to a pin */
        TopologyNodes = (*Pin).NodesConnectedTo;
        TopologyNodesCount = (*Pin).NodesConnectedToCount;

        /* use pins which are attached to this pin */
        TopologyPins = (*Pin).PinConnectedTo;
        TopologyPinsCount = (*Pin).PinConnectedToCount;
    }

    /* reset visited status */
    MMixerResetTopologyVisitStatus(Topology);

    /* sanity check */
    ASSERT!((*Topology).TopologyPinsCount > PinIndex);

    /* add pins which are directly connected to this pin */
    Index = 0;
    while Index < TopologyPinsCount {
        /* add them to pin array */
        MMixerAddPinIndexToArray(
            MixerContext,
            *TopologyPins.offset(Index as isize) as ULONG,
            (*Topology).TopologyPinsCount,
            OutPinsCount,
            OutPins,
        );
        Index = Index.wrapping_add(1);
    }

    /* now visit all up / down stream pins & nodes */
    Index = 0;
    while Index < TopologyNodesCount {
        /* explore all connected pins with helper */
        MMixerGetAllUpOrDownstreamPinsFromNodeIndex(
            MixerContext,
            Topology,
            (**TopologyNodes.offset(Index as isize)).NodeIndex,
            bUpStream,
            OutPinsCount,
            OutPins,
        );
        Index = Index.wrapping_add(1);
    }

    /* done */
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetAllUpOrDownstreamNodesFromPinIndex(
    mut MixerContext: PMIXER_CONTEXT,
    mut Topology: PTOPOLOGY,
    mut PinIndex: ULONG,
    mut bUpStream: ULONG,
    mut OutNodesCount: PULONG,
    mut OutNodes: PULONG,
) {
    let mut Index: ULONG = 0;
    let mut TopologyNodesCount: ULONG = 0;
    let mut Pin: PPIN = core::ptr::null_mut();
    let mut TopologyNodes: *mut PTOPOLOGY_NODE = core::ptr::null_mut();

    /* mark them as empty */
    *OutNodesCount = 0;

    /* get pin */
    Pin = (*Topology).TopologyPins.offset(PinIndex as isize) as *mut PIN as PPIN;

    if bUpStream != 0 {
        /* use nodes to which a pin is attached to */
        TopologyNodes = (*Pin).NodesConnectedFrom;
        TopologyNodesCount = (*Pin).NodesConnectedFromCount;
    } else {
        /* use nodes which are attached to a node */
        TopologyNodes = (*Pin).NodesConnectedTo;
        TopologyNodesCount = (*Pin).NodesConnectedToCount;
    }

    /* reset visited status */
    MMixerResetTopologyVisitStatus(Topology);

    /* sanity check */
    ASSERT!((*Topology).TopologyPinsCount > PinIndex);

    /* now visit all up / down stream pins & nodes */
    Index = 0;
    while Index < TopologyNodesCount {
        /* add node to array */
        MMixerAddPinIndexToArray(
            MixerContext,
            (**TopologyNodes.offset(Index as isize)).NodeIndex,
            (*Topology).TopologyNodesCount,
            OutNodesCount,
            OutNodes,
        );

        /* explore all connected nodes with helper */
        MMixerGetAllUpOrDownstreamNodesFromNodeIndex(
            MixerContext,
            Topology,
            (**TopologyNodes.offset(Index as isize)).NodeIndex,
            bUpStream,
            OutNodesCount,
            OutNodes,
        );

        Index = Index.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetNextNodesFromPinIndex(
    mut MixerContext: PMIXER_CONTEXT,
    mut Topology: PTOPOLOGY,
    mut PinIndex: ULONG,
    mut bUpStream: ULONG,
    mut OutNodesCount: PULONG,
    mut OutNodes: PULONG,
) {
    let mut Pin: PPIN = core::ptr::null_mut::<PIN>();
    let mut TopologyNodes: *mut *mut TOPOLOGY_NODE = core::ptr::null_mut::<*mut TOPOLOGY_NODE>();
    let mut TopologyNodesCount: ULONG = 0;
    let mut Index: ULONG = 0;

    /* sanity check */
    ASSERT!(PinIndex < (*Topology).TopologyPinsCount);

    /* get pin */
    Pin = (*Topology).TopologyPins.offset(PinIndex as isize) as *mut PIN as PPIN;

    if bUpStream != 0 {
        /* get up stream nodes */
        TopologyNodes = (*Pin).NodesConnectedFrom as *mut *mut TOPOLOGY_NODE;
        TopologyNodesCount = (*Pin).NodesConnectedFromCount;
    } else {
        /* get down stream nodes */
        TopologyNodes = (*Pin).NodesConnectedTo as *mut *mut TOPOLOGY_NODE;
        TopologyNodesCount = (*Pin).NodesConnectedToCount;
    }

    /* store topology nodes ids */
    Index = 0;
    while Index < TopologyNodesCount {
        *OutNodes.offset(Index as isize) =
            (**TopologyNodes.offset(Index as isize)).NodeIndex as ULONG;
        Index = Index.wrapping_add(1);
    }

    /* store topology nodes count */
    *OutNodesCount = TopologyNodesCount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetNextNodesFromNodeIndex(
    MixerContext: PMIXER_CONTEXT,
    Topology: PTOPOLOGY,
    NodeIndex: ULONG,
    bUpStream: ULONG,
    OutNodesCount: PULONG,
    OutNodes: PULONG,
) {
    let mut TopologyNodes: *mut *mut TOPOLOGY_NODE;
    let mut TopologyNodesCount: ULONG;
    let mut Index: ULONG;

    /* sanity check */
    ASSERT!(NodeIndex < (*Topology).TopologyNodesCount);

    if bUpStream != 0 {
        /* get up stream nodes */
        TopologyNodes = (*(*Topology).TopologyNodes.offset(NodeIndex as isize)).NodeConnectedFrom
            as *mut *mut TOPOLOGY_NODE;
        TopologyNodesCount =
            (*(*Topology).TopologyNodes.offset(NodeIndex as isize)).NodeConnectedFromCount;
    } else {
        /* get down stream nodes */
        TopologyNodes = (*(*Topology).TopologyNodes.offset(NodeIndex as isize)).NodeConnectedTo
            as *mut *mut TOPOLOGY_NODE;
        TopologyNodesCount =
            (*(*Topology).TopologyNodes.offset(NodeIndex as isize)).NodeConnectedToCount;
    }

    /* store topology nodes ids */
    Index = 0;
    while Index < TopologyNodesCount {
        *OutNodes.offset(Index as isize) =
            (**TopologyNodes.offset(Index as isize)).NodeIndex as ULONG;
        Index = Index.wrapping_add(1);
    }

    /* store topology nodes count */
    *OutNodesCount = TopologyNodesCount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetTopologyPinCount(Topology: PTOPOLOGY, PinCount: PULONG) {
    /* store pin count */
    *PinCount = (*Topology).TopologyPinsCount as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn MMixerAllocateTopologyPinArray(
    mut MixerContext: PMIXER_CONTEXT,
    mut Topology: PTOPOLOGY,
    mut OutPins: *mut PULONG,
) -> MIXER_STATUS {
    let mut Pins: PULONG = ::core::ptr::null_mut::<::core::ffi::c_ulong>();
    let mut Index: ULONG = 0;

    /* sanity check */
    ASSERT!((*Topology).TopologyPinsCount != 0);

    /* allocate topology pins */
    Pins = (*MixerContext).Alloc.unwrap_unchecked()(
        ((*Topology).TopologyPinsCount as usize)
            .wrapping_mul(::core::mem::size_of::<ULONG>() as usize) as ULONG,
    ) as PULONG;

    if Pins.is_null() {
        /* out of memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* mark index as unused */
    Index = 0 as ULONG;
    while Index < (*Topology).TopologyPinsCount {
        *Pins.offset(Index as isize) = MAXULONG as ::core::ffi::c_ulong;
        Index = Index.wrapping_add(1);
    }

    /* store result */
    *OutPins = Pins;

    /* done */
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerAllocateTopologyNodeArray(
    mut MixerContext: PMIXER_CONTEXT,
    mut Topology: PTOPOLOGY,
    mut OutNodes: *mut PULONG,
) -> MIXER_STATUS {
    let mut Nodes: PULONG = ::core::ptr::null_mut::<::core::ffi::c_ulong>();
    let mut Index: ULONG = 0;

    /* sanity check */
    ASSERT!((*Topology).TopologyNodesCount != 0);

    /* allocate topology pins */
    Nodes = (*MixerContext).Alloc.unwrap_unchecked()(
        ((*Topology).TopologyNodesCount as usize)
            .wrapping_mul(::core::mem::size_of::<ULONG>() as usize) as ULONG,
    ) as PULONG;

    if Nodes.is_null() {
        /* out of memory */
        return MM_STATUS_NO_MEMORY;
    }

    /* mark index as unused */
    Index = 0 as ULONG;
    while Index < (*Topology).TopologyNodesCount {
        *Nodes.offset(Index as isize) = MAXULONG as ::core::ffi::c_ulong;
        Index = Index.wrapping_add(1);
    }

    /* store result */
    *OutNodes = Nodes;

    /* done */
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerIsNodeTerminator(
    Topology: PTOPOLOGY,
    NodeIndex: ULONG,
    bTerminator: *mut ULONG,
) {
    /* sanity check */
    ASSERT!(NodeIndex < (*Topology).TopologyNodesCount);

    /* check if node has multiple parents */
    if (*(*Topology).TopologyNodes.offset(NodeIndex as isize)).NodeConnectedFromCount > 1
    {
        /* node is connected to multiple other nodes */
        *bTerminator = (TRUE as u32);

        /* done */
        return;
    }

    /* check if node is mux / sum node */
    if IsEqualGUIDAligned(
        core::ptr::addr_of_mut!((*(*Topology).TopologyNodes.offset(NodeIndex as isize)).NodeType) as LPGUID,
        core::ptr::addr_of!(KSNODETYPE_SUM) as LPGUID,
    ) != false
        || IsEqualGUIDAligned(
            core::ptr::addr_of_mut!((*(*Topology).TopologyNodes.offset(NodeIndex as isize)).NodeType) as LPGUID,
            core::ptr::addr_of!(KSNODETYPE_MUX) as LPGUID,
        ) != false
    {
        /* classic terminator */
        *bTerminator = (TRUE as u32);

        /* done */
        return;
    }

    /* node is not a terminator */
    *bTerminator = (FALSE as u32);
}
#[no_mangle]
pub unsafe extern "C" fn MMixerIsNodeConnectedToPin(
    MixerContext: PMIXER_CONTEXT,
    Topology: PTOPOLOGY,
    NodeIndex: ULONG,
    PinId: ULONG,
    bUpStream: ULONG,
    bConnected: PULONG,
) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS;
    let mut Index: ULONG;
    let mut PinsCount: ULONG;
    let mut Pins: PULONG = core::ptr::null_mut();

    /* allocate pin index array */
    Status = MMixerAllocateTopologyPinArray(MixerContext, Topology, core::ptr::addr_of_mut!(Pins));

    if Status != MM_STATUS_SUCCESS {
        /* failed to allocate */
        return Status;
    }

    /* now get connected pins */
    PinsCount = 0;
    MMixerGetAllUpOrDownstreamPinsFromNodeIndex(
        MixerContext,
        Topology,
        NodeIndex,
        bUpStream,
        core::ptr::addr_of_mut!(PinsCount),
        Pins,
    );

    /* set to false */
    *bConnected = (FALSE as u32);

    Index = 0;
    while Index < PinsCount {
        if *Pins.offset(Index as isize) == PinId {
            /* pin is connected */
            *bConnected = (TRUE as u32);
            break;
        }
        Index = Index.wrapping_add(1);
    }

    /* free pin index array */
    (*MixerContext).Free.unwrap_unchecked()(Pins as PVOID);

    /* done */
    return MM_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetConnectedFromLogicalTopologyPins(
    mut Topology: PTOPOLOGY,
    mut NodeIndex: ULONG,
    mut OutPinCount: PULONG,
    mut OutPins: PULONG,
) {
    let mut Index: ULONG;
    let mut Node: PTOPOLOGY_NODE;

    /* sanity check */
    ASSERT!(NodeIndex < (*Topology).TopologyNodesCount);

    /* get node */
    Node = (*Topology).TopologyNodes.offset(NodeIndex as isize);

    Index = 0;
    while Index < (*Node).NodeConnectedFromCount {
        /* copy logical pin id */
        *OutPins.offset(Index as isize) =
            *(*Node).LogicalPinNodeConnectedFrom.offset(Index as isize);
        Index = Index.wrapping_add(1);
    }

    /* store pin count */
    *OutPinCount = (*Node).NodeConnectedFromCount;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerGetNodeTypeFromTopology(
    mut Topology: PTOPOLOGY,
    mut NodeIndex: ULONG,
) -> LPGUID {
    if !(NodeIndex < (*Topology).TopologyNodesCount) {
        ASSERT!(NodeIndex < (*Topology).TopologyNodesCount);
    } else {
    }
    return &raw mut (*(*Topology).TopologyNodes.offset(NodeIndex as isize)).NodeType;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerSetTopologyPinReserved(Topology: PTOPOLOGY, PinId: ULONG) {
    /* sanity check */
    ASSERT!(PinId < (*Topology).TopologyPinsCount);

    /* set reserved */
    (*(*Topology).TopologyPins.offset(PinId as isize)).Reserved = TRUE as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerIsTopologyPinReserved(
    Topology: PTOPOLOGY,
    PinId: ULONG,
    bReserved: PULONG,
) {
    if !(PinId < (*Topology).TopologyPinsCount) {
        ASSERT!("PinId < Topology->TopologyPinsCount");
    }

    *bReserved = (*(*Topology).TopologyPins.offset(PinId as isize)).Reserved as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerSetTopologyNodeReserved(
    Topology: PTOPOLOGY,
    NodeIndex: ULONG,
) {
    /* sanity check */
    ASSERT!(NodeIndex < (*Topology).TopologyNodesCount);

    /* set reserved */
    (*(*Topology).TopologyNodes.add(NodeIndex as usize)).Reserved = TRUE as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerIsTopologyNodeReserved(
    Topology: PTOPOLOGY,
    NodeIndex: ULONG,
    bReserved: PULONG,
) {
    ASSERT!(NodeIndex < (*Topology).TopologyNodesCount);

    /* get reserved status */
    *bReserved = (*(*Topology).TopologyNodes.offset(NodeIndex as isize)).Reserved;
}
#[no_mangle]
pub unsafe extern "C" fn MMixerCreateTopology(
    MixerContext: PMIXER_CONTEXT,
    PinCount: ULONG,
    NodeConnections: PKSMULTIPLE_ITEM,
    NodeTypes: PKSMULTIPLE_ITEM,
    OutTopology: *mut PTOPOLOGY,
) -> MIXER_STATUS {
    let mut Status: MIXER_STATUS;
    let mut Topology: PTOPOLOGY = core::ptr::null_mut();

    /* allocate topology */
    Status = MMixerAllocateTopology(MixerContext, (*NodeTypes).Count, PinCount, core::ptr::addr_of_mut!(Topology));

    if Status != MM_STATUS_SUCCESS {
        /* failed to allocate topology */
        return Status;
    }

    /* initialize topology nodes */
    MMixerInitializeTopologyNodes(MixerContext, NodeTypes, Topology);

    /* explore topology */
    Status = MMixerExploreTopology(MixerContext, NodeConnections, NodeTypes, Topology);

    if Status != MM_STATUS_SUCCESS {
        /* failed to allocate topology */
        return Status;
    }

    MMixerPrintTopology(Topology);

    /* store result */
    *OutTopology = Topology;

    /* done */
    return MM_STATUS_SUCCESS;
}
