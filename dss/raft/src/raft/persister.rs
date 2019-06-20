//! Support for Raft and kvraft to save persistent
//! Raft state (log &c) and k/v server snapshots.
//!
//! we will use the original persister.rs to test your code for grading.
//! so, while you can modify this code to help you debug, please
//! test with the original before submitting.

use std::sync::{Arc, Mutex};

pub trait Persister: Send + 'static {
    fn raft_state(&self) -> Vec<u8>;
    fn save_raft_state(&self, state: Vec<u8>);
    fn save_state_and_snapshot(&self, state: Vec<u8>, snapshot: Vec<u8>);
    fn snapshot(&self) -> Vec<u8>;
}

impl<T: ?Sized + Persister> Persister for Box<T> {
    fn raft_state(&self) -> Vec<u8> {
        (**self).raft_state()
    }
    fn save_raft_state(&self, state: Vec<u8>) {
        (**self).save_raft_state(state)
    }
    fn save_state_and_snapshot(&self, state: Vec<u8>, snapshot: Vec<u8>) {
        (**self).save_state_and_snapshot(state, snapshot)
    }
    fn snapshot(&self) -> Vec<u8> {
        (**self).snapshot()
    }
}

impl<T: ?Sized + Sync + Persister> Persister for Arc<T> {
    fn raft_state(&self) -> Vec<u8> {
        (**self).raft_state()
    }
    fn save_raft_state(&self, state: Vec<u8>) {
        (**self).save_raft_state(state)
    }
    fn save_state_and_snapshot(&self, state: Vec<u8>, snapshot: Vec<u8>) {
        (**self).save_state_and_snapshot(state, snapshot)
    }
    fn snapshot(&self) -> Vec<u8> {
        (**self).snapshot()
    }
}

pub struct SimplePersister {
    states: Mutex<(
        Vec<u8>, // raft state
        Vec<u8>, // snapshot
    )>,
}

impl SimplePersister {
    pub fn new() -> SimplePersister {
        SimplePersister {
            states: Mutex::default(),
        }
    }
}

impl Persister for SimplePersister {
    fn save_raft_state(&self, state: Vec<u8>) {
        self.states.lock().unwrap().0 = state;
    }

    fn raft_state(&self) -> Vec<u8> {
        self.states.lock().unwrap().0.clone()
    }

    fn save_state_and_snapshot(&self, state: Vec<u8>, snapshot: Vec<u8>) {
        self.states.lock().unwrap().0 = state;
        self.states.lock().unwrap().1 = snapshot;
    }

    fn snapshot(&self) -> Vec<u8> {
        self.states.lock().unwrap().1.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_safety() {
        let sp = SimplePersister::new();
        sp.save_raft_state(vec![111]);
        let obj: Box<dyn Persister + Sync> = Box::new(sp);
        assert_eq!(obj.raft_state(), vec![111]);
        obj.save_state_and_snapshot(vec![222], vec![123]);
        assert_eq!(obj.raft_state(), vec![222]);
        assert_eq!(obj.snapshot(), vec![123]);

        let cloneable_obj: Arc<dyn Persister> = Arc::new(obj);
        assert_eq!(cloneable_obj.raft_state(), vec![222]);
        assert_eq!(cloneable_obj.snapshot(), vec![123]);

        let cloneable_obj_ = cloneable_obj.clone();
        cloneable_obj.save_raft_state(vec![233]);
        assert_eq!(cloneable_obj_.raft_state(), vec![233]);
        assert_eq!(cloneable_obj_.snapshot(), vec![123]);

        let sp = SimplePersister::new();
        let obj: Arc<dyn Persister + Sync> = Arc::new(sp);
        let _box_obj: Box<dyn Persister> = Box::new(obj);
    }
}
