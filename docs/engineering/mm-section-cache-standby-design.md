# Cached section pages as standby: design

Status (2026-10-17): steps 1 and 2 are implemented and measured (below). Step 2 is `mm/standby.rs`.
Owner: wow64-amd64 lane (`vibeos-fex`).

## The problem, measured

A page of an image or data-file section that the ReactOS Mm allocated (`MmAllocPage(MC_USER)`,
`MI_IS_ROS_PFN`) stays `ActiveAndValid` and owned by its segment after the last view of it
goes. Windows moves such a page to the standby list: it still names its section, it counts as
available, and the allocator repurposes it synchronously when the free and zero lists run dry.

Here (`--tripwire MEMCENSUS=1`, appstorm64 under FEX, column `ros`):

- 2G: `ros` grows 4.4k → 30.9k pages across the sweep, and available memory falls with it.
- 1152M: the same growth; the low point is 13.8k free pages, so the balancer never trims.
- 1024M / 1088M: apps fail at 7-21 MB free with 55-97 MB still in `ros`, and the sweep wedges.

Two reasons nothing came back:

1. **arm64 could not free these pages at all.** `MmPageOutPhysicalAddress` returned
   `STATUS_UNSUCCESSFUL` for a page no process maps, skipping the segment path
   (`MmGetSectionAssociation` → `MmCheckDirtySegment`) that x86 takes there. It dated from the
   Milestone-11 bring-up (`411417cc538`). Fixed in `mm/rmap.rs`, which now also refuses (loudly,
   `stale-segment-association`) a page whose segment entry no longer names it.
2. **The balancer trimmed only at the cliff.** `MiTrimMemoryConsumer` trimmed `MC_USER` below
   `MiMinimumAvailablePages` (256 pages), and the allocator only kicked it below
   `MmMinimumFreePages`. A user fault that found no page failed the app instead of waiting.

## Step 1: Windows-like behaviour through the balancer (implemented)

- `mm/balance.rs`: below `MmLowMemoryThreshold` (about 30 MB on a 1 GB machine,
  `ARM3/mminit.rs`) the balancer trims `MC_USER` back up to it, in aging mode (Priority 0), so a
  page a process still touches keeps its frame. The 256-page hard mode is unchanged.
- `ARM3/pfnlist.rs`: crossing `MmLowMemoryThreshold` downward kicks the balancer. It also runs
  on its 2 s timer.
- `ARM3/pagfault.rs`: a user fault that got `STATUS_NO_MEMORY`, waited on the balancer, and
  then finds more than `4 * MmMinimumFreePages` free returns success, so the access re-runs. With
  no progress it still fails the app, which keeps the 768M whole-machine stall
  (`20261010-054207-p5m0hi2a`) from coming back.

Measured with step 1 (appstorm64 under FEX, MEMCENSUS):

| VM | before | after |
|---|---|---|
| 2G | 54/54 | 54/54 (`20261017-025200-g9stt2oy`), no reclaim needed |
| 1152M | 53/54 | 54/54 (`20261017-050134-vaqi2lil`), no reclaim needed |
| 1088M | wedged after ~174 process exits | 54/54 (`20261017-050119-kl4yjrk3`): ~16k pages reclaimed, `ros` held at 21k instead of 31k, low point 10.1k free pages |
| 1024M | crashes, then wedged | rapps fails, then a `WrFreePage` wait at 49 free pages with ~13.8k cached pages all in use: the workload does not fit beside the 800 MB ramdisk |

What step 1 does not give: the cached pages still do not count as available
(`GlobalMemoryStatus`, the census `avail`), and reclaim is asynchronous, so a burst can still
exhaust the free list before the balancer runs.

## Step 2: a ROS standby list (implemented, `mm/standby.rs`)

The obstacle was ownership. ARM3 repurposes a standby page under the PFN lock at DISPATCH and
restores its owner through `PteAddress`/`OriginalPte`. A ROS page's owner is a segment page
entry (SSE) guarded by the segment's fast mutex, which cannot be taken there, and its
`OriginalPte` slot holds the rmap head (field manual, "The `OriginalPte` / `RmapListHead`
union overlay"). What made it tractable: the segment page tables are NONPAGED and live until
segment teardown, and a cached page's rmap list is exactly one entry, the segment association,
which already names its table and index.

- **Insert**, in `MmUnsharePageEntrySectionSegment` where the C keeps a page cached after its
  last view goes. The page must be clean, have refcount 1 (no MDL, not mid balancer walk), have
  no swap copy, and have an rmap list that is only the segment association. It leaves the user
  LRU for the standby list, using the same `NextLRU`/`PreviousLRU` links. `PteAddress`
  records its slot (ROS never used that field; `MmAllocPage` now clears it). It leaves
  `MC_USER`'s `PagesUsed` and joins `MmAvailablePages`. Only clean pages qualify, which is
  sound because dirt reaches the SSE first: a user view's PTE dirty bit goes to the unshare,
  and Cc calls `MmMakeSegmentDirty` while its VACB still maps the page.
- **Claim**: `_MmGetPageEntrySectionSegment` and `_MmSetPageEntrySectionSegment` take a
  standby page back to the LRU, under the PFN lock, before anyone reads or rewrites its slot.
  So nothing outside `standby.rs` ever sees a standby page, and a repurpose cannot race a
  segment-locked write. The one internal read that only asks "is a page here" (Set's
  `LastPage` walk) reads raw instead, so that it does not pull pages back.
- **Repurpose**: `MiRemoveAnyPage`/`MiRemoveZeroPage` take the oldest standby page under the
  PFN lock once both the free and zeroed lists are empty. That branch was empty upstream. The
  slot gets `MM_REPURPOSED_SSE` (swap-format, which Get reports as 0, "not resident"). The
  page's one rmap entry is detached onto a deferred list, with a segment reference taken for
  it. Nothing at DISPATCH touches pool or a segment mutex.
- **Drain**: the balancer thread, at passive level, clears each slot that still holds the
  marker (under the segment lock), drops the reference, and frees the rmap entry. If the segment
  reused the slot first, the new entry inherited the marker's reference, and the drain only
  drops its own.
- Guards: repurposing a standby page whose slot or rmap changed without a claim trips
  `standby.MiRosStandbyTakeForRepurpose page-changed-on-standby` and returns the page to the
  LRU. Freeing a page still on the list trips `freelist.MmDereferencePage page-on-standby`
  and bugchecks. Neither fired in any run below. MEMCENSUS prints `rosstandby`.

Measured (appstorm64 under FEX, MEMCENSUS, arm64):

| VM | step 1 | step 2 |
|---|---|---|
| 2G | 54/54; avail 274.5k -> 233.1k pages | 54/54 (`20261017-122823-nj2zrsc3`); avail 272.8k -> 247.8k; up to 16.7k pages on standby, none repurposed |
| 1088M | 54/54 | 54/54 (`20261017-122823-zzusy1xh`); 7k+ pages repurposed synchronously, low point 13.6k available |
| 1024M | rapps fails, then a `WrFreePage` wedge | 54/54 (`20261017-122823-0fb5ujzb`); 10k+ repurposed, plus 16k reclaimed by the balancer |

Plus smoke on all three arches and the FEX ladder (27/27, `20261017-122114-rw9sm82p`).

**Left of the 2G drift** (25k pages): not a leak. The census's `other` column grew 368 -> 10k
in both runs because the ROS `MmDeleteVirtualMapping` (arm64/i386 `page.rs`) leaves a ROS page
in `TransitionPage` once its last mapping goes, and the census counted ROS pages only under
`ActiveAndValid`. Put back, the ROS total matches `MC_USER`'s `PagesUsed`. Step 1: 30.1k + 9.7k
= 39.8k vs 40.2k. Step 2: 30.2k + 9.7k - 16.7k on standby = 23.2k vs 23.5k. The census now
counts ROS pages in either location, and prints `rosidle` for the unmapped ones that are not on
standby.

What kept unmapped pages off standby, fixed after the first measurement:
- **Readahead.** `MmMakeSegmentResident` reads 16 KiB, and the pages the fault did not ask for
  were never mapped, so no unshare ever offered them to standby. They now go to standby
  straight from the read, as Windows' prefetch does.
- **Claim by inspection.** That same readahead scan, and `MmCheckDirtySegment` called from a
  flush, read each slot through Get, which pulled every standby page they passed back to the
  LRU. The scan now peeks (`MiPeekPageEntrySectionSegment`) at neighbours outside the requested
  range, and a flush puts a clean, unmapped page back on standby.

Re-measured with both (`20261017-132301-sdn0bd9x` 2G, `20261017-132301-nlm83rh0` 1024M,
both 54/54): at 2G avail goes 273.8k -> 255.4k (18.4k, from 41.4k in step 1). At the end,
24.2k pages are on standby and only 2.1k are unmapped but off it. 1024M repurposed 16k+ pages
and finished in 1529 s (1934 s before). What remains of the 2G drift is pages in use: ROS pages
still mapped (+11k over boot, the live desktop's images), private +3.1k, paged pool +1.7k,
page tables +1k.

Since COW and paged-pool commit charging (2026-10-17, `ARM3/commit.rs`), 1024M is limited by
commit rather than by pages. The ramdisk leaves a 129 MB limit, and iexplore under FEX is
refused there and dies (`20261017-141001-or20qi28`, 53/54). 1088M and 2G stay 54/54.
