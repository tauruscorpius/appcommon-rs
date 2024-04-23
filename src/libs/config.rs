use serde::{Deserialize};
use tokio::sync::Mutex;
use lazy_static::lazy_static;
use log::error;

#[derive(Deserialize, Default, Debug, Clone)]
pub struct Redis {
    pub host : String,
    pub port : u16,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct EtcdEndPoint {
    pub host : String,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct NodeLookup {
    pub host : String,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct ExampleConfig {
    redis : Vec<Redis>,
    etcd_endpoints : Vec<EtcdEndPoint>,
    node_lookup_nodes : Vec<NodeLookup>,
}

pub fn get_home() -> String {
    match std::env::var("HOME") {
        Ok(v) =>  v,
        Err(e) => {
            println!("cant find HOME env, err : {}\n", e);
            return "".to_string()
        }
    }
}

impl ExampleConfig {
    pub fn new() -> Self{
        Self{
            redis: vec![],
            etcd_endpoints: vec![],
            node_lookup_nodes: vec![],
        }
    }
    pub fn parse(&mut self, config_file : &str) -> bool {
        let cfg_file = get_home() + "/etc/" + config_file;
        let cfg_file_content = match std::fs::read_to_string(cfg_file) {
            Ok(v) => v,
            Err(e) => {
                error!("read config file content failed, error : {}\n", e);
                return false;
            }
        };
        let parsed = match toml::from_str(cfg_file_content.as_str()) {
            Ok(v) => {
                *self = v;
                true
            }
            Err(e) => {
                error!("parse config file failed, error : {}\n", e);
                false
            }
        };
        parsed
    }
    pub fn get_redis_config(&self) -> Vec<String> {
        self.redis.iter().map(|x| "redis://".to_owned() + &x.host.clone() + &*":".to_string() + &*x.port.to_string()).collect()
    }
    pub fn get_etcd_endpoints(&self) -> Vec<String> {
        self.etcd_endpoints.iter().map(|x| "http://".to_owned() + &x.host.clone()).collect()
    }
    pub fn get_node_lookup_nodes(&self) -> Vec<String> {
        self.node_lookup_nodes.iter().map(|x| x.host.clone()).collect()
    }
    pub fn get_esl_inbound_info(&self) -> (String, String, String) {
        (self.get_esl_inbound_addr(), self.get_esl_inbound_password(), self.get_esl_inbound_gw())
    }
}

lazy_static!(
  static ref CONFIG_INSTANCE : Mutex<ExampleConfig> = Mutex::new(ExampleConfig::new());
);

pub fn get_config() -> &'static Mutex<ExampleConfig> { &*CONFIG_INSTANCE }
