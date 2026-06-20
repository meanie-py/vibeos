//! virtio-net device bring-up + the smoltcp `phy::Device` adapter.
//!
//! `bringup()` performs the PCI find + bus-master enable + virtio handshake and
//! returns a high-level `VirtIONet`. `VirtioPhy` wraps it as a smoltcp device.
//! Per the design (no zero-copy in v1) the RX path copies each frame into an
//! owned buffer and immediately recycles the virtio buffer — which also cleanly
//! sidesteps the smoltcp "RxToken + TxToken returned together" borrow problem
//! (the RxToken owns its bytes; only the TxToken borrows the device).

use alloc::vec;
use alloc::vec::Vec;

use smoltcp::phy::{self, Device, DeviceCapabilities, Medium};
use smoltcp::time::Instant;
use virtio_drivers::device::net::{TxBuffer, VirtIONet};
use virtio_drivers::transport::pci::{bus::PciRoot, PciTransport};

use crate::hal::NetHal;
use crate::pci::{enable_bus_master, find_virtio_net, HalPciAccess};

pub const QUEUE_SIZE: usize = 16;
pub const BUF_LEN: usize = 2048;

pub type NetDev = VirtIONet<NetHal, PciTransport, QUEUE_SIZE>;

/// Find, claim, and initialise the virtio-net device.
pub fn bringup() -> Option<NetDev> {
    let df = find_virtio_net()?;
    enable_bus_master(df);
    let mut root = PciRoot::new(HalPciAccess);
    let transport = PciTransport::new::<NetHal, _>(&mut root, df).ok()?;
    VirtIONet::new(transport, BUF_LEN).ok()
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
}

pub struct VirtioRxToken {
    data: Vec<u8>,
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
