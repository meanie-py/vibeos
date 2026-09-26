# xHCI review — 2026-09-26

Reviewed the Rust controller, endpoint, root-hub and hardware modules at
`91c11442467`, plus usbport's miniport locking, common-buffer allocation and
transfer mapping/completion paths. Fixes are deliberately uncommitted.

## Fixed

- **Untracked DMA after queue exhaustion.** SubmitTransfer wrote live TRBs before
  discovering that its 32-entry pending queue was full. Capacity is now checked
  first. The first TRB stays CPU-owned until the whole request and its pending
  record exist; an active controller need not wait for another doorbell.
- **Lost command completions on SMP.** The waiter address was recorded after
  publishing its command. It is now installed before publication, under the event
  consumer's lock.
- **Ignored Setup-stage errors.** Transfer-event matching checked only Data and
  Status TRBs. It now covers the request's complete ring span, including wrap
  links, so an error on Setup completes the request and records the halt.
- **Cancellation falsely completing another request.** Stop Endpoint events were
  treated as ordinary failed completions. Aborting a later request could therefore
  complete the active head, even though hardware would resume it. Stop events now
  record progress without completing; cancellation also rechecks completion after
  the command wait, which can itself consume a racing real completion.
- **Endpoint event lookup lifetime.** Removal/publication now takes `evt_lock`,
  pairing with event consumption before usbport can free the endpoint object.
- **Lost notifications during synchronous command waits.** Those waits can consume
  port/transfer events. Their notifications are now latched for a later DPC/poll.
- **False success after failed context/recovery commands.** EP0 opening no longer
  accepts a failed Evaluate Context, and Reset Endpoint/Set TR Dequeue failures
  retain the halted state. Successful recovery rings the doorbell for queued work.
- **Payload cache coherency.** Each endpoint now owns a 64 KiB common-buffer
  arena. Transfers reserve whole pages with a bitmap, so concurrent requests and
  unrelated caller data cannot share a DMA cache line. OUT bytes are copied into
  that arena and cleaned; IN pages are cleaned before handoff, invalidated after
  completion/stop, and only the received bytes are copied back. The controller
  never writes directly into a caller's pool buffer. TRBs split at page boundaries.
  usbport still builds S/G for request splitting, but skips its final HAL flush
  for xHCI, preventing an unused HAL bounce mapping from overwriting those bytes;
  it still frees its map registers.
- **Unsafe buffer release on failed commands.** A timed-out command now halts the
  controller before its shared input context can be reused. Failed Stop Endpoint,
  endpoint drop, Disable Slot, and recovery commands do the same before any buffer
  release. Pending requests complete with device-not-responding, new commands are
  blocked, and subsequent transfers complete as errors rather than retrying forever.
  If hardware will not halt, panic-common bugchecks instead of returning DMA-owned
  memory to usbport. Acknowledged hardware errors are latched for CheckController.
- **Incomplete power management reporting success.** Root-port suspend/resume and
  controller resume now report unsupported. Controller suspend quiesces DMA. This
  prevents the old USB2 U3-to-U0 shortcut or no-op resume from claiming success.

The hardware rules were checked against Intel's
[xHCI 1.2b specification](https://cdrdv2-public.intel.com/625472/625472_xHCI_Rev1_2b.pdf),
especially sections 4.6.9 (Stop Endpoint), 4.9 (rings), and 4.10 (transfer events).

## Limits and follow-up

The cache and failure fixes above address code-path findings; they have not been
validated on a physical Pi. QEMU cannot validate non-coherent DMA. The fail-stop
policy is deliberately conservative: usbport's controller-reset callback is
unimplemented, so a failed controller requires a PnP restart/reboot rather than
automatic recovery. Existing feature limits remain: no downstream-hub topology,
SuperSpeed root hub, isochronous transfers, or suspend/resume support.

The private arena permits 16 simultaneous one-page requests or one 64 KiB request
per endpoint. If contiguous arena pages are unavailable, submission returns busy
before changing the hardware ring; usbport retries after completions release pages.
This trades memory copies and 64 KiB per pipe for explicit DMA ownership. The
general ARM64 HAL still assumes coherent DMA; this change protects usbxhci without
claiming to repair other drivers' DMA paths.

Keyboard and mouse interrupt-IN pipes are both implemented. The mouse regression
requires a report accepted by mouclass, using a one-time mouhid delivery marker.

## Regression coverage

- `python3 tests/usbxhci-host/run.py` runs the shipping Rust modules with NT
  service stubs on the Windows host (WSL, installed MSVC/SDK, repo LLVM linker).
  It tests ring-wrap publication, full-queue rejection, Setup errors, stopped
  transfer progress, control short packets, cancellation of a later TD, private
  IN/OUT data, arena exhaustion/reuse, timeout containment, and failed stop/drop.
- `VIBEOS_QEMU_XHCI=1 VIBEOS_QEMU_SMP=4 python -m pytest
  tests/qemu/test_xhci.py --arch arm64 --skip-clean-build -q` uses the existing
  QEMU fixture and checks media freshness before starting it. It requires a real
  xHCI boot, injects keys and mouse motion, and requires keyboard/class delivery
  and a mouse report accepted by mouclass.
  Run with the project's Python environment and freshly built media.

Host tests model command failures with fake registers and events, not real
hardware timing, and cannot execute ARM64 cache instructions.

## Validation — 2026-09-26

All results below cover the final code changes, left uncommitted for review.
CLI run IDs resolve under `/tmp/vibeos-cli/<run-id>/summary.json`.

| Check | Result | Run ID / log |
| --- | --- | --- |
| Cold ARM64 build | OK | `20260926-155210-0ceosq4u-build-arm64` |
| Cold amd64 build | OK | `20260926-155211-cmc44u4q-build-amd64` |
| Cold i386 build | OK | `20260926-155210-d7kmu1ej-build-i386` |
| ARM64 SMP4 xHCI smoke | Boot reached desktop; WARN only for opt-in SMP | `20260926-155503-jifxa53t-smoke-arm64` |
| amd64 xHCI smoke | OK | `20260926-155631-vr7309k1-smoke-amd64` |
| i386 xHCI smoke | OK | `20260926-155631-0l47bq4p-smoke-i386` |
| Source check | OK, no new lint findings | `20260926-155211-0emmhws0-check-arm64` |
| Tooling check | All 74 passed | `/tmp/usbxhci-check-tooling-final.log` |
| Host regressions | 11 passed | `/tmp/usbxhci-host-regressions-final.log` |
| ARM64 SMP4 keyboard/mouse | 2 passed; injected mouse dx=20/dy=10 accepted by mouclass | `/tmp/usbxhci-keyboard-mouse-final.log` |

All three smoke screenshots were inspected. Smokes used `VIBEOS_QEMU_XHCI=1`
and `--no-net`; networking was not tested. Physical Pi DMA/cache validation remains
outstanding. `git diff --check` also passed.
