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

pub mod bcos2sdk;
pub mod bcos3sdk;
pub mod bcossdkutil;

#[cfg(test)]
mod tests {
    //set RUST_TEST_NOCAPTURE=1
    use crate::bcos2sdk::bcos2client::Bcos2Client;
    use crate::bcossdkutil;
    use crate::bcossdkutil::kisserror::KissError;
    use fisco_bcos_rust_gears_sdk::console::cli_common::Cli;
    use structopt::StructOpt;

    // use super::*;
    pub fn default_config(params: &Vec<String>) -> String {
        if params.len() >= 1 {
            return params[0].clone();
        } else {
            return "conf/config.toml".to_string();
        }
    }

    #[test]
    fn lib_test_init() -> Result<(), KissError> {
        let cli: Cli = Cli::from_args();
        println!("cli : {:?}", cli);
        let configfile = default_config(&cli.params);
        let bcossdk = Bcos2Client::new_from_config(configfile.as_str())?;
        println!("bcossdkutil : {:?}", bcossdk.to_summary());
        assert!(bcossdk.account.privkey.len() > 0);
        Ok(())
    }

    #[test]
    fn lib_test_getNodeVersion() -> Result<(), KissError> {
        let cli: Cli = Cli::from_args();
        println!("cli : {:?}", cli);
        bcossdkutil::macrodef::set_debugprint(true);
        let configfile = default_config(&cli.params);
        let mut bcossdk = Bcos2Client::new_from_config(configfile.as_str())?;
        println!("start to getNodeVersion");
        let result = bcossdk.getNodeVersion()?;
        println!("lib_test_getNodeVersion:  {:?}", result);
        assert!(result["FISCO-BCOS Version"].as_str() != None);
        Ok(())
    }
}
