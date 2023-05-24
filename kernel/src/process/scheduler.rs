use super::{Process, ProcessId};
use crate::arch::interrupt::StackFrame;
use crate::arch::thread;
use core::ptr::NonNull;
use spin::Mutex;

const MAX_SCHEDULED_PROCS: usize = 256;

struct Scheduler {
    procs: [ProcessId; MAX_SCHEDULED_PROCS],
    count: usize,
    cur: usize,
}

impl Scheduler {
    fn push(&mut self, id: ProcessId) {
        if self.cur >= MAX_SCHEDULED_PROCS {
            panic!("too many scheduled processes");
        }
        self.procs[self.count] = id;
        self.count += 1;
    }

    fn index_in_steps(&self, steps: usize) -> usize {
        (self.cur + steps) % self.count.max(1)
    }

    fn get(&self, steps: usize) -> Option<ProcessId> {
        let index = self.index_in_steps(steps);
        if self.count != 0 {
            Some(self.procs[index])
        } else {
            None
        }
    }

    fn next(&self) -> Option<ProcessId> {
        self.get(1)
    }

    fn get_proc(&self, id: ProcessId) -> Option<NonNull<Process>> {
        super::get(id)
    }

    fn advance(&mut self) {
        self.cur += 1;
        self.cur %= self.count.max(1);
    }
}

static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler {
    procs: [ProcessId(0); MAX_SCHEDULED_PROCS],
    count: 0,
    cur: 0,
});

pub fn schedule(process_id: ProcessId) {
    debug!("scheduling process '{}'", process_id);
    SCHEDULER.lock().push(process_id);
}

pub fn deschedule(_process_id: ProcessId) {
    unimplemented!()
}

pub unsafe fn step(stackframe: *mut StackFrame) {
    let mut scheduler = SCHEDULER.lock();
    let cur_thread = unsafe { thread::cur_thread().as_mut() };
    cur_thread.update_stackframe(stackframe.read());
    match scheduler.next() {
        Some(id)
            if let Some(mut proc) = scheduler.get_proc(id) =>  {
                let proc = unsafe { proc.as_mut() };
                let thread_ptr = proc.threads[0];
                let thread = thread_ptr.as_ref();
                stackframe.write(thread.get_stackframe());
                thread::set_thread(thread_ptr);
                debug!("running processes '{id}'");
            }
        _ => {}
    }
    scheduler.advance();
}
