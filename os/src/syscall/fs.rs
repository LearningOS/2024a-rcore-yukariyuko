//! File and filesystem-related syscalls

use core::mem::size_of;

use crate::fs::{get_inode_id, open_file, OpenFlags, Stat, StatMode};
use crate::mm::{translated_byte_buffer, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let mut path = translated_str(token, path);
    let mut inner = task.inner_exclusive_access();
    let mut create_new: bool = true;
    if let Some(pt) = inner.link_table.get(&path) {
        path = pt.clone();
        create_new = false;
    } else if !OpenFlags::from_bits(flags).unwrap().is_create() {
        return -1;
    }
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        if create_new {
            inner.link_table.insert(path.clone(), path.clone());
        }
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        inner.fd_name.insert(fd, path);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let task = current_task().unwrap();
    let token = current_user_token();
    let bytes = translated_byte_buffer(token, st as *const u8, size_of::<Stat>());
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    let name = inner.fd_name.get(&fd).unwrap();
    let mut cnt = 0;
    for (_, tname) in &inner.link_table {
        if *tname == *name {
            cnt += 1;
        }
    }
    let stt = Stat::new(get_inode_id(name).unwrap() as u64, StatMode::FILE, cnt);
    let mut stp = &stt as *const Stat as *const u8;
    for byte in bytes {
        for b in byte {
            unsafe {
                *b = *stp;
                stp = stp.add(1);
            }
        }
    }
    0
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_linkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let old_name = translated_str(current_user_token(), old_name);
    let new_name = translated_str(current_user_token(), new_name);
    if old_name == new_name {
        return -1;
    }
    let inner = current_task().unwrap();
    let link_table = &mut inner.inner_exclusive_access().link_table;
    if let Some(tf_name) = link_table.get(&old_name) {
        link_table.insert(new_name, tf_name.clone());
        0
    } else {
        -1
    }
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let name = translated_str(current_user_token(), name);
    let inner = current_task().unwrap();
    let link_table = &mut inner.inner_exclusive_access().link_table;
    if link_table.remove(&name).is_none() {
        -1
    } else {
        0
    }
}
