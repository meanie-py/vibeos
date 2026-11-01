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

## RESUME HERE (parked 2026-11-01, fleet wind-down for the XFS migration)

**State:** branch `win11-flip`, committed and clean, off master `558865d4c44`. Submitted to
merge-integrator at the tip that carries this note. The landing verdict was still pending
when I parked, so check `land-status` / the integrator's message before redoing anything.

**What it carries:** AppPolicyGet* (all eight), FlsGetValue2 and GetFileInformationByName in
kernel32-rust. Windows 8's token pseudo-handles (-4/-5/-6) in ob-rust's
ObReferenceObjectByHandle, and NtClose of -3..-6. Two winref-graded probes in
`tools/payloads/hostapps` (`w11api.c`, `w11tok.c`) with Windows 11's answers in
`w11*.win11.txt`. The version-flip audit closed. The i386 special-pool bug moved to the
roadmap registry.

**Green at submit:**
- check and check-tooling (110/110);
- smoke (warm), all three: `20261101-001548-y15lonbk` i386, `-41xtq54u` amd64, `-ttbctt9i`
  arm64;
- amd64 conformance, ratchet holds 107/107: `20261101-001803-gaobodf6`;
- hostapps with the probes: `20261031-231841-s7ryfcpn`. The guest diff against
  `w11*.win11.txt` matches line for line except the three divergences the compat roadmap
  lists.

Those `/tmp/vibeos-cli` directories do not survive the migration; their verdicts are
recorded here and in the commit messages.

**Next, in order:**
1. If the integrator bounced win11-flip, start from its artifact.
2. `FileStorageInfo` / `FileAlignmentInfo` on fastfat and cdfs. winref answers both; we give
   ERROR_NOT_SUPPORTED and 0. Measure Windows' FAT32 and CDFS answers first.
3. gpg-agent cannot start: CreateProcess's `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`. This is the
   one red line in hostapps (`gpg --symmetric` exits 2).
4. SVCFAIL tripwire on i386 and arm64 (amd64 only today).
5. Open questions in the roadmap registry:
   - the i386 `SPECIALPOOL=*` 0x0F: latent on master, lock `0x80741810`;
   - why arm64 ZwAllocateVirtualMemory's two threads swapped finishing order.

**Scratch worktree:** `../vibeos-spbase` is detached at master `a319085fd99` with the
self-test fix staged. It holds nothing unique, so `git worktree remove --force` it once
nobody needs the i386 special-pool A/B rebuilt.
