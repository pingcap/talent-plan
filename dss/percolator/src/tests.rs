use crate::client::Client;
use crate::server::{MemoryStorage, TimestampOracle};
use crate::service::{add_transaction_service, add_tso_service, TSOClient, TransactionClient};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use labrpc::*;
use prost::Message;

struct CommitHooks {
    drop_req: AtomicBool,
    drop_resp: AtomicBool,
    fail_primary: AtomicBool,
}

impl RpcHooks for CommitHooks {
    fn before_dispatch(&self, fq_name: &str, req: &[u8]) -> Result<()> {
        if self.drop_req.load(Ordering::Relaxed) && fq_name == "transaction.commit" {
            let m = crate::msg::CommitRequest::decode(req).unwrap();
            if m.is_primary && !self.fail_primary.load(Ordering::Relaxed) {
                return Ok(());
            }
            return Err(Error::Other("reqhook".to_owned()));
        }
        Ok(())
    }
    fn after_dispatch(&self, fq_name: &str, resp: Result<Vec<u8>>) -> Result<Vec<u8>> {
        if self.drop_resp.load(Ordering::Relaxed) && fq_name == "transaction.commit" {
            return Err(Error::Other("resphook".to_owned()));
        }
        resp
    }
}

fn init_logger() {
    use std::sync::Once;
    static LOGGER_INIT: Once = Once::new();
    LOGGER_INIT.call_once(env_logger::init);
}

fn init(num_clinet: usize) -> (Network, Vec<Client>, Arc<CommitHooks>) {
    init_logger();

    let mut clients = vec![];
    let rn = Network::new();
    let tso_server_name = "tso_server";
    let mut tso_server_builder = ServerBuilder::new(tso_server_name.to_owned());
    let server_name = "server";
    let mut server_builder = ServerBuilder::new(server_name.to_owned());
    let tso: TimestampOracle = Default::default();
    add_tso_service(tso, &mut tso_server_builder).unwrap();
    let store: MemoryStorage = Default::default();
    add_transaction_service(store, &mut server_builder).unwrap();
    let tso_server = tso_server_builder.build();
    let server = server_builder.build();
    rn.add_server(tso_server.clone());
    rn.add_server(server.clone());
    let hook = Arc::new(CommitHooks {
        drop_req: AtomicBool::new(false),
        drop_resp: AtomicBool::new(false),
        fail_primary: AtomicBool::new(false),
    });
    for i in 0..num_clinet {
        let txn_name_string = format!("txn{}", i);
        let txn_name = txn_name_string.as_str();
        let cli = rn.create_client(txn_name.to_owned());
        cli.set_hooks(hook.clone());
        let txn_client = TransactionClient::new(cli);
        rn.enable(txn_name, true);
        rn.connect(txn_name, server_name);
        let tso_name_string = format!("tso{}", i);
        let tso_name = tso_name_string.as_str();
        let cli = rn.create_client(tso_name.to_owned());
        let tso_client = TSOClient::new(cli);
        rn.enable(tso_name, true);
        rn.connect(tso_name, tso_server_name);
        clients.push(crate::client::Client::new(tso_client, txn_client));
    }

    (rn, clients, hook)
}

#[test]
fn test_get_timestamp_under_unreliable_network() {
    let (rn, clients, _) = init(3);
    let mut children = vec![];

    for (i, _) in clients.iter().enumerate() {
        let client = clients[i].to_owned();
        let tso_name_string = format!("tso{}", i);
        rn.enable(tso_name_string.as_str(), false);
        children.push(thread::spawn(move || {
            let res = client.get_timestamp();
            if i == 2 {
                assert_eq!(res, Err(Error::Timeout));
            } else {
                assert!(res.is_ok());
            }
        }));
    }

    thread::sleep(Duration::from_millis(100));
    rn.enable("tso0", true);
    thread::sleep(Duration::from_millis(200));
    rn.enable("tso1", true);
    thread::sleep(Duration::from_millis(400));
    rn.enable("tso2", true);

    for child in children {
        child.join().unwrap();
    }
}

#[test]
// https://github.com/ept/hermitage/blob/master/sqlserver.md#predicate-many-preceders-pmp
fn test_predicate_many_preceders_read_predicates() {
    let (_, clients, _) = init(3);

    let mut client0 = clients[0].to_owned();
    client0.begin();
    client0.set(b"1".to_vec(), b"10".to_vec());
    client0.set(b"2".to_vec(), b"20".to_vec());
    assert_eq!(client0.commit(), Ok(true));

    let mut client1 = clients[1].to_owned();
    client1.begin();
    assert_eq!(client1.get(b"3".to_vec()), Ok(Vec::new()));

    let mut client2 = clients[2].to_owned();
    client2.begin();
    client2.set(b"3".to_vec(), b"30".to_vec());
    assert_eq!(client2.commit(), Ok(true));

    assert_eq!(client1.get(b"3".to_vec()), Ok(Vec::new()));
}

#[test]
// https://github.com/ept/hermitage/blob/master/sqlserver.md#predicate-many-preceders-pmp
fn test_predicate_many_preceders_write_predicates() {
    let (_, clients, _) = init(3);

    let mut client0 = clients[0].to_owned();
    client0.begin();
    client0.set(b"1".to_vec(), b"10".to_vec());
    client0.set(b"2".to_vec(), b"20".to_vec());
    assert_eq!(client0.commit(), Ok(true));

    let mut client1 = clients[1].to_owned();
    client1.begin();

    let mut client2 = clients[2].to_owned();
    client2.begin();

    client1.set(b"1".to_vec(), b"20".to_vec());
    client1.set(b"2".to_vec(), b"30".to_vec());
    assert_eq!(client1.get(b"2".to_vec()), Ok(b"20".to_vec()));

    client2.set(b"2".to_vec(), b"40".to_vec());
    assert_eq!(client1.commit(), Ok(true));
    assert_eq!(client2.commit(), Ok(false));
}

#[test]
// https://github.com/ept/hermitage/blob/master/sqlserver.md#lost-update-p4
fn test_lost_update() {
    let (_, clients, _) = init(3);

    let mut client0 = clients[0].to_owned();
    client0.begin();
    client0.set(b"1".to_vec(), b"10".to_vec());
    client0.set(b"2".to_vec(), b"20".to_vec());
    assert_eq!(client0.commit(), Ok(true));

    let mut client1 = clients[1].to_owned();
    client1.begin();

    let mut client2 = clients[2].to_owned();
    client2.begin();

    assert_eq!(client1.get(b"1".to_vec()), Ok(b"10".to_vec()));
    assert_eq!(client2.get(b"1".to_vec()), Ok(b"10".to_vec()));

    client1.set(b"1".to_vec(), b"11".to_vec());
    client2.set(b"1".to_vec(), b"11".to_vec());
    assert_eq!(client1.commit(), Ok(true));
    assert_eq!(client2.commit(), Ok(false));
}

#[test]
// https://github.com/ept/hermitage/blob/master/sqlserver.md#read-skew-g-single
fn test_read_skew_read_only() {
    let (_, clients, _) = init(3);

    let mut client0 = clients[0].to_owned();
    client0.begin();
    client0.set(b"1".to_vec(), b"10".to_vec());
    client0.set(b"2".to_vec(), b"20".to_vec());
    assert_eq!(client0.commit(), Ok(true));

    let mut client1 = clients[1].to_owned();
    client1.begin();

    let mut client2 = clients[2].to_owned();
    client2.begin();

    assert_eq!(client1.get(b"1".to_vec()), Ok(b"10".to_vec()));
    assert_eq!(client2.get(b"1".to_vec()), Ok(b"10".to_vec()));
    assert_eq!(client2.get(b"2".to_vec()), Ok(b"20".to_vec()));

    client2.set(b"1".to_vec(), b"12".to_vec());
    client2.set(b"2".to_vec(), b"18".to_vec());
    assert_eq!(client2.commit(), Ok(true));

    assert_eq!(client1.get(b"2".to_vec()), Ok(b"20".to_vec()));
}

#[test]
// https://github.com/ept/hermitage/blob/master/sqlserver.md#read-skew-g-single
fn test_read_skew_predicate_dependencies() {
    let (_, clients, _) = init(3);

    let mut client0 = clients[0].to_owned();
    client0.begin();
    client0.set(b"1".to_vec(), b"10".to_vec());
    client0.set(b"2".to_vec(), b"20".to_vec());
    assert_eq!(client0.commit(), Ok(true));

    let mut client1 = clients[1].to_owned();
    client1.begin();

    let mut client2 = clients[2].to_owned();
    client2.begin();

    assert_eq!(client1.get(b"1".to_vec()), Ok(b"10".to_vec()));
    assert_eq!(client1.get(b"2".to_vec()), Ok(b"20".to_vec()));

    client2.set(b"3".to_vec(), b"30".to_vec());
    assert_eq!(client2.commit(), Ok(true));

    assert_eq!(client1.get(b"3".to_vec()), Ok(Vec::new()));
}

#[test]
// https://github.com/ept/hermitage/blob/master/sqlserver.md#read-skew-g-single
fn test_read_skew_write_predicate() {
    let (_, clients, _) = init(3);

    let mut client0 = clients[0].to_owned();
    client0.begin();
    client0.set(b"1".to_vec(), b"10".to_vec());
    client0.set(b"2".to_vec(), b"20".to_vec());
    assert_eq!(client0.commit(), Ok(true));

    let mut client1 = clients[1].to_owned();
    client1.begin();

    let mut client2 = clients[2].to_owned();
    client2.begin();

    assert_eq!(client1.get(b"1".to_vec()), Ok(b"10".to_vec()));
    assert_eq!(client2.get(b"1".to_vec()), Ok(b"10".to_vec()));
    assert_eq!(client2.get(b"2".to_vec()), Ok(b"20".to_vec()));

    client2.set(b"1".to_vec(), b"12".to_vec());
    client2.set(b"2".to_vec(), b"18".to_vec());
    assert_eq!(client2.commit(), Ok(true));

    client1.set(b"2".to_vec(), b"30".to_vec());
    assert_eq!(client1.commit(), Ok(false));
}

#[test]
// https://github.com/ept/hermitage/blob/master/sqlserver.md#write-skew-g2-item
fn test_write_skew() {
    let (_, clients, _) = init(3);

    let mut client0 = clients[0].to_owned();
    client0.begin();
    client0.set(b"1".to_vec(), b"10".to_vec());
    client0.set(b"2".to_vec(), b"20".to_vec());
    assert_eq!(client0.commit(), Ok(true));

    let mut client1 = clients[1].to_owned();
    client1.begin();

    let mut client2 = clients[2].to_owned();
    client2.begin();

    assert_eq!(client1.get(b"1".to_vec()), Ok(b"10".to_vec()));
    assert_eq!(client1.get(b"2".to_vec()), Ok(b"20".to_vec()));
    assert_eq!(client2.get(b"1".to_vec()), Ok(b"10".to_vec()));
    assert_eq!(client2.get(b"2".to_vec()), Ok(b"20".to_vec()));

    client1.set(b"1".to_vec(), b"11".to_vec());
    client2.set(b"2".to_vec(), b"21".to_vec());

    assert_eq!(client1.commit(), Ok(true));
    assert_eq!(client2.commit(), Ok(true));
}

#[test]
// https://github.com/ept/hermitage/blob/master/sqlserver.md#anti-dependency-cycles-g2
fn test_anti_dependency_cycles() {
    let (_, clients, _) = init(4);

    let mut client0 = clients[0].to_owned();
    client0.begin();
    client0.set(b"1".to_vec(), b"10".to_vec());
    client0.set(b"2".to_vec(), b"20".to_vec());
    assert_eq!(client0.commit(), Ok(true));

    let mut client1 = clients[1].to_owned();
    client1.begin();

    let mut client2 = clients[2].to_owned();
    client2.begin();

    client1.set(b"3".to_vec(), b"30".to_vec());
    client2.set(b"4".to_vec(), b"42".to_vec());

    assert_eq!(client1.commit(), Ok(true));
    assert_eq!(client2.commit(), Ok(true));

    let mut client3 = clients[3].to_owned();
    client3.begin();
    assert_eq!(client3.get(b"3".to_vec()), Ok(b"30".to_vec()));
    assert_eq!(client3.get(b"4".to_vec()), Ok(b"42".to_vec()));
}

#[test]
fn test_commit_primary_drop_secondary_requests() {
    let (_, clients, hook) = init(2);

    let mut client0 = clients[0].to_owned();
    client0.begin();
    client0.set(b"3".to_vec(), b"30".to_vec());
    client0.set(b"4".to_vec(), b"40".to_vec());
    client0.set(b"5".to_vec(), b"50".to_vec());
    hook.drop_req.store(true, Ordering::Relaxed);
    assert_eq!(client0.commit(), Ok(true));

    let mut client1 = clients[1].to_owned();
    client1.begin();
    assert_eq!(client1.get(b"3".to_vec()).unwrap(), b"30");
    assert_eq!(client1.get(b"4".to_vec()).unwrap(), b"40");
    assert_eq!(client1.get(b"5".to_vec()).unwrap(), b"50");
}

#[test]
fn test_commit_primary_success() {
    let (_, clients, hook) = init(2);

    let mut client0 = clients[0].to_owned();
    client0.begin();
    client0.set(b"3".to_vec(), b"30".to_vec());
    client0.set(b"4".to_vec(), b"40".to_vec());
    client0.set(b"5".to_vec(), b"50".to_vec());
    hook.drop_req.store(true, Ordering::Relaxed);
    assert_eq!(client0.commit(), Ok(true));

    let mut client1 = clients[1].to_owned();
    client1.begin();
    assert_eq!(client1.get(b"3".to_vec()).unwrap(), b"30");
    assert_eq!(client1.get(b"4".to_vec()).unwrap(), b"40");
    assert_eq!(client1.get(b"5".to_vec()).unwrap(), b"50");
}

#[test]
fn test_commit_primary_success_without_response() {
    let (_, clients, hook) = init(2);

    let mut client0 = clients[0].to_owned();
    client0.begin();
    client0.set(b"3".to_vec(), b"30".to_vec());
    client0.set(b"4".to_vec(), b"40".to_vec());
    client0.set(b"5".to_vec(), b"50".to_vec());
    hook.drop_resp.store(true, Ordering::Relaxed);
    assert_eq!(client0.commit(), Err(Error::Other("resphook".to_owned())));

    let mut client1 = clients[1].to_owned();
    client1.begin();
    assert_eq!(client1.get(b"3".to_vec()).unwrap(), b"30");
    assert_eq!(client1.get(b"4".to_vec()).unwrap(), b"40");
    assert_eq!(client1.get(b"5".to_vec()).unwrap(), b"50");
}

#[test]
fn test_commit_primary_fail() {
    let (_, clients, hook) = init(2);

    let mut client0 = clients[0].to_owned();
    client0.begin();
    client0.set(b"3".to_vec(), b"30".to_vec());
    client0.set(b"4".to_vec(), b"40".to_vec());
    client0.set(b"5".to_vec(), b"50".to_vec());
    hook.drop_req.store(true, Ordering::Relaxed);
    hook.fail_primary.store(true, Ordering::Relaxed);
    assert_eq!(client0.commit(), Ok(false));

    let mut client1 = clients[1].to_owned();
    client1.begin();
    assert_eq!(client1.get(b"3".to_vec()), Ok(Vec::new()));
    assert_eq!(client1.get(b"4".to_vec()), Ok(Vec::new()));
    assert_eq!(client1.get(b"5".to_vec()), Ok(Vec::new()));
}
