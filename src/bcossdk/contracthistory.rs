#![allow(
    clippy::unreadable_literal,
    clippy::upper_case_acronyms,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    overflowing_literals,
    unused_variables,
    unused_assignments
)]

use std::path::Path;

use serde_derive::{Deserialize, Serialize};
use toml;
use toml::value::*;

use crate::bcossdk::kisserror::{KissErrKind, KissError};
use crate::bcossdk::{fileutils, liteutils};

//use chrono::format::{DelayedFormat, StrftimeItems};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ContractHistory {
    address: Map<String, Value>,
    history: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ContractRecord {
    pub address: String,
    pub name: String,
    pub timestamp: String,
    pub blocknum: u32,
}
impl ContractRecord {
    pub fn decord(address: &str, raw: &str) -> ContractRecord {
        //0x1269231e2fee34a9b117d47a347eaecea40babcf = 'HelloWorld;2021-07-19 21:57:01;54'
        let sp: Vec<&str> = raw.split(";").collect();
        let name = sp.get(0).unwrap();
        ContractRecord {
            address: address.to_string(),
            name: sp.get(0).unwrap().to_string(),
            timestamp: sp.get(1).unwrap().to_string(),
            blocknum: u32::from_str_radix(sp[2], 10).unwrap(),
        }
    }
    pub fn encode(&self, withaddress: bool) -> String {
        if withaddress {
            return format!(
                "{};{};{};{}",
                self.name, self.timestamp, self.blocknum, self.address
            );
        } else {
            return format!("{};{};{}", self.name, self.timestamp, self.blocknum);
        }
    }

    pub fn make_record_msg(contract_name: &str, address: &str, blocknum: u32) -> String {
        let str_datetime = liteutils::datetime_str();
        let record = ContractRecord {
            address: address.to_string(),
            name: contract_name.to_string(),
            timestamp: str_datetime,
            blocknum: blocknum,
        };
        return record.encode(false);
    }
}

impl ContractHistory {
    pub fn default_file_name(path: &str) -> String {
        let chfile = format!("{}/contracthistory.toml", path);
        chfile
    }

    pub fn load_from_path(path: &str) -> Result<ContractHistory, KissError> {
        let file = ContractHistory::default_file_name(path);
        ContractHistory::load(file.as_str())
    }

    pub fn load(history_file: &str) -> Result<ContractHistory, KissError> {
        let loadres = fileutils::readstring(history_file)?;
        let res = toml::from_str(loadres.as_str());
        match res {
            Ok(history) => Ok(history),
            Err(e) => {
                kisserr!(
                    KissErrKind::EFormat,
                    "contract history wrong format {}",
                    history_file
                )
            }
        }
    }
    pub fn add(&mut self, contract_name: &str, address: &str, blocknum: u32) {
        self.address
            .insert(contract_name.to_string(), Value::from(address));
        self.add_history_record(contract_name, address, blocknum);
    }

    pub fn add_history_record(&mut self, contract_name: &str, address: &str, blocknum: u32) {
        let str_datetime = liteutils::datetime_str();
        let msg = ContractRecord::make_record_msg(contract_name, address, blocknum);
        self.history.insert(address.to_string(), Value::from(msg));
    }

    pub fn save(&self, filename: &str) -> Result<(), KissError> {
        let result = toml::to_string_pretty(self);
        match result {
            Ok(content) => fileutils::writestring(filename, content),
            Err(e) => {
                kisserr!(KissErrKind::EFormat, "format error")
            }
        }
    }

    pub fn getlast(&self, contract_name: &str) -> Result<String, KissError> {
        if self.address.contains_key(contract_name) {
            let v = self.address.get(contract_name).unwrap().as_str().unwrap();
            //v = v.trim_start_matches("\"").to_string();
            //v = v.trim_end_matches("\"").to_string();
            //printlnex!("get last address is [{}]",v);
            Ok(v.to_string())
        } else {
            kisserr!(
                KissErrKind::Error,
                "contract latest history not found {}",
                contract_name
            )
        }
    }
    pub fn find_record_by_address(&self, address: &str) -> Result<ContractRecord, KissError> {
        printlnex!("history :{:?}", self);
        if self.history.contains_key(address) {
            let v = self
                .history
                .get(address)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            //printlnex!("found contract name {} for address {}",&name,address);
            let r = ContractRecord::decord(address, v.as_str());
            return Ok(r);
        }
        kisserr!(
            KissErrKind::EFormat,
            "contract history not found {}",
            address
        )
    }
    pub fn history_file(path: &str) -> String {
        let chfile = format!("{}/contracthistory.toml", path);
        return chfile;
    }

    pub fn save_to_file(
        history_file: &str,
        contract_name: &str,
        addr: &str,
        blocknum: u32,
    ) -> Result<(), KissError> {
        let mut ch = ContractHistory::default();
        let p = Path::new(history_file);
        if p.exists() {
            ch = ContractHistory::load(history_file)?;
        }
        ch.add(contract_name, addr, blocknum);
        ch.save(history_file)
    }
    pub fn get_last_from_file(
        history_file: &str,
        contract_name: &str,
    ) -> Result<String, KissError> {
        let ch = ContractHistory::load(history_file)?;
        ch.getlast(contract_name)
    }

    pub fn check_address(contract_name: &str, addressinput: &str) -> Result<String, KissError> {
        ContractHistory::check_address_from_file(
            "contracts/contract.toml",
            contract_name,
            addressinput,
        )
    }
    ///如果输入的地址是last | latest 则去指定文件里,按指定的合约名寻找最新的地址, 如果是合格地址（todo），则直接返回
    pub fn check_address_from_file(
        fullfilepath: &str,
        contract_name: &str,
        addressinput: &str,
    ) -> Result<String, KissError> {
        match addressinput {
            "last" | "latest" => {
                let addr = ContractHistory::get_last_from_file(fullfilepath, contract_name)?;
                printlnex!("get from history addr is [{}]", &addr);
                return Ok(addr);
            }
            _ => return Ok(addressinput.to_string()),
        };
    }
}

pub fn test_toml() {
    let start = time::now();
    let history_name = "contract.toml";
    let mut ch = ContractHistory::load(history_name).unwrap();
    println!("{:?}", ch);
    let addr = ch.getlast("HelloWorld").unwrap();
    println!("get by name {}", addr);
    ch.add("a", "0xabcdefg", 0);
    ch.add("HelloWorld", "0xef1234567890", 99);
    //let res = ch.save(history_name);
    let end = time::now();
    let stamp = end - start;
    println!("using {}", stamp.num_milliseconds())
}
