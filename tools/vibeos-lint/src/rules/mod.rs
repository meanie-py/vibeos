//! Register each plugin here; plugins have no subprocess or filesystem scan.
use crate::Rule;
pub mod const_narrowing_cast;
pub mod double_fetch;
pub mod dowhile_continue;
pub mod fnptr_abi_transmute;
pub mod hresult_literals;
pub mod info_class_literals;
pub mod init_then_move;
pub mod let_shadow;
pub mod list_walk_from_head;
pub mod misaligned_cast;
pub mod read_after_clear;
pub mod variadic_array_arg;

pub fn registry() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(list_walk_from_head::ListWalkFromHead),
        Box::new(dowhile_continue::DowhileContinue),
        Box::new(variadic_array_arg::VariadicArrayArg),
        Box::new(let_shadow::LetShadow),
        Box::new(double_fetch::DoubleFetch),
        Box::new(hresult_literals::HresultLiterals),
        Box::new(init_then_move::InitThenMove),
        Box::new(read_after_clear::ReadAfterClear),
        Box::new(const_narrowing_cast::ConstNarrowingCast),
        Box::new(info_class_literals::InfoClassLiterals),
        Box::new(misaligned_cast::MisalignedCast),
        Box::new(fnptr_abi_transmute::FnptrAbiTransmute),
    ]
}
