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

pub mod assert_to_ub;
pub mod bindgen_padding_index;
pub mod bitwise_not_bool;
pub mod bugcheck_codes;
pub mod byte_step_one;
pub mod byteptr_stride;
pub mod calloc_elem_type;
pub mod cdecl_alias;
pub mod containing_record_null;
pub mod dangling_transmute;
pub mod fourcc_literals;
pub mod gmem_flags;
pub mod match_arm_break;
pub mod narrow_vs_wide_literal;
pub mod noop_goto;
pub mod opaque_ptr_arith;
pub mod ptr_highword_mask;
pub mod raw_bitfield;
pub mod raw_uart_writes;
pub mod typed_byte_copy;
pub mod wide_byte_copy;

pub fn registry() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(byte_step_one::ByteStepOne),
        Box::new(wide_byte_copy::WideByteCopy),
        Box::new(typed_byte_copy::TypedByteCopy),
        Box::new(calloc_elem_type::CallocElemType),
        Box::new(containing_record_null::ContainingRecordNull),
        Box::new(dangling_transmute::DanglingTransmute),
        Box::new(bitwise_not_bool::BitwiseNotBool),
        Box::new(opaque_ptr_arith::OpaquePtrArith),
        Box::new(bugcheck_codes::BugcheckCodes),
        Box::new(raw_bitfield::RawBitfield),
        Box::new(gmem_flags::GmemFlags),
        Box::new(byteptr_stride::ByteptrStride),
        Box::new(noop_goto::NoopGoto),
        Box::new(cdecl_alias::CdeclAlias),
        Box::new(narrow_vs_wide_literal::NarrowVsWideLiteral),
        Box::new(ptr_highword_mask::PtrHighwordMask),
        Box::new(bindgen_padding_index::BindgenPaddingIndex),
        Box::new(raw_uart_writes::RawUartWrites),
        Box::new(fourcc_literals::FourccLiterals),
        Box::new(match_arm_break::MatchArmBreak),
        Box::new(assert_to_ub::AssertToUb),
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
