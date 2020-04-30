use std::collections::HashMap;

use super::model::{EventKind, Events, Model, Operations};

#[derive(Clone, Debug)]
pub enum Op {
    GET,
    PUT,
    APPEND,
}

#[derive(Clone, Debug)]
pub struct KvInput {
    pub op: Op,
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct KvOutput {
    pub value: String,
}

#[derive(Clone, Default)]
pub struct KvModel {}

impl Model for KvModel {
    type State = String;
    type Input = KvInput;
    type Output = KvOutput;

    fn partition(
        &self,
        history: Operations<Self::Input, Self::Output>,
    ) -> Vec<Operations<Self::Input, Self::Output>> {
        let mut map = HashMap::new();
        for op in history {
            let v = map.entry(op.input.key.clone()).or_insert_with(|| vec![]);
            (*v).push(op);
        }
        let mut ret = vec![];
        for (_, ops) in map {
            ret.push(ops);
        }
        ret
    }

    fn partition_event(
        &self,
        history: Events<Self::Input, Self::Output>,
    ) -> Vec<Events<Self::Input, Self::Output>> {
        let mut m = HashMap::new();
        let mut matched: HashMap<usize, String> = HashMap::new();
        for event in history {
            match event.kind {
                EventKind::CallEvent => {
                    let key = event.value.input().key.clone();
                    matched.insert(event.id, key.clone());
                    m.entry(key).or_insert_with(|| vec![]).push(event);
                }
                EventKind::ReturnEvent => {
                    let key = matched[&event.id].clone();
                    m.entry(key).or_insert_with(|| vec![]).push(event);
                }
            }
        }
        let mut ret = vec![];
        for (_, v) in m {
            ret.push(v);
        }
        ret
    }

    fn init(&self) -> Self::State {
        // note: we are modeling a single key's value here;
        // we're partitioning by key, so this is okay
        "".to_string()
    }

    fn step(
        &self,
        state: &Self::State,
        input: &Self::Input,
        output: &Self::Output,
    ) -> (bool, Self::State) {
        match input.op {
            Op::GET => (&output.value == state, state.clone()),
            Op::PUT => (true, input.value.clone()),
            Op::APPEND => (true, state.clone() + &input.value),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufRead, BufReader, Result};

    use super::super::check_events;
    use super::{KvInput, KvModel, KvOutput, Op};
    use crate::model::{Event, EventKind, Events, Model, Value};
    use regex::Regex;

    fn check_kv(log_name: String, correct: bool) {
        let model = KvModel {};

        let file_name = format!("../linearizability/test_data/{}.txt", &log_name);
        let events = match parse_kv_log(&file_name) {
            Ok(events) => events,
            Err(e) => panic!("parse kv log {} failed: {}", &file_name, e),
        };
        assert_eq!(check_events(model, events), correct);
    }

    fn parse_kv_log(
        file_name: &str,
    ) -> Result<Events<<KvModel as Model>::Input, <KvModel as Model>::Output>> {
        lazy_static::lazy_static! {
            static ref INVOKE_GET: Regex = Regex::new(
                r#"\{:process (\d+), :type :invoke, :f :get, :key "(.*)", :value nil\}"#
            )
            .unwrap();
            static ref INVOKE_PUT: Regex = Regex::new(
                r#"\{:process (\d+), :type :invoke, :f :put, :key "(.*)", :value "(.*)"\}"#
            )
            .unwrap();
            static ref INVOKE_APPEND: Regex = Regex::new(
                r#"\{:process (\d+), :type :invoke, :f :append, :key "(.*)", :value "(.*)"\}"#
            )
            .unwrap();
            static ref RETURN_GET: Regex =
                Regex::new(r#"\{:process (\d+), :type :ok, :f :get, :key ".*", :value "(.*)"\}"#)
                    .unwrap();
            static ref RETURN_PUT: Regex =
                Regex::new(r#"\{:process (\d+), :type :ok, :f :put, :key ".*", :value ".*"\}"#)
                    .unwrap();
            static ref RETURN_APPEND: Regex =
                Regex::new(r#"\{:process (\d+), :type :ok, :f :append, :key ".*", :value ".*"\}"#)
                    .unwrap();
        }

        let f = File::open(file_name)?;
        let buf_reader = BufReader::new(f);
        let mut events = vec![];
        let mut id = 0;
        let mut procid_map: HashMap<isize, usize> = HashMap::new();

        for line in buf_reader.lines() {
            let contents = line.unwrap();
            if let Some(args) = INVOKE_GET.captures(&contents) {
                events.push(Event {
                    kind: EventKind::CallEvent,
                    value: Value::Input(KvInput {
                        op: Op::GET,
                        key: args[2].to_string(),
                        value: "".to_string(),
                    }),
                    id,
                });
                procid_map.insert(args[1].to_string().parse().unwrap(), id);
                id += 1;
            } else if let Some(args) = INVOKE_PUT.captures(&contents) {
                events.push(Event {
                    kind: EventKind::CallEvent,
                    value: Value::Input(KvInput {
                        op: Op::PUT,
                        key: args[2].to_string(),
                        value: args[3].to_string(),
                    }),
                    id,
                });
                procid_map.insert(args[1].to_string().parse().unwrap(), id);
                id += 1;
            } else if let Some(args) = INVOKE_APPEND.captures(&contents) {
                events.push(Event {
                    kind: EventKind::CallEvent,
                    value: Value::Input(KvInput {
                        op: Op::APPEND,
                        key: args[2].to_string(),
                        value: args[3].to_string(),
                    }),
                    id,
                });
                procid_map.insert(args[1].to_string().parse().unwrap(), id);
                id += 1;
            } else if let Some(args) = RETURN_GET.captures(&contents) {
                let match_id = procid_map
                    .remove(&args[1].to_string().parse().unwrap())
                    .unwrap();
                events.push(Event {
                    kind: EventKind::ReturnEvent,
                    value: Value::Output(KvOutput {
                        value: args[2].to_string(),
                    }),
                    id: match_id,
                });
            } else if let Some(args) = RETURN_PUT.captures(&contents) {
                let match_id = procid_map
                    .remove(&args[1].to_string().parse().unwrap())
                    .unwrap();
                events.push(Event {
                    kind: EventKind::ReturnEvent,
                    value: Value::Output(KvOutput {
                        value: "".to_string(),
                    }),
                    id: match_id,
                });
            } else if let Some(args) = RETURN_APPEND.captures(&contents) {
                let match_id = procid_map
                    .remove(&args[1].to_string().parse().unwrap())
                    .unwrap();
                events.push(Event {
                    kind: EventKind::ReturnEvent,
                    value: Value::Output(KvOutput {
                        value: "".to_string(),
                    }),
                    id: match_id,
                });
            } else {
                unreachable!();
            }
        }

        for (_, match_id) in procid_map {
            events.push(Event {
                kind: EventKind::ReturnEvent,
                value: Value::Output(KvOutput {
                    value: "".to_string(),
                }),
                id: match_id,
            })
        }
        Ok(events)
    }

    #[test]
    fn test_kv_1client_ok() {
        check_kv("c01-ok".to_string(), true)
    }

    #[test]
    fn test_kv_1client_bad() {
        check_kv("c01-bad".to_string(), false)
    }

    #[test]
    fn test_kv_10client_ok() {
        check_kv("c10-ok".to_string(), true)
    }

    #[test]
    fn test_kv_10client_bad() {
        check_kv("c10-bad".to_string(), false)
    }

    #[test]
    fn test_kv_50client_ok() {
        check_kv("c50-ok".to_string(), true)
    }

    #[test]
    fn test_kv_50client_bad() {
        check_kv("c50-bad".to_string(), false)
    }
}
