use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut WkstaInfo502: WKSTA_INFO_502 = unsafe { core::mem::zeroed() };
#[no_mangle]
pub unsafe extern "C" fn InitWorkstationInfo() {
    let mut h_info_key: HKEY = core::ptr::null_mut();
    let mut dw_type: u32 = 0;
    let mut dw_size: u32 = 0;
    let mut dw_value: u32 = 0;
    let mut dw_error: u32;

    dw_error = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("System\\CurrentControlSet\\Services\\LanmanWorkstation\\Parameters"),
        0,
        KEY_READ,
        &mut h_info_key,
    ) as DWORD;
    if dw_error != ERROR_SUCCESS {
        return;
    }

    // CharWait
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("CharWait"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS && dw_value <= 65535 {
        WkstaInfo502.wki502_char_wait = dw_value;
    } else {
        WkstaInfo502.wki502_char_wait = 0;
    }

    // CollectionTime
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("CollectionTime"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS && dw_value <= 65535000 {
        WkstaInfo502.wki502_collection_time = dw_value;
    } else {
        WkstaInfo502.wki502_collection_time = 250;
    }

    // MaximumCollectionCount
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("MaximumCollectionCount"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS && dw_value <= 65535 {
        WkstaInfo502.wki502_maximum_collection_count = dw_value;
    } else {
        WkstaInfo502.wki502_maximum_collection_count = 16;
    }

    // KeepConn
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("KeepConn"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS && dw_value >= 1 && dw_value <= 65535 {
        WkstaInfo502.wki502_keep_conn = dw_value;
    } else {
        WkstaInfo502.wki502_keep_conn = 600;
    }

    // MaxCmds
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("MaxCmds"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS && dw_value >= 50 && dw_value <= 65535 {
        WkstaInfo502.wki502_max_cmds = dw_value;
    } else {
        WkstaInfo502.wki502_max_cmds = 50;
    }

    // SessTimeout
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("SessTimeout"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS && dw_value >= 60 && dw_value <= 65535 {
        WkstaInfo502.wki502_sess_timeout = dw_value;
    } else {
        WkstaInfo502.wki502_sess_timeout = 60;
    }

    // SizCharBuf
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("SizCharBuf"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS && dw_value >= 64 && dw_value <= 4096 {
        WkstaInfo502.wki502_siz_char_buf = dw_value;
    } else {
        WkstaInfo502.wki502_siz_char_buf = 512;
    }

    // MaxThreads
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("MaxThreads"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS && dw_value >= 1 && dw_value <= 256 {
        WkstaInfo502.wki502_max_threads = dw_value;
    } else {
        WkstaInfo502.wki502_max_threads = 17;
    }

    // LockQuota (first)
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("LockQuota"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_lock_quota = dw_value;
    } else {
        WkstaInfo502.wki502_lock_quota = 6144;
    }

    // LockQuota (duplicate)
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("LockQuota"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_lock_quota = dw_value;
    } else {
        WkstaInfo502.wki502_lock_quota = 6144;
    }

    // LockIncrement
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("LockIncrement"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_lock_increment = dw_value;
    } else {
        WkstaInfo502.wki502_lock_increment = 10;
    }

    // LockMaximum
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("LockMaximum"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_lock_maximum = dw_value;
    } else {
        WkstaInfo502.wki502_lock_maximum = 500;
    }

    // PipeIncrement
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("PipeIncrement"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_pipe_increment = dw_value;
    } else {
        WkstaInfo502.wki502_pipe_increment = 10;
    }

    // PipeMaximum
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("PipeMaximum"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_pipe_maximum = dw_value;
    } else {
        WkstaInfo502.wki502_pipe_maximum = 500;
    }

    // CacheFileTimeout
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("CacheFileTimeout"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_cache_file_timeout = dw_value;
    } else {
        WkstaInfo502.wki502_cache_file_timeout = 40;
    }

    // DormantFileLimit
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("DormantFileLimit"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS && dw_value >= 1 {
        WkstaInfo502.wki502_dormant_file_limit = dw_value;
    } else {
        WkstaInfo502.wki502_dormant_file_limit = 1;
    }

    // ReadAheadThroughput
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("ReadAheadThroughput"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_read_ahead_throughput = dw_value;
    } else {
        WkstaInfo502.wki502_read_ahead_throughput = 0;
    }

    // MailslotBuffers
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("MailslotBuffers"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_num_mailslot_buffers = dw_value;
    } else {
        WkstaInfo502.wki502_num_mailslot_buffers = 3;
    }

    // ServerAnnounceBuffers
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("ServerAnnounceBuffers"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_num_srv_announce_buffers = dw_value;
    } else {
        WkstaInfo502.wki502_num_srv_announce_buffers = 20;
    }

    // NumIllegalDatagramEvents
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("NumIllegalDatagramEvents"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_max_illegal_datagram_events = dw_value;
    } else {
        WkstaInfo502.wki502_max_illegal_datagram_events = 5;
    }

    // IllegalDatagramResetTime
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("IllegalDatagramResetTime"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_illegal_datagram_event_reset_frequency = dw_value;
    } else {
        WkstaInfo502.wki502_illegal_datagram_event_reset_frequency = 3600;
    }

    // LogElectionPackets
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("LogElectionPackets"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_log_election_packets = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_log_election_packets = 0;
    }

    // UseOpportunisticLocking
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("UseOpportunisticLocking"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_use_opportunistic_locking = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_use_opportunistic_locking = 1;
    }

    // UseUnlockBehind
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("UseUnlockBehind"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_use_unlock_behind = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_use_unlock_behind = 1;
    }

    // UseCloseBehind
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("UseCloseBehind"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_use_close_behind = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_use_close_behind = 1;
    }

    // BufNamedPipes
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("BufNamedPipes"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_buf_named_pipes = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_buf_named_pipes = 1;
    }

    // UseLockReadUnlock
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("UseLockReadUnlock"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_use_lock_read_unlock = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_use_lock_read_unlock = 1;
    }

    // UtilizeNtCaching
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("UtilizeNtCaching"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_utilize_nt_caching = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_utilize_nt_caching = 1;
    }

    // UseRawRead
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("UseRawRead"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_use_raw_read = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_use_raw_read = 1;
    }

    // UseRawWrite
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("UseRawWrite"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_use_raw_write = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_use_raw_write = 1;
    }

    // UseWriteRawData
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("UseWriteRawData"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_use_write_raw_data = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_use_write_raw_data = 0;
    }

    // UseEncryption
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("UseEncryption"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_use_encryption = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_use_encryption = 1;
    }

    // BufFilesDenyWrite
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("BufFilesDenyWrite"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_buf_files_deny_write = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_buf_files_deny_write = 0;
    }

    // BufReadOnlyFiles
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("BufReadOnlyFiles"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_buf_read_only_files = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_buf_read_only_files = 0;
    }

    // ForceCoreCreateMode
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("ForceCoreCreateMode"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_force_core_create_mode = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_force_core_create_mode = 0;
    }

    // Use512ByteMaxTransfer
    dw_size = (core::mem::size_of::<u32>()) as u32;
    dw_error = RegQueryValueExW(
        h_info_key,
        w!("Use512ByteMaxTransfer"),
        core::ptr::null_mut(),
        &mut dw_type,
        core::ptr::addr_of_mut!(dw_value) as *mut u8,
        &mut dw_size) as DWORD;
    if dw_error == ERROR_SUCCESS {
        WkstaInfo502.wki502_use_512_byte_max_transfer = (dw_value) as i32;
    } else {
        WkstaInfo502.wki502_use_512_byte_max_transfer = 0;
    }

    RegCloseKey(h_info_key);
}
#[no_mangle]
pub unsafe extern "C" fn SaveWorkstationInfo(Level: u32) {
    let mut hInfoKey: HKEY = core::ptr::null_mut();
    let dwError: u32;


    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("System\\CurrentControlSet\\Services\\LanmanWorkstation\\Parameters"),
        0,
        KEY_WRITE,
        &mut hInfoKey,
    ) as DWORD;

    if dwError != ERROR_SUCCESS {
        return;
    }

    match Level {
        502 => {
            RegSetValueExW(hInfoKey, w!("CharWait"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_char_wait) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("CollectionTime"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_collection_time) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("MaximumCollectionCount"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_maximum_collection_count) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("KeepConn"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_keep_conn) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("MaxCmds"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_max_cmds) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("SessTimeout"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_sess_timeout) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("SizCharBuf"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_siz_char_buf) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("MaxThreads"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_max_threads) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("LockQuota"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_lock_quota) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("LockIncrement"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_lock_increment) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("LockMaximum"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_lock_maximum) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("PipeIncrement"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_pipe_increment) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("PipeMaximum"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_pipe_maximum) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("CacheFileTimeout"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_cache_file_timeout) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("DormantFileLimit"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_dormant_file_limit) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("ReadAheadThroughput"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_read_ahead_throughput) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("MailslotBuffers"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_num_mailslot_buffers) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("ServerAnnounceBuffers"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_num_srv_announce_buffers) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("NumIllegalDatagramEvents"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_max_illegal_datagram_events) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("IllegalDatagramResetTime"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_illegal_datagram_event_reset_frequency) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("LogElectionPackets"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_log_election_packets) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("UseOpportunisticLocking"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_use_opportunistic_locking) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("UseUnlockBehind"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_use_unlock_behind) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("UseCloseBehind"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_use_close_behind) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("BufNamedPipes"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_buf_named_pipes) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("UseLockReadUnlock"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_use_lock_read_unlock) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("UtilizeNtCaching"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_utilize_nt_caching) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("UseRawRead"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_use_raw_read) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("UseRawWrite"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_use_raw_write) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("UseWriteRawData"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_use_write_raw_data) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("UseEncryption"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_use_encryption) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("BufFilesDenyWrite"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_buf_files_deny_write) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("BufReadOnlyFiles"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_buf_read_only_files) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("ForceCoreCreateMode"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_force_core_create_mode) as *const u8, (core::mem::size_of::<i32>()) as u32);

            RegSetValueExW(hInfoKey, w!("Use512ByteMaxTransfer"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_use_512_byte_max_transfer) as *const u8, (core::mem::size_of::<i32>()) as u32);
        }
        1013 => {
            RegSetValueExW(hInfoKey, w!("KeepConn"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_keep_conn) as *const u8, (core::mem::size_of::<i32>()) as u32);
        }
        1018 => {
            RegSetValueExW(hInfoKey, w!("SessTimeout"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_sess_timeout) as *const u8, (core::mem::size_of::<i32>()) as u32);
        }
        1046 => {
            RegSetValueExW(hInfoKey, w!("DormantFileLimit"), 0, REG_DWORD, core::ptr::addr_of!(WkstaInfo502.wki502_dormant_file_limit) as *const u8, (core::mem::size_of::<i32>()) as u32);
        }
        _ => {}
    }

    RegCloseKey(hInfoKey);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Services
//  * FILE:             base/services/wkssvc/info.c
//  * PURPOSE:          Workstation service
//  * PROGRAMMER:       Eric Kohl
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(wkssvc);
//
// /* GLOBALS *******************************************************************/
//
// WKSTA_INFO_502 WkstaInfo502;
//
//
// /* FUNCTIONS *****************************************************************/
//
// VOID
// InitWorkstationInfo(VOID)
// {
//     HKEY hInfoKey = NULL;
//     DWORD dwType, dwSize, dwValue;
//     DWORD dwError;
//
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"System\\CurrentControlSet\\Services\\LanmanWorkstation\\Parameters",
//                             0,
//                             KEY_READ,
//                             &hInfoKey) as DWORD;
//     if (dwError != ERROR_SUCCESS)
//     {
//         ERR("RegOpenKeyExW() failed!\n");
//         return;
//     }
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"CharWait",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if ((dwError == ERROR_SUCCESS) && (dwValue <= 65535))
//         WkstaInfo502.wki502_char_wait = dwValue;
//     else
//         WkstaInfo502.wki502_char_wait = 0;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"CollectionTime",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if ((dwError == ERROR_SUCCESS) && (dwValue <= 65535000))
//         WkstaInfo502.wki502_collection_time = dwValue;
//     else
//         WkstaInfo502.wki502_collection_time = 250;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"MaximumCollectionCount",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if ((dwError == ERROR_SUCCESS) && (dwValue <= 65535))
//         WkstaInfo502.wki502_maximum_collection_count = dwValue;
//     else
//         WkstaInfo502.wki502_maximum_collection_count = 16;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"KeepConn",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if ((dwError == ERROR_SUCCESS) && (dwValue >= 1) && (dwValue <= 65535))
//         WkstaInfo502.wki502_keep_conn = dwValue;
//     else
//         WkstaInfo502.wki502_keep_conn = 600;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"MaxCmds",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if ((dwError == ERROR_SUCCESS) && (dwValue >= 50) && (dwValue <= 65535))
//         WkstaInfo502.wki502_max_cmds = dwValue;
//     else
//         WkstaInfo502.wki502_max_cmds = 50;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"SessTimeout",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if ((dwError == ERROR_SUCCESS) && (dwValue >= 60) && (dwValue <= 65535))
//         WkstaInfo502.wki502_sess_timeout = dwValue;
//     else
//         WkstaInfo502.wki502_sess_timeout = 60;
//
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"SizCharBuf",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if ((dwError == ERROR_SUCCESS) && (dwValue >= 64) && (dwValue <= 4096))
//         WkstaInfo502.wki502_siz_char_buf = dwValue;
//     else
//         WkstaInfo502.wki502_siz_char_buf = 512;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"MaxThreads",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if ((dwError == ERROR_SUCCESS) && (dwValue >= 1) && (dwValue <= 256))
//         WkstaInfo502.wki502_max_threads = dwValue;
//     else
//         WkstaInfo502.wki502_max_threads = 17;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"LockQuota",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_lock_quota = dwValue;
//     else
//         WkstaInfo502.wki502_lock_quota = 6144;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"LockQuota",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_lock_quota = dwValue;
//     else
//         WkstaInfo502.wki502_lock_quota = 6144;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"LockIncrement",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_lock_increment = dwValue;
//     else
//         WkstaInfo502.wki502_lock_increment = 10;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"LockMaximum",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_lock_maximum = dwValue;
//     else
//         WkstaInfo502.wki502_lock_maximum = 500;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"PipeIncrement",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_pipe_increment = dwValue;
//     else
//         WkstaInfo502.wki502_pipe_increment = 10;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"PipeMaximum",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_pipe_maximum = dwValue;
//     else
//         WkstaInfo502.wki502_pipe_maximum = 500;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"CacheFileTimeout",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_cache_file_timeout = dwValue;
//     else
//         WkstaInfo502.wki502_cache_file_timeout = 40;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"DormantFileLimit",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS && dwValue >= 1)
//         WkstaInfo502.wki502_dormant_file_limit = dwValue;
//     else
//         WkstaInfo502.wki502_dormant_file_limit = 1;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"ReadAheadThroughput",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_read_ahead_throughput = dwValue;
//     else
//         WkstaInfo502.wki502_read_ahead_throughput = 0;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"MailslotBuffers",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_num_mailslot_buffers = dwValue;
//     else
//         WkstaInfo502.wki502_num_mailslot_buffers = 3;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"ServerAnnounceBuffers",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_num_srv_announce_buffers = dwValue;
//     else
//         WkstaInfo502.wki502_num_srv_announce_buffers = 20;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"NumIllegalDatagramEvents",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_max_illegal_datagram_events = dwValue;
//     else
//         WkstaInfo502.wki502_max_illegal_datagram_events = 5;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"IllegalDatagramResetTime",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_illegal_datagram_event_reset_frequency = dwValue;
//     else
//         WkstaInfo502.wki502_illegal_datagram_event_reset_frequency = 3600;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"LogElectionPackets",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_log_election_packets = dwValue;
//     else
//         WkstaInfo502.wki502_log_election_packets = 0;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"UseOpportunisticLocking",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_use_opportunistic_locking = dwValue;
//     else
//         WkstaInfo502.wki502_use_opportunistic_locking = 1;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"UseUnlockBehind",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_use_unlock_behind = dwValue;
//     else
//         WkstaInfo502.wki502_use_unlock_behind = 1;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"UseCloseBehind",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_use_close_behind = dwValue;
//     else
//         WkstaInfo502.wki502_use_close_behind = 1;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"BufNamedPipes",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_buf_named_pipes = dwValue;
//     else
//         WkstaInfo502.wki502_buf_named_pipes = 1;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"UseLockReadUnlock",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_use_lock_read_unlock = dwValue;
//     else
//         WkstaInfo502.wki502_use_lock_read_unlock = 1;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"UtilizeNtCaching",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_utilize_nt_caching = dwValue;
//     else
//         WkstaInfo502.wki502_utilize_nt_caching = 1;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"UseRawRead",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_use_raw_read = dwValue;
//     else
//         WkstaInfo502.wki502_use_raw_read = 1;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"UseRawWrite",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_use_raw_write = dwValue;
//     else
//         WkstaInfo502.wki502_use_raw_write = 1;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"UseWriteRawData",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_use_write_raw_data = dwValue;
//     else
//         WkstaInfo502.wki502_use_write_raw_data = 0;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"UseEncryption",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_use_encryption = dwValue;
//     else
//         WkstaInfo502.wki502_use_encryption = 1;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"BufFilesDenyWrite",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_buf_files_deny_write = dwValue;
//     else
//         WkstaInfo502.wki502_buf_files_deny_write = 0;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"BufReadOnlyFiles",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_buf_read_only_files = dwValue;
//     else
//         WkstaInfo502.wki502_buf_read_only_files = 0;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"ForceCoreCreateMode",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_force_core_create_mode = dwValue;
//     else
//         WkstaInfo502.wki502_force_core_create_mode = 0;
//
//     dwSize = sizeof(dwValue);
//     dwError = RegQueryValueExW(hInfoKey,
//                                L"Use512ByteMaxTransfer",
//                                0,
//                                &dwType,
//                                (PBYTE)&dwValue,
//                                &dwSize) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//         WkstaInfo502.wki502_use_512_byte_max_transfer = dwValue;
//     else
//         WkstaInfo502.wki502_use_512_byte_max_transfer = 0;
//
//     RegCloseKey(hInfoKey);
// }
//
//
// VOID
// SaveWorkstationInfo(
//      DWORD Level)
// {
//     HKEY hInfoKey = NULL;
//     DWORD dwError;
//
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"System\\CurrentControlSet\\Services\\LanmanWorkstation\\Parameters",
//                             0,
//                             KEY_WRITE,
//                             &hInfoKey) as DWORD;
//     if (dwError != ERROR_SUCCESS)
//     {
//         ERR("RegOpenKeyExW() failed!\n");
//         return;
//     }
//
//     switch (Level)
//     {
//         case 502:
//             RegSetValueExW(hInfoKey,
//                            L"CharWait",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_char_wait,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"CollectionTime",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_collection_time,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"MaximumCollectionCount",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_maximum_collection_count,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"KeepConn",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_keep_conn,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"MaxCmds",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_max_cmds,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"SessTimeout",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_sess_timeout,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"SizCharBuf",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_siz_char_buf,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"MaxThreads",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_max_threads,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"LockQuota",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_lock_quota,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"LockIncrement",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_lock_increment,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"LockMaximum",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_lock_maximum,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"PipeIncrement",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_pipe_increment,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"PipeMaximum",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_pipe_maximum,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"CacheFileTimeout",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_cache_file_timeout,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"DormantFileLimit",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_dormant_file_limit,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"ReadAheadThroughput",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_read_ahead_throughput,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"MailslotBuffers",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_num_mailslot_buffers,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"ServerAnnounceBuffers",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_num_srv_announce_buffers,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"NumIllegalDatagramEvents",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_max_illegal_datagram_events,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"IllegalDatagramResetTime",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_illegal_datagram_event_reset_frequency,
//                            sizeof(DWORD));
//
//             RegSetValueExW(hInfoKey,
//                            L"LogElectionPackets",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_log_election_packets,
//                            sizeof(BOOL));
//
//             RegSetValueExW(hInfoKey,
//                            L"UseOpportunisticLocking",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_use_opportunistic_locking,
//                            sizeof(BOOL));
//
//             RegSetValueExW(hInfoKey,
//                            L"UseUnlockBehind",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_use_unlock_behind,
//                            sizeof(BOOL));
//
//             RegSetValueExW(hInfoKey,
//                            L"UseCloseBehind",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_use_close_behind,
//                            sizeof(BOOL));
//
//             RegSetValueExW(hInfoKey,
//                            L"BufNamedPipes",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_buf_named_pipes,
//                            sizeof(BOOL));
//
//             RegSetValueExW(hInfoKey,
//                            L"UseLockReadUnlock",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_use_lock_read_unlock,
//                            sizeof(BOOL));
//
//             RegSetValueExW(hInfoKey,
//                            L"UtilizeNtCaching",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_utilize_nt_caching,
//                            sizeof(BOOL));
//
//             RegSetValueExW(hInfoKey,
//                            L"UseRawRead",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_use_raw_read,
//                            sizeof(BOOL));
//
//             RegSetValueExW(hInfoKey,
//                            L"UseRawWrite",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_use_raw_write,
//                            sizeof(BOOL));
//
//             RegSetValueExW(hInfoKey,
//                            L"UseWriteRawData",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_use_write_raw_data,
//                            sizeof(BOOL));
//
//             RegSetValueExW(hInfoKey,
//                            L"UseEncryption",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_use_encryption,
//                            sizeof(BOOL));
//
//             RegSetValueExW(hInfoKey,
//                            L"BufFilesDenyWrite",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_buf_files_deny_write,
//                            sizeof(BOOL));
//
//             RegSetValueExW(hInfoKey,
//                            L"BufReadOnlyFiles",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_buf_read_only_files,
//                            sizeof(BOOL));
//
//             RegSetValueExW(hInfoKey,
//                            L"ForceCoreCreateMode",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_force_core_create_mode,
//                            sizeof(BOOL));
//
//             RegSetValueExW(hInfoKey,
//                            L"Use512ByteMaxTransfer",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_use_512_byte_max_transfer,
//                            sizeof(BOOL));
//             break;
//
//         case 1013:
//             RegSetValueExW(hInfoKey,
//                            L"KeepConn",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_keep_conn,
//                            sizeof(DWORD));
//             break;
//
//         case 1018:
//             RegSetValueExW(hInfoKey,
//                            L"SessTimeout",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_sess_timeout,
//                            sizeof(DWORD));
//             break;
//
//         case 1046:
//             RegSetValueExW(hInfoKey,
//                            L"DormantFileLimit",
//                            0,
//                            REG_DWORD,
//                            (PBYTE)&WkstaInfo502.wki502_dormant_file_limit,
//                            sizeof(DWORD));
//             break;
//     }
//
//     RegCloseKey(hInfoKey);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
