#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use kvs::{KvStore, KvsServer, Result};
use log::LevelFilter;
use std::env;
use std::net::SocketAddr;
use std::process::exit;
use structopt::StructOpt;

const DEFAULT_LISTENING_ADDRESS: &'static str = "127.0.0.1:4000";

#[derive(StructOpt, Debug)]
#[structopt(name = "kvs-server")]
struct Opt {
    #[structopt(
        long,
        help = "Sets the listening address",
        value_name = "IP:PORT",
        raw(default_value = "DEFAULT_LISTENING_ADDRESS"),
        parse(try_from_str)
    )]
    addr: SocketAddr,
    #[structopt(
        long,
        help = "Sets the storage engine",
        value_name = "ENGINE-NAME",
        default_value = "kvs",
        raw(possible_values = "&Engine::variants()")
    )]
    engine: Engine,
}

arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Debug)]
    enum Engine {
        kvs,
        sled
    }
}

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();
    let opt = Opt::from_args();
    if let Err(e) = run(opt) {
        error!("{}", e);
        exit(1);
    }
}

fn run(opt: Opt) -> Result<()> {
    info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    info!("Storage engine: {}", opt.engine);
    info!("Listening on {}", opt.addr);
    match opt.engine {
        Engine::kvs => {
            let engine = KvStore::open(env::current_dir()?)?;
            let server = KvsServer::new(engine);
            server.run(opt.addr)?;
        }
        Engine::sled => unimplemented!(),
    }
    Ok(())
}
