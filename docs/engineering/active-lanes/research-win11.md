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

## RESUME HERE (parked 2026-11-22, Sunday wind-down)

**State:** everything is landed through the integrator. The weekend's work: scsiport/cdfs/
fastfat (`6f395eed157`), netstack's bind-time ephemeral port, which gets gpg-agent starting
(`4804d979d88`), SVCFAIL on all arches (`2ea58ab9976`), `SPECIALPOOL=*` booting on all three
arches with the `scan_guard_drop_late` lint (`b895e8fddbf`), fastfat spinlocks taking the lock
on 64-bit (`09657ac1b49`), and `PROC_THREAD_ATTRIBUTE_HANDLE_LIST` applied by CreateProcess
(this branch, last). Overnight amd64 soak: 180/180 clean (`20261122-002255-hmEvEu`).

**HANDLE_LIST design** (kernel32 proc_create.rs, ps-rust query.rs, ob-rust inherit_list.rs):
kernel32 enforces Windows' ERROR_INVALID_PARAMETER cases and stages the list on the creating
thread through a private NtSetInformationThread class (`0x56420001`, defined in both
crates). ob-rust resolves the list to parent handle-table entries, and ObpDuplicateHandleCallback
inherits only those. The hostapps `inhlist` probe grades it against `inhlist.win11.txt`.

**Next, in order:**
0. The `inhlist` probe checks handle values, not identity. Under `list {A}` our child shows C
   open with inherit_flag=0: that is its own startup handle in the freed slot, not a leak (see
   the probe header). Name the events and compare names, then rerun winref. Winref timed out
   3/3 under load 26-31 on 2026-11-22, so run it on a quiet host.
1. `PROC_THREAD_ATTRIBUTE_PARENT_PROCESS` is still accepted and ignored (proc_create.rs
   comment). It needs CreateProcessInternalW to stop assuming NtCurrentProcess as the parent,
   and inherit_list.rs only filters inheritance from the creating process.
2. The amd64 in-page 0x1E (registry): an overnight soak gave 180/180 clean. If it is worth more,
   soak with the hostapps payload, where it was seen.
3. Registry open question: arm64 ZwAllocateVirtualMemory's thread order (benign).
4. Audit leftovers (roadmap fastfat/cdfs entry): the cdfs FsRtlIsLeadDbcsCharacter stand-in and
   SBCS-only RtlOemStringToCountedUnicodeSize, which matter only on DBCS OEM code pages.

**Scratch worktree:** `../vibeos-spbase` (detached, holds nothing unique). Leave it for
control-plane's prune.
