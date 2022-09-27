use std::collections::HashMap;
use std::str::FromStr;
use std::stringify;

use serde_json::Value as JsonValue;

use fisco_bcos_rust_gears_sdk::bcossdk::kisserror::{KissErrKind, KissError};

use crate::{Cli, kisserr};

///--------------在这个文件里聚合一下查询接口,用简单的公共方法提供----------------------------------------
///方法命令刻意和https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html
///保持大小写和拼写一致，以便查找，局部不遵循rust命令规范

type CMD_FUNCS = fn(&Cli) -> Result<(), KissError>;
#[macro_export]
macro_rules! cmdmap {
            ($m:expr,$x:ident) => {
                $m.insert(stringify!($x).trim_start_matches("").to_lowercase() ,($x) )
            };
}

#[derive(Default, Clone)]
pub struct CliCmdMap {
    pub name: String,
    pub cmd_func_map: HashMap<String, CMD_FUNCS>,
}

impl CliCmdMap {
    pub fn new(name_: &str) -> Self {
        CliCmdMap {
            name: name_.to_string(),
            cmd_func_map: HashMap::new(),
        }
    }
}

impl CliCmdMap {
    pub fn in_cmd(&self, cmd: &str) -> bool {
        let seekkey = cmd.to_lowercase();
        return self.cmd_func_map.contains_key(seekkey.as_str());
    }

    pub fn handle_cmd(self, cli: &Cli) -> Result<(), KissError>
    {
        //println!("BcosSDK: {}", bcossdk.to_summary());
        let cmd = &cli.cmd;
        let seekkey = cli.cmd.to_lowercase();
        if !self.in_cmd(seekkey.as_str()) {
            println!("cmd [ {} ]not implement yet.Valid cmds: ", cmd);
            self.print_cmds(true);
            return kisserr!(KissErrKind::Error,"cmd {} not implement yet ",cmd);
        }
        let func = self.cmd_func_map.get(seekkey.as_str()).unwrap();
        //println!("\n{} -->",cli.cmd.as_str());
        func(&cli)?;
        Ok(())
    }

    pub fn print_cmds(&self,crlf:bool)
    {
        let mut i = 1;
        println!("CMDMAP: {}",self.name);
        for (k, v) in self.cmd_func_map.iter() {
            print!("\t{:02})->\t{}", i, k);
            if crlf {
                println!("");
            }else if i%4==0 {println!("")}
            i += 1;
        }
        println!("");
    }
}



