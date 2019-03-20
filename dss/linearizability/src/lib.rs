mod bitset;
pub mod model;
pub mod models;

use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::bitset::Bitset;
use crate::model::{Event, EventKind, Events, Model, Operations, Value};

enum EntryKind {
    CallEntry,
    ReturnEntry,
}

struct Entry<T> {
    pub kind: EntryKind,
    pub value: T,
    pub id: usize,
    pub time: i64,
}

fn make_entries<I: Debug, O: Debug>(history: Operations<I, O>) -> Vec<Entry<Value<I, O>>> {
    let mut entries = Vec::new();
    for (id, elem) in history.into_iter().enumerate() {
        entries.push(Entry {
            kind: EntryKind::CallEntry,
            value: Value::Input(elem.input),
            id,
            time: elem.call,
        });
        entries.push(Entry {
            kind: EntryKind::ReturnEntry,
            value: Value::Output(elem.output),
            id,
            time: elem.finish,
        })
    }
    entries.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
    entries
}

struct LinkedNodes<T: Debug> {
    head: Option<LinkNode<T>>,
}

impl<T: Debug> LinkedNodes<T> {
    pub fn new() -> Self {
        LinkedNodes { head: None }
    }

    pub fn head(&self) -> Option<LinkNode<T>> {
        self.head.clone()
    }

    pub fn from_entries(entries: Vec<Entry<T>>) -> Self {
        let mut matches: HashMap<usize, LinkNode<T>> = HashMap::new();
        let mut nodes = Self::new();

        for entry in entries.into_iter().rev() {
            nodes.push_front(match entry.kind {
                EntryKind::CallEntry => Rc::new(RefCell::new(Node {
                    value: entry.value,
                    matched: matches.get(&entry.id).cloned(),
                    id: entry.id,
                    next: None,
                    prev: None,
                })),
                EntryKind::ReturnEntry => {
                    let node = Rc::new(RefCell::new(Node {
                        value: entry.value,
                        matched: None,
                        id: entry.id,
                        next: None,
                        prev: None,
                    }));
                    matches.insert(entry.id, node.clone());
                    node
                }
            })
        }

        nodes
    }

    pub fn len(&self) -> usize {
        let mut len = 0;
        let mut entry = self.head.clone();
        while let Some(e) = entry {
            entry = e.borrow().next.clone();
            len += 1;
        }
        len
    }

    pub fn push_front(&mut self, new_head: LinkNode<T>) {
        match self.head.take() {
            Some(old_head) => {
                old_head.borrow_mut().prev = Some(new_head.clone());
                new_head.borrow_mut().next = Some(old_head);
                self.head = Some(new_head);
            }
            None => {
                self.head = Some(new_head);
            }
        }
    }
}

type LinkNode<T> = Rc<RefCell<Node<T>>>;

struct Node<T: Debug> {
    pub value: T,
    pub matched: Option<LinkNode<T>>,
    pub id: usize,
    pub next: Option<LinkNode<T>>,
    pub prev: Option<LinkNode<T>>,
}

fn renumber<T>(events: Vec<Event<T>>) -> Vec<Event<T>> {
    let mut e = Vec::new();
    let mut m: HashMap<usize, usize> = HashMap::new(); // renumbering
    let mut id: usize = 0;
    for event in events {
        e.push(Event {
            kind: event.kind,
            value: event.value,
            id: *m.entry(event.id).or_insert_with(|| {
                id += 1;
                id - 1
            }),
        });
    }
    e
}

fn convert_entries<T>(events: Vec<Event<T>>) -> Vec<Entry<T>> {
    let mut entries = Vec::new();
    for event in events {
        entries.push(match event.kind {
            EventKind::CallEvent => Entry {
                kind: EntryKind::CallEntry,
                value: event.value,
                id: event.id,
                time: -1,
            },
            EventKind::ReturnEvent => Entry {
                kind: EntryKind::ReturnEntry,
                value: event.value,
                id: event.id,
                time: -1,
            },
        })
    }
    entries
}

struct CacheEntry<T> {
    linearized: Bitset,
    state: T,
}

fn cache_contains<M: Model>(
    model: &M,
    cache: &HashMap<u64, Vec<CacheEntry<M::State>>>,
    entry: &CacheEntry<M::State>,
) -> bool {
    if cache.contains_key(&entry.linearized.hash()) {
        for elem in &cache[&entry.linearized.hash()] {
            if entry.linearized.equals(&elem.linearized) && model.equal(&entry.state, &elem.state) {
                return true;
            }
        }
    }
    false
}

struct CallsEntry<V: Debug, T> {
    entry: Option<LinkNode<V>>,
    state: T,
}

fn lift<T: Debug>(entry: &LinkNode<T>) {
    let prev = Ref::map(entry.borrow(), |e| e.prev.as_ref().unwrap());
    prev.borrow_mut().next = entry.borrow().next.clone();
    let next = Ref::map(entry.borrow(), |e| e.next.as_ref().unwrap());
    next.borrow_mut().prev = entry.borrow().prev.clone();

    let matched = Ref::map(entry.borrow(), |e| e.matched.as_ref().unwrap());
    let matched_prev = Ref::map(matched.borrow(), |e| e.prev.as_ref().unwrap());
    matched_prev.borrow_mut().next = matched.borrow().next.clone();
    if matched.borrow().next.is_some() {
        let matched_next = Ref::map(matched.borrow(), |e| e.next.as_ref().unwrap());
        matched_next.borrow_mut().prev = matched.borrow().prev.clone();
    }
}

fn unlift<T: Debug>(entry: &LinkNode<T>) {
    {
        let matched = Ref::map(entry.borrow(), |e| e.matched.as_ref().unwrap());
        let matched_prev = Ref::map(matched.borrow(), |e| e.prev.as_ref().unwrap());
        matched_prev.borrow_mut().next = Some(matched.clone());
        if matched.borrow().next.is_some() {
            let matched_next = Ref::map(matched.borrow(), |e| e.next.as_ref().unwrap());
            matched_next.borrow_mut().prev = Some(matched.clone());
        }
    }

    let prev = Ref::map(entry.borrow(), |e| e.prev.as_ref().unwrap());
    prev.borrow_mut().next = Some(entry.clone());
    let next = Ref::map(entry.borrow(), |e| e.next.as_ref().unwrap());
    next.borrow_mut().prev = Some(entry.clone());
}

fn check_single<M: Model>(
    model: M,
    mut subhistory: LinkedNodes<Value<M::Input, M::Output>>,
    kill: Arc<AtomicBool>,
) -> bool {
    let n = subhistory.len() / 2;
    let mut linearized = Bitset::new(n);
    let mut cache = HashMap::new();
    let mut calls = vec![];

    let mut state = model.init();
    subhistory.push_front(Rc::new(RefCell::new(Node {
        value: Value::None,
        matched: None,
        id: usize::max_value(),
        prev: None,
        next: None,
    })));
    let head_entry = subhistory.head().unwrap();
    let mut entry = head_entry.borrow().next.clone();
    while head_entry.borrow().next.is_some() {
        if kill.load(Ordering::SeqCst) {
            return false;
        }
        let matched = entry.as_ref().unwrap().borrow().matched.clone();
        entry = if let Some(matching) = matched {
            // the return entry
            let res = model.step(
                &state,
                entry.as_ref().unwrap().borrow().value.input(),
                matching.borrow().value.output(),
            );
            match res {
                (true, new_state) => {
                    let mut new_linearized = linearized.clone();
                    new_linearized.set(entry.as_ref().unwrap().borrow().id);
                    let new_cache_entry = CacheEntry {
                        linearized: new_linearized.clone(),
                        state: new_state.clone(),
                    };
                    if !cache_contains(&model, &cache, &new_cache_entry) {
                        let hash = new_linearized.hash();
                        cache.entry(hash).or_default().push(new_cache_entry);
                        calls.push(CallsEntry {
                            entry: entry.clone(),
                            state,
                        });
                        state = new_state;
                        linearized.set(entry.as_ref().unwrap().borrow().id);
                        lift(entry.as_ref().unwrap());
                        head_entry.borrow().next.clone()
                    } else {
                        entry.as_ref().unwrap().borrow().next.clone()
                    }
                }
                (false, _) => entry.as_ref().unwrap().borrow().next.clone(),
            }
        } else {
            if calls.is_empty() {
                return false;
            }
            let calls_top = calls.pop().unwrap();
            entry = calls_top.entry;
            state = calls_top.state;
            linearized.clear(entry.as_ref().unwrap().borrow().id);
            unlift(entry.as_ref().unwrap());
            entry.as_ref().unwrap().borrow().next.clone()
        }
    }
    true
}

pub fn check_operations<M: Model>(model: M, history: Operations<M::Input, M::Output>) -> bool {
    check_operations_timeout(model, history, Duration::new(0, 0))
}

// timeout = 0 means no timeout
// if this operation times out, then a false positive is possible
pub fn check_operations_timeout<M: Model>(
    model: M,
    history: Operations<M::Input, M::Output>,
    timeout: Duration,
) -> bool {
    let partitions = model.partition(history);

    let (tx, rx) = channel();
    let mut handles = vec![];
    let kill = Arc::new(AtomicBool::new(false));
    let count = partitions.len();
    for subhistory in partitions {
        let tx = tx.clone();
        let kill = Arc::clone(&kill);
        let m = model.clone();
        let handle = thread::spawn(move || {
            let l = LinkedNodes::from_entries(make_entries(subhistory));
            let _ = tx.send(check_single(m, l, kill));
        });
        handles.push(handle);
    }

    let res = wait_res(rx, kill, count, timeout);
    for handle in handles {
        handle.join().unwrap();
    }
    res
}

pub fn check_events<M: Model>(model: M, history: Events<M::Input, M::Output>) -> bool {
    check_events_timeout(model, history, Duration::new(0, 0))
}

// timeout = 0 means no timeout
// if this operation times out, then a false positive is possible
pub fn check_events_timeout<M: Model>(
    model: M,
    history: Events<M::Input, M::Output>,
    timeout: Duration,
) -> bool {
    let partitions = model.partition_event(history);

    let (tx, rx) = channel();
    let mut handles = vec![];
    let kill = Arc::new(AtomicBool::new(false));
    let count = partitions.len();
    for subhistory in partitions {
        let tx = tx.clone();
        let kill = Arc::clone(&kill);
        let m = model.clone();
        let handle = thread::spawn(move || {
            let l = LinkedNodes::from_entries(convert_entries(renumber(subhistory)));
            let _ = tx.send(check_single(m, l, kill));
        });
        handles.push(handle);
    }

    let res = wait_res(rx, kill, count, timeout);
    for handle in handles {
        handle.join().unwrap();
    }
    res
}

fn wait_res(
    rx: Receiver<bool>,
    kill: Arc<AtomicBool>,
    mut count: usize,
    timeout: Duration,
) -> bool {
    let mut ok = true;
    loop {
        match if timeout.as_secs() == 0 && timeout.subsec_nanos() == 0 {
            rx.recv().map_err(From::from)
        } else {
            rx.recv_timeout(timeout)
        } {
            Ok(res) => {
                ok = ok && res;
                if !ok {
                    kill.store(true, Ordering::SeqCst);
                    break;
                }
                count -= 1;
                if count == 0 {
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => break,
            Err(e) => panic!("recv err: {}", e),
        }
    }
    ok
}
