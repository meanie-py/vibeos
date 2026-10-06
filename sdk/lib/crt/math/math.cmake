
if (MSVC)
    # Disable warning C4146: unary minus operator applied to unsigned type, result still unsigned
    add_compile_options(/wd4146)
endif()

include_directories(libm_sse2)

list(APPEND LIBCNTPR_MATH_SOURCE
    math/_chgsignf.c
    math/_dclass.c
    math/_dsign.c
    math/_dtest.c
    math/_fdclass.c
    math/_fdsign.c
    math/_fdtest.c
    math/_finite.c
    math/_finitef.c
    math/_isnan.c
    math/_isnanf.c
    math/_invoke_matherr.c
    math/abs.c
    math/div.c
    math/exp2f.c
    math/labs.c
    math/nextafter.c
    math/nextafterf.c
    math/nexttowardf.c
    math/round.c
    math/roundf.c
    math/scalbn.c
    math/scalbnf.c
    math/sincos.c
)

if(ARCH STREQUAL "i386")
    list(APPEND LIBCNTPR_MATH_SOURCE
        math/fabs.c
        math/fabsf.c
        math/i386/ci.c
        math/i386/cicos.c
        math/i386/cilog.c
        math/i386/cipow.c
        math/i386/cisin.c
        math/i386/cisqrt.c
        math/i386/ldexp.c
        math/i386/libm_sse2.c
    )
    list(APPEND LIBCNTPR_MATH_ASM_SOURCE
        math/i386/alldiv_asm.s
        math/i386/alldvrm_asm.s
        math/i386/allmul_asm.s
        math/i386/allrem_asm.s
        math/i386/allshl_asm.s
        math/i386/allshr_asm.s
        math/i386/atan_asm.s
        math/i386/atan2_asm.s
        math/i386/aulldiv_asm.s
        math/i386/aulldvrm_asm.s
        math/i386/aullrem_asm.s
        math/i386/aullshr_asm.s
        math/i386/ceil_asm.s
        math/i386/cos_asm.s
        # math/i386/fabs_asm.s # FIXME
        math/i386/floor_asm.s
        math/i386/ftol_asm.s
        math/i386/ftol2_asm.s
        math/i386/ftoul2_legacy_asm.s
        math/i386/log_asm.s
        math/i386/log10_asm.s
        math/i386/pow_asm.s
        math/i386/sin_asm.s
        math/i386/sqrt_asm.s
        math/i386/tan_asm.s
    )
    list(APPEND CRT_MATH_ASM_SOURCE
        math/i386/ceilf.S
        math/i386/floorf.S
        math/i386/exp_asm.s
        math/i386/fmod_asm.s
        math/i386/fmodf_asm.s
    )
    list(APPEND CRT_MATH_SOURCE
        math/_hypotf.c
    )
elseif(ARCH STREQUAL "amd64")
    list(APPEND LIBCNTPR_MATH_SOURCE
        math/amd64/_set_statfp.c
        # math/libm_sse2/_chgsign.c
        # math/libm_sse2/_chgsignf.c
        # math/libm_sse2/_copysign.c
        # math/libm_sse2/_copysignf.c
        # math/libm_sse2/_finite.c
        # math/libm_sse2/_finitef.c
        math/libm_sse2/_handle_error.c
        math/libm_sse2/acos.c
        math/libm_sse2/acosf.c
        math/libm_sse2/asin.c
        math/libm_sse2/asinf.c
        math/libm_sse2/atan.c
        math/libm_sse2/atan2.c
        math/libm_sse2/atan2f.c
        math/libm_sse2/atanf.c
        # math/libm_sse2/cabs.c
        # math/libm_sse2/cabsf.c
        math/libm_sse2/ceil.c
        math/libm_sse2/ceilf.c
        math/libm_sse2/cosh.c
        math/libm_sse2/coshf.c
        math/libm_sse2/exp_special.c
        math/libm_sse2/exp2.c
        math/libm_sse2/floor.c
        math/libm_sse2/floorf.c
        math/libm_sse2/fma3_available.c
        math/libm_sse2/hypot.c
        math/libm_sse2/hypotf.c
        math/libm_sse2/L2_by_pi_bits.c
        math/libm_sse2/ldexp.c
        # math/libm_sse2/ldexpf.c
        math/libm_sse2/log_128_lead_tail_table.c
        math/libm_sse2/log_256_lead_tail_table.c
        math/libm_sse2/log_F_inv_dword_table.c
        math/libm_sse2/log_F_inv_qword_table.c
        math/libm_sse2/log_special.c
        math/libm_sse2/log10_128_lead_tail_table.c
        math/libm_sse2/log10_256_lead_tail_table.c
        math/libm_sse2/logb.c
        math/libm_sse2/logbf.c
        math/libm_sse2/Lsincos_array.c
        math/libm_sse2/Lsincosf_array.c
        math/libm_sse2/modf.c
        math/libm_sse2/modff.c
        math/libm_sse2/pow_special.c
        # math/libm_sse2/remainder.c
        math/libm_sse2/remainder_piby2.c
        # math/libm_sse2/remainder_piby2f.c
        # math/libm_sse2/remainderf.c
        math/libm_sse2/sincos_special.c
        math/libm_sse2/sinh.c
        math/libm_sse2/sinhf.c
        math/libm_sse2/sqrt.c
        math/libm_sse2/sqrtf.c
        math/libm_sse2/tan.c
        math/libm_sse2/tanf.c
        math/libm_sse2/tanh.c
        math/libm_sse2/tanhf.c
        math/libm_sse2/two_to_jby64_head_tail_table.c
        math/libm_sse2/two_to_jby64_table.c
    )
    if(USE_LLVM_MINGW)
        # llvm-mingw's Clang cannot assemble MASM-format .asm files.
        # Use generic C implementations instead of optimized SSE2 assembly.
        list(APPEND LIBCNTPR_MATH_SOURCE
            math/cos.c
            math/sin.c
            math/arm64/exp.c
            math/arm64/fmod.c
            math/arm64/log.c
            math/arm64/log10.c
            math/arm64/pow.c
        )
        list(APPEND LIBCNTPR_MATH_ASM_SOURCE
            math/amd64/fabs.S
            math/amd64/fabsf.S
        )
        list(APPEND CRT_MATH_SOURCE
            math/sinf.c
            math/cosf.c
            math/expf.c
            math/fmodf.c
        )
    else()
        list(APPEND LIBCNTPR_MATH_ASM_SOURCE
            math/libm_sse2/fm.inc
            math/libm_sse2/cos.asm
            math/libm_sse2/cosf.asm
            math/libm_sse2/exp.asm
            math/libm_sse2/expf.asm
            math/amd64/fabs.S
            math/amd64/fabsf.S
            math/libm_sse2/fmod.asm
            math/libm_sse2/fmodf.asm
            math/libm_sse2/log.asm
            math/libm_sse2/log10.asm
            math/libm_sse2/pow.asm
            math/libm_sse2/remainder_piby2_forAsm.asm
            math/libm_sse2/remainder_piby2_forFMA3.asm
            math/libm_sse2/remainder_piby2f_forAsm.asm
            math/libm_sse2/remainder_piby2f_forC.asm
            math/libm_sse2/sin.asm
            math/libm_sse2/sinf.asm
        )
    endif()
elseif(ARCH STREQUAL "arm64")
    list(APPEND LIBCNTPR_MATH_SOURCE
        math/cos.c
        math/ceilf.c
        math/fabs.c
        math/fabsf.c
        math/floorf.c
        math/sin.c
        math/sqrt.c
        math/sqrtf.c
        math/arm64/atan.c
        math/arm64/atan2.c
        math/arm64/ceil.c
        math/arm64/exp.c
        math/arm64/floor.c
        math/arm64/fmod.c
        math/arm64/ldexp.c
        math/arm64/log.c
        math/arm64/log10.c
        math/arm64/logb.c
        math/arm64/pow.c
        math/arm64/tan.c
    )
    list(APPEND CRT_MATH_SOURCE
        math/_hypotf.c
        math/arm64/fma3_available.c
        math/acosf.c
        math/asinf.c
        math/atan2f.c
        math/atanf.c
        math/coshf.c
        math/expf.c
        math/fmodf.c
        math/modff.c
        math/sinf.c
        math/sinhf.c
        math/tanf.c
        math/tanhf.c
    )
endif()

if(NOT ARCH STREQUAL "i386")
    list(APPEND CRT_MATH_SOURCE
        math/_copysignf.c
        math/log10f.c
        math/stubs.c
    )
endif()

if(NOT ARCH STREQUAL "amd64")
    list(APPEND CRT_MATH_SOURCE
        math/acos.c
        math/asin.c
        math/cosh.c
        math/cosf.c
        math/exp2.c
        math/hypot.c
        math/modf.c
        math/s_modf.c
        math/sinh.c
        math/tanh.c
    )
endif()

list(APPEND CRT_MATH_SOURCE
    ${LIBCNTPR_MATH_SOURCE}
    math/adjust.c
    math/cabs.c
    math/fdivbug.c
    math/frexp.c
    math/huge_val.c
    # The Bessel functions: cnt-rust's _j0.._yn over the `libm` crate. The vendored
    # FDLIBM bodies (math/ieee754) are gone; these wrappers stay on disk as the
    # cnt-difftest oracle for the errno shell.
    # math/j0_y0.c
    # math/j1_y1.c
    # math/jn_yn.c
    math/ldiv.c
    math/logf.c
    math/powf.c
    math/trunc.c
    math/truncf.c
)

list(APPEND CRT_MATH_ASM_SOURCE
    ${LIBCNTPR_MATH_ASM_SOURCE}
)

# `atan2`, the one libm function gdi32 links (its Wine EMF driver's arc angles): Rust,
# sdk/lib/rust/atan2-rust (libm::atan2). It was a different C/asm per arch -- i386
# math/i386/atan2_asm.s, amd64 AMD libm_sse2 atan2.c with _handle_error/_invoke_matherr/
# _set_statfp, arm64 math/arm64/{atan2,atan}.c -- and the amd64 one was the last shipped use
# of the vendored libm_sse2 tree. SSE2 on i386, as msvcrt-crt-rust: libm's f64 code needs it.
set(_ATAN2_ENV)
if(ARCH STREQUAL "i386")
    set(_ATAN2_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# gdi32 names the archive BY PATH, ${ATAN2_LIB} (gcc.cmake): through an INTERFACE target it
# landed behind ntdll's import library, which exports atan2 on arm64.
rust_staticlib_variant(atan2_rust atan2-rust "" TRUE ${_ATAN2_ENV})
if(NOT atan2_rust_LIB STREQUAL ATAN2_LIB)
    message(FATAL_ERROR "ATAN2_LIB (${ATAN2_LIB}) is not atan2-rust's archive (${atan2_rust_LIB})")
endif()
