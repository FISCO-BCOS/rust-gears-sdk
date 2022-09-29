use std::path::{Path, PathBuf};
use std::process::{Command, Output};

//use log::info;

use crate::bcossdkutil::bcosclientconfig::{BcosCryptoKind, ClientConfig};
use crate::bcossdkutil::kisserror::{KissErrKind, KissError};

///编译合约。传入合约名字和配置文件路径
///因为不需要连接节点，纯本地运行，采用静态方法实现，避免加载各种库，也无需连接网络
pub fn sol_compile(contract_name: &str, configfile: &str) -> Result<Output, KissError> {
    let config = ClientConfig::load(configfile)?;
    let mut solc_path = match config.common.crypto {
        BcosCryptoKind::ECDSA => config.common.solc,
        BcosCryptoKind::GM => config.common.solcgm,
    };
    if cfg!(target_os = "windows") {
        solc_path = format!("{}.exe", solc_path);
    }

    if !Path::new(solc_path.as_str()).exists() {
        return kisserr!(
            KissErrKind::EFileMiss,
            "solc [{}] is not exists,check the solc setting in config file [{}]",
            solc_path,
            configfile
        );
    }

    let mut solfullpath = PathBuf::from(&config.common.contractpath);
    let options = ["--abi", "--bin", "--bin-runtime", "--overwrite", "--hashes"];

    solfullpath = solfullpath.join(format!("{}.sol", contract_name));
    //println!("solc fullpath {:?}",solfullpath);
    if !solfullpath.exists() {
        return kisserr!(KissErrKind::EFileMiss,"contract solfile [{}] is not exists,check the config setting in  [{}]->contractpath[{}]",
                solfullpath.to_str().unwrap(),configfile,config.common.contractpath);
    }
    println!(
        "compile sol  {} ,use solc {},outputdir:{} options: {:?} ",
        solfullpath.to_str().unwrap(),
        solc_path,
        config.common.contractpath.as_str(),
        options
    );
    let outputres = Command::new(solc_path)
        .args(&options)
        .arg("-o")
        .arg(config.common.contractpath.as_str())
        .arg(solfullpath.to_str().unwrap())
        .output();
    //println!("compile result : {:?}", &outputres);

    match outputres {
        Ok(out) => {
            if !out.status.success() {
                println!(
                    "compile command is NOT success! stderr:\n{}",
                    String::from_utf8(out.stderr).unwrap()
                );
                return kisserr!(
                    KissErrKind::Error,
                    "compile [{}] error status :{}",
                    contract_name,
                    out.status.to_string()
                );
            }
            return Ok(out);
        }
        Err(e) => {
            return kisserr!(
                KissErrKind::Error,
                "compile [{}] error :{:?}",
                contract_name,
                e
            );
        }
    }
}
