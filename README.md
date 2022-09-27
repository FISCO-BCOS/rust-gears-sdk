# FISCO BCOS SDK : Rust Gears
--- 
Rust SDK for FISCO-BCOS  ,like some  rusted  but solid gears , help to build  blockchain application with FISCO-BCOS

[FISCO BCOS](https://github.com/FISCO-BCOS/FISCO-BCOS/tree/master)的轻量级版本Rust SDK, 基础网络、国密非国密算法支持、合约解析能力较为完备，附带命令行交互控制台。

- 本项目的定位是一个学习/研究/编程兴趣的项目，仅供学习参考。如有正式的使用需求，建议在熟悉rust语言的前提下，仅部分参考本项目和FISCO BCOS相关的实现，去开发自己的生产级sdk，并经过严谨测试验证后使用。
  
- 本项目并非唯一且官方的fisco bcos rust sdk，社区陆续会有其他优秀的rust的sdk实现，提供多种选择和参考可能性



## 关键特性


支持FISCO BCOS3.0 release版本，支持FISCO BCOS2.x的RPC/Channel协议，国密和非国密算法，友好解析交易(Transaction)、回执(Receipt)、事件（Event）

有命令行控制台，支持创建和查询账户，以及合约类（部署调用），查询类的操作。

- 支持FISCO BCOS 3.x接口: [JSON-RPC](https://fisco-bcos-doc.readthedocs.io/zh_CN/latest/docs/develop/api.html)。对于FISCO BCOS3.x，客户端采用ffi方式包装C语言开发的底层SDK库，由SDK库负责网络协议封装和安全通信细节。
- 支持FISCO BCOS 2.x接口: [JSON-RPC](https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/api.html)。对于FISCO BCOS2.x，客户端基于[Channel协议](https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/design/protocol_description.html#channelmessage)与FISCO BCOS进行通信，保证节点与SDK安全加密通信的同时，可接收节点推送的消息。
- FISCO BCOS2.0国密和非国密的[Channel协议](https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/design/protocol_description.html#channelmessage)
- 支持交易的部署、发送交易、call 
- 支持解析解析功能：包括交易输入、交易输出、Event Log等ABI数据的拼装和解析。
- 支持基于pem文件的账户创建和私钥读取。
- 引用WeDPR的密码库进行HASH，签名, 该密码库支持ECDSA和SM2,3,4
- 控制台支持Struct参数，数组等复杂数据结构，SDK调用方法参见[src/sample/structdemo.rs](src/sample/structdemo.rs)
- 支持FISCO BCOS2.0 合约事件监听，参见相应调用


## rustc环境

已经验证的rust版本 

rustc 1.54.0-nightly (ed597e7e1 2021-06-08) 

rustc 1.55.0-nightly (67b03007c 2021-07-23)

rustc 1.66.0-nightly (f5193a9fc 2022-09-25)

rust版本的本身更新较快，请按rust官网指引安装配置。


## crates.io引用方式
考虑到稳定性和完备性，暂未发布最新版本，建议直接套用源代码，并在开发使用中完善。

## 代码调用示例
```
(FISCO BCOS2.x):

fn main() {
    //enable log
    log4rs::init_file("log4rs.yml", Default::default()).unwrap(); 
    //load config and init the bcossdk
    let mut bcossdk = BcosSDK::new_from_config("conf/config.toml").unwrap(); 
    //get node version,other apis see fisco-bcos document or sample
    let res = bcossdk.getNodeVersion();
    println!("res {:?}",res);
}

(FISCO BCOS3.x):
fn main() {
    //enable log
    log4rs::init_file("log4rs.yml", Default::default()).unwrap(); 
    //load config and init the bcossdk
    let bcos3client = Bcos3Client::new(cli.default_configfile().as_str()).unwarap(); 
    //get blocknumber,other apis see fisco-bcos document or sample
    let res = bcossdk.getBlockNumber();
    println!("res {:?}",res);
}


Tips:调用sdk代码前，应保证调用过一次日志初始化语句： 
log4rs::init_file("log4rs.yml", Default::default()).unwrap();
```



## 目录结构

- src/bcossdk : 主要的sdk代码，代码本身已经比较模块化，有注释，欢迎查阅
- src/bcos3sdk : 整合FISCO BCOS3.x的[C语言SDK](https://fisco-bcos-doc.readthedocs.io/zh_CN/latest/docs/develop/sdk/c_sdk/)
- src/console : 命令行控制台实现代码 
- src/sample: 陆续增加一些sample
- conf : 配置文件,运行前，先将client_config.toml.sample复制为client_config.toml,并仔细理解和修改里面的配置值，尤其是ip端口和证书
- sdk  : 连接节点的证书，国密和非国密的都放到这里
- contracts : 合约的sol，abi，bin等
- gm: 考虑到同时连接国密和非国密节点，推荐建立一个gm目录，包含以上的conf,sdk,contracts等目录结构，保存国密节点所需的配置、证书、私钥、合约定义和代码等
- log: 控制台运行后会自动生成log目录，保存日志文件。日志配置见log4rs，默认配置是输出滚动日志，可根据实际需要修改。
  

## FISCO BCOS3.x的C语言SDK库 

FISCO BCOS 3.x的rust sdk通过FFI方式封装C语言实现的接口库，协议、通信、证书等细节封装在C/C++库里。

ABI格式编解码和2.x的客户端一样依旧用rust实现。

**重要:**

最新版本的C语言的SDK库文件可到[文件下载连接](https://fisco-bcos-doc.readthedocs.io/zh_CN/latest/docs/develop/sdk/c_sdk/dylibs.html),下载相应操作系统的库文件。

如windows平台上的bcos-c-sdk.dll/.lib,linux平台上的libbcos-c-sdk.so等。

下载后放到当前目录的编译环境路径和运行环境路径下，具体路径取决于开发者的特定项目结构、环境配置。总之一定要在编译期和运行期能映射到C语言SDK库。

**C语言SDK接口实现代码**

[https://github.com/FISCO-BCOS/bcos-c-sdk](https://github.com/FISCO-BCOS/bcos-c-sdk)

[技术文档](https://fisco-bcos-doc.readthedocs.io/zh_CN/latest/docs/develop/sdk/c_sdk/index.html)

**C++客户端代码**
[https://github.com/FISCO-BCOS/bcos-cpp-sdk](https://github.com/FISCO-BCOS/bcos-cpp-sdk)

## 配置文件
主要配置文件是 conf/config.toml，项目提供了conf/config.toml.sample,将其复制或去掉sample后缀即可。

目前的配置文件同时包含FISCO BCOS2/FISCO BCOS3的配置，初始化时会全部加载，后续可以优化为特定版本客户端只加载特定的配置

配置项解释：


```
[common]
crypto = "ECDSA"
accountpem = "conf/client.pem"
contractpath = "./contracts"
solc = "./bin/solc"
solcgm = "./bin/solc-gm"

#------------------FISCO BCOS3.0 Begin----------------------------------------
[bcos3]
# FISCO BCOS3.0的配置段，如连接FISCO BCOS2.0版本，无需关心此段
# FISCO BCOS3.0 c底层sdk的配置，都在bcos3_config_file里，无需配置在此文件
sdk_config_file ="./bcos3sdklib/bcos3_sdk_config.ini"
group = "group0"
#-------------------FISCO BCOS3.0 End-----------------------------------------


#------------------FISCO BCOS2.0 Begin----------------------------------------
[bcos2]
chainid = 1
groupid = 1
protocol = "CHANNEL"

[rpc]
url = "http://127.0.0.1:8545"
timeout = 3


[channel]
ip = "127.0.0.1"
port = 20200
tlskind = "ECDSA"
timeout = 10
nativelib_echo_mode = 0
cacert = "sdk/ca.crt"
sdkcert = "sdk/sdk.crt"
sdkkey = "sdk/sdk.key"
gmcacert = "sdk/gmca.crt"
gmsdkcert = "sdk/gmsdk.crt"
gmsdkkey = "sdk/gmsdk.key"
gmensdkcert = "sdk/gmensdk.crt"
gmensdkkey = "sdk/gmensdk.key"
#------------------FISCO BCOS2.0 End----------------------------------------
```


## 控制台使用帮助：
```
cargo run -- --help       控制台本身的help(采用StructOpt库默认格式）

典型命令如 

cargo run --  bcos3 deploy HelloWorld 

命令行选项：

    -c, --config <configfile>
            -c 配置文件，全路径如-c conf/config.toml

    -n, --contractname <contractname>
            -n 显式的指定合约名，主要是供解析交易和回执时使用，不用带后缀，如"HelloWorld"


```
```
cargo run -- usage      bcossdk的操作命令字帮助，建议查看包括 usage account，usage contract，usage get或usage all

注意，控制台调用区块链的RPC接口时，在此版本开始需要区分bcos2，bcos3的客户端，如

查询类：

cargo run -- bcos2 getBlockNumber
cargo run -- bcos3 getBlockNumber

更多命令参见cargo run -- usage get

合约：

cargo run -- bcos2 deploy HelloWorld
cargo run -- bcos2 sendtx HelloWorld latest set "new data"
cargo run -- bcos3 deploy HelloWorld
cargo run -- bcos3 sendtx HelloWorld latest set "new data"
```


账户管理、合约编译这些无节点版本区别的，则不需要加bcos2/bcos3参数

## Solc编译器下载说明

下载链接参见 [Github Release:包含多版本/多平台](https://github.com/FISCO-BCOS/solidity/releases)

根据实际操作系统版本、国密或非国密，下载相应的二进制文件，解压并放到bin/目录下，或者和配置路径对应

建议同时下载0.4.25和6.x的solc

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
cargo run -- bcos3 sendtx TestStruct latest addUser ('alice',23)

两个参数，第二个参数是结构体
cargo run -- bcos3 sendtx TestStruct latest addbyname alice ('alice',23)

结构体数组参数
cargo run -- bcos3 sendtx TestStruct latest addUsers [('alice',23),('bob',28)]

查询，返回的是结构体
cargo run -- bcos3 call TestStruct latest getUser alice
```
**重要提示:**
* 输入数据的 \\, "等字符可能会被转义，所以参数应尽量不包含牵涉转义的各种字符，保持简单。
* 如输入的数据一定要包含转移字符，建议按urlencode,b64等模式先做编码，避免转义问题
* 或者修改src/bcossdk/liteutils.rs里的split_param方法，修改其转义实现，这个方法在数组参数，结构体参数，数组嵌套结构体参数解析时会被多次调用，
导致转义次数会不止一次，最终输出的结果可能会不如输入者预期。

## FISCO BCOS2.x channel协议中的SSL库使用说明

channel协议用于FISCO BCOS 2.x。FISCO BCOS 3.x采用了新的协议，不是这个版本的channel协议，如使用3.x的节点，无需关注此节。

channel协议要求在客户端和节点之间TLS长连接，使用证书握手和加密，详细参见：[Channel协议](https://fisco-bcos-documentation.readthedocs.io/zh_CN/latest/docs/design/protocol_description.html#channelmessage)

TLS实现分为国密和非国密两种，国密的证书私钥文件会比非国密多两个（sdk加密证书和key）。

对非国密TLS，本项目直接使用了ssl库，在系统上要求安装了ssl。在linux上要设置OPENSSL_DIR=[如：/usr/local/ssl],否则编译时rust的openssl-sys库会报错。

对国密版本，采用了TASSL开源库以及简单的C LIB封装对接，需要在不同平台上手动编译出动态库（dll或so)，并确保这些动态库和相关的依赖库，都部署在项目目录或系统目录下，可加载。

**如果想使用国密版本channel协议，请仔细阅读[native_ssock_lib下的README](./native_ssock_lib/)**

**要将c语言项目编译出来的动态库，全部复制到和可执行程序相同的目录或系统目录下下，才可以加载成功**

**在linux上，可以配置LD_LIBRARY_PATH=[so库所在目录]并生效**

**加载动态库的rust实现参见src/bcossdk/bcos_tls_native.rs，如有兴趣建议仔细走读，并进行优化，目前的实现在生命周期和稳定性方面有优化空间**

## todo list:
- AMOP的完整实现（有待补充，需在多节点多机构之间开发测试验证）
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
