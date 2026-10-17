//! mmixer's topology walkers, shipping Rust against the genuine topology.c.
//!
//! On a TREE (no node reachable along two paths) the C is well-defined and the Rust must
//! match it exactly: the same sorted pin/node arrays, the same counts. Where paths converge,
//! the C lists a node or pin twice, re-walks it, and trips its own ASSERTs (an RtlAssert in
//! this DBG=1 tree); the Rust lists each once. Those cases are graded against an independent
//! reachability model, and the test pins that the C still misbehaves there.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused_mut, dead_code)]

use std::{cell::Cell, ffi::c_void};

type ULONG = u32;
type PULONG = *mut u32;
type PVOID = *mut c_void;
const TRUE: i32 = 1;
const FALSE: i32 = 0;
const MAXULONG: u32 = u32::MAX;
macro_rules! ASSERT {
    ($e:expr $(,)?) => {
        if !($e) {
            RUST_ASSERTS.with(|c| c.set(c.get() + 1));
        }
    };
}

#[repr(C)]
pub struct MIXER_CONTEXT {
    Copy: Option<unsafe extern "C" fn(PVOID, PVOID, ULONG)>,
}
type PMIXER_CONTEXT = *mut MIXER_CONTEXT;
#[repr(C)]
pub struct TOPOLOGY_NODE {
    NodeType: [u8; 16],
    NodeIndex: ULONG,
    NodeConnectedToCount: ULONG,
    NodeConnectedTo: *mut PTOPOLOGY_NODE,
    NodeConnectedFromCount: ULONG,
    NodeConnectedFrom: *mut PTOPOLOGY_NODE,
    LogicalPinNodeConnectedFrom: PULONG,
    PinConnectedFromCount: ULONG,
    PinConnectedFrom: PULONG,
    PinConnectedToCount: ULONG,
    PinConnectedTo: PULONG,
    Visited: ULONG,
    Reserved: ULONG,
}
type PTOPOLOGY_NODE = *mut TOPOLOGY_NODE;
#[repr(C)]
pub struct PIN {
    PinId: ULONG,
    NodesConnectedToCount: ULONG,
    NodesConnectedTo: *mut PTOPOLOGY_NODE,
    NodesConnectedFromCount: ULONG,
    NodesConnectedFrom: *mut PTOPOLOGY_NODE,
    PinConnectedFromCount: ULONG,
    PinConnectedFrom: PULONG,
    PinConnectedToCount: ULONG,
    PinConnectedTo: PULONG,
    Visited: ULONG,
    Reserved: ULONG,
}
type PPIN = *mut PIN;
#[repr(C)]
pub struct TOPOLOGY {
    TopologyPinsCount: ULONG,
    TopologyPins: PPIN,
    TopologyNodesCount: ULONG,
    TopologyNodes: PTOPOLOGY_NODE,
}
type PTOPOLOGY = *mut TOPOLOGY;

thread_local! {
    static C_ASSERTS: Cell<u32> = const { Cell::new(0) };
    static RUST_ASSERTS: Cell<u32> = const { Cell::new(0) };
}
#[no_mangle]
extern "C" fn fixture_assert_failed() {
    C_ASSERTS.with(|c| c.set(c.get() + 1));
}

include!(concat!(env!("OUT_DIR"), "/topology_shipping.rs"));

extern "C" {
    fn ref_MMixerAddPinIndexToArray(c: PMIXER_CONTEXT, id: ULONG, max: ULONG, n: PULONG, out: PULONG);
    fn ref_MMixerGetUpOrDownStreamPins(c: PMIXER_CONTEXT, t: PTOPOLOGY, node: PTOPOLOGY_NODE, up: ULONG, n: PULONG, out: PULONG);
    fn ref_MMixerGetUpOrDownstreamNodes(c: PMIXER_CONTEXT, t: PTOPOLOGY, node: PTOPOLOGY_NODE, up: ULONG, n: PULONG, out: PULONG);
}

/// wdmaud's context Copy: RtlMoveMemory(Destination, Source, Length) -- overlap-safe.
unsafe extern "C" fn copy(dst: PVOID, src: PVOID, n: ULONG) {
    std::ptr::copy(src as *const u8, dst as *mut u8, n as usize);
}

/// A topology from edge lists, owning every array the C structs point into.
struct Graph {
    topology: TOPOLOGY,
    nodes: Vec<TOPOLOGY_NODE>,
    pins: Vec<PIN>,
    _node_to: Vec<Vec<PTOPOLOGY_NODE>>,
    _node_from: Vec<Vec<PTOPOLOGY_NODE>>,
    _pin_to: Vec<Vec<ULONG>>,
    _pin_from: Vec<Vec<ULONG>>,
}

/// `edges`: node a -> node b (downstream). `pins_out`: node -> pin (the node's
/// PinConnectedTo). `pins_in`: pin -> node (the node's PinConnectedFrom).
fn graph(nodes: usize, pins: usize, edges: &[(usize, usize)], pins_out: &[(usize, u32)], pins_in: &[(u32, usize)]) -> Box<Graph> {
    let mut g = Box::new(Graph {
        topology: TOPOLOGY { TopologyPinsCount: pins as u32, TopologyPins: std::ptr::null_mut(), TopologyNodesCount: nodes as u32, TopologyNodes: std::ptr::null_mut() },
        nodes: (0..nodes)
            .map(|i| TOPOLOGY_NODE {
                NodeType: [0; 16],
                NodeIndex: i as u32,
                NodeConnectedToCount: 0,
                NodeConnectedTo: std::ptr::null_mut(),
                NodeConnectedFromCount: 0,
                NodeConnectedFrom: std::ptr::null_mut(),
                LogicalPinNodeConnectedFrom: std::ptr::null_mut(),
                PinConnectedFromCount: 0,
                PinConnectedFrom: std::ptr::null_mut(),
                PinConnectedToCount: 0,
                PinConnectedTo: std::ptr::null_mut(),
                Visited: 0,
                Reserved: 0,
            })
            .collect(),
        pins: (0..pins)
            .map(|i| PIN {
                PinId: i as u32,
                NodesConnectedToCount: 0,
                NodesConnectedTo: std::ptr::null_mut(),
                NodesConnectedFromCount: 0,
                NodesConnectedFrom: std::ptr::null_mut(),
                PinConnectedFromCount: 0,
                PinConnectedFrom: std::ptr::null_mut(),
                PinConnectedToCount: 0,
                PinConnectedTo: std::ptr::null_mut(),
                Visited: 0,
                Reserved: 0,
            })
            .collect(),
        _node_to: vec![Vec::new(); nodes],
        _node_from: vec![Vec::new(); nodes],
        _pin_to: vec![Vec::new(); nodes],
        _pin_from: vec![Vec::new(); nodes],
    });
    let base = g.nodes.as_mut_ptr();
    for &(a, b) in edges {
        unsafe {
            g._node_to[a].push(base.add(b));
            g._node_from[b].push(base.add(a));
        }
    }
    for &(n, p) in pins_out {
        g._pin_to[n].push(p);
    }
    for &(p, n) in pins_in {
        g._pin_from[n].push(p);
    }
    for i in 0..nodes {
        let (to, from, pt, pf) = (g._node_to[i].as_mut_ptr(), g._node_from[i].as_mut_ptr(), g._pin_to[i].as_mut_ptr(), g._pin_from[i].as_mut_ptr());
        let (nto, nfrom, npt, npf) = (g._node_to[i].len(), g._node_from[i].len(), g._pin_to[i].len(), g._pin_from[i].len());
        let n = &mut g.nodes[i];
        (n.NodeConnectedTo, n.NodeConnectedToCount) = (to, nto as u32);
        (n.NodeConnectedFrom, n.NodeConnectedFromCount) = (from, nfrom as u32);
        (n.PinConnectedTo, n.PinConnectedToCount) = (pt, npt as u32);
        (n.PinConnectedFrom, n.PinConnectedFromCount) = (pf, npf as u32);
    }
    g.topology.TopologyNodes = g.nodes.as_mut_ptr();
    g.topology.TopologyPins = g.pins.as_mut_ptr();
    g
}

type Walk = unsafe extern "C" fn(PMIXER_CONTEXT, PTOPOLOGY, PTOPOLOGY_NODE, ULONG, PULONG, PULONG);

#[derive(Debug, PartialEq)]
struct Run {
    count: u32,
    out: Vec<u32>,
    c_asserts: u32,
}

/// One walk from `start`, on a freshly reset graph (the callers' MMixerResetTopologyVisitStatus).
unsafe fn walk(f: Walk, g: &mut Graph, start: usize, up: bool, slots: usize) -> Run {
    for n in g.nodes.iter_mut() {
        n.Visited = 0;
    }
    for p in g.pins.iter_mut() {
        p.Visited = 0;
    }
    C_ASSERTS.with(|c| c.set(0));
    let mut ctx = MIXER_CONTEXT { Copy: Some(copy) };
    let mut out = vec![MAXULONG; slots];
    let mut count = 0u32;
    let node = g.nodes.as_mut_ptr().add(start);
    f(&mut ctx, &mut g.topology, node, up as u32, &mut count, out.as_mut_ptr());
    Run { count, out, c_asserts: C_ASSERTS.with(|c| c.get()) }
}

/// What a walk from `start` should list: every pin (or other node) reachable, once, sorted.
fn reachable(g: &Graph, start: usize, up: bool, want_pins: bool, slots: usize) -> Vec<u32> {
    let mut seen = vec![false; g.nodes.len()];
    let mut stack = vec![start];
    let mut got = std::collections::BTreeSet::new();
    seen[start] = true;
    while let Some(n) = stack.pop() {
        let next = if up { &g._node_from[n] } else { &g._node_to[n] };
        if want_pins {
            got.extend(if up { &g._pin_from[n] } else { &g._pin_to[n] }.iter().copied());
        }
        for &p in next {
            let i = unsafe { p.offset_from(g.nodes.as_ptr()) } as usize;
            if !want_pins && i != start {
                got.insert(i as u32);
            }
            if !seen[i] {
                seen[i] = true;
                stack.push(i);
            }
        }
    }
    let mut v: Vec<u32> = got.into_iter().collect();
    v.resize(slots, MAXULONG);
    v
}

fn rng(seed: u32) -> impl FnMut(u32) -> u32 {
    let mut x = seed.wrapping_mul(2654435761) | 1;
    move |n| {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        x % n
    }
}

/// A random tree on `nodes` nodes: node i > 0 hangs off one earlier node, each node gets
/// distinct pins, so nothing is reachable twice.
fn tree(nodes: usize, seed: u32) -> Box<Graph> {
    let mut r = rng(seed);
    let edges: Vec<(usize, usize)> = (1..nodes).map(|i| (r(i as u32) as usize, i)).collect();
    let mut pins_out = Vec::new();
    let mut pins_in = Vec::new();
    let mut next = 0u32;
    for n in 0..nodes {
        for _ in 0..r(3) {
            pins_out.push((n, next));
            next += 1;
        }
        for _ in 0..r(2) {
            pins_in.push((next, n));
            next += 1;
        }
    }
    graph(nodes, next as usize, &edges, &pins_out, &pins_in)
}

#[test]
fn sorted_insert_matches_c_without_duplicates_and_keeps_a_set_with_them() {
    unsafe {
        let mut ctx = MIXER_CONTEXT { Copy: Some(copy) };
        for seed in 1..200u32 {
            let mut r = rng(seed);
            let max = 1 + r(12) as usize;
            let ids: Vec<u32> = (0..max + r(4) as usize).map(|_| r(max as u32)).collect();
            let run = |f: unsafe extern "C" fn(PMIXER_CONTEXT, ULONG, ULONG, PULONG, PULONG), ids: &[u32], ctx: &mut MIXER_CONTEXT| {
                let mut out = vec![MAXULONG; max];
                let mut n = 0u32;
                for &id in ids {
                    f(ctx, id, max as u32, &mut n, out.as_mut_ptr());
                }
                (n, out)
            };
            let rust = run(MMixerAddPinIndexToArray, &ids, &mut ctx);
            let mut set: Vec<u32> = ids.iter().copied().collect::<std::collections::BTreeSet<_>>().into_iter().collect();
            let unique = set.len() as u32;
            set.resize(max, MAXULONG);
            assert_eq!(rust, (unique, set.clone()), "ids {ids:?}");
            if ids.len() as u32 == unique {
                assert_eq!(run(ref_MMixerAddPinIndexToArray, &ids, &mut ctx), rust, "ids {ids:?}");
            }
        }
        // The C's missing equality case: 3 again lands before the next larger entry.
        let mut out = vec![MAXULONG; 4];
        let mut n = 0u32;
        for id in [3, 1, 3] {
            ref_MMixerAddPinIndexToArray(&mut ctx, id, 4, &mut n, out.as_mut_ptr());
        }
        assert_eq!((n, out), (3, vec![1, 3, 3, MAXULONG]));
    }
}

#[test]
fn walks_match_c_on_trees() {
    unsafe {
        for seed in 1..120u32 {
            let nodes = 1 + (seed as usize % 9);
            let mut g = tree(nodes, seed);
            let slots_pins = g.pins.len().max(1);
            for start in 0..nodes {
                for up in [false, true] {
                    let ctx = format!("seed {seed} start {start} up {up}");
                    let c = walk(ref_MMixerGetUpOrDownStreamPins, &mut g, start, up, slots_pins);
                    let rust = walk(MMixerGetUpOrDownStreamPins, &mut g, start, up, slots_pins);
                    assert_eq!(c.c_asserts, 0, "a tree is the C's defined case: {ctx}");
                    assert_eq!((rust.count, &rust.out), (c.count, &c.out), "pins {ctx}");
                    assert_eq!(rust.out, reachable(&g, start, up, true, slots_pins), "pins {ctx}");
                    let c = walk(ref_MMixerGetUpOrDownstreamNodes, &mut g, start, up, nodes);
                    let rust = walk(MMixerGetUpOrDownstreamNodes, &mut g, start, up, nodes);
                    assert_eq!(c.c_asserts, 0, "{ctx}");
                    assert_eq!((rust.count, &rust.out), (c.count, &c.out), "nodes {ctx}");
                    assert_eq!(rust.out, reachable(&g, start, up, false, nodes), "nodes {ctx}");
                }
            }
        }
    }
}

#[test]
fn converging_paths_are_listed_once_where_the_c_lists_and_asserts_twice() {
    unsafe {
        // 0 -> 1 -> 3, 0 -> 2 -> 3, 3 -> 4: node 3 (and its pin 0, and node 4) reachable twice.
        let mut g = graph(5, 2, &[(0, 1), (0, 2), (1, 3), (2, 3), (3, 4)], &[(3, 0), (4, 1)], &[]);
        let pins = walk(MMixerGetUpOrDownStreamPins, &mut g, 0, false, 2);
        assert_eq!((pins.count, pins.out.clone()), (2, vec![0, 1]));
        let c = walk(ref_MMixerGetUpOrDownStreamPins, &mut g, 0, false, 2);
        assert!(c.c_asserts > 0, "the C re-walks node 3 and trips its Visited ASSERT: {c:?}");
        let nodes = walk(MMixerGetUpOrDownstreamNodes, &mut g, 0, false, 5);
        assert_eq!((nodes.count, nodes.out.clone()), (4, reachable(&g, 0, false, false, 5)));
        let c = walk(ref_MMixerGetUpOrDownstreamNodes, &mut g, 0, false, 5);
        assert!(c.count > 4, "the C counts node 3 and 4 once per path: {c:?}");
        // The same diamond walked upstream from its sink.
        let up = walk(MMixerGetUpOrDownstreamNodes, &mut g, 4, true, 5);
        assert_eq!((up.count, up.out), (4, reachable(&g, 4, true, false, 5)));
    }
}

#[test]
fn a_cycle_terminates() {
    unsafe {
        // 0 -> 1 -> 2 -> 0: the C would recurse until the stack overflowed (not run here).
        let mut g = graph(3, 1, &[(0, 1), (1, 2), (2, 0)], &[(2, 0)], &[]);
        let pins = walk(MMixerGetUpOrDownStreamPins, &mut g, 0, false, 1);
        assert_eq!((pins.count, pins.out), (1, vec![0]));
        let nodes = walk(MMixerGetUpOrDownstreamNodes, &mut g, 0, false, 3);
        assert_eq!((nodes.count, nodes.out), (2, vec![1, 2, MAXULONG]));
    }
}
