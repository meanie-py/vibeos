/* Host fixture for mmixer's topology walkers: precomp.h's TOPOLOGY, TOPOLOGY_NODE and PIN
 * verbatim (field for field, so the Rust test's #[repr(C)] mirrors serve both sides), and a
 * MIXER_CONTEXT holding only the Copy hook the walkers call. A failed ASSERT is RECORDED,
 * not fatal: this tree builds DBG=1, where it is an RtlAssert, and the test reports it. */
#include <stdint.h>
#include <stddef.h>
#include <string.h>
#define IN
#define OUT
#define VOID void
typedef uint32_t ULONG, *PULONG;
typedef void *PVOID;
typedef struct { uint8_t bytes[16]; } GUID;
#define TRUE 1
#define FALSE 0
#define MAXULONG 0xFFFFFFFFu
typedef VOID (*PMIXER_COPY)(PVOID Dst, PVOID Src, ULONG NumBytes);
typedef struct { PMIXER_COPY Copy; } MIXER_CONTEXT, *PMIXER_CONTEXT;
typedef struct __TOPOLOGY_NODE__ {
    GUID NodeType;
    ULONG NodeIndex;
    ULONG NodeConnectedToCount;
    struct __TOPOLOGY_NODE__ **NodeConnectedTo;
    ULONG NodeConnectedFromCount;
    struct __TOPOLOGY_NODE__ **NodeConnectedFrom;
    PULONG LogicalPinNodeConnectedFrom;
    ULONG PinConnectedFromCount;
    PULONG PinConnectedFrom;
    ULONG PinConnectedToCount;
    PULONG PinConnectedTo;
    ULONG Visited;
    ULONG Reserved;
} TOPOLOGY_NODE, *PTOPOLOGY_NODE;
typedef struct {
    ULONG PinId;
    ULONG NodesConnectedToCount;
    PTOPOLOGY_NODE *NodesConnectedTo;
    ULONG NodesConnectedFromCount;
    PTOPOLOGY_NODE *NodesConnectedFrom;
    ULONG PinConnectedFromCount;
    PULONG PinConnectedFrom;
    ULONG PinConnectedToCount;
    PULONG PinConnectedTo;
    ULONG Visited;
    ULONG Reserved;
} PIN, *PPIN;
typedef struct {
    ULONG TopologyPinsCount;
    PPIN TopologyPins;
    ULONG TopologyNodesCount;
    PTOPOLOGY_NODE TopologyNodes;
} TOPOLOGY, *PTOPOLOGY;
void fixture_assert_failed(void);
#define ASSERT(x) ((x) ? (void)0 : fixture_assert_failed())
#define IsEqualGUIDAligned(a, b) (!memcmp((a), (b), sizeof(GUID)))
static VOID MMixerResetTopologyVisitStatus(PTOPOLOGY Topology)
{
    for (ULONG i = 0; i < Topology->TopologyNodesCount; i++) Topology->TopologyNodes[i].Visited = FALSE;
    for (ULONG i = 0; i < Topology->TopologyPinsCount; i++) Topology->TopologyPins[i].Visited = FALSE;
}
