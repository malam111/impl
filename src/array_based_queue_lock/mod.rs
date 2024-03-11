use std::sync;
use std::ops::{Deref, DerefMut};
use std::cell;

#[derive(Debug, PartialEq)]
enum Qstat {
    Go,
    Wait,
    Empty,
}

#[derive(Debug)]
enum Qerror {
    OutOfTicket,
}

pub struct InnerQ {
    // static sized?
    queue: Vec<Qstat>,
    last_q: usize,
    load: usize,
}

pub struct Qlock {
    inner: sync::Mutex<()>,
    data: cell::UnsafeCell<InnerQ>,
}

unsafe impl Send for Qlock {}
unsafe impl Sync for Qlock {}

impl Qlock {

    pub fn new(num: usize) -> Self {
        let mut vec = Vec::with_capacity(num);
        for i in 0..num {
            vec.push(Qstat::Empty);
        }
        Qlock {
            inner: sync::Mutex::new(()),
            data: cell::UnsafeCell::new(
                InnerQ {
                    queue: vec,
                    last_q: 0,
                    load: 0,
                }
            )
        }
    }

    fn get_mut(&self) -> &mut InnerQ {
        unsafe { &mut *self.data.get() }
    }

    pub fn queue(&self, idx: usize) -> Result<usize, Qerror> {
        let _lock = self.inner.lock().unwrap();
        let mut innerq = self.get_mut();
        if innerq.queue[innerq.last_q] == Qstat::Wait {
            return Err(Qerror::OutOfTicket);
        }
        innerq.queue[innerq.last_q] = Qstat::Wait;
        let ret = innerq.last_q;
        if innerq.load == 0 {
            innerq.queue[innerq.last_q] = Qstat::Go;
        }
        innerq.last_q = (innerq.last_q + 1) % innerq.queue.capacity();
        innerq.load += 1;
        Self::log(format!("Q{}", idx).as_str(), innerq);
        Ok(ret) 
    }

    // this ticketing system is bad, because we're asking and trusting the user to hand their true
    // ticket, need a wrapper
    pub fn check(&self, ticket: usize, idx: usize) -> bool {
        let _lock = self.inner.lock().unwrap();
        let mut innerq = self.get_mut();
        if innerq.queue[ticket] != Qstat::Go { 
            std::hint::spin_loop(); return false; }
        Self::log(format!("C{}", idx).as_str(), innerq);
        true  
    }

    pub fn unlock(&self, ticket: usize, idx: usize) {
        let _lock = self.inner.lock().unwrap();
        let mut innerq = self.get_mut();
        let next = (ticket + 1) % innerq.queue.capacity();
        if innerq.queue[next] == Qstat::Wait {
            innerq.queue[next] = Qstat::Go;
        }
        innerq.queue[ticket] = Qstat::Empty;
        innerq.load -= 1;
        Self::log(format!("U{}", idx).as_str(), innerq);
    }

    pub fn log(msg: &str, mutex: &mut InnerQ) {
        println!("{} -> {:?}", msg, mutex.queue);
    }
}

struct MTD<T> {
    lock: Qlock, 
    inner: T,
}

impl<T: AsRef<str>> MTD<T> {
    pub fn new(num: usize, inner: T) -> Self {
        MTD {
            lock: Qlock::new(num),
            inner
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::sync;
    use std::thread;
    use std::time::Duration;
    
    use rand::Rng;

    const THREAD_NUM: usize = 4;

    #[test]
    fn main_test() {
        let string = "hello".to_string();
        let queue = sync::Arc::new(Qlock::new(THREAD_NUM));
        let mut handles = vec![];
        let mut rng = rand::thread_rng();
        for idx in 0..THREAD_NUM {
            let thread: sync::Arc<Qlock> = sync::Arc::clone(&queue);
            let rng = rng.gen_range(0..=2);
            let handle = thread::spawn( move || {
                loop {
                    let ticket = thread.queue(idx).unwrap();
                    while !thread.check(ticket, idx) { std::hint::spin_loop(); }
                    thread::sleep(Duration::from_secs(rng));
                    thread.unlock(ticket, idx);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
        assert!(true);
    }
}
