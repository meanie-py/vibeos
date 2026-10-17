# Cached section pages as standby: design

Status (2026-10-17): step 1 is implemented and in the sweep; step 2 is this design, not code.
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

What step 1 does not give: the cached pages still do not count as available
(`GlobalMemoryStatus`, the census `avail`), and reclaim is asynchronous, so a burst can still
exhaust the free list before the balancer runs.

## Step 2: a real standby state for segment pages (not started)

The obstacle is ownership. ARM3 repurposes a standby page under the PFN lock at DISPATCH and
restores its owner through `PteAddress`/`OriginalPte`. A ROS page's owner is a segment page
entry (SSE) guarded by the segment's fast mutex, which cannot be taken there, and its
`OriginalPte` slot holds the rmap head (field manual, "The `OriginalPte` / `RmapListHead`
union overlay").

Proposed shape:

1. **A transition SSE.** When `MmUnsharePageEntrySectionSegment` drops a clean page's share count
   to 0, the segment entry gets a transition bit (as Windows' transition prototype PTE) and the
   PFN moves to a ROS-standby list under the PFN lock, counted in `MmAvailablePages`.
2. **Repurpose under the PFN lock only.** `MiRemoveAnyPage`/`MiRemoveZeroPage`, finding free and
   zero empty, take a ROS-standby page. They do not touch the SSE; they mark the PFN repurposed
   and leave the segment association for the segment side to clean up.
3. **Claim back under segment lock + PFN lock.** The section fault path, seeing a transition SSE,
   takes the PFN lock: if the page is still standby it unlinks it and re-shares it (a soft fault);
   if it was repurposed it clears the SSE and reads the page again.
4. **Segment teardown** frees its standby pages through the same claim path.

Validation: MEMCENSUS shows `avail` flat across a 2G sweep; the 1024M sweep degrades (slower,
re-faults) instead of failing apps; smoke on all three arches and the FEX ladder, since the
section fault path is in every boot.
