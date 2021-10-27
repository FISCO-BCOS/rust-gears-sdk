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

use crate::bcossdk::contractabi::ContractABI;
use crate::bcossdk::bcos_channel_threads_worker::{BcosChannelWorker};
use crate::bcossdk::bcos_channel_handler_manager::{IChannelPushHandlerFacade, HANDLE_FACADE_OBJ};
use crate::bcossdk::channelpack::{ChannelPack, CHANNEL_PACK_TYPE};
use std::sync::{Arc, Mutex};
use crate::bcossdk::contracthistory::ContractHistory;
use crate::bcossdk::kisserror::KissError;
use serde_derive::{Serialize,Deserialize};
use std::time::Duration;
use std::thread;
use crate::bcossdk::{bcos_channel_threads_worker, channelpack};
use crate::bcossdk::liteutils::datetime_str;
use serde_json::{ Value as JsonValue};

pub struct EventHandler
{
    pub worker:Arc::<Mutex<BcosChannelWorker>>,
    pub contract: ContractABI
}



 /*
        {
      "fromBlock": "latest",
      "toBlock": "latest",
      "addresses": [
        0xca5ed56862869c25da0bdf186e634aac6c6361ee
      ],
      "topics": [
        "0x91c95f04198617c60eaf2180fbca88fc192db379657df0e412a9f7dd4ebbe95d"
      ],
      "groupID": "1",
      "filterID": "bb31e4ec086c48e18f21cb994e2e5967"
    }*/
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RegisterEventRequest
{
    fromBlock:String,
    toBlock:String,
    addresses: Vec<String>,
    topics:Vec<String>,
    groupID:u8,
    filterID:String
}
impl RegisterEventRequest{
    pub fn new()->Self
    {
        RegisterEventRequest{
            fromBlock:"latest".to_string(),
            toBlock:"latest".to_string(),
            addresses:vec!(),
            topics: vec!(),
            groupID:1,
            filterID:"bb31e4ec086c48e18f21cb994e2e5968".to_string() //ChannelPack::make_seq()
        }
    }
}


impl EventHandler{
    pub fn new(configfile:&str,contract_name:&str)->Self{
        let worker = BcosChannelWorker::new(configfile);
        let contract = ContractABI::new_by_name(contract_name,
                                            worker.bcossdk.config.contract.contractpath.as_str(),
                                            &worker.bcossdk.hashtype).unwrap();

        EventHandler {
            contract:contract,
            worker:Arc::new(Mutex::new(worker))
        }
    }
    pub fn register_eventlog_filter(&mut self,eventcallback:HANDLE_FACADE_OBJ,
                                    address:&Vec<String>,event_name:&str,indexed_values:&Vec<&str>)
    {
        //注册回调的入口
        self.worker.lock().unwrap().handlemanager.set_handle
        (&(CHANNEL_PACK_TYPE::EVENT_LOG_PUSH as u16),eventcallback);

        //构造注册消息监听的请求包json
        let mut req = RegisterEventRequest::new();
        req.addresses.append(&mut address.clone());
        println!("event name is {}",event_name);
        let eventres = self.contract.find_event_by_name(event_name);
        //self.contract.print_event_namehash();
        let event = match eventres{
            Some(ev)=>{ev},
            None=>{
                //没有找到event定义，返回了
                //println!("not found event for {}",event_name);
                return
            }
        };
        //这些打印都可以删掉
        println!("event  is {:?}",event);
        let evhash = self.contract.event_abi_utils.event_signature(&event);
        req.topics.push(format!("0x{}",hex::encode(evhash)));
        for t in &req.topics
        {
            //println!("topic: {:?}",format!("0x{}",hex::encode(evhash)));
        }
        let indexedevents = self.contract.event_abi_utils.indexed_params(&event,true);
        let mut i = 0;
        for e in indexedevents {
            if i>=indexed_values.len() {break;} //根据输入的参数个数
            //将输入的参数值转换为indexed类型的topic
            let  topic = self.contract.event_abi_utils.topic_by_indexed_params
            (&e.kind,&indexed_values.get(i).unwrap());
            i=i+1;
            println!("topic is {},eventparam:{:?}",&topic,e);
            //将topic加入到请求包体
            req.topics.push(topic);
         }
        let reqencode = serde_json::to_string(&req).unwrap();
        //println!("reqencode {}",reqencode);
        //注册包是个amop类型的消息
        let registerreqdata = channelpack::pack_amop(&Vec::from(""),&Vec::from(reqencode));
        println!("register amop data : {:?}",registerreqdata);
        //let (amoptopic ,amopdata ) = channelpack::unpack_amop(&registerreqdata);
        //println!("amop data unpack {}",String::from_utf8(amopdata).unwrap());
        let pack  = channelpack::make_channel_pack_by_rawdata(CHANNEL_PACK_TYPE::CLIENT_REGISTER_EVENT_LOG,
                    &registerreqdata
        );
        let packdata  = pack.unwrap().pack();
        // 发送请求包
        let mut worker = self.worker.lock().unwrap();
        let res = worker.bcossdk.netclient.channel_client.try_send(&packdata);

        //println!("register event result {:?},packdata {:?}",res,packdata);
    }
}

///事件回调演示，要实现IChannelPushHandlerFacade接口，包含的contract对象用来解析事件
pub struct DemoEventHandler{
    pub contract: ContractABI
}

impl DemoEventHandler{
    pub fn new(ct:&ContractABI)->Self{
        DemoEventHandler{
            contract:ct.clone()
        }
    }
}

///事件回调的处理，简单解析和打印一下事件
impl IChannelPushHandlerFacade for DemoEventHandler{
    fn handle(&self,pack: &ChannelPack) {

        println!("!!!!!!!  EVENT CALL BACKUP {}",pack.detail());
        let value :JsonValue= serde_json::from_str(&String::from_utf8(pack.data.clone()).unwrap().as_str()).unwrap();
        let parsed_logs = self.contract.parse_receipt_logs(&value["logs"]);
        println!("{:?}",parsed_logs);
    }
}


pub async fn  event_demo(configfile:&str)->Result<(),KissError>{

    let contract_name = "HelloEvent";
    let mut evh = EventHandler::new(configfile,contract_name);
    let history_file = ContractHistory::history_file(evh.worker.lock().unwrap().bcossdk.config.contract.contractpath.as_str());
    let address = ContractHistory::get_last_from_file(history_file.as_str(),contract_name)?;
    println!("address is {}",address);
    println!("contract abi is {} ",evh.contract.abi_file);
        //let handler = Arc::new(DemoEventHandler::new());
    let demohandler = Arc::new(Mutex::new(DemoEventHandler::new(&evh.contract)));
    let event_name = "on_two_indexed"; //具体定义参见合约sol文件和abi
    //event_name = "on_set";
    //event里的indexed类型的参数，传入时全部用字符串形式
    let indexed_value = vec!("5","key123");
    //注册事件监听
    evh.register_eventlog_filter(demohandler,&vec!(address.clone()),event_name,&indexed_value);


    println!("\n>>>>>>>>>>>>>>>>>>>>demo helloEvent settwo");
    //发个交易触发一下事件
    let method = "settwo";
    let paramsvec = vec!(format!("Test string for helloEvent: {}",datetime_str()),"5".to_string(),"key123".to_string());
    let txres = evh.worker.lock().unwrap().bcossdk.send_raw_transaction(&evh.contract, &address, &method.to_string(), paramsvec.as_slice());
    println!("send_raw_transaction result {:?}", txres);


    println!("go start worker");
    bcos_channel_threads_worker::start_bcos_channel_worker(&evh.worker).await;
    loop{
        thread::sleep(Duration::from_secs(1));
       // tokio::time::sleep(Duration::from_micros(500)).await;
    }
    //Ok(())

}