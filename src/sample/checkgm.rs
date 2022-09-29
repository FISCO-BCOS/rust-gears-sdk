use fisco_bcos_rust_gears_sdk::bcos2sdk::bcos2client::Bcos2Client;

pub fn demo() {
    let mut bcossdk = Bcos2Client::new_from_config("gm/conf/config.toml").unwrap();
    let block_limit = bcossdk.getBlockLimit();
    println!("block limit {:?}", block_limit);
    let version = bcossdk.getNodeVersion();
    println!("node version {:?}", version);
}
