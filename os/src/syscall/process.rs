//! Process management syscalls
use crate::{config::PAGE_SIZE, mm::{MapPermission, PTEFlags, PageTable, VirtAddr, VirtPageNum, translated_byte_buffer, translated_byte_buffer_mut}, task::{TASK_MANAGER, change_program_brk, exit_current_and_run_next, suspend_current_and_run_next}, timer::get_time_us};
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
            let buffers = translated_byte_buffer(current_user_token(), ptr, 1);
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
            let mut buffers = translated_byte_buffer_mut(current_user_token(),ptr,1);
            buffers[0][0] = _data as u8;
            return 0;

        },
        2 =>{
            return TASK_MANAGER.get_syscall_num(_id);
        },
        _ =>{
            return -1;
        },
    }


    }


// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    //检查一下port
    if(_port & !0x7) != 0x0{
        return -1;
    }
    if (_port & 0x7) == 0 {
        return -1;
    }

    let mut permission = MapPermission::empty();
    //然后转换为mappermission
    if(_port & 0x1) != 0{
        permission |= MapPermission::R;
    }
    if(_port & 0x2) != 0{
        permission |= MapPermission::W;
    }
    if(_port & 0x4) != 0{
        permission |= MapPermission::X;
    }
   permission |= MapPermission::U;

   //检查一下start有没有按页对齐
    if _start % PAGE_SIZE != 0{
        return -1;
    }
    if _len == 0{
         return -1;  
    }
    //然后确定虚拟页的范围
   let end = _start + _len;
   let start_va = VirtAddr::from(_start);
   let end_va = VirtAddr::from(end);
   let start_vpn = start_va.floor();
   let end_vpn = end_va.ceil();

    //本来不应该直接访问TASK_MANAGER.inner的，但是我感觉时间不够交题了，后面再重新写这一块吧
    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    let memory_set = &mut inner.tasks[current].memory_set;

   //然后检查[start, start + len) 中是否存在已经被映射的页
   for vpn_num in start_vpn.0..end_vpn.0{
        let vpn = VirtPageNum(vpn_num);

        if let Some(pte) = memory_set.translate(vpn){
            if pte.is_valid(){
                return -1;
            }
        }
   }

   //然后建立映射
    memory_set.insert_framed_area(start_va,end_va,permission,);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");

    if _start % PAGE_SIZE != 0{
        return -1;
    }
    if _len == 0{
        return -1;
    }

    let end = _start + _len;
    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(end);
    let start_vpn = start_va.floor();
    let end_vpn = end_va.ceil();

    let mut inner = TASK_MANAGER.inner.exclusive_access();
    let current = inner.current_task;
    let memory_set = &mut inner.tasks[current].memory_set;

    for vpn_num in start_vpn.0..end_vpn.0{
        let vpn = VirtPageNum(vpn_num);

        match memory_set.translate(vpn){
            Some(pte) if pte.is_valid() =>{}
            _ => return -1,
        }
    }

    let success = memory_set.remove_area(start_va, end_va);
    
    if success{
        0
    }else {
        -1
    }
    }

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
