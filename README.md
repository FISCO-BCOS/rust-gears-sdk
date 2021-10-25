# FISCO BCOS SDK : Rust Gears
--- 
Rust SDK for FISCO-BCOS  ,like some  rusted  but solid gears , help to build  blockchain application with FISCO-BCOS

[FISCO BCOS](https://github.com/FISCO-BCOS/FISCO-BCOS/tree/master)的轻量级版本Rust SDK, 基础网络、国密非国密算法支持、合约解析能力较为完备，附带命令行交互控制台。

- 本项目的定位是一个学习/研究/编程兴趣的项目，仅供学习参考。如有正式的使用需求，建议在熟悉rust语言的前提下，仅部分参考本项目和FISCO BCOS相关的实现，去开发自己的生产级sdk，并经过严谨测试验证后使用。
  
- 本项目并非唯一且官方的fisco bcos rust sdk，社区陆续会有其他优秀的rust的sdk实现，提供多种选择和参考可能性

## rustc环境

已经验证的rust版本 

rustc 1.54.0-nightly (ed597e7e1 2021-06-08) 

rustc 1.55.0-nightly (67b03007c 2021-07-23)

rust版本的本身更新较快，请按rust官网指引安装配置。


## crates.io引用方式
crates.io地址: (https://crates.io/crates/fisco-bcos-rust-gears-sdk)

在应用项目的Cargo.toml的dependencies增加引用: ```fisco-bcos-rust-gears-sdk = "0.3.141"``` (可按crates.io上实际已经发布的版本更新版本号)

调用示例:
```
use fisco_bcos_rust_gears_sdk::bcossdk::bcossdk::BcosSDK;

fn main() {
    //enable log
    log4rs::init_file("log4rs.yml", Default::default()).unwrap(); 
    //load config and init the bcossdk
    let mut bcossdk = BcosSDK::new_from_config("conf/config.toml").unwrap(); 
    //get node version,other apis see fisco-bcos document or sample
    let res = bcossdk.getNodeVersion();
    println!("res {:?}",res);
}
```


## 关键特性

概要:

全面支持RPC/Channel协议，国密和非国密算法，友好解析交易(Transaction)、回执(Receipt)、事件（Event）

有命令行控制台，支持创建和查询账户，以及合约类（部署调用），查询类的操作。

- FISCO BCOS 2.0 [JSON-RPC](https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html)的Rust API
- 支持json rpc的同步请求模式
- 支持国密和非国密的[Channel协议](https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/design/protocol_description.html#channelmessage)
- 支持交易的部署、发送交易、call 
- 支持解析解析功能：包括交易输入、交易输出、Event Log等ABI数据的拼装和解析。
- 支持基于pem文件的账户创建和私钥读取。
- 引用WeDPR的密码库进行HASH，签名, 该密码库支持ECDSA和SM2,3,4
- 控制台支持Struct参数，数组等复杂数据结构，SDK调用方法参见[src/sample/structdemo.rs](src/sample/structdemo.rs)
- 支持合约事件监听，参见[src/bcossdk/eventhandler.rs](src/bcossdk/eventhandler.rs)
- 客户端sdk支持多线程调用

## 目录结构：

- src/bcossdk : 主要的sdk代码，代码本身已经比较模块化，有注释，欢迎查阅
- src/console : 命令行控制台实现代码 
- src/sample: 陆续增加一些sample
- conf : 配置文件,运行前，先将client_config.toml.sample复制为client_config.toml,并仔细理解和修改里面的配置值，尤其是ip端口和证书
- sdk  : 连接节点的证书，国密和非国密的都放到这里
- contracts : 合约的sol，abi，bin等
- gm: 考虑到同时连接国密和非国密节点，推荐建立一个gm目录，包含以上的conf,sdk,contracts等目录结构，保存国密节点所需的配置、证书、私钥、合约定义和代码等
- log: 控制台运行后会自动生成log目录，保存日志文件。日志配置见log4rs，默认配置是输出滚动日志，可根据实际需要修改。
```
tips:调用sdk代码前，应保证调用过一次日志初始化语句： 
log4rs::init_file("log4rs.yml", Default::default()).unwrap();
```
  

## 配置文件
主要配置文件是 conf/config.toml，项目提供了conf/config.toml.sample,将其复制或去掉sample后缀即可。配置项解释：
```
[chain]  #链基础配置 
    chainid=1 #默认链id
    groupid=1 #默认组id
    crypto="ECDSA"  #链采用的密码学算法，ECDSA或GM
    accountpem="conf/client.pem" #客户端的默认账户私钥文件
    protocol="RPC"  # 客户端和节点的连接方式 CHANNEL或RPC

[contract]
contractpath="./contracts"  #合约的abi，bin，sol以及历史记录文件都在这个目录
solc = "./bin/solc"         #非国密版的solc编译器全路径
solcgm = "./bin/solc-gm"    #国密版本的solc编译器全路径

[rpc]
    url="http://127.0.0.1:8545" #rpc通信url，改为实际的服务器ip和端口
    timeout = 3  


[channel]
    ip = "127.0.0.1"  #channel协议连接的节点ip地址
    port = 20200      #节点channel端口
    tlskind = "ECDSA"  # channel协议采用的密码算法，ECDSA或GM
    timeout=10          
    nativelib_echo_mode = 0  #native库是否打印调试信息的配置
    cacert= "sdk/ca.crt"     #非国密的ca证书，共3个，从节点的node[x]/sdk目录下获取
    sdkcert = "sdk/sdk.crt"
    sdkkey = "sdk/sdk.key"
    gmcacert= "sdk/gmca.crt"  #从这里开始是国密的证书，5个，从节点的node[x]/sdk/gm目录下获取
    gmsdkcert = "sdk/gmsdk.crt"
    gmsdkkey = "sdk/gmsdk.key"
    gmensdkcert = "sdk/gmensdk.crt"
    gmensdkkey = "sdk/gmensdk.key"
```

## 控制台使用帮助：
```
cargo run -- --help       控制台本身的help(采用StructOpt库默认格式）

典型命令如 

cargo run --  deploy HelloWorld 

命令行选项：

    -v
            -v -vv -vvv...打开详细的打印

    -c, --config <configfile>
            -c 配置文件，全路径如-c conf/config.toml

    -n, --contractname <contractname>
            -n 显式的指定合约名，主要是供解析交易和回执时使用，不用带后缀，如"HelloWorld"


```
```
cargo run -- usage      bcossdk的操作命令字帮助，建议查看包括 usage account，usage contract，usage get或usage all

当前配置文件路径:conf/config.toml
--所有命令--
1)

--Account:账户相关的命令--

    account new [名字]，创建新的账户,名字可选，用于保存时的文件名,如未指定，则用地址（address）作为文件名

    account show [名字]，显示指定名字的账户信息，如未指定名字，则展示配置文件指定目录下所有的账户（.pem）信息

    写入和寻找账户文件的路径与配置文件同级。当前账户文件目录:conf
2)

--Contract:合约相关的命令--

    deploy [合约名] [合约构造的初始化参数...], 如 deploy HelloWorld [参数1] [参数2]

    sendtx [合约名] [地址或latest/last] [方法名] [方法对应的参数...], 如 sendtx HelloWorld latest  set "hello"

    call   [合约名] [地址或latest/last] [方法名] [方法对应的参数...], 如 call HelloWorld latest  get
    
    compile [合约名]  调用配置好的solc编译器，编译合约，默认合约sol文件和输出都在配置的contracts目录，solc下载参见contrats目录下的README（注：用deploy指令部署合约时，会先尝试编译）

    合约成功部署后，新地址会写入合约目录的contracthistory.toml文件，后续就可以用lastest/last代替地址调用了

    写入历史和寻找合约ABI文件的路径以配置文件里的[contract]contractpath=项为准。

合约当前的路径:./contracts

3)

--Get:查询类命令--
--查询类指令包含在全部RPC接口指令里，常用的指令如下--

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

全部指令参见url：https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html
```


## 控制台输入复杂数据类型概要说明

**数组**

合约数组如uint256[3] nums，那么在rust层面，其参数构造可以是 [1,2,3],同理，字符串数组对应['a','b','c']

在控制台输入时，数组参数需要加上中括号，比如[1, 2, 3]，数组中是字符串或字节类型，加双引号或单引号，例如[“alice”, ”bob”]，注意数组参数中不要有空格；布尔类型为true或者false。

**结构体**

合约结构体如
```
    struct User {
        string name;
        uint256 age;
     }
```
对应rust的tuple类型，如 ('alice',23)

如果是结构体数组 User[] _users, 则对应tuple数组如[('alice',23),('bob',28)]

在控制台输入时，按以上格式输入即可。举例
```
单个结构体参数
cargo run -- sendtx TestStruct latest addUser ('alice',23)

两个参数，第二个参数是结构体
cargo run -- sendtx TestStruct latest addbyname alice ('alice',23)

结构体数组参数
cargo run -- sendtx TestStruct latest addUsers [('alice',23),('bob',28)]

查询，返回的是结构体
cargo run -- call TestStruct latest getUser alice
```
**重要提示:**
* 输入数据的 \\, "等字符可能会被转义，所以参数应尽量不包含牵涉转义的各种字符，保持简单。
* 如输入的数据一定要包含转移字符，建议按urlencode,b64等模式先做编码，避免转义问题
* 或者修改src/bcossdk/liteutils.rs里的split_param方法，修改其转义实现，这个方法在数组参数，结构体参数，数组嵌套结构体参数解析时会被多次调用，
导致转义次数会不止一次，最终输出的结果可能会不如输入者预期。

## channel协议中的SSL库使用说明

channel协议，要求在客户端和节点之间TLS长连接，使用证书握手和加密，详细参见：[Channel协议](https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/design/protocol_description.html#channelmessage)

TLS实现分为国密和非国密两种，国密的证书私钥文件会比非国密多两个（sdk加密证书和key）。

对非国密TLS，本项目直接使用了ssl库，在系统上要求安装了ssl。在linux上要设置OPENSSL_DIR=[如：/usr/local/ssl],否则编译时rust的openssl-sys库会报错。

对国密版本，采用了TASSL开源库以及简单的C LIB封装对接，需要在不同平台上手动编译出动态库（dll或so)，并确保这些动态库和相关的依赖库，都部署在项目目录或系统目录下，可加载。

**如果想使用国密版本channel协议，请仔细阅读[native_ssock_lib下的README](./native_ssock_lib/)**

**要将c语言项目编译出来的动态库，全部复制到和可执行程序相同的目录或系统目录下下，才可以加载成功**

**在linux上，可以配置LD_LIBRARY_PATH=[so库所在目录]并生效**

**加载动态库的rust实现参见src/bcossdk/bcos_tls_native.rs，如有兴趣建议仔细走读，并进行优化，目前的实现在生命周期和稳定性方面有优化空间**

## todo list:

- 补全JSON RPC的接口（2021.10 大部分done）
- 在mac，arm等多平台上进行测试适配( 2021.08 windows/linux Ubuntu/CentOS已经适配)
- 支持多线程的channel长连接,实现异步风格的调用模式 （2021.10done）
- 支持event回调监听 （2021.10done）
- 支持AMOP (2021.10 amop实现组包解包，待实现整体框架)
- 优化代码风格，以更符合rust领域的规范
- 优化错误处理，包括错误码，错误逻辑，边界异常等 
- 优化生命周期/内存管理细节
- 优化性能

## 贡献代码
- 欢迎clone,fork，可以参考/复制所需代码,欢迎交流讨论。

- 欢迎并非常感谢您的贡献，请参阅[代码贡献流程](https://mp.weixin.qq.com/s/hEn2rxqnqp0dF6OKH6Ua-A
)。
- 如项目对您有帮助，欢迎star支持！


## 加入社区
**FISCO BCOS开源社区**是国内活跃的开源社区，社区长期为机构和个人开发者提供各类支持与帮助。已有来自各行业的数千名技术爱好者在研究和使用FISCO BCOS。如您对FISCO BCOS开源技术及应用感兴趣，欢迎加入社区获得更多支持与帮助。

## License
Rust SDK的开源协议为[MIT License](https://opensource.org/licenses/MIT). 详情参考[LICENSE](./LICENSE)。
