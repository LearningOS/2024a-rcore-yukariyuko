//! Process management syscalls
use core::mem::size_of;

use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE},
    mm::{translated_byte_buffer, MapPermission, VirtAddr},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_app_memory_set,
        get_task_info, suspend_current_and_run_next, TaskStatus,
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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

/// get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let bytes = translated_byte_buffer(current_user_token(), ts as *const u8, size_of::<TimeVal>());
    let us = get_time_us();
    let tst = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let mut tstp = &tst as *const TimeVal as *const u8;
    for byte in bytes {
        for b in byte {
            unsafe {
                *b = *tstp;
                tstp = tstp.add(1);
            }
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let (status, times, time) = get_task_info();
    let mut syscall_times = [0; MAX_SYSCALL_NUM];
    for (i, cnt) in times {
        syscall_times[i] = cnt;
    }
    let bytes =
        translated_byte_buffer(current_user_token(), ti as *const u8, size_of::<TaskInfo>());
    let ti = TaskInfo {
        status,
        syscall_times,
        time,
    };
    let mut tip = &ti as *const TaskInfo as *const u8;
    for byte in bytes {
        for b in byte {
            unsafe {
                *b = *tip;
                tip = tip.add(1);
            }
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    if (port & !0x7 != 0) || (port & 0x7 == 0) {
        return -1;
    }
    let mut pages_num = len / PAGE_SIZE;
    if len % PAGE_SIZE != 0 {
        pages_num += 1;
    }
    let len = (len + PAGE_SIZE - 1) / PAGE_SIZE * PAGE_SIZE;
    let memory_set = get_app_memory_set();
    for i in 0..pages_num {
        unsafe {
            let vpa = VirtAddr(start + i * PAGE_SIZE);
            if (*memory_set).translate(vpa.into()).is_some()
                && (*memory_set).translate(vpa.into()).unwrap().is_valid()
            {
                return -1;
            }
        }
    }
    let mut permission = MapPermission::U;
    if port & 1 != 0 {
        permission |= MapPermission::R;
    }
    if port & 2 != 0 {
        permission |= MapPermission::W;
    }
    if port & 4 != 0 {
        permission |= MapPermission::X;
    }
    unsafe {
        (*memory_set).insert_framed_area(start.into(), (start + len).into(), permission);
    }
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    let mut pages_num = len / PAGE_SIZE;
    if len % PAGE_SIZE != 0 {
        pages_num += 1;
    }
    let memory_set = get_app_memory_set();
    for i in 0..pages_num {
        unsafe {
            if (*memory_set)
                .translate((start + i * PAGE_SIZE).into())
                .is_none()
                || !(*memory_set)
                    .translate((start + i * PAGE_SIZE).into())
                    .unwrap()
                    .is_valid()
            {
                return -1;
            }
        }
    }
    unsafe {
        (*memory_set).pop(VirtAddr::from(start).into());
    }
    0
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
