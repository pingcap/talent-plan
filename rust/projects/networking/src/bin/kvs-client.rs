use clap::{App, AppSettings, Arg, SubCommand};
use kvs::{KvStore, Result};
use std::env::current_dir;
use std::net::SocketAddr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "kvs-client",
    raw(global_settings = "&[\
                           AppSettings::DisableHelpSubcommand,\
                           AppSettings::SubcommandRequiredElseHelp,\
                           AppSettings::VersionlessSubcommands]")
)]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
    #[structopt(
        long,
        help = "Sets the server address",
        value_name = "IP:PORT",
        default_value = "127.0.0.1:4000",
        parse(try_from_str)
    )]
    addr: SocketAddr,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "get", about = "Get the string value of a given string key")]
    Get {
        #[structopt(name = "KEY", help = "A string key")]
        key: String,
    },
    #[structopt(name = "set", about = "Set the value of a string key to a string")]
    Set {
        #[structopt(name = "KEY", help = "A string key")]
        key: String,
        #[structopt(name = "VALUE", help = "The string value of the key")]
        value: String,
    },
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    println!("{:?}", opt);
    Ok(())
}
