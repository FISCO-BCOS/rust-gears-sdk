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

//use crate::bcossdkutil::bcos_ssl_native::BcosNativeTlsClient;
//use crate::bcossdkutil::bcos_ssl_normal::BcosSSLClient;
//use crate::bcossdkutil::bcosclientconfig::{BcosCryptoKind, ChannelConfig};
//use crate::bcossdkutil::bufferqueue::BufferQueue;
use crate::bcos2sdk::channelpack::{make_channel_pack, ChannelPack, CHANNEL_PACK_TYPE};
//use crate::bcossdkutil::kisserror::{KissErrKind, KissError};
use crate::bcos2sdk::bcos2client::Bcos2Client;
use crate::bcos2sdk::bcos_channel_handler_manager::ChannelPushHandlerManager;
use crate::bcos2sdk::bcosrpcwraper::RpcRequestData;
use crate::bcossdkutil::kisserror::KissError;
use lazy_static::lazy_static;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

type BcosSDKArc = Arc<Mutex<Bcos2Client>>;
type BcosChannelWorkerArc = Arc<Mutex<BcosChannelWorker>>;
type BcosSDKMap = Arc<Mutex<HashMap<String, Bcos2Client>>>;
lazy_static! {
    static ref BCOS_SDK_MAP: BcosSDKMap = {
        let sdkmap: BcosSDKMap = Arc::new(Mutex::new(HashMap::new()));
        sdkmap
    };
}

///用线程模式读写channel连接上的数据，并回调处理者
pub struct BcosChannelWorker {
    pub bcossdk: Bcos2Client,
    pub handlemanager: ChannelPushHandlerManager,
    pub is_working: bool,
}

impl BcosChannelWorker {
    pub fn new(configfile: &str) -> Self {
        BcosChannelWorker {
            bcossdk: Bcos2Client::new_from_config(configfile).unwrap(),
            handlemanager: ChannelPushHandlerManager::default(),
            is_working: true,
        }
    }
}
//unsafe impl Send for BcosChannelWorker{}
//unsafe impl Sync for BcosChannelWorker{}

pub fn read_packets(worker_arc: &BcosChannelWorkerArc) -> Result<Vec<ChannelPack>, KissError> {
    let mut worker = worker_arc.lock().unwrap();
    worker.bcossdk.netclient.channel_client.read_packets()
}

pub fn send_packet(
    worker_arc: &BcosChannelWorkerArc,
    pack: &ChannelPack,
) -> Result<i32, KissError> {
    let mut worker = worker_arc.lock().unwrap();
    worker
        .bcossdk
        .netclient
        .channel_client
        .try_send(&pack.pack())
}

fn is_working(worker_arc: &BcosChannelWorkerArc) -> bool {
    let worker = worker_arc.lock().unwrap();
    let working = worker.is_working;
    working
}

async fn read_thread(worker_arc: BcosChannelWorkerArc, tx: &Sender<ChannelPack>) {
    println!("read thread running");
    //let queue = BufferQueue::default();
    loop {
        if !is_working(&worker_arc) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;

        //let mut worker = worker_arc.lock().unwrap();
        let recvres = read_packets(&worker_arc);
        match recvres {
            Ok(res) => {
                //println!("read from channel ,total {}", res.len());
                let mut i = 0;
                for pack in res {
                    i += 1;
                    println!("{})-> {}", i, pack.detail());
                    let res = tx.send(pack).await;
                }
            }
            Err(e) => {
                println!("read_thread error {:?}", e);
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    println!("read thread done!!!")
}

pub fn getNodeVersionPack() -> Option<ChannelPack> {
    let groupid = 1;
    let cmd = "getClientVersion";
    let params_value = json!([groupid]);

    let req = RpcRequestData {
        method: cmd.to_string(),
        params: params_value.clone(),
        jsonrpc: "2.0".to_string(),
        id: 1,
    };
    println!("{:?}", req);
    make_channel_pack(CHANNEL_PACK_TYPE::RPC, req.encode().unwrap().as_str())
}

///每3秒钟发一次heartbeat
async fn heart_beat_thread(worker_arc: BcosChannelWorkerArc) {
    let mut last_heartbeat_time = time::now() - chrono::Duration::seconds(10);

    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
        if !is_working(&worker_arc) {
            break;
        }

        if time::now() - last_heartbeat_time < chrono::Duration::seconds(5) {
            continue;
        }
        //let heartbeatpack = make_channel_pack(CHANNEL_PACK_TYPE::HEART_BEAT, "");
        let heartbeatpack = getNodeVersionPack();
        last_heartbeat_time = time::now();
        println!(
            "heartbeat send {:?}",
            heartbeatpack.as_ref().unwrap().detail()
        );
        let res = send_packet(&worker_arc, &heartbeatpack.unwrap());

        println!("heartbeat send result {:?}", res);
        //let recvres = worker.bcossdkutil.netclient.channel_client.try_recv();
        //println!("heartbeat recv result {:?}",recvres);
    }
    println!("heartbeat thread done!!")
}

pub async fn start(configfile: &str) {
    println!("start worker");
    let worker = BcosChannelWorker::new(configfile);
    let worker_arc = Arc::new(Mutex::new(worker));
    start_bcos_channel_worker(&worker_arc).await;
}

pub fn handle_packet(worker_arc: &BcosChannelWorkerArc, pack: &ChannelPack) {
    println!(">>>handle packet {}", pack.detail());
    let worker = worker_arc.lock().unwrap();
    println!("worker ->>>> {}", worker.handlemanager.count_handler());
    let handleOpt = worker.handlemanager.get_handle(&pack.packtype);
    match handleOpt {
        Some(handle) => {
            handle.lock().unwrap().handle(&pack);
        }
        None => {
            println!("Handle not found for type {}", pack.packtype);
        }
    }
}

pub async fn start_bcos_channel_worker(worker_arc: &Arc<Mutex<BcosChannelWorker>>) {
    println!("start worker");
    let (readthread_sender, mut readthread_recver) = mpsc::channel::<ChannelPack>(32);

    //let worker = BcosChannelWorker::new(configfile);
    //let worker_arc = Arc::new(Mutex::new(worker));
    println!("start heatbeat thread");
    let worker_heartbeat = worker_arc.clone();
    tokio::spawn(async move {
        heart_beat_thread(worker_heartbeat).await;
    });
    println!("start read thread");

    let worker_read = worker_arc.clone();
    tokio::spawn(async move {
        read_thread(worker_read, &readthread_sender.clone()).await;
    });

    println!("local waiting");

    let mut counter = 1;
    let mut working = true;
    loop {
        println!("still waiting");

        match readthread_recver.recv().await {
            Some(pack) => {
                counter += 1;
                println!("rx recv packet {:?}", pack.detail());
                let worker_local = worker_arc.clone();
                handle_packet(&worker_local, &pack);

                if counter > 100 {
                    println!("shutdown threads");
                    //worker_local = worker_arc.clone();
                    worker_local.lock().unwrap().is_working = false;
                    working = false;
                    break;
                }
            }
            None => {
                tokio::time::sleep(Duration::from_micros(1000)).await;
            }
        }
    }
    tokio::time::sleep(Duration::from_millis(3000)).await;
    println!("local thread done");
}
