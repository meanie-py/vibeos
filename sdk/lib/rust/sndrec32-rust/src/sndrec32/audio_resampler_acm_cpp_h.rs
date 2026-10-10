use crate::prelude::*;
use core::ffi::c_void;





impl audio_resampler_acm {
    pub unsafe fn init_(&mut self) {
        let this_: *mut Self = self;
        /* Zeroing structures */
        ZeroMemory!(core::ptr::addr_of_mut!((*this_).acm_header) as *mut core::ffi::c_void, core::mem::size_of::<ACMSTREAMHEADER>() as usize);
        ZeroMemory!(core::ptr::addr_of_mut!((*this_).wformat_src) as *mut core::ffi::c_void, core::mem::size_of::<WAVEFORMATEX>() as usize);
        ZeroMemory!(core::ptr::addr_of_mut!((*this_).wformat_dst) as *mut core::ffi::c_void, core::mem::size_of::<WAVEFORMATEX>() as usize);

        /* Setting structures sizes */
        (*this_).acm_header.cbStruct = core::mem::size_of::<ACMSTREAMHEADER>() as DWORD;
        (*this_).wformat_src.cbSize = core::mem::size_of::<WAVEFORMATEX>() as WORD;
        (*this_).wformat_dst.cbSize = core::mem::size_of::<WAVEFORMATEX>() as WORD;

        /* Setting WAVEFORMATEX structure parameters
           according to `audio_format' in/out classes */

        (*this_).wformat_src.wFormatTag = WAVE_FORMAT_PCM as WORD;
        (*this_).wformat_src.nSamplesPerSec = (*this_).audfmt_in.sample_rate() as DWORD;
        (*this_).wformat_src.nChannels = (*this_).audfmt_in.channels() as WORD;
        (*this_).wformat_src.wBitsPerSample = (*this_).audfmt_in.bits() as WORD;
        (*this_).wformat_src.nAvgBytesPerSec = (*this_).audfmt_in.byte_rate() as DWORD;
        (*this_).wformat_src.nBlockAlign = (*this_).audfmt_in.block_align() as WORD;

        (*this_).wformat_dst.wFormatTag = WAVE_FORMAT_PCM as WORD;
        (*this_).wformat_dst.nSamplesPerSec = (*this_).audfmt_out.sample_rate() as DWORD;
        (*this_).wformat_dst.nChannels = (*this_).audfmt_out.channels() as WORD;
        (*this_).wformat_dst.wBitsPerSample = (*this_).audfmt_out.bits() as WORD;
        (*this_).wformat_dst.nAvgBytesPerSec = (*this_).audfmt_out.byte_rate() as DWORD;
        (*this_).wformat_dst.nBlockAlign = (*this_).audfmt_out.block_align() as WORD;

        /* Init acm structures completed successful */
    }
    pub unsafe fn open(&mut self) {
        let this_: *mut Self = self;
        let mut err: MMRESULT = 0;

        /* Opens ACM stream */
        err = acmStreamOpen(
            core::ptr::addr_of_mut!((*this_).acm_stream),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*this_).wformat_src),
            core::ptr::addr_of_mut!((*this_).wformat_dst),
            core::ptr::null_mut(), 0, 0,
            ACM_STREAMOPENF_NONREALTIME,
        );

        if err != MMSYSERR_NOERROR {
            /* TODO: throw error */
            MessageBox(core::ptr::null_mut(), (&[0x0061, 0x0063, 0x006d, 0x004f, 0x0070, 0x0065, 0x006e, 0x0020, 0x0065, 0x0072, 0x0072, 0x006f, 0x0072, 0x003a, 0x0020, 0x0025, 0x0069, 0x0000u16]).as_ptr() /* _T "acmOpen error: %i" */, (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* _T "ERROR" */, MB_ICONERROR);
        }

        /* Calcs source buffer length */
        (*this_).src_buflen = (((*this_).audfmt_in.byte_rate() as f32) * ((*this_).buf_secs as f32)) as u32;

        /* Calcs destination source buffer length with help of ACM apis */
        err = acmStreamSize(
            (*this_).acm_stream,
            (*this_).src_buflen,
            core::ptr::addr_of_mut!((*this_).dst_buflen),
            ACM_STREAMSIZEF_SOURCE,
        );

        if err != MMSYSERR_NOERROR {
            /* TODO: throw error */
            MessageBox(core::ptr::null_mut(), (&[0x0061, 0x0063, 0x006d, 0x0053, 0x0074, 0x0072, 0x0065, 0x0061, 0x006d, 0x0053, 0x0069, 0x007a, 0x0065, 0x0020, 0x0065, 0x0072, 0x0072, 0x006f, 0x0072, 0x0000u16]).as_ptr() /* _T "acmStreamSize error" */, (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* _T "ERROR" */, MB_ICONERROR);
        }

        /* Initialize ACMSTREAMHEADER structure,
           and alloc memory for source and destination buffers */

        (*this_).acm_header.fdwStatus = 0;
        (*this_).acm_header.dwUser = 0;

        (*this_).acm_header.pbSrc = cpp_new_array::<BYTE>((*this_).src_buflen) as LPBYTE;
        (*this_).acm_header.cbSrcLength = (*this_).src_buflen;
        (*this_).acm_header.cbSrcLengthUsed = 0;
        (*this_).acm_header.dwSrcUser = ((*this_).src_buflen) as _;

        (*this_).acm_header.pbDst = cpp_new_array::<BYTE>((*this_).dst_buflen) as LPBYTE;
        (*this_).acm_header.cbDstLength = (*this_).dst_buflen;
        (*this_).acm_header.cbDstLengthUsed = 0;
        (*this_).acm_header.dwDstUser = ((*this_).dst_buflen) as _;

        /* Give ACMSTREAMHEADER initialized correctly to the driver */
        err = acmStreamPrepareHeader(
            (*this_).acm_stream,
            core::ptr::addr_of_mut!((*this_).acm_header),
            0,
        );
        if err != MMSYSERR_NOERROR {
            /* TODO: throw error */
            MessageBox(core::ptr::null_mut(), (&[0x0061, 0x0063, 0x006d, 0x0053, 0x0074, 0x0072, 0x0065, 0x0061, 0x006d, 0x0050, 0x0072, 0x0065, 0x0070, 0x0061, 0x0072, 0x0065, 0x0048, 0x0065, 0x0061, 0x0064, 0x0065, 0x0072, 0x0020, 0x0065, 0x0072, 0x0072, 0x006f, 0x0072, 0x0000u16]).as_ptr() /* _T "acmStreamPrepareHeader error" */, (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* _T "ERROR" */, MB_ICONERROR);
        }

        /* ACM stream successfully opened */
        (*this_).stream_opened = true;
    }
    pub unsafe fn close(&mut self) {
        let this_: *mut Self = self;
        let mut err: u32;

        if !(*this_).acm_stream.is_null() {
            if (*this_).acm_header.fdwStatus & ACMSTREAMHEADER_STATUSF_PREPARED != 0 {
                (*this_).acm_header.cbSrcLength = (*this_).src_buflen;
                (*this_).acm_header.cbDstLength = (*this_).dst_buflen;

                err = acmStreamUnprepareHeader((*this_).acm_stream, core::ptr::addr_of_mut!((*this_).acm_header), 0);
                if err != MMSYSERR_NOERROR {
                    /* Free buffer memory */
                    if (*this_).acm_header.pbSrc != core::ptr::null_mut() {
                        cpp_delete_array((*this_).acm_header.pbSrc);
                        (*this_).acm_header.pbSrc = core::ptr::null_mut();
                    }

                    if (*this_).acm_header.pbDst != core::ptr::null_mut() {
                        cpp_delete_array((*this_).acm_header.pbDst);
                        (*this_).acm_header.pbDst = core::ptr::null_mut();
                    }

                    /* Re-init structures */
                    (*this_).init_();
                    /* Updating status */
                    (*this_).stream_opened = false;
                    /* TODO: throw error */
                    MessageBox(core::ptr::null_mut(), (&[0x0061, 0x0063, 0x006d, 0x0053, 0x0074, 0x0072, 0x0065, 0x0061, 0x006d, 0x0055, 0x006e, 0x0050, 0x0072, 0x0065, 0x0070, 0x0061, 0x0072, 0x0065, 0x0048, 0x0065, 0x0061, 0x0064, 0x0065, 0x0072, 0x0020, 0x0065, 0x0072, 0x0072, 0x006f, 0x0072, 0x0000u16]).as_ptr() /* _T "acmStreamUnPrepareHeader error" */, (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* _T "ERROR" */, MB_ICONERROR);
                }
            }

            err = acmStreamClose((*this_).acm_stream, 0);
            (*this_).acm_stream = core::ptr::null_mut();

            if err != MMSYSERR_NOERROR {
                /* Free buffer memory */
                if (*this_).acm_header.pbSrc != core::ptr::null_mut() {
                    cpp_delete_array((*this_).acm_header.pbSrc);
                    (*this_).acm_header.pbSrc = core::ptr::null_mut();
                }

                if (*this_).acm_header.pbDst != core::ptr::null_mut() {
                    cpp_delete_array((*this_).acm_header.pbDst);
                    (*this_).acm_header.pbDst = core::ptr::null_mut();
                }

                /* Re-init structures */
                (*this_).init_();
                /* Updating status */
                (*this_).stream_opened = false;
                /* TODO: throw error! */
                MessageBox(core::ptr::null_mut(), (&[0x0061, 0x0063, 0x006d, 0x0053, 0x0074, 0x0072, 0x0065, 0x0061, 0x006d, 0x0043, 0x006c, 0x006f, 0x0073, 0x0065, 0x0020, 0x0065, 0x0072, 0x0072, 0x006f, 0x0072, 0x0000u16]).as_ptr() /* _T "acmStreamClose error" */, (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* _T "ERROR" */, MB_ICONERROR);
            }
        } /* if acm_stream != 0 */

        /* Free buffer memory */
        if (*this_).acm_header.pbSrc != core::ptr::null_mut() {
            cpp_delete_array((*this_).acm_header.pbSrc);
        }

        if (*this_).acm_header.pbDst != core::ptr::null_mut() {
            cpp_delete_array((*this_).acm_header.pbDst);
        }

        /* Re-init structures */
        (*this_).init_();
        /* Updating status */
        (*this_).stream_opened = false;

        /* ACM sream successfully closed */
    }
    pub unsafe fn audio_receive__body(&mut self, data: *mut u8, size: u32) {
        let this_: *mut Self = self;
        let mut err: MMRESULT;

        /* Checking for acm stream opened */
        if (*this_).stream_opened {
            /* Copy audio data from extern to internal source buffer */
            memcpy(((*this_).acm_header.pbSrc).cast(), data as *const c_void, size as usize);

            (*this_).acm_header.cbSrcLength = size;
            (*this_).acm_header.cbDstLengthUsed = 0;

            err = acmStreamConvert(
                (*this_).acm_stream,
                core::ptr::addr_of_mut!((*this_).acm_header),
                ACM_STREAMCONVERTF_BLOCKALIGN
            );

            if err != MMSYSERR_NOERROR {
                /* TODO: throw error */
                MessageBox(
                    core::ptr::null_mut(),
                    (&[0x0061, 0x0063, 0x006d, 0x0053, 0x0074, 0x0072, 0x0065, 0x0061, 0x006d, 0x0043, 0x006f, 0x006e, 0x0076, 0x0065, 0x0072, 0x0074, 0x0020, 0x0065, 0x0072, 0x0072, 0x006f, 0x0072, 0x0000u16]).as_ptr() /* _T "acmStreamConvert error" */,
                    (&[0x0045, 0x0052, 0x0052, 0x004f, 0x0052, 0x0000u16]).as_ptr() /* _T "ERROR" */,
                    MB_ICONERROR
                );
            }

            /* Wait for sound conversion */
            while (ACMSTREAMHEADER_STATUSF_DONE & (*this_).acm_header.fdwStatus) == 0 {}

            /* Copy resampled audio, to destination buffer */
            //memcpy(pbOutputData, acm_header.pbDst, acm_header.cbDstLengthUsed);
        }
    }
}
