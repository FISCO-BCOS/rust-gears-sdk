use crate::bcossdk::kisserror::KissError;
use crate::bcossdk::bcosclientconfig::ClientConfig;
use structopt::StructOpt;
#[derive(StructOpt,Debug)]
#[structopt(about = "Fisco Bcos rust sdk console")]
pub struct Cli {
     /// 操作指令字，如 usage,deploy，sendtx，call，account，getXXX等.
     ///
     /// 输入 usage account/contract/get/all 查看对应的指令列表
     ///
     ///
     pub cmd: String,
     ///
     /// 当前操作的参数,根据操作命令字的不同会有所变化
     //#[structopt(parse(from_os_str))]
    pub params : Vec<String>,
    ///-c 配置文件，全路径如-c conf/config.toml
    #[structopt(short = "c", long = "config") ]
    pub configfile : Option<String>,
    ///-n 显式的指定合约名，不用带后缀，如"HelloWorld"
    #[structopt(short = "n", long = "contractname")]
    pub contractname : Option<String>,
    ///-v -vv -vvv...打开详细的打印
    #[structopt(short = "v",parse(from_occurrences))]
    pub verbos : u32,
}

impl  Cli{
    pub fn default_configfile(&self)->String{
        let configfile = match &self.configfile{
            Option::None =>{"conf/config.toml"},
            Some(f)=>{f.as_str()}
        };
        configfile.to_string()
    }
    pub fn default_config(&self)->Result<ClientConfig,KissError>{
        let configfile =self.default_configfile();
        ClientConfig::load(configfile.as_str())
    }

}
