use crate::prelude::*;

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const E1000_TAG: u32 = 0x3030_3165; // '001e' multi-char constant




#[repr(C)]
pub struct _E1000_ADAPTER_MULTICAST_ENTRY {
    pub MacAddress: [UCHAR; (IEEE_802_ADDR_LENGTH as usize)],
}

pub unsafe extern "C" fn E1000ReadUlong(
    mut Adapter: PE1000_ADAPTER,
    mut Address: ULONG,
    mut Value: PULONG,
) {
    NdisReadRegisterUlong!(
        (*Adapter).IoBase.offset(Address as isize) as PULONG,
        Value,
    );
}

pub unsafe extern "C" fn E1000WriteUlong(
    mut Adapter: PE1000_ADAPTER,
    mut Address: ULONG,
    mut Value: ULONG,
) {
    NdisWriteRegisterUlong!((*Adapter).IoBase.add(Address as usize) as PULONG, Value);
}

pub unsafe extern "C" fn E1000WriteIoUlong(
    Adapter: PE1000_ADAPTER,
    Address: ULONG,
    Value: ULONG,
) {
    let mut Dummy: ULONG = 0;

    NdisRawWritePortUlong!((*Adapter).IoPort as PULONG, Address);
    NdisReadRegisterUlong!(
        (*Adapter).IoBase.offset(E1000_REG_STATUS as isize),
        core::ptr::addr_of_mut!(Dummy),
    );
    NdisRawWritePortUlong!(
        (*Adapter).IoPort.offset(4 as isize) as PULONG,
        Value,
    );
}

pub unsafe extern "C" fn NICApplyInterruptMask(Adapter: PE1000_ADAPTER) {
    E1000WriteUlong(Adapter, E1000_REG_IMS as ULONG, (*Adapter).InterruptMask as ULONG);
}

pub unsafe extern "C" fn NICDisableInterrupts(Adapter: PE1000_ADAPTER) {
    E1000WriteUlong(Adapter, E1000_REG_IMC, !(0 as ULONG));
}
