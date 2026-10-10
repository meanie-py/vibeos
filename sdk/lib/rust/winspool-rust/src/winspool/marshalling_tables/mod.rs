//! The `static const MARSHALLING` tables of win32ss/printing/include/marshalling/*.h as
//! winspool sees them: one module per header, each written by
//! `python -m rustconv.c_data_port win32ss/printing/base/winspool/<h>.c --crate winspool-rust \
//!  --header marshalling/<h>.h --headers-only --per-arch --public --no-mod`
//! (clang decides every byte, per arch). Re-exported flat from the crate prelude, under
//! their C names.
pub mod forms;
pub mod jobs;
pub mod monitors;
pub mod ports;
pub mod printerdrivers;
pub mod printers;
pub mod printprocessors;

// Named, not globbed: every header module also defines its own layout twins (`MARSHALLING_x5`).
pub use forms::{FormInfo1Marshalling, FormInfo2Marshalling, pFormInfoMarshalling};
pub use jobs::{AddJobInfo1Marshalling, JobInfo1Marshalling, JobInfo2Marshalling, pJobInfoMarshalling};
pub use monitors::{MonitorInfo1Marshalling, MonitorInfo2Marshalling, pMonitorInfoMarshalling};
pub use ports::{PortInfo1Marshalling, PortInfo2Marshalling, pPortInfoMarshalling};
pub use printerdrivers::{PrinterDriver1Marshalling, PrinterDriver2Marshalling, PrinterDriver3Marshalling, PrinterDriver4Marshalling, PrinterDriver5Marshalling, PrinterDriver6Marshalling, PrinterDriver8Marshalling, pPrinterDriverMarshalling};
pub use printers::{PrinterInfoStressMarshalling, PrinterInfo1Marshalling, PrinterInfo2Marshalling, PrinterInfo3Marshalling, PrinterInfo4Marshalling, PrinterInfo5Marshalling, PrinterInfo6Marshalling, PrinterInfo7Marshalling, PrinterInfo8Marshalling, PrinterInfo9Marshalling, pPrinterInfoMarshalling};
pub use printprocessors::{DatatypesInfo1Marshalling, PrintProcessorInfo1Marshalling};
