#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use kvs::{KvStore, KvStoreEngine, KvsServer, Result};
use std::env;
use std::net::SocketAddr;
use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "kvs-server")]
struct Opt {
    #[structopt(
        long,
        help = "Sets the listening address",
        value_name = "IP:PORT",
        default_value = "127.0.0.1:4000",
        parse(try_from_str)
    )]
    addr: SocketAddr,
    #[structopt(
        long,
        help = "Sets the storage engine",
        value_name = "ENGINE-NAME",
        default_value = "kvs",
        raw(possible_values = "&Engine::variants()", case_insensitive = "true")
    )]
    engine: Engine,
}

arg_enum! {
    #[derive(Debug)]
    enum Engine {
        Kvs,
        Sled
    }
}

fn main() {
    env_logger::init();
    let opt = Opt::from_args();
    if let Err(e) = run(opt) {
        error!("{}", e);
        exit(1);
    }
}

fn run(opt: Opt) -> Result<()> {
    match opt.engine {
        Engine::Kvs => {
            let engine = KvStoreEngine::new(KvStore::open(env::current_dir()?)?);
            let server = KvsServer::new(engine);
            server.run(opt.addr)?;
        }
        Engine::Sled => unimplemented!(),
    }
    Ok(())
}
