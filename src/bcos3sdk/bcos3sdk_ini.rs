use std::collections::HashMap;

use ini::Ini;

use crate::bcossdkutil::kisserror::{KissErrKind, KissError};
use crate::kisserr;
//用来读c sdk附带的ini文件，注意rust-ini模块要开启features = ["inline-comment"]
pub struct Bcos3sdkIni {
    pub values: toml::Value,
    pub peers: HashMap<String, String>,
}

impl Bcos3sdkIni {
    pub fn load(config_file: &str) -> Result<Bcos3sdkIni, KissError> {
        let mut sdkini = Bcos3sdkIni {
            values: toml::Value::String("".to_string()),
            peers: HashMap::new(),
        };
        let confres = Ini::load_from_file(config_file);
        if confres.is_err() {
            return kisserr!(KissErrKind::EFormat,"load config file error: {},{:?}",config_file,confres.err());
        }
        let config = confres.unwrap();
        let peersseg = config.section(Some("peers"));
        if !peersseg.is_none() {
            let peers = peersseg.unwrap();
            for (key, value) in peers.iter() {
                sdkini.peers.insert(key.to_string(), value.to_string());
            }
        }

        Ok(sdkini)
    }
}
