//! Process management syscalls
use crate::{mm::{PTEFlags, translated_byte_buffer,translated_byte_buffer_mut,PageTable,VirtAddr}, task::{change_program_brk, exit_current_and_run_next, suspend_current_and_run_next}, timer::get_time_us};
use crate::task::current_user_token;
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
///
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let time_val = TimeVal{
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let size = core::mem::size_of::<TimeVal>();
    let ptr1 = &time_val as *const TimeVal as *const u8;
    let src = unsafe {
        core::slice::from_raw_parts(ptr1, size)
    };
    let buffers = translated_byte_buffer_mut(current_user_token(),_ts as *mut u8,size);
    let mut offset = 0;
    for buffer in buffers{
        let len  = buffer.len();
        buffer.copy_from_slice(&src[offset..offset + len]);
        offset += len;
    }
    0

}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    let page_table = PageTable::from_token(current_user_token());
            let va = VirtAddr::from(_id);
            let vpn = va.floor();
    match _trace_request{
        0 =>{
            let pte = match page_table.translate(vpn){
                Some(pte) => pte,
                None => return -1,
            };
            if !(pte.is_valid()&&pte.readable()&&pte.flags().contains(PTEFlags::U)){
                return -1;
            }

            let ptr = _id as *const u8;
            let mut buffers = translated_byte_buffer(current_user_token(), ptr, 1);
            return buffers[0][0] as isize;
        },
        1 =>{
            let pte = match page_table.translate(vpn){
                Some(pte) => pte,
                None => return -1,
            };
            if !(pte.is_valid()&&pte.writable()&&pte.flags().contains(PTEFlags::U)){
                return -1;
            }

            let  ptr = _id as *mut u8;
            let buffers = translated_byte_buffer_mut(current_user_token(),ptr,1);
            buffers[0][0] = _data as u8;
            return 0;

        },
        2 =>{

        },
        _ =>{
            return -1;
        },
    }


    }


// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}}
