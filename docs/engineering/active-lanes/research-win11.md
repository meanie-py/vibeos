# research / win11-flip: behave like Windows 11

Worktree `vibeos-research`. Lane owner: the `research` session. The queues this lane works
from live in the docs below; this file holds only the lane's state.

- [../version-flip-audit.md](../version-flip-audit.md): the NT 5.2 -> 10.0 flip. **Closed
  2026-10-31.** The flip (NT 10.0 build 22631, Win8 handle biasing, kmtest floors re-seeded)
  landed as master `558865d4c44`, gate `20261031-231411-lh1fopt3`.
- [../modern-windows-compat-roadmap.md](../modern-windows-compat-roadmap.md): the API
  surface that is left. Its hostapps bullet holds the "Still owed" list.
- [../winref.md](../winref.md): the Windows 11 reference VM. Every contract on this lane is
  measured there first, and the probe and its Windows output are committed beside the code.

## RESUME HERE (parked 2026-11-08, fleet token-budget wind-down)

**State:** nothing unlanded. The lane's branch (`fsaudit-2`) is master plus this note. Landed
this week: win11-flip; Windows' storage/alignment queries on fastfat and cdfs
(`6dc85e47444`); and a line-by-line audit of both drivers' translations, with fixes in
`6dc85e47444` and `9c4f8342cc1`. The second round includes cdfs device IOCTLs on volume
handles, which had all been failing; the FAT32 BPB check; LBO truncation in
FatLookupLastMcbEntry; and the FSCTL_FILESYSTEM_GET_STATISTICS counters, which C stubs had
left at zero. The audit's leftovers are in the roadmap's fastfat/cdfs entry.

**Next, in order:**
1. The CD volume's `IOCTL_STORAGE_QUERY_PROPERTY` alignment query returns err=87, where
   Windows returns err=1 (`fsquery.win11.txt`, "===CD D:==="). This comes from cdrom/classpnp,
   not CDFS.
2. The audit's two open decisions (roadmap): cdfs's `FILE_OPEN_REQUIRING_OPLOCK` block is
   `#[cfg(any())]`, and FatCommonCreateCallout has no try/except.
3. gpg-agent cannot start because of CreateProcess's `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`. It
   is the one red line in hostapps.
4. SVCFAIL tripwire on i386 and arm64 (amd64 only today).
5. Soak candidate, never run: the amd64 `STATUS_NONEXISTENT_SECTOR` in-page 0x1E. It was seen
   once (hostapps `20261101-123017-7ue7skcb`) and has been clean since; see the roadmap
   registry. Run `./vibeos-cli farm --arches amd64 --minutes 480`, then `farm-report`.
6. Registry open questions: the i386 `SPECIALPOOL=*` 0x0F (deterministic, so debug it rather
   than soak it), and arm64 ZwAllocateVirtualMemory's thread order (benign).

**Scratch worktree:** `../vibeos-spbase` (detached, holds nothing unique). Leave it for
control-plane's prune.
