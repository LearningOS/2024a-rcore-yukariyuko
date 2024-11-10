use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    let check = process_inner.enable_dead_lock_check;
    drop(process_inner);
    drop(process);
    if check && !deadlock_check(0, mutex_id) {
        -0xDEAD
    } else {
        let binding = current_task().unwrap();
        let mut binding = binding.inner_exclusive_access();
        let need = &mut binding.mutex_need;
        need[mutex_id] += 1;
        drop(binding);
        mutex.lock();
        let binding = current_task().unwrap();
        let mut binding = binding.inner_exclusive_access();
        let need = &mut binding.mutex_need;
        need[mutex_id] -= 1;
        let alllocation = &mut binding.mutex_allocation;
        alllocation[mutex_id] += 1;
        0
    }
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    let binding = current_task().unwrap();
    let mut binding = binding.inner_exclusive_access();
    let alllocation = &mut binding.sem_allocation;
    alllocation[mutex_id] -= 1;
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    println!("up");
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let binding = current_task().unwrap();
    let mut binding = binding.inner_exclusive_access();
    let alllocation = &mut binding.sem_allocation;
    alllocation[sem_id] -= 1;
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    sem.up();
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    println!("down");
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let check = process_inner.enable_dead_lock_check;
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    drop(process_inner);
    drop(process);
    if check && !deadlock_check(1, sem_id) {
        -0xDEAD
    } else {
        println!("safe");
        let binding = current_task().unwrap();
        let mut binding = binding.inner_exclusive_access();
        let need = &mut binding.sem_need;
        need[sem_id] += 1;
        drop(binding);
        sem.down();
        let binding = current_task().unwrap();
        let mut binding = binding.inner_exclusive_access();
        let need = &mut binding.sem_need;
        need[sem_id] -= 1;
        let allocation = &mut binding.sem_allocation;
        allocation[sem_id] += 1;
        0
    }
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    let process = current_process();
    let mut procinr = process.inner_exclusive_access();
    if enabled == 1 {
        procinr.enable_dead_lock_check = true;
        0
    } else if enabled == 0 {
        procinr.enable_dead_lock_check = false;
        0
    } else {
        -1
    }
}

fn deadlock_check(type_: i32, id: usize) -> bool {
    println!("checking");
    let (mut work, allocation, mut need) = init(type_);
    let mut finish = vec![false; need.len()];
    let tid = current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .tid;
    need[tid][id] += 1;
    //println!("tid:{}\nw{:?}\na{:?}\nn{:?}", tid, work, allocation, need);
    loop {
        let mut find = false;
        for (i, f) in finish.iter_mut().enumerate() {
            if !*f {
                let mut can_fin = true;
                for (j, n) in need[i].iter().enumerate() {
                    if work[j] < *n {
                        can_fin = false;
                        break;
                    }
                }
                if can_fin {
                    for (j, n) in allocation[i].iter().enumerate() {
                        work[j] += *n;
                    }
                    *f = true;
                    find = true;
                }
            }
        }
        if !find {
            break;
        }
    }
    !finish.contains(&false)
}

fn init(type_: i32) -> (Vec<usize>, Vec<Vec<usize>>, Vec<Vec<usize>>) {
    let process = current_process();
    let procinr = process.inner_exclusive_access();
    let resources = 8;
    let mut work = if type_ == 0 {
        vec![1; resources]
    } else {
        let mut v = vec![0; resources];
        let list = &procinr.semaphore_list;
        for (i, s) in list.iter().enumerate() {
            if let Some(s) = s {
                v[i] = s.total;
            }
        }
        v
    };
    //println!("work_before: {:?}", work);
    let mut allocation = Vec::new();
    let mut need = Vec::new();
    let mut map = BTreeMap::new();
    let tasks = &procinr.tasks;
    for task in tasks {
        if let Some(task) = task.as_ref() {
            let task = task.inner_exclusive_access();
            if task.res.is_none() {
                allocation.push(vec![0; resources]);
                need.push(vec![0; resources]);
                continue;
            }
            let tid = task.res.as_ref().unwrap().tid;
            map.insert(tid, map.len());
            allocation.push(vec![0; resources]);
            need.push(vec![0; resources]);
            let (av, nv) = if type_ == 0 {
                (&task.mutex_allocation, &task.mutex_need)
            } else {
                (&task.sem_allocation, &task.sem_need)
            };
            // println!("id:{} a{:?} n{:?}", tid, av, nv);
            let idx = need.len() - 1;
            for (iav, a) in av.iter().enumerate() {
                allocation[idx][iav] += a;
                work[iav] -= a;
            }
            for (inv, n) in nv.iter().enumerate() {
                need[idx][inv] += n;
            }
        }
    }
    (work, allocation, need)
}
