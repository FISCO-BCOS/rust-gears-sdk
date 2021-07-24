use crate::bcossdk::bcossdk::BcosSDK;

pub fn demo(){

    let mut bcossdk = BcosSDK::new_from_config("gm/conf/config.toml").unwrap();
    let block_limit = bcossdk.getBlockLimit();
    println!("block limit {:?}",block_limit);
    let version  = bcossdk.getNodeVersion();
    println!("node version {:?}",version);

}