use crate::Cli;
use crate::bcossdk::bcosclientconfig::ClientConfig;
use std::path::PathBuf;

pub fn usage_account(config:&ClientConfig){
    println!("\n--Account:账户相关的命令--");
    let msg=r###"
    account new [名字]，创建新的账户,名字可选，用于保存时的文件名,如未指定，则用地址（address）作为文件名

    account show [名字]，显示指定名字的账户信息，如未指定名字，则展示配置文件指定目录下所有的账户（.pem）信息

    写入和寻找账户文件的路径与配置文件同级。"###;
    let mut p = PathBuf::from(&config.configfile.as_ref().unwrap().as_str());
    p.pop();
    print!("{}",msg);
    println!("当前账户文件目录:{}",p.to_str().unwrap());
}
pub fn usage_contract(config:&ClientConfig){
    println!("\n--Contract:合约相关的命令--");
    let msg=r###"
    deploy [合约名] [合约构造的初始化参数...], 如 deploy HelloWorld [参数1] [参数2]

    sendtx [合约名] [地址或latest/last] [方法名] [方法对应的参数...], 如 sendtx HelloWorld latest  set "hello"

    call   [合约名] [地址或latest/last] [方法名] [方法对应的参数...], 如 call HelloWorld latest  get

    compile [合约名]  调用配置好的solc编译器，编译合约，默认合约sol文件和输出都在配置的contracts目录，solc下载参见contrats目录下的README（注：用deploy指令部署合约时，会先尝试编译）

    合约成功部署后，新地址会写入合约目录的contracthistory.toml文件，后续就可以用lastest/last代替地址调用了

    写入历史和寻找合约ABI文件的路径以配置文件里的[contract]contractpath=项为准。
    "###;

    println!("{}\n合约当前的路径:{}\n",msg,config.contract.contractpath);
}

pub fn usage_get(config:&ClientConfig){
    println!("\n--Get:查询类命令--");
    println!("--查询类指令包含在全部RPC接口指令里，常用的指令如下--");
    let  msg = r###"
    节点类-->
    getBlockNumber，getClientVersion，getNodeInfo
    getPeers，getPbtView，getSealList，getObserverList， getSyncStatus
    getNodeIDList，getGroupList，getGroupPeers(groupid)
    getSystemConfigByKey(key）

    区块类-->
    getBlockByHash(hash,bool),getBlockByNumber(number,bool),
    getBlockHeaderByHash(hash,bool),getBlockHashByNumber(number)

    交易类-->
    getTransactionByHash(hash), getTransactionReceipt(hash)
    getTransactionByBlockHashAndIndex(blockhash,index)
    getTransactionByBlockNumberAndIndex(blocknumber,index)
    getPendingTransactions，getTotalTransactionCount
    getBatchReceiptsByBlockNumberAndRange(blocknumber,from,count,compressflag)
    getBatchReceiptsByBlockHashAndRange(blockhash,from,count,compressflag)

    群组操作类(:todo)->
    generateGroup,startGroup,stopGroup,removeGroup,recoverGroup,queryGroupStatus
    "###;
    println!("{}",msg);
    println!("全部指令参见url：https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html")
}
pub fn usage_all(config:&ClientConfig){
    println!("--所有命令--");
    println!("1)");
    usage_account(&config);
    println!("2)");
    usage_contract(&config);
    println!("3)");
    usage_get(&config);
}


pub fn usage(cli:&Cli){
    let mut catagory:String = "all".to_string();
    if cli.params.len()>=1
    {
        catagory = cli.params[0].clone().to_lowercase();
    }

    let configfile = match &cli.configfile{
        Option::None =>{"conf/config.toml"},
        Some(f)=>{f.as_str()}
    };
    println!("当前配置文件路径:{}",configfile);
    let configres = ClientConfig::load(configfile);
    let config = match configres {
        Ok(c)=>{c},
        Err(e)=>{
            println!("-->未加载配置文件");
            println!("请确认配置文件存在 {},或用-c选项指定特定目录下的配置文件",configfile);
            return
        }
    };

    match catagory.as_str(){
        "account"=>{
           usage_account(&config);
            return;
        },
        "contract"=>{
            usage_contract(&config);
            return;
        },
        "get"=>{
            usage_get(&config);
            return;
        },
        "all"=>{
            usage_all(&config);
            return;
        },
        _=>{
            println!("\n\n输入： usage account / contract / get / all");
        }
    }
}