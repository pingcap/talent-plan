use clap::arg_enum;
use kvs::*;
use log::LevelFilter;
use log::{error, info, warn};
use std::env::current_dir;
use std::fs;
use std::net::SocketAddr;
use std::process::exit;
use structopt::StructOpt;

const DEFAULT_LISTENING_ADDRESS: &str = "127.0.0.1:4000";
const DEFAULT_ENGINE: Engine = Engine::kvs;

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
        raw(possible_values = "&Engine::variants()")
    )]
    engine: Option<Engine>,
}

arg_enum! {
    #[allow(non_camel_case_types)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    enum Engine {
        kvs,
        sled
    }
}

fn main() {
    //这里应该是设置一个日志过滤级别
    env_logger::builder().filter_level(LevelFilter::Info).init();
    //structopt获取命令行参数并映射到具体的结构体上
    let mut opt = Opt::from_args();
    let res = current_engine().and_then(move |curr_engine| {
        if opt.engine.is_none() {
            opt.engine = curr_engine;
        }
        //之前使用的引擎与新定的引擎不同以非0代码退出.
        if curr_engine.is_some() && opt.engine != curr_engine {
            error!("Wrong engine!");
            exit(1);
        }
        //以当前配置执行
        run(opt)
    });
    if let Err(e) = res {
        error!("{}", e);
        exit(1);
    }
}

fn run(opt: Opt) -> Result<()> {
    let engine = opt.engine.unwrap_or(DEFAULT_ENGINE);
    info!("kvs-server {}", env!("CARGO_PKG_VERSION"));
    info!("Storage engine: {}", engine);
    info!("Listening on {}", opt.addr);

    // write engine to engine file
    fs::write(current_dir()?.join("engine"), format!("{}", engine))?;
    //选择对应的引擎开始执行
    match engine {
        //这里的KvStore与project2中并无区别
        Engine::kvs => run_with_engine(KvStore::open(current_dir()?)?, opt.addr),
        Engine::sled => run_with_engine(SledKvsEngine::new(sled::open(current_dir()?)?), opt.addr),
    }
}

fn run_with_engine<E: KvsEngine>(engine: E, addr: SocketAddr) -> Result<()> {
    let server = KvsServer::new(engine);
    server.run(addr)
}

//获取当前在使用的引擎
fn current_engine() -> Result<Option<Engine>> {
    let engine = current_dir()?.join("engine");
    if !engine.exists() {
        return Ok(None);
    }

    match fs::read_to_string(engine)?.parse() {
        Ok(engine) => Ok(Some(engine)),
        Err(e) => {
            warn!("The content of engine file is invalid: {}", e);
            Ok(None)
        }
    }
}
