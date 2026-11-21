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

**Done this session:** RESUME items 1 and 2.
- The CD alignment query's err=87 came from scsiport (UniATA's port), not cdrom or CDFS: its
  LUN PDO answered unknown properties with STATUS_INVALID_PARAMETER_1. It now answers
  STATUS_INVALID_DEVICE_REQUEST, which gives Win32 1 like Windows 11 (`e701132fc12`, integrator
  batch 177).
- cdfs refuses `FILE_OPEN_REQUIRING_OPLOCK`, as measured on Windows 11 (fsquery's new
  `oplockopen` line), and fastfat's `FatCommonCreateCallout` has its try/except.

**Next, in order:**
1. gpg-agent cannot start because of CreateProcess's `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`. It
   is the one red line in hostapps.
2. SVCFAIL tripwire on i386 and arm64 (amd64 only today).
3. Soak candidate, never run: the amd64 `STATUS_NONEXISTENT_SECTOR` in-page 0x1E. It was seen
   once (hostapps `20261101-123017-7ue7skcb`) and has been clean since; see the roadmap
   registry. Run `./vibeos-cli farm --arches amd64 --minutes 480`, then `farm-report`.
4. Registry open questions: the i386 `SPECIALPOOL=*` 0x0F (deterministic, so debug it rather
   than soak it), and arm64 ZwAllocateVirtualMemory's thread order (benign).

**Scratch worktree:** `../vibeos-spbase` (detached, holds nothing unique). Leave it for
control-plane's prune.
