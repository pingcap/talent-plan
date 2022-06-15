use std::cmp::PartialEq;
use std::fmt::Debug;
use std::fmt::Display;
use std::marker::Send;

#[derive(Debug)]
pub enum Value<I: Debug, O: Debug> {
    Input(I),
    Output(O),
    None,
}

impl<I: Debug, O: Debug> Value<I, O> {
    pub fn input(&self) -> &I {
        if let Value::Input(i) = self {
            i
        } else {
            panic!("Not a input")
        }
    }

    pub fn output(&self) -> &O {
        if let Value::Output(o) = self {
            o
        } else {
            panic!("Not a output")
        }
    }
}

#[derive(Debug, Clone)]
pub struct Operation<I: Debug, O: Debug> {
    pub input: I,
    pub call: i64, // invocation time
    pub output: O,
    pub finish: i64, // response time
}

pub enum EventKind {
    CallEvent,
    ReturnEvent,
}

pub struct Event<T> {
    pub kind: EventKind,
    pub value: T,
    pub id: usize,
}

pub type Operations<I, O> = Vec<Operation<I, O>>;
pub type Events<I, O> = Vec<Event<Value<I, O>>>;

pub trait Model: Clone + Send + 'static {
    type State: Clone + Display + PartialEq;
    type Input: Send + Debug + 'static;
    type Output: Send + Debug + 'static;

    // Partition functions, such that a history is linearizable if an only
    // if each partition is linearizable. If you don't want to implement
    // this, you can always use the `NoPartition` functions implemented
    // below.
    fn partition(
        &self,
        history: Operations<Self::Input, Self::Output>,
    ) -> Vec<Operations<Self::Input, Self::Output>> {
        vec![history]
    }

    fn partition_event(
        &self,
        history: Events<Self::Input, Self::Output>,
    ) -> Vec<Events<Self::Input, Self::Output>> {
        vec![history]
    }

    // Initial state of the system.
    fn init(&self) -> Self::State;

    // Step function for the system. Returns whether or not the system
    // could take this step with the given inputs and outputs and also
    // returns the new state. This should not mutate the existing state.
    fn step(
        &self,
        state: &Self::State,
        input: &Self::Input,
        output: &Self::Output,
    ) -> (bool, Self::State);

    // Equality on states. If you are using a simple data type for states,
    // you can use the `ShallowEqual` function implemented below.
    fn equal(&self, state1: &Self::State, state2: &Self::State) -> bool {
        state1 == state2
    }
}
