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

## RESUME HERE (2026-11-21, fleet resumed)

**Done this session:** RESUME items 1-4, and the i386 `SPECIALPOOL=*` 0x0F.
- The CD alignment query's err=87 came from scsiport (UniATA's port), not cdrom or CDFS: its
  LUN PDO answered unknown properties with STATUS_INVALID_PARAMETER_1. It now answers
  STATUS_INVALID_DEVICE_REQUEST, which gives Win32 1 like Windows 11 (`e701132fc12`, integrator
  batch 177).
- cdfs refuses `FILE_OPEN_REQUIRING_OPLOCK`, as measured on Windows 11 (fsquery's new
  `oplockopen` line), and fastfat's `FatCommonCreateCallout` has its try/except.
- gpg-agent starts and hostapps is 34/34 green. The cause was not CreateProcess's
  `PROC_THREAD_ATTRIBUTE_HANDLE_LIST` (still unapplied, but harmless here): netstack left a
  port-0 bind at port 0, so getsockname gave libassuan port 0. The `w32sock` probe grades it.

- `--tripwire SVCFAIL=<image|*>` logs failing system calls on i386 and arm64 too
  (`svcfail_log` in tripwire.rs, called from each arch's dispatcher).
- The i386 `SPECIALPOOL=*` 0x0F was the PFN lock held across a pool free in
  `MmDeleteSectionAssociation`. `SPECIALPOOL=*` now boots on all three arches.
- fastfat's spinlocks lock on amd64 and arm64: the C glue's `KeAcquireSpinLockRaiseToDpc`
  stand-in, which only raised IRQL, had captured the DDK macro there.

**Next, in order:**
1. The amd64 in-page 0x1E (registry): an overnight soak gave 180/180 clean
   (`20261122-002255-hmEvEu`). If it is worth more, soak with the hostapps payload, which is
   where it was seen, rather than plain smoke.
2. Registry open question: arm64 ZwAllocateVirtualMemory's thread order (benign).

**Scratch worktree:** `../vibeos-spbase` (detached, holds nothing unique). Leave it for
control-plane's prune.
