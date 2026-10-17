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

pub mod call_arg_swap;
pub mod comment_swallowed_code;
pub mod extern_arity;
pub mod formatless_printf;
pub mod ptr_typedef_cast_read;
pub mod sse_helpers;

pub mod constant_placement;
pub mod dropped_null_guard;
pub mod ptr_byte_diff;
pub mod raising_probes;

pub mod store_on_failure;

pub mod arg_side_effects;
pub mod assert_polarity;
pub mod case_bitor_pattern;
pub mod octal_literals;
pub mod raw_user_access;
pub mod safe_probe_deref;

pub mod doc_sections;
pub mod orphan_modules;
pub mod pack_drift;

pub mod bool_width;
pub mod bugcheck_stubs;
pub mod cc_mismatch;
pub mod dropped_asserts;
pub mod dropped_prints;
pub mod dropped_probes;
pub mod dropped_seh_pairs;
pub mod extern_static_width;
pub mod import_cc;
pub mod inline_flag_literals;
pub mod macro_fn_shadow;
pub mod nightly_surface;
pub mod rawdylib_linkname;
pub mod res_named_types;
pub mod type_alias_contradictions;
pub mod varargs_spec_arity;
pub mod wide_literals;
pub mod winerror_consts;

pub mod com_double_release;
pub mod duplicate_symbols;
pub mod export_shadows;
pub mod extern_decl_drift;
pub mod irql_balance;
pub mod ob_refcount;
pub mod unaligned_wide;
pub mod unpruned_walk;
pub mod varargs_shim_parity;

pub mod const_contradictions;
pub mod const_vs_island;
pub mod const_vs_own_island;
pub mod const_within_crate;

pub fn registry() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(const_contradictions::ConstContradictions),
        Box::new(const_vs_island::ConstVsIsland),
        Box::new(const_vs_own_island::ConstVsOwnIsland),
        Box::new(const_within_crate::ConstWithinCrate),
        Box::new(irql_balance::IrqlBalance),
        Box::new(ob_refcount::ObRefcount),
        Box::new(com_double_release::ComDoubleRelease),
        Box::new(unpruned_walk::UnprunedWalk),
        Box::new(extern_decl_drift::ExternDeclDrift),
        Box::new(duplicate_symbols::DuplicateSymbols),
        Box::new(export_shadows::ExportShadows),
        Box::new(unaligned_wide::UnalignedWide),
        Box::new(varargs_shim_parity::VarargsShimParity),
        Box::new(winerror_consts::WinerrorConsts),
        Box::new(wide_literals::WideLiterals),
        Box::new(bool_width::BoolWidth),
        Box::new(type_alias_contradictions::TypeAliasContradictions),
        Box::new(inline_flag_literals::InlineFlagLiterals),
        Box::new(res_named_types::ResNamedTypes),
        Box::new(dropped_prints::DroppedPrints),
        Box::new(dropped_probes::DroppedProbes),
        Box::new(dropped_seh_pairs::DroppedSehPairs),
        Box::new(dropped_asserts::DroppedAsserts),
        Box::new(nightly_surface::NightlySurface),
        Box::new(bugcheck_stubs::BugcheckStubs),
        Box::new(macro_fn_shadow::MacroFnShadow),
        Box::new(varargs_spec_arity::VarargsSpecArity),
        Box::new(import_cc::ImportCc),
        Box::new(cc_mismatch::CcMismatch),
        Box::new(extern_static_width::ExternStaticWidth),
        Box::new(rawdylib_linkname::RawdylibLinkname),
        Box::new(orphan_modules::OrphanModules),
        Box::new(pack_drift::PackDrift),
        Box::new(doc_sections::DocSections),
        Box::new(octal_literals::OctalLiterals),
        Box::new(case_bitor_pattern::CaseBitorPattern),
        Box::new(arg_side_effects::ArgSideEffects),
        Box::new(assert_polarity::AssertPolarity),
        Box::new(raw_user_access::RawUserAccess),
        Box::new(safe_probe_deref::SafeProbeDeref),
        Box::new(store_on_failure::StoreOnFailure),
        Box::new(dropped_null_guard::DroppedNullGuard),
        Box::new(ptr_byte_diff::PtrByteDiff),
        Box::new(constant_placement::ConstantPlacement),
        Box::new(raising_probes::RaisingProbes),
        Box::new(comment_swallowed_code::CommentSwallowedCode),
        Box::new(formatless_printf::FormatlessPrintf),
        Box::new(ptr_typedef_cast_read::PtrTypedefCastRead),
        Box::new(call_arg_swap::CallArgSwap),
        Box::new(sse_helpers::SseHelpers),
        Box::new(extern_arity::ExternArity),
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
