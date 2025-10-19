//! Process management syscalls
use crate::{mm::{translated_addressr, translated_addressw, MapPermission, VirtAddr}, task::{change_program_brk, current_user_token, exit_current_and_run_next, get_syscall_trace, mmap,munmap, suspend_current_and_run_next}, timer::get_time_us};

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
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let pa = match translated_addressw(current_user_token(),_ts as usize){
        Some(pa) => pa,
        None => return -1
    } as *mut TimeVal;
    let us = get_time_us();
    unsafe {
        *pa = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request{
        0 => {
            match translated_addressr(current_user_token(), _id){
                Some(pa) => {
                    unsafe {
                        (pa as *const u8).read_volatile() as isize   
                    }
                },
                None => -1,
            }
        },
        1 => {
            match translated_addressw(current_user_token(), _id){
                Some(pa) => {
                    unsafe {
                        (pa as *mut u8).write(_data as u8);
                        0
                    }
                },
                None => -1,
            }
        },
        2 => {
            get_syscall_trace(_id) as isize
        },
        _ => {-1 as isize}
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_map {:#x} {:#x} {:#x}",_start,_len,_port);
    if _start & 0xfff != 0 || _port & !0x7 != 0 || _port & 0x7 == 0{
        return -1;
    }
    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(_start+_len);
    let mut map_permission = MapPermission::U;
    if _port & 0x1 != 0 {map_permission = map_permission|MapPermission::R;}
    if _port & 0x2 != 0 {map_permission = map_permission|MapPermission::W;}
    if _port & 0x4 != 0 {map_permission = map_permission|MapPermission::X;}
    let result = mmap(start_va, end_va, map_permission);
    result
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap {:#x} {:#x}",_start,_len);
    if _start & 0xfff != 0{
        return -1;
    }
    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(_start+_len);
    let result = munmap(start_va, end_va);
    result
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
