use clap::{App, AppSettings, Arg, SubCommand};
use kvs::{KvStore, Result};
use std::env::current_dir;

fn main() -> Result<()> {
    let server_addr_arg = Arg::with_name("addr")
        .help("Sets the server address")
        .long("addr")
        .value_name("IP:PORT")
        .default_value("127.0.0.1:4000");
    let matches = App::new("kvs-client")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(
            SubCommand::with_name("set")
                .about("Set the value of a string key to a string")
                .usage("kvs-client set <KEY> <VALUE> [--addr IP:PORT]")
                .arg(Arg::with_name("KEY").help("A string key").required(true))
                .arg(
                    Arg::with_name("VALUE")
                        .help("The string value of the key")
                        .required(true),
                )
                .arg(server_addr_arg.clone()),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get the string value of a given string key")
                .usage("kvs-client get <KEY> [--addr IP:PORT]")
                .arg(Arg::with_name("KEY").help("A string key").required(true))
                .arg(server_addr_arg),
        )
        .get_matches();

    match matches.subcommand() {
        ("set", Some(matches)) => {
            let key = matches.value_of("KEY").expect("KEY argument missing");
            let value = matches.value_of("VALUE").expect("VALUE argument missing");

            let mut store = KvStore::open(current_dir()?)?;
            store.set(key.to_string(), value.to_string())?;
        }
        ("get", Some(matches)) => {
            let key = matches.value_of("KEY").expect("KEY argument missing");

            let mut store = KvStore::open(current_dir()?)?;
            if let Some(value) = store.get(key.to_string())? {
                println!("{}", value);
            } else {
                println!("Key not found");
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}
