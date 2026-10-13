// etw.c's EtwRegister / EtwUnregister / EtwWrite are not defined here any more: ntoskrnl
// exports the real ones (wmi-rust's etw.rs) on every arch. This library is linked INTO
// drivers (wdf01000, kmtest_drv, the storage class drivers), so a copy here answered
// STATUS_NOT_IMPLEMENTED to every driver that linked it. (ntoskrnl itself has not linked
// this library since its cargo link dropped libntoskrnl_vista.a.)
