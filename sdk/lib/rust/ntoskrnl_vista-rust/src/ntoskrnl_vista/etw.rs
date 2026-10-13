// etw.c's EtwRegister / EtwUnregister / EtwWrite are not defined here any more: ntoskrnl
// exports the real ones (wmi-rust's etw.rs) on every arch. This library is linked INTO
// drivers (wdf01000, kmtest_drv, the storage class drivers) and into ntoskrnl itself, so a
// copy here answered STATUS_NOT_IMPLEMENTED to every driver that linked it, and in the
// kernel it was a second definition of an export whose winner was link order.
