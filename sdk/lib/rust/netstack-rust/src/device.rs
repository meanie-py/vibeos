//! virtio-net device bring-up + the smoltcp `phy::Device` adapter.
//!
//! `bringup()` performs the PCI find + bus-master enable + virtio handshake and
//! returns a virtio-net device. `VirtioPhy` wraps it as a smoltcp device.
//! Per the design (no zero-copy in v1) the RX path copies each frame into an
//! owned buffer and immediately recycles the virtio buffer — which also cleanly
//! sidesteps the smoltcp "RxToken + TxToken returned together" borrow problem
//! (the RxToken owns its bytes; only the TxToken borrows the device).

use alloc::vec;
use alloc::vec::Vec;
use core::mem::size_of;

use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;
use virtio_drivers::device::net::TxBuffer;
use virtio_drivers::queue::VirtQueue;
use virtio_drivers::transport::{DeviceStatus, InterruptStatus, Transport};
use virtio_drivers::transport::pci::{bus::PciRoot, PciTransport};

use crate::hal::NetHal;
use crate::pci::{enable_bus_master, find_virtio_net, HalPciAccess};

pub const QUEUE_SIZE: usize = 16;
pub const BUF_LEN: usize = 2048;
const QUEUE_RECEIVE: u16 = 0;
const QUEUE_TRANSMIT: u16 = 1;
const VIRTIO_NET_HDR_LEN: usize = 12;
const VIRTIO_NET_HDR_LEGACY_LEN: usize = 10;
const FEAT_MAC: u64 = 1 << 5;
const FEAT_STATUS: u64 = 1 << 16;
const FEAT_MRG_RXBUF: u64 = 1 << 15;
const FEAT_RING_INDIRECT_DESC: u64 = 1 << 28;
const FEAT_RING_EVENT_IDX: u64 = 1 << 29;
const FEAT_VERSION_1: u64 = 1 << 32;
const SUPPORTED_FEATURES: u64 =
    FEAT_MAC | FEAT_STATUS | FEAT_RING_INDIRECT_DESC | FEAT_RING_EVENT_IDX | FEAT_VERSION_1;

pub struct NetDev {
    transport: PciTransport,
    mac: [u8; 6],
    recv_queue: VirtQueue<NetHal, QUEUE_SIZE>,
    send_queue: VirtQueue<NetHal, QUEUE_SIZE>,
    rx_buffers: [Option<RxBuffer>; QUEUE_SIZE],
    legacy_header: bool,
}

/// Find, claim, and initialise the virtio-net device.
pub fn bringup() -> Option<NetDev> {
    let df = find_virtio_net()?;
    enable_bus_master(df);
    let mut root = PciRoot::new(HalPciAccess);
    let mut transport = PciTransport::new::<NetHal, _>(&mut root, df).ok()?;

    transport.set_status(DeviceStatus::empty());
    transport.set_status(DeviceStatus::ACKNOWLEDGE | DeviceStatus::DRIVER);
    let features = transport.read_device_features();
    let negotiated = features & SUPPORTED_FEATURES;
    transport.write_driver_features(negotiated);
    transport.set_status(
        DeviceStatus::ACKNOWLEDGE | DeviceStatus::DRIVER | DeviceStatus::FEATURES_OK,
    );
    if !transport.get_status().contains(DeviceStatus::FEATURES_OK) {
        return None;
    }

    let mac = transport.read_config_space::<[u8; 6]>(0).ok()?;
    let indirect = negotiated & FEAT_RING_INDIRECT_DESC != 0;
    let event_idx = negotiated & FEAT_RING_EVENT_IDX != 0;
    let send_queue =
        VirtQueue::<NetHal, QUEUE_SIZE>::new(&mut transport, QUEUE_TRANSMIT, indirect, event_idx)
            .ok()?;
    let mut recv_queue =
        VirtQueue::<NetHal, QUEUE_SIZE>::new(&mut transport, QUEUE_RECEIVE, indirect, event_idx)
            .ok()?;
    transport.finish_init();

    let legacy_header = negotiated & FEAT_VERSION_1 == 0 && negotiated & FEAT_MRG_RXBUF == 0;
    const NONE_BUF: Option<RxBuffer> = None;
    let mut rx_buffers = [NONE_BUF; QUEUE_SIZE];
    for (i, slot) in rx_buffers.iter_mut().enumerate() {
        let mut rx = RxBuffer::new(i, BUF_LEN, legacy_header);
        let token = unsafe { recv_queue.add(&[], &mut [rx.as_bytes_mut()]).ok()? };
        if token as usize != i {
            return None;
        }
        if recv_queue.should_notify() {
            transport.notify(QUEUE_RECEIVE);
        }
        *slot = Some(rx);
    }

    Some(NetDev {
        transport,
        mac,
        recv_queue,
        send_queue,
        rx_buffers,
        legacy_header,
    })
}

pub struct VirtioPhy {
    dev: NetDev,
}

impl VirtioPhy {
    pub fn new(dev: NetDev) -> Self {
        Self { dev }
    }
    pub fn mac(&self) -> [u8; 6] {
        self.dev.mac_address()
    }
    /// Enable the device's queue interrupts (clears the avail-ring NO_INTERRUPT
    /// suppression so the used-ring advance asserts INTx). Called once at PASSIVE
    /// before the ISR is connected (M10).
    pub fn enable_interrupts(&mut self) {
        self.dev.enable_interrupts();
    }
    /// Raw pointer to the underlying device, for the M10 ISR's `ack_interrupt`.
    /// This is the ONLY cross-thread access to the device; see `ack_interrupt`.
    pub fn dev_ptr(&mut self) -> *mut NetDev {
        &mut self.dev
    }
}

/// Acknowledge a virtio interrupt from ISR context and report the raw ISR status
/// bits (bit0 = QUEUE_INTERRUPT — the used ring advanced). The read clears the
/// device's ISR status register, deasserting legacy INTx.
///
/// SAFETY / aliasing: `dev` is the worker's device, which the worker also holds
/// `&mut` to during `iface.poll`. This is sound because (a) the underlying
/// operation is a single volatile read of the ISR *MMIO register*, which is
/// disjoint from every queue descriptor / `NetDev` field the worker mutates, and
/// (b) a read-to-clear of that register is safe to race with the worker on
/// another CPU, since the worker never reads it. The ISR must touch nothing else
/// on the device.
pub unsafe fn ack_interrupt(dev: *mut NetDev) -> u32 {
    (*dev).ack_interrupt().bits()
}

pub struct VirtioRxToken {
    data: Vec<u8>,
}

pub struct RxBuffer {
    buf: Vec<usize>,
    packet_len: usize,
    idx: u16,
    legacy_header: bool,
}

impl RxBuffer {
    fn new(idx: usize, buf_len: usize, legacy_header: bool) -> Self {
        Self {
            buf: vec![0; buf_len / size_of::<usize>()],
            packet_len: 0,
            idx: idx as u16,
            legacy_header,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        let len = self.buf.len() * size_of::<usize>();
        unsafe { core::slice::from_raw_parts(self.buf.as_ptr() as *const u8, len) }
    }

    fn as_bytes_mut(&mut self) -> &mut [u8] {
        let len = self.buf.len() * size_of::<usize>();
        unsafe { core::slice::from_raw_parts_mut(self.buf.as_mut_ptr() as *mut u8, len) }
    }

    pub fn packet(&self) -> &[u8] {
        let hdr_size = if self.legacy_header {
            VIRTIO_NET_HDR_LEGACY_LEN
        } else {
            VIRTIO_NET_HDR_LEN
        };
        &self.as_bytes()[hdr_size..hdr_size + self.packet_len]
    }
}

impl NetDev {
    pub fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    pub fn ack_interrupt(&mut self) -> InterruptStatus {
        self.transport.ack_interrupt()
    }

    pub fn enable_interrupts(&mut self) {
        self.send_queue.set_dev_notify(true);
        self.recv_queue.set_dev_notify(true);
    }

    pub fn can_recv(&self) -> bool {
        self.recv_queue.peek_used().is_some()
    }

    pub fn can_send(&self) -> bool {
        self.send_queue.available_desc() >= 2
    }

    pub fn receive(&mut self) -> Result<RxBuffer, ()> {
        let token = self.recv_queue.peek_used().ok_or(())?;
        let mut rx = self.rx_buffers[token as usize].take().ok_or(())?;
        if token != rx.idx {
            return Err(());
        }
        let len = unsafe {
            self.recv_queue
                .pop_used(token, &[], &mut [rx.as_bytes_mut()])
                .map_err(|_| ())?
        } as usize;
        let hdr_size = if rx.legacy_header {
            VIRTIO_NET_HDR_LEGACY_LEN
        } else {
            VIRTIO_NET_HDR_LEN
        };
        rx.packet_len = len.saturating_sub(hdr_size);
        Ok(rx)
    }

    pub fn recycle_rx_buffer(&mut self, mut rx: RxBuffer) -> Result<(), ()> {
        let token = unsafe {
            self.recv_queue
                .add(&[], &mut [rx.as_bytes_mut()])
                .map_err(|_| ())?
        };
        if self.rx_buffers[token as usize].is_some() {
            return Err(());
        }
        if self.recv_queue.should_notify() {
            self.transport.notify(QUEUE_RECEIVE);
        }
        rx.idx = token;
        self.rx_buffers[token as usize] = Some(rx);
        Ok(())
    }

    pub fn send(&mut self, tx_buf: TxBuffer) -> Result<(), ()> {
        let header = [0u8; VIRTIO_NET_HDR_LEN];
        let header_len = if self.legacy_header {
            VIRTIO_NET_HDR_LEGACY_LEN
        } else {
            VIRTIO_NET_HDR_LEN
        };
        self.send_queue
            .add_notify_wait_pop(
                &[&header[..header_len], tx_buf.packet()],
                &mut [],
                &mut self.transport,
            )
            .map(|_| ())
            .map_err(|_| ())
    }
}

impl phy::RxToken for VirtioRxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.data)
    }
}

pub struct VirtioTxToken<'a> {
    dev: &'a mut NetDev,
}

impl<'a> phy::TxToken for VirtioTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut buf = vec![0u8; len];
        let r = f(&mut buf);
        let _ = self.dev.send(TxBuffer::from(&buf));
        r
    }
}

impl Device for VirtioPhy {
    type RxToken<'a> = VirtioRxToken;
    type TxToken<'a> = VirtioTxToken<'a>;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        if !self.dev.can_recv() {
            return None;
        }
        let rxbuf = self.dev.receive().ok()?;
        let data = rxbuf.packet().to_vec();
        let _ = self.dev.recycle_rx_buffer(rxbuf);
        Some((VirtioRxToken { data }, VirtioTxToken { dev: &mut self.dev }))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        if !self.dev.can_send() {
            return None;
        }
        Some(VirtioTxToken { dev: &mut self.dev })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.medium = Medium::Ethernet;
        caps.max_transmission_unit = 1500;
        caps.max_burst_size = Some(1);
        caps
    }
}
