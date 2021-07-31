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

pub mod bcossdk;


#[cfg(test)]
mod tests {
//set RUST_TEST_NOCAPTURE=1
    use crate::bcossdk::kisserror::{ KissError};
    use crate::bcossdk::bcossdk::BcosSDK;
    use crate::bcossdk::cli_common::Cli;
    use structopt::StructOpt;
    use crate::bcossdk;

    // use super::*;
    pub fn default_config(params:&Vec<String>)->String
   {
       if params.len() >= 1 {
           return params[0].clone();
       }
       else{
           return "conf/config.toml".to_string();
        }
   }

    #[test]
    fn lib_test_init() ->Result<(),KissError>{
        let cli:Cli = Cli::from_args();
        println!("cli : {:?}",cli);
        let configfile = default_config(&cli.params);
        let bcossdk = BcosSDK::new_from_config(configfile.as_str())?;
        println!("bcossdk : {:?}", bcossdk.to_summary());
        assert!(bcossdk.account.privkey.len()>0);
        Ok(())
    }

    #[test]
    fn lib_test_getNodeVersion()-> Result<(),KissError>{
        let cli:Cli = Cli::from_args();
        println!("cli : {:?}",cli);
        bcossdk::macrodef::set_debugprint(true);
        let configfile = default_config(&cli.params);
        let mut bcossdk = BcosSDK::new_from_config(configfile.as_str())?;
        println!("start to getNodeVersion");
        let result = bcossdk.getNodeVersion()?;
        println!("lib_test_getNodeVersion:  {:?}",result);
        assert!( result["FISCO-BCOS Version"].as_str()!=None);
        Ok(())
    }
}