use fisco_bcos_rust_gears_sdk::bcossdk::kisserror::KissError;
use fisco_bcos_rust_gears_sdk::bcossdk::solcompile::sol_compile;
use crate::{Cli, kisserr};

pub fn console_compile(cli:&Cli) ->Result<(),KissError> {

    //let config = ClientConfig::load(cli.default_configfile().as_str())?;
    let contract_name = cli.params[0].clone();
    let outputres = sol_compile(contract_name.as_str(),cli.default_configfile().as_str());
    //println!("compile [{}] done。",contract_name);
    match outputres
    {
        Ok(output)=>{

            println!("compiler  status : {}",output.status);
            if output.stdout.len() > 0{
                println!("stdout: {}",String::from_utf8(output.stdout).unwrap());
            }
            if output.stderr.len() > 0{
                println!("stderr: {}",String::from_utf8(output.stderr).unwrap());
            }
        }
        Err(e)=>{
            println!("compile error : {:?}",e);
            return Err(e);
        }
    }

    Ok(())
}