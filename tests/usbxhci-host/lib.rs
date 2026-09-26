//! Host regressions using the shipping miniport modules, with only NT services
//! replaced. On WSL run: python3 tests/usbxhci-host/run.py
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]

#[path = "../../sdk/lib/rust/usbxhci-rust/src/types.rs"]
mod types;
#[path = "../../sdk/lib/rust/usbxhci-rust/src/hw.rs"]
mod hw;
#[path = "../../sdk/lib/rust/usbxhci-rust/src/controller.rs"]
mod controller;
#[path = "../../sdk/lib/rust/usbxhci-rust/src/endpoint.rs"]
mod endpoint;
#[path = "../../sdk/lib/rust/usbxhci-rust/src/roothub.rs"]
mod roothub;

#[macro_export]
macro_rules! xlog {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {{ $(let _ = $arg;)* }};
}

static mut RegPacket: types::USBPORT_REGISTRATION_PACKET = unsafe { core::mem::zeroed() };

#[no_mangle]
unsafe extern "system" fn KeAcquireSpinLockRaiseToDpc(lock: *mut usize) -> u8 {
    let lock = &*(lock as *const std::sync::atomic::AtomicUsize);
    while lock.compare_exchange(0, 1, std::sync::atomic::Ordering::Acquire,
                                std::sync::atomic::Ordering::Relaxed).is_err() {
        std::thread::yield_now();
    }
    0
}

#[no_mangle]
unsafe extern "system" fn KeReleaseSpinLock(lock: *mut usize, _irql: u8) {
    (&*(lock as *const std::sync::atomic::AtomicUsize))
        .store(0, std::sync::atomic::Ordering::Release);
}

#[no_mangle]
unsafe extern "system" fn KeStallExecutionProcessor(_us: u32) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[repr(C, align(4096))]
    struct Page([u8; 4096]);

    #[repr(C, align(4096))]
    struct Payload([u8; 65536]);

    unsafe fn zero<T>() -> Box<T> {
        Box::new(core::mem::zeroed())
    }

    unsafe fn controller_memory(x: &mut controller::XhciExtension) -> Vec<Box<Page>> {
        let mut pages: Vec<_> = (0..5).map(|_| Box::new(Page([0; 4096]))).collect();
        x.cmd.init(pages[0].0.as_mut_ptr(), 0x3000);
        x.evt_va = pages[1].0.as_mut_ptr() as *mut hw::Trb;
        x.evt_pa = 0x4000;
        x.evt_cycle = 1;
        x.rt = pages[2].0.as_mut_ptr();
        x.db = pages[3].0.as_mut_ptr();
        x.op = pages[4].0.as_mut_ptr();
        hw::wr32(x.op, hw::USBSTS, hw::USBSTS_HCH);
        hw::wr32(x.op, hw::USBCMD, hw::USBCMD_RS | hw::USBCMD_INTE);
        x.started = true;
        pages
    }

    unsafe fn submit(x: &mut controller::XhciExtension, ep: &mut endpoint::XhciEndpoint,
                     td: &mut endpoint::XhciTransfer, params: &mut types::USBPORT_TRANSFER_PARAMETERS,
                     buffer: *mut u8) -> types::MPSTATUS {
        let mut sg: types::USBPORT_SCATTER_GATHER_LIST = core::mem::zeroed();
        sg.MappedSystemVa = buffer as _;
        endpoint::XHCI_SubmitTransfer(x as *mut _ as _, ep as *mut _ as _, params,
                                      td as *mut _ as _, &mut sg)
    }

    #[test]
    fn dma_in_copies_only_received_bytes_and_preserves_neighboring_storage() {
        unsafe {
            let mut x: Box<controller::XhciExtension> = zero();
            let _memory = controller_memory(&mut x);
            let mut ep: Box<endpoint::XhciEndpoint> = zero();
            let mut td: Box<endpoint::XhciTransfer> = zero();
            let mut params: Box<types::USBPORT_TRANSFER_PARAMETERS> = zero();
            let mut ring = Box::new(Page([0; 4096]));
            let mut dma = Box::new(Payload([0; 65536]));
            let mut caller = [0xcc; 16];
            ep.own_ring.init(ring.0.as_mut_ptr(), 0x1000);
            ep.ring = &mut ep.own_ring;
            ep.payload_va = dma.0.as_mut_ptr();
            ep.payload_pa = 0x500000;
            ep.slot = 1;
            ep.dci = 3;
            ep.props.EndpointAddress = 0x81;
            ep.props.MaxPacketSize = 8;
            x.eps[1][3] = &mut *ep;
            params.TransferBufferLength = 8;
            assert_eq!(submit(&mut x, &mut ep, &mut td, &mut params, caller.as_mut_ptr().add(1)), 0);
            assert_eq!((*ep.own_ring.va).param_lo, 0x500000);
            assert_eq!(ep.payload_used, 1);
            dma.0[..5].copy_from_slice(&[1, 2, 3, 4, 5]); // device DMA
            // A separate CPU write sharing the caller's cache line must survive.
            caller[0] = 0x42;
            endpoint::on_transfer_event(&mut x, td.last_pa, (hw::CC_SHORT_PACKET << 24) | 3,
                                         (1 << 24) | (3 << 16));
            endpoint::XHCI_PollEndpoint(&mut *x as *mut _ as _, &mut *ep as *mut _ as _);
            assert_eq!(&caller[..9], &[0x42, 1, 2, 3, 4, 5, 0xcc, 0xcc, 0xcc]);
            assert_eq!(ep.payload_used, 0);
            assert_eq!(ep.count, 0);
        }
    }

    #[test]
    fn dma_arena_backpressure_leaves_ring_unchanged_and_out_data_is_copied() {
        unsafe {
            let mut x: Box<controller::XhciExtension> = zero();
            let _memory = controller_memory(&mut x);
            let mut ep: Box<endpoint::XhciEndpoint> = zero();
            let mut params: Box<types::USBPORT_TRANSFER_PARAMETERS> = zero();
            let mut ring = Box::new(Page([0; 4096]));
            let mut dma = Box::new(Payload([0; 65536]));
            let mut caller = [0x7e; 8];
            let mut tds: Vec<Box<endpoint::XhciTransfer>> = (0..17).map(|_| zero()).collect();
            ep.own_ring.init(ring.0.as_mut_ptr(), 0x1000);
            ep.ring = &mut ep.own_ring;
            ep.payload_va = dma.0.as_mut_ptr();
            ep.payload_pa = 0x500000;
            ep.slot = 1;
            ep.dci = 2;
            ep.props.EndpointAddress = 1;
            ep.props.MaxPacketSize = 8;
            x.eps[1][2] = &mut *ep;
            params.TransferBufferLength = 8;
            for i in 0..16 {
                assert_eq!(submit(&mut x, &mut ep, &mut tds[i], &mut params, caller.as_mut_ptr()), 0);
                assert_eq!(&dma.0[i * 4096..i * 4096 + 8], &caller);
            }
            assert_eq!(ep.payload_used, 0xffff);
            let before = ep.own_ring.enqueue;
            assert_eq!(submit(&mut x, &mut ep, &mut tds[16], &mut params, caller.as_mut_ptr()), types::MP_STATUS_FAILURE);
            assert_eq!(ep.own_ring.enqueue, before);
            endpoint::on_transfer_event(&mut x, tds[0].last_pa, hw::CC_SUCCESS << 24, (1 << 24) | (2 << 16));
            endpoint::XHCI_PollEndpoint(&mut *x as *mut _ as _, &mut *ep as *mut _ as _);
            assert_eq!(submit(&mut x, &mut ep, &mut tds[16], &mut params, caller.as_mut_ptr()), 0);
            assert_eq!(tds[16].payload_first, 0);
        }
    }

    #[test]
    fn command_timeout_halts_before_failing_requests_and_blocks_new_commands() {
        unsafe {
            let (mut x, mut ep, td) = event_fixture();
            let _memory = controller_memory(&mut x);
            let result = controller::command(&mut x, 0, 0, hw::TRB_NOOP_COMMAND << hw::TRB_TYPE_SHIFT);
            assert_eq!(result.code, 0);
            assert!(x.failed.load(Ordering::Acquire));
            assert!(!x.started);
            assert_eq!(hw::rd32(x.op, hw::USBCMD) & (hw::USBCMD_RS | hw::USBCMD_INTE), 0);
            assert!(td.done.load(Ordering::Acquire));
            assert!(ep.halted);
            let before = x.cmd.enqueue;
            controller::command(&mut x, 0, 0, hw::TRB_NOOP_COMMAND << hw::TRB_TYPE_SHIFT);
            assert_eq!(x.cmd.enqueue, before);
            let mut next: Box<endpoint::XhciTransfer> = zero();
            let mut params: Box<types::USBPORT_TRANSFER_PARAMETERS> = zero();
            assert_eq!(submit(&mut x, &mut ep, &mut next, &mut params, core::ptr::null_mut()), 0);
            assert!(next.done.load(Ordering::Acquire));
            assert_eq!(next.cc, 0);
        }
    }

    #[test]
    fn failed_stop_halts_before_abort_returns_the_buffer() {
        unsafe {
            let (mut x, mut ep, mut td) = event_fixture();
            let _memory = controller_memory(&mut x);
            *x.evt_va = hw::Trb {
                param_lo: 0x3000, param_hi: 0, status: hw::CC_TRB << 24,
                control: (hw::TRB_EV_COMMAND_COMPLETION << hw::TRB_TYPE_SHIFT) | 1,
            };
            let mut completed = 99;
            endpoint::XHCI_AbortTransfer(&mut *x as *mut _ as _, &mut *ep as *mut _ as _,
                                         &mut *td as *mut _ as _, &mut completed);
            assert!(x.failed.load(Ordering::Acquire));
            assert_eq!(hw::rd32(x.op, hw::USBCMD) & hw::USBCMD_RS, 0);
            assert_eq!(ep.count, 0);
            assert_eq!(completed, 0);
        }
    }

    #[test]
    fn failed_endpoint_drop_halts_before_close_returns() {
        unsafe {
            let (mut x, mut ep, _td) = event_fixture();
            let _memory = controller_memory(&mut x);
            let mut common = vec![0u64; controller::COMMON_BUFFER_SIZE / 8];
            x.va = common.as_mut_ptr() as *mut u8;
            x.input_ctx_va = x.va.add(controller::PAGE);
            x.ctx_size = 32;
            x.slots[1].in_use = true;
            x.slots[1].max_dci = 3;
            ep.dci = 3;
            ep.count = 0;
            x.eps[1][1] = core::ptr::null_mut();
            x.eps[1][3] = &mut *ep;
            *x.evt_va = hw::Trb {
                param_lo: 0x3000, param_hi: 0, status: hw::CC_TRB << 24,
                control: (hw::TRB_EV_COMMAND_COMPLETION << hw::TRB_TYPE_SHIFT) | 1,
            };
            endpoint::XHCI_CloseEndpoint(&mut *x as *mut _ as _, &mut *ep as *mut _ as _, 0);
            assert!(x.failed.load(Ordering::Acquire));
            assert_eq!(hw::rd32(x.op, hw::USBCMD) & hw::USBCMD_RS, 0);
            assert!(x.eps[1][3].is_null());
        }
    }

    #[test]
    fn hidden_td_stays_unowned_across_wrap_until_published() {
        unsafe {
            let mut mem = Box::new(Page([0; 4096]));
            let mut ring: Box<controller::Ring> = zero();
            ring.init(mem.0.as_mut_ptr(), 0x1000);
            ring.enqueue = 254;
            ring.push_hidden(0, 8, (hw::TRB_SETUP << hw::TRB_TYPE_SHIFT) | hw::TRB_IDT);
            ring.push(0, 0, (hw::TRB_STATUS << hw::TRB_TYPE_SHIFT) | hw::TRB_IOC);
            assert_eq!((*ring.va.add(254)).control & hw::TRB_CYCLE, 0);
            assert_eq!((*ring.va.add(255)).control & hw::TRB_CYCLE, 1);
            assert_eq!((*ring.va).control & hw::TRB_CYCLE, 0);
            ring.publish(254, 1);
            assert_eq!((*ring.va.add(254)).control & hw::TRB_CYCLE, 1);
            assert_eq!(ring.enqueue, 1);
            assert_eq!(ring.cycle, 0);
        }
    }

    #[test]
    fn full_software_queue_does_not_touch_the_hardware_ring() {
        unsafe {
            let mut x: Box<controller::XhciExtension> = zero();
            let mut ep: Box<endpoint::XhciEndpoint> = zero();
            let mut td: Box<endpoint::XhciTransfer> = zero();
            let mut params: Box<types::USBPORT_TRANSFER_PARAMETERS> = zero();
            let mut sg: Box<types::USBPORT_SCATTER_GATHER_LIST> = zero();
            let mut page = Box::new(Page([0; 4096]));
            ep.own_ring.init(page.0.as_mut_ptr(), 0x1000);
            ep.ring = &mut ep.own_ring;
            ep.slot = 1;
            ep.count = 32;
            let result = endpoint::XHCI_SubmitTransfer(
                &mut *x as *mut _ as _, &mut *ep as *mut _ as _,
                &mut *params, &mut *td as *mut _ as _, &mut *sg);
            assert_eq!(result, types::MP_STATUS_FAILURE);
            assert_eq!(ep.own_ring.enqueue, 0);
            assert_eq!(ep.count, 32);
            assert_eq!((*ep.own_ring.va).control, 0);
        }
    }

    unsafe fn event_fixture() -> (Box<controller::XhciExtension>, Box<endpoint::XhciEndpoint>, Box<endpoint::XhciTransfer>) {
        let mut x: Box<controller::XhciExtension> = zero();
        let mut ep: Box<endpoint::XhciEndpoint> = zero();
        let mut td: Box<endpoint::XhciTransfer> = zero();
        ep.is_control = true;
        ep.slot = 1;
        ep.dci = 1;
        ep.own_ring.pa = 0x1000;
        ep.ring = &mut ep.own_ring;
        ep.pending[0] = &mut *td;
        ep.count = 1;
        x.eps[1][1] = &mut *ep;
        td.first = 0;
        td.ntrbs = 3;
        td.ndata = 1;
        td.data_pa[0] = 0x1010;
        td.data_len[0] = 8;
        td.requested = 8;
        td.last_pa = 0x1020;
        (x, ep, td)
    }

    #[test]
    fn setup_stage_error_completes_and_halts_control_endpoint() {
        unsafe {
            let (mut x, ep, td) = event_fixture();
            endpoint::on_transfer_event(&mut x, 0x1000, hw::CC_USB_TRANSACTION << 24,
                                        (1 << 24) | (1 << 16));
            assert!(td.done.load(Ordering::SeqCst));
            assert_eq!(td.cc, hw::CC_USB_TRANSACTION);
            assert!(ep.halted);
        }
    }

    #[test]
    fn stop_event_records_progress_without_completing_the_head() {
        unsafe {
            let (mut x, ep, td) = event_fixture();
            endpoint::on_transfer_event(&mut x, 0x1010, (hw::CC_STOPPED << 24) | 3,
                                        (1 << 24) | (1 << 16));
            assert!(!td.done.load(Ordering::SeqCst));
            assert_eq!(td.transferred, 5);
            assert!(!ep.halted);
        }
    }

    #[test]
    fn control_short_packet_waits_for_status_and_keeps_actual_length() {
        unsafe {
            let (mut x, _ep, td) = event_fixture();
            let control = (1 << 24) | (1 << 16);
            endpoint::on_transfer_event(&mut x, 0x1010, (hw::CC_SHORT_PACKET << 24) | 3, control);
            assert!(!td.done.load(Ordering::SeqCst));
            endpoint::on_transfer_event(&mut x, 0x1020, hw::CC_SUCCESS << 24, control);
            assert!(td.done.load(Ordering::SeqCst));
            assert_eq!(td.transferred, 5);
        }
    }

    #[test]
    fn aborting_a_later_td_keeps_the_stopped_head_pending() {
        unsafe {
            let (mut x, mut ep, head) = event_fixture();
            let mut target: Box<endpoint::XhciTransfer> = zero();
            let mut transfers = Box::new(Page([0; 4096]));
            let mut commands = Box::new(Page([0; 4096]));
            let mut events = Box::new(Page([0; 4096]));
            let mut runtime = Box::new(Page([0; 4096]));
            let mut doorbells = Box::new(Page([0; 4096]));
            ep.own_ring.va = transfers.0.as_mut_ptr() as *mut hw::Trb;
            ep.own_ring.enqueue = 4;
            ep.own_ring.cycle = 1;
            ep.slot = 1;
            ep.dci = 1;
            ep.count = 2;
            ep.pending[1] = &mut *target;
            target.first = 3;
            target.first_cycle = 1;
            target.ntrbs = 1;
            target.end = 4;
            (*ep.own_ring.va.add(3)).control = (hw::TRB_NORMAL << hw::TRB_TYPE_SHIFT) | 1;
            x.cmd.init(commands.0.as_mut_ptr(), 0x3000);
            x.evt_va = events.0.as_mut_ptr() as *mut hw::Trb;
            x.evt_pa = 0x4000;
            x.evt_cycle = 1;
            x.rt = runtime.0.as_mut_ptr();
            x.db = doorbells.0.as_mut_ptr();
            *x.evt_va = hw::Trb {
                param_lo: 0x1010, param_hi: 0, status: (hw::CC_STOPPED << 24) | 3,
                control: (hw::TRB_EV_TRANSFER << hw::TRB_TYPE_SHIFT) | (1 << 24) | (1 << 16) | 1,
            };
            *x.evt_va.add(1) = hw::Trb {
                param_lo: 0x3000, param_hi: 0, status: hw::CC_SUCCESS << 24,
                control: (hw::TRB_EV_COMMAND_COMPLETION << hw::TRB_TYPE_SHIFT) | 1,
            };
            let mut completed = 99;
            endpoint::XHCI_AbortTransfer(&mut *x as *mut _ as _, &mut *ep as *mut _ as _,
                                         &mut *target as *mut _ as _, &mut completed);
            assert_eq!(ep.count, 1);
            assert!(!head.done.load(Ordering::SeqCst));
            assert_eq!(head.transferred, 5);
            assert_eq!(hw::trb_type((*ep.own_ring.va.add(3)).control), 8);
            assert_eq!(completed, 0);
            assert_eq!(ep.deq, 0);
            // The command consumer must preserve the notification for the DPC.
            assert!(x.pending_transfer.load(Ordering::Acquire));
        }
    }
}
