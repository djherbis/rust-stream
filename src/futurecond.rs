use std::sync::{Arc, Mutex, Condvar};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ops::Drop;
use futures::stream::Stream;
use futures::{Async, Poll};
use futures::task;
use std::io; // do we really need this? no error can be returned...

pub struct State {
    size: usize,
    open: bool,
}

pub struct FutureCond {
    cond: Arc<(Mutex<State>, Condvar)>,
}

impl FutureCond {
    pub fn new() -> FutureCond {
        FutureCond {
            cond: Arc::new((Mutex::new(State {
                size: 0,
                open: true,
            }),
                            Condvar::new())),
        }
    }

    pub fn wrote(&self, n: usize) {
        let &(ref lock, ref cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        state.size += n;
        cvar.notify_all();
    }

    pub fn listener(&self) -> Listener {
        Listener { off: AtomicUsize::new(0), cond: self.cond.clone() }
    }
}

impl Drop for FutureCond {
    fn drop(&mut self) {
        let &(ref lock, ref cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        state.open = false;
        cvar.notify_all();
    }
}

pub struct Listener {
    off: AtomicUsize, // may not need to be atomic
    cond: Arc<(Mutex<State>, Condvar)>,
}

impl Listener {
    pub fn off(&self, off: u64) {
        self.off.swap(off as usize, Ordering::SeqCst);
    }

    pub fn state(&self, off: u64) -> (usize, bool) {
        let &(ref lock, _) = &*self.cond;
        let state = lock.lock().unwrap();
        (state.size - off as usize, state.open)
    }
}

impl Stream for Listener {
    type Item = u64;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<u64>, io::Error> {
        let state = self.off.load(Ordering::SeqCst);
        let (off, open) = self.state(state as u64);
        if off > 0 {
            self.off.swap(off+state, Ordering::SeqCst);
            return Ok(Async::Ready(Some(off as u64)))
        }
        if !open {
            return Ok(Async::Ready(None));
        }
        // now how do we get the task handle?
        let readTask = task::park();
        // chain unpark on the next write?
        Ok(Async::NotReady)
    }
}