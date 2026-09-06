// 🛡️ 紫电 AI - 2026 SOTA Windows 内核级进程生命周期看护者 (process_guardian.rs)
// 职责边界：利用 Win32 Job Object (KILL_ON_JOB_CLOSE)，确保主程序退出时 100% 自动终结所有子进程

#[cfg(target_os = "windows")]
pub mod win32_guardian {
    use std::ffi::c_void;
    use std::sync::OnceLock;

    type HANDLE = *mut c_void;
    type BOOL = i32;
    type DWORD = u32;

    #[repr(C)]
    struct JOBOBJECT_BASIC_LIMIT_INFORMATION {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: DWORD,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: DWORD,
        affinity: usize,
        priority_class: DWORD,
        scheduling_class: DWORD,
    }

    #[repr(C)]
    struct IO_COUNTERS {
        read_operation_count: u64,
        write_operation_count: u64,
        other_operation_count: u64,
        read_transfer_count: u64,
        write_transfer_count: u64,
        other_transfer_count: u64,
    }

    #[repr(C)]
    struct JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
        basic_limit_information: JOBOBJECT_BASIC_LIMIT_INFORMATION,
        io_info: IO_COUNTERS,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_used: usize,
        peak_job_memory_used: usize,
    }

    const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: DWORD = 0x00002000;
    const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION: i32 = 9;
    const PROCESS_SET_QUOTA: DWORD = 0x0100;
    const PROCESS_TERMINATE: DWORD = 0x0001;

    extern "system" {
        fn CreateJobObjectW(lpJobAttributes: *mut c_void, lpName: *const u16) -> HANDLE;
        fn SetInformationJobObject(
            hJob: HANDLE,
            JobObjectInfoClass: i32,
            lpJobObjectInfo: *const c_void,
            cbJobObjectInfoLength: DWORD,
        ) -> BOOL;
        fn AssignProcessToJobObject(hJob: HANDLE, hProcess: HANDLE) -> BOOL;
        fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;
        fn CloseHandle(hObject: HANDLE) -> BOOL;
    }

    static GLOBAL_JOB_OBJECT: OnceLock<usize> = OnceLock::new();

    /// 根据操作系统 PID 将新启动的子进程绑定到全局作业对象
    pub fn attach_child_pid(pid: u32) {
        let job_handle_val = *GLOBAL_JOB_OBJECT.get_or_init(|| unsafe {
            let job = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
            if !job.is_null() {
                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                info.basic_limit_information.limit_flags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                let ok = SetInformationJobObject(
                    job,
                    JOB_OBJECT_EXTENDED_LIMIT_INFORMATION,
                    &info as *const _ as *const c_void,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as DWORD,
                );
                if ok != 0 {
                    tracing::info!("🛡️ [ProcessGuardian] Windows Job Object 已激活 (KILL_ON_JOB_CLOSE 就绪)");
                }
            }
            job as usize
        });

        if job_handle_val != 0 {
            unsafe {
                let h_proc = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, 0, pid);
                if !h_proc.is_null() {
                    let _ = AssignProcessToJobObject(job_handle_val as HANDLE, h_proc);
                    CloseHandle(h_proc);
                }
            }
        }
    }
}

pub fn protect_child_pid(_pid: Option<u32>) {
    #[cfg(target_os = "windows")]
    if let Some(pid) = _pid {
        win32_guardian::attach_child_pid(pid);
    }
}
