use std::sync::{Arc, Mutex, Condvar};
use std::ops::Drop;

struct State {
    size: usize,
    open: bool,
}

pub struct Broadcaster {
    cond: Arc<(Mutex<State>, Condvar)>,
}

impl Broadcaster {
    pub fn new() -> Broadcaster {
        Broadcaster {
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
        Listener { cond: self.cond.clone() }
    }
}

impl Drop for Broadcaster {
    fn drop(&mut self) {
        let &(ref lock, ref cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        state.open = false;
        cvar.notify_all();
    }
}

pub struct Listener {
    cond: Arc<(Mutex<State>, Condvar)>,
}

impl Listener {
    pub fn wait(&self, off: u64) -> (usize, bool) {
        let &(ref lock, ref cvar) = &*self.cond;
        let mut state = lock.lock().unwrap();
        while state.open && state.size <= off as usize {
            state = cvar.wait(state).unwrap();
        }
        (state.size - off as usize, state.open)
    }
}
