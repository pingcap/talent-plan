use structopt::clap::{AppSettings, Arg, SubCommand};
use kvs::KvStore;
use structopt::clap;
// use structopt::StructOpt;
/*
* env就是打印Cargo.toml里面的各种配置
*/
// #[derive(StructOpt, Debug)]
// #[structopt(name = "basic")]
// struct Opt{
//     opt:String,
//     key:String,
//     value:String
// }


fn main() {
    let matches = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        //关闭help子命令
        .setting(AppSettings::DisableHelpSubcommand)
        //处理不存在的子命令并优雅的退出(无参数的时候也算)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        //关闭-v或者--version
        .setting(AppSettings::VersionlessSubcommands)
        //这是里自定义的各种子命令
        .subcommand(
            SubCommand::with_name("set")
                //子命令描述
                .about("Set the value of a string key to a string")
                //参数名,参数的help text,是否必填
                .arg(Arg::with_name("KEY").help("A string key").required(true))
                .arg(
                    Arg::with_name("VALUE")
                        .help("The string value of the key")
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get the string value of a given string key")
                .arg(Arg::with_name("KEY").help("A string key").required(true)),
        )
        .subcommand(
            SubCommand::with_name("rm")
                .about("Remove a given key")
                .arg(Arg::with_name("KEY").help("A string key").required(true)),
        )
        .get_matches();
    //匹配到具体的命令应该怎么处理,目前没有任何处理。直接退出
    let mut store = KvStore::new();
    match matches.subcommand() {
        ("set", Some(matches)) => {
            let key = matches.value_of("KEY").unwrap();
            let value = matches.value_of("VALUE").unwrap();
            store.set(key.to_string(), value.to_string());
            println!("{},{}",key,value);
        }
        ("get", Some(matches)) => {
            let key = matches.value_of("KEY").unwrap();
            let value=store.get(key.to_string()).unwrap();
            println!("{}",value);
        }
        ("rm", Some(matches)) => {
            let key = matches.value_of("KEY").unwrap();
            store.remove(key.to_string());
        }
        //没匹配到的执行这个处理
        _ => unreachable!(),
    }
}
