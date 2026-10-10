use crate::prelude::*;
use core::ffi::c_void;
















impl audio_membuffer {
    pub unsafe fn alloc_mem_(&mut self, bytes: u32) {
        let this_: *mut Self = self;
        /* Some checking */
        if bytes == 0 {
            return;
        }

        /* Checks previously alloc'd memory and frees it */
        if !(*this_).audio_data.is_null() {
            cpp_delete_array((*this_).audio_data);
        }

        /* Allocs new memory and zeros it */
        (*this_).audio_data = cpp_new_array::<BYTE>(bytes);
        memset(((*this_).audio_data).cast(), 0, (bytes * core::mem::size_of::<BYTE>() as u32) as _);

        /* Sets the correct buffer size */
        (*this_).buf_size = bytes;
        (*this_).init_size = bytes;
    }
    pub unsafe fn free_mem_(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).audio_data.is_null() {
            cpp_delete_array((*this_).audio_data);
        }

        (*this_).buf_size = 0;
        (*this_).audio_data = core::ptr::null_mut();
    }
    pub unsafe fn resize_mem_(&mut self, new_size: u32) {
        let this_: *mut Self = self;
        if new_size == 0 {
            return;
        }

        /* The new_size, cannot be <= of the `bytes_received' member value of the
           parent class `audio_receiver'. We cannot touch received audio data,
           so we have to alloc at least bytes_received+1 bytes. But we can truncate
           unused memory, so `new_size' can be < of `buf_size' */
        if new_size <= (*this_).__base0.bytes_received {
            return;
        }

        let mut new_mem: *mut BYTE;

        /* Allocs new memory and zeros it */
        new_mem = cpp_new_array::<BYTE>(new_size);
        memset((new_mem).cast(), 0, new_size as usize * core::mem::size_of::<BYTE>());

        if !(*this_).audio_data.is_null() {
            /* Copies received audio data, and discard unused memory */
            memcpy((new_mem).cast(), (*this_).audio_data as *const c_void, (*this_).__base0.bytes_received as usize);
            /* Frees old memory */
            cpp_delete_array((*this_).audio_data);
            /* Commit new memory */
            (*this_).audio_data = new_mem;
            (*this_).buf_size = new_size;
        } else {
            (*this_).audio_data = new_mem;
            (*this_).buf_size = new_size;
        }

        if let Some(f) = (*this_).buffer_resized {
            f(new_size);
        }
    }
    pub unsafe fn truncate_(&mut self) {
        let this_: *mut Self = self;
        /* If `buf_size' is already = to the `bytes_received' of audio data,
           then this operation is useless; simply return */
        if (*this_).__base0.bytes_received == (*this_).buf_size {
            return;
        }

        if !(*this_).audio_data.is_null() {
            /* Allocs a new buffer */
            let newbuf: *mut BYTE = cpp_new_array::<BYTE>((*this_).__base0.bytes_received);
            /* Copies audio data */
            memcpy((newbuf).cast(), (*this_).audio_data as *const c_void, (*this_).__base0.bytes_received as usize);
            /* Frees old memory */
            cpp_delete_array((*this_).audio_data);
            /* Commit the new buffer */
            (*this_).audio_data = newbuf;
            (*this_).buf_size = (*this_).__base0.bytes_received;

            /* Buffer truncation successful. Now the buffer size is exactly big
               as much audio data was received */
        }
    }
    pub unsafe fn clear(&mut self) {
        let this_: *mut Self = self;
        (*this_).free_mem_();
        (*this_).__base0.bytes_received = 0;
    }
    pub unsafe fn reset(&mut self) {
        let this_: *mut Self = self;
        /* Frees memory and reset to initial state */
        (*this_).clear();
        /* Alloc memory of size specified at the constructor */
        (*this_).alloc_mem_((*this_).init_size);
    }
    pub unsafe fn alloc_bytes(&mut self, bytes: u32) {
        let this_: *mut Self = self;
        (*this_).alloc_mem_(bytes);
    }
    pub unsafe fn alloc_seconds__0(&mut self, secs: u32) {
        let this_: *mut Self = self;
        (*this_).alloc_mem_((*this_).aud_info.byte_rate() * secs);
    }
    pub unsafe fn alloc_seconds__1(&mut self, secs: f32) {
        let this_: *mut Self = self;
        (*this_).alloc_mem_(((*this_).aud_info.byte_rate() as f32 * secs) as u32);
    }
    pub unsafe fn resize_bytes(&mut self, bytes: u32) {
        let this_: *mut Self = self;
        (*this_).resize_mem_(bytes);
    }
    pub unsafe fn resize_seconds__0(&mut self, secs: u32) {
        let this_: *mut Self = self;
        (*this_).resize_mem_((*this_).aud_info.byte_rate().wrapping_mul(secs));
    }
    pub unsafe fn resize_seconds__1(&mut self, secs: f32) {
        let this_: *mut Self = self;
        (*this_).resize_mem_((((*this_).aud_info.byte_rate() as f32) * secs) as u32);
    }
    pub unsafe fn audio_receive__body(&mut self, data: *mut u8, size: u32) {
        let this_: *mut Self = self;
        /* If there isn't a buffer, allocs memory for it of size*2, and copies audio data arrival */
        if ((*this_).audio_data == core::ptr::null_mut()) || ((*this_).buf_size == 0)
        {
            (*this_).alloc_mem_(size.wrapping_mul(2));
            memcpy(((*this_).audio_data).cast(), data as *const c_void, size as usize);
            return;
        }

        /* If buffer's free memory is < of `size', we have to realloc buffer memory
           of buf_size*2, while free memory is enough to contain `size' bytes.
           In this case free memory is represented by `buf_size - bytes_recorded' */
        let mut tot_mem: u32 = (*this_).buf_size;
        let mut free_mem: u32 = (*this_).buf_size.wrapping_sub((*this_).__base0.bytes_received);
        if free_mem < size
        {
            /* Calcs new buffer size */
            /* TODO: flags for other behaviour? */
            while free_mem < size
            {
                tot_mem = tot_mem.wrapping_mul(2);
                free_mem = tot_mem.wrapping_sub((*this_).__base0.bytes_received);
            }

            /* Resize buffer memory */
            (*this_).resize_mem_(tot_mem);
        }

        /* Now we have enough free space in the buffer, so let's copy audio data arrivals */
        memcpy(
            ((*this_).audio_data.add((*this_).__base0.bytes_received as usize)).cast(),
            data as *const c_void,
            size as usize,
        );

        if let Some(f) = (*this_).audio_arrival {
            f((*this_).aud_info.samples_in_bytes(size));
        }
    }
    pub unsafe fn read__body(&mut self, out_buf: *mut BYTE, bytes: u32) -> u32 {
        let this_: *mut Self = self;
        /* Some checking */
        if (*this_).audio_data.is_null() {
            return 0;
        }

        if (*this_).__base1.bytes_played_ >= (*this_).__base0.bytes_received {
            return 0;
        }

        let mut to_play: u32 = (*this_).__base0.bytes_received - (*this_).__base1.bytes_played_;
        let mut to_copy: u32 = if bytes > to_play { to_play } else { bytes };

        /* Copies the audio data out */
        if (!out_buf.is_null()) && (to_copy != 0) && (!(*this_).audio_data.is_null()) {
            memcpy(out_buf as *mut core::ffi::c_void, (*this_).audio_data.add((*this_).__base1.bytes_played_ as usize) as *const core::ffi::c_void, to_copy as usize);
        }

        /* Increments the number of total bytes played (audio data gone out from
           the `audio_producer' object) */
        (*this_).__base1.bytes_played_ += to_copy;

        if let Some(f) = (*this_).audio_arrival {
            f((*this_).aud_info.samples_in_bytes(to_copy));
        }

        /* Returns the exact size of audio data produced */
        return to_copy;
    }
    pub unsafe fn finished__body(&mut self) -> bool {
        let this_: *mut Self = self;
        if (*this_).__base1.bytes_played_ < (*this_).__base0.bytes_received {
            return false;
        } else {
            return true;
        }
    }
}
