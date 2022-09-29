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

use std::collections::HashMap;
use std::path::Path;

use serde_derive::{Deserialize, Serialize};
use toml;

use crate::bcossdkutil::kisserror::{KissErrKind, KissError};
use crate::bcossdkutil::{fileutils, liteutils};

//use chrono::format::{DelayedFormat, StrftimeItems};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ContractHistory {
    address: HashMap<String, HashMap<String, String>>,
    history: HashMap<String, HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ContractRecord {
    pub address: String,
    pub name: String,
    pub timestamp: String,
    pub blocknum: u64,
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
            blocknum: u64::from_str_radix(sp[2], 10).unwrap(),
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

    pub fn make_record_msg(contract_name: &str, address: &str, blocknum: u64) -> String {
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
    pub fn add(&mut self, segment: &str, contract_name: &str, address: &str, blocknum: u64) {
        //先找到tag的指定段

        let mut segment_map: HashMap<String, String> = HashMap::new();
        if self.address.contains_key(segment) {
            segment_map = self.address.get(segment).unwrap().clone();
        }
        segment_map.insert(contract_name.to_string(), address.to_string());
        self.address.insert(segment.to_string(), segment_map);
        self.add_history_record(segment, contract_name, address, blocknum);
    }

    pub fn add_history_record(
        &mut self,
        segment: &str,
        contract_name: &str,
        address: &str,
        blocknum: u64,
    ) {
        let str_datetime = liteutils::datetime_str();
        let msg = ContractRecord::make_record_msg(contract_name, address, blocknum);
        let mut segment_map: HashMap<String, String> = HashMap::new();
        if self.history.contains_key(segment) {
            segment_map = self.history.get(segment).unwrap().clone();
        }
        segment_map.insert(address.to_string(), msg.to_string());
        self.history.insert(segment.to_string(), segment_map);
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

    pub fn getlast(&self, segment: &str, contract_name: &str) -> Result<String, KissError> {
        if self.address.contains_key(segment) {
            let segment_map = self.address.get(segment).unwrap().clone();
            if segment_map.contains_key(contract_name) {
                let v = segment_map.get(contract_name).unwrap();
                return Ok(v.to_string());
            }
        }
        return kisserr!(
            KissErrKind::Error,
            "contract latest history not found {}",
            contract_name
        );
    }
    pub fn find_record_by_address(
        &self,
        segment: &str,
        address: &str,
    ) -> Result<ContractRecord, KissError> {
        printlnex!("history :{:?}", self);
        if self.history.contains_key(segment) {
            let segment_map = self.history.get(segment).unwrap().clone();
            if segment_map.contains_key(address) {
                let v = segment_map.get(address).unwrap();
                //printlnex!("found contract name {} for address {}",&name,address);
                let r = ContractRecord::decord(address, v.as_str());
                return Ok(r);
            }
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
        segment: &str,
        contract_name: &str,
        addr: &str,
        blocknum: u64,
    ) -> Result<(), KissError> {
        let mut ch = ContractHistory::default();
        let p = Path::new(history_file);
        if p.exists() {
            ch = ContractHistory::load(history_file)?;
        }
        ch.add(segment, contract_name, addr, blocknum);
        ch.save(history_file)
    }
    pub fn get_last_from_file(
        history_file: &str,
        segment: &str,
        contract_name: &str,
    ) -> Result<String, KissError> {
        let ch = ContractHistory::load(history_file)?;
        ch.getlast(segment, contract_name)
    }

    ///如果输入的地址是last | latest 则去指定文件里,按指定的合约名寻找最新的地址, 如果是合格地址（todo），则直接返回
    pub fn check_address_from_file(
        fullfilepath: &str,
        segment: &str,
        contract_name: &str,
        addressinput: &str,
    ) -> Result<String, KissError> {
        match addressinput {
            "last" | "latest" => {
                let addr =
                    ContractHistory::get_last_from_file(fullfilepath, segment, contract_name)?;
                printlnex!("get from history addr is [{}]", &addr);
                return Ok(addr);
            }
            _ => return Ok(addressinput.to_string()),
        };
    }
}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct GroupData {
    pub name: HashMap<String, HashMap<String, String>>,
}

pub fn test_multi() {
    //let mut gd:HashMap<String,HashMap<String,String>> = HashMap::new();
    let mut gd = GroupData {
        name: HashMap::new(),
    };
    let mut gd1: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut car: HashMap<String, String> = HashMap::new();
    car.insert("c260".to_string(), "benz".to_string());
    car.insert("a3".to_string(), "audi".to_string());
    car.insert("x3".to_string(), "bmw".to_string());

    let mut fruit: HashMap<String, String> = HashMap::new();
    fruit.insert("apple".to_string(), "red".to_string());
    fruit.insert("banana".to_string(), "yellow".to_string());

    gd.name.insert("carbrand".to_string(), car.clone());
    gd.name.insert("fruitcolor".to_string(), fruit.clone());

    gd1.insert("carbrand".to_string(), car.clone());
    gd1.insert("fruitcolor".to_string(), fruit.clone());

    let res = toml::to_string_pretty(&gd1).unwrap();
    println!("{}", res);

    let mut v: HashMap<String, HashMap<String, String>> = toml::from_str(res.as_str()).unwrap();
    println!("from toml:{:?}", v);
    let mut car = v.get("carbrand").unwrap().clone();
    car.insert("x3".to_string(), "bwm_2022".to_string());
    v.insert("carbrand".to_string(), car.clone());
    println!("after change {}", toml::to_string_pretty(&v).unwrap());
}

pub fn test_toml() {
    //return test_multi();

    let history_name = "contracts/contracthistory1.toml";
    let mut ch = ContractHistory::load(history_name).unwrap();
    println!("{:?}", ch);
    let addr = ch.getlast("seg2", "HelloWorld").unwrap();
    println!("get by name {}", addr);
    ch.add("seg1", "a", "0xabcdefg", 0);
    ch.add("seg2", "HelloWorld1", "0xef1234567890", 99);
    let res = ch.save(history_name);

    println!("getlast {:?}", ch.getlast("seg2", "HelloWorld"));
    println!(
        "find by address {:?}",
        ch.find_record_by_address("seg2", "0x02687d278477a84328446e580f79cfb12ab219e4")
    );
}
