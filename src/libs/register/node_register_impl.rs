// register this node to NodeLookup

use std::collections::HashMap;
use std::sync::{Arc};
use log::{info, warn, error, debug};
use tokio::sync::Mutex;
use crate::libs::{http2};
use crate::libs::json::json_impl;
use lazy_static::lazy_static;
use crate::libs::register::node_types;
use tokio::time;
use warp::http;
use crate::libs::register::node_types::{HttpRegisterNodes, HttpServiceQueryRequest, RegisterNode, ServiceNode};
use crate::libs::register::node_uri_path;
use async_channel::{Receiver, Sender};

#[derive(Debug)]
struct RegisterNodeRR {
    node : Vec<RegisterNode>,
    rr   : u32,
}

impl RegisterNodeRR {
    fn new() -> Self {
        RegisterNodeRR{
            node: vec![],
            rr: 0,
        }
    }

    #[allow(dead_code)]
    fn get(&mut self) -> String {
        if self.node.is_empty() {
            return "".to_string();
        }
        if self.rr >= self.node.len() as u32 {
            self.rr = 0;
        }
        let ss = self.node[self.rr as usize].api_root.clone();
        self.rr += 1;
        ss
    }

    #[allow(dead_code)]
    fn erase(&mut self, uid : String) {
        // update node
        let mut nodes : Vec<RegisterNode> = vec![];
        for j in &self.node {
            if uid != j.uid {
                nodes.push(j.clone());
            }
        }
        nodes.sort();
        self.node = nodes;
    }

    fn update(&mut self, vv : &Vec<RegisterNode>) {
        self.node = vv.clone();
        self.node.sort();
    }
}

pub struct RegisterStub {
    exit_flag : Mutex<bool>,
    node_type_store: Arc<Mutex<HashMap<String, Mutex<RegisterNodeRR>>>>,
    uuid_store: Arc<Mutex<HashMap<String, RegisterNode>>>,
    update_locker : Mutex<()>,
    update_nodes : (Sender<String>, Receiver<String>)
}

impl RegisterStub {
    fn new() -> Self {
        let (s, r) = async_channel::unbounded();
        RegisterStub{
            exit_flag : Mutex::new(false),
            node_type_store: Arc::new(Mutex::new(HashMap::new())),
            uuid_store: Arc::new(Mutex::new(HashMap::new())),
            update_locker: Default::default(),
            update_nodes: (s, r)
        }
    }

    pub async fn set_exit(&self) {
        let mut x = self.exit_flag.lock().await;
        *x = true;
    }
    pub async fn is_exit(&self) -> bool {
        let x = self.exit_flag.lock().await;
        *x
    }

    pub async fn waiting_update_nodes(&self) {
        match self.update_nodes.1.recv().await {
            Ok(_) => {}
            Err(e) => {
                error!("receive notify update nodes message failed, err {}\n", e);
            }
        }
    }

    pub async fn notify_update_nodes(&self) {
        // match self.update_nodes.0.send("_".to_string()).await
        // _ will let action().await not execute at all
        // why ???
        match self.update_nodes.0.send("_".to_string()).await {
            Ok(_) => {}
            Err(e) => {
                error!("send notify update nodes message failed, err {}\n", e);
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) async fn get(&self, uuid_ : String, node_type : String) -> String {
        if uuid_.len() > 0 {
            let uuid_hash = self.uuid_store.lock().await;
            let specific = uuid_hash.get(&uuid_);
            match specific {
                Some(v) => {
                    // routed by from uid
                    return v.api_root.clone();
                }
                None => {
                }
            }
        }
        let node_hash = self.node_type_store.lock().await;
        let node_result = node_hash.get(&node_type);
        match node_result {
            Some(v) => {
                return v.lock().await.get();
            }
            None => {
            }
        }
        "".to_string()
    }

    pub fn wrap_http_url(&self, root : String, path : String) -> String {
        let ss = "https://".to_string();
        ss + &root + &path
    }

    async fn register(
        &self,
        schema_to_be_register: String,
        host_to_be_register : String,
        host_node_lookup: String,
        path : String,
        this_node_type : String,
        uuid : String
    ) -> bool {
        let url = self.wrap_http_url(host_node_lookup, path);
        let mut req = ServiceNode::new();
        req.uid = uuid;
        req.node_type = this_node_type;
        req.api_root = host_to_be_register;
        if schema_to_be_register == "http" {
            req.scheme = schema_to_be_register.clone();
        }

        let ser = json_impl::marshal(&req);
        let req_body = match ser {
            Ok(v) => {
                v
            }
            Err(e) => {
                error!("error marshal data while send to lookup and for register self, err : {:?}\n", e);
                return false;
            }
        };

        let (status, _) = http2::http2_client_impl::http2_client_post(url, req_body, false).await;

        if status != http::StatusCode::OK.as_u16() {
            error!("get error status code while send to lookup and for register self, status {}\n", status);
            return false;
        }

        true
    }

    pub(crate) async fn update(&self,
                               host_node_lookup: String,
                               app_uuid : String,
    ) -> bool {

        let _x = self.update_locker.lock().await;

        let url = self.wrap_http_url(host_node_lookup, node_uri_path::PATH_HTTP_NODE_QUERY.to_string());

        let mut req = HttpServiceQueryRequest::new();
        req.from_uid = app_uuid;
        req.type_filter.exclude.push(node_types::TYPES_NODE_TYPE_LOOKUP_NODE.to_string());

        let ser = json_impl::marshal(&req);
        let req_body = match ser {
            Ok(v) => {
                v
            }
            Err(e) => {
                error!("error marshal data while send to lookup and for request nodes, err : {:?}\n", e);
                return false;
            }
        };

        let (status, body) = http2::http2_client_impl::http2_client_post(url, req_body, true).await;

        if status != http::StatusCode::OK.as_u16() {
            error!("get error status code while send to lookup and for request nodes, status {}\n", status);
            return false;
        }

        //deserialize
        let mut http_data = HttpRegisterNodes::new();
        match json_impl::unmarshal(&body, &mut http_data) {
            Ok(_) => {}
            Err(e) => {
                error!("unmarshal http_body {} err {}\n", body, e);
                return false;
            }
        }

        // update nodes
        http_data.nodes.sort_by(|a, b| a.node_type.partial_cmp(&b.node_type).unwrap());

        // uuid index
        {
            let mut hash = HashMap::new();
            for i in &http_data.nodes {
                hash.insert(i.uid.clone(), i.clone());
            }
            let mut uuid_hash = self.uuid_store.lock().await;
            *uuid_hash = hash;
        }

        debug!("http_data : {:?}", http_data);

        if http_data.nodes.is_empty() {
            info!("retrieved registered nodes from node lookup is empty\n");
            return true;
        }

        let mut last = 0;
        for i in 0..http_data.nodes.len() + 1 {
            if i == http_data.nodes.len() || http_data.nodes[last].node_type != http_data.nodes[i].node_type {
                debug!("add slice nodes find idx {} type : {}", last, &http_data.nodes[last].node_type);
                let n : Result<RegisterNodeRR, ()> = match self.node_type_store.lock().await.get(&http_data.nodes[last].node_type) {
                    Some(r) => {
                        debug!("add slice nodes e : {:?}", &http_data.nodes[last..i]);
                        r.lock().await.update(&http_data.nodes[last..i].to_vec());
                        Err(())
                    }
                    None => {
                        let mut node = RegisterNodeRR::new();
                        debug!("add slice nodes n : {:?}", &http_data.nodes[last..i]);
                        node.update(&http_data.nodes[last..i].to_vec());
                        Ok(node)
                    }
                };
                match n {
                    Ok(v) => {
                        debug!("add node_type {} nodes {:?} \n", http_data.nodes[last].node_type.clone(), v);
                        self.node_type_store.lock().await.insert(http_data.nodes[last].node_type.clone(), Mutex::new(v));
                    }
                    Err(_) => {}
                }
                last = i;
            }
        }
        true
    }
}

lazy_static!(
  static ref SINGLETON_INSTANCE : RegisterStub = RegisterStub::new();
);

pub fn get_register() -> &'static RegisterStub {
    &*SINGLETON_INSTANCE
}

async fn register_ac(
    reg : bool,
    schema_to_be_register : &String,
    host_to_be_register: &String,
    host_node_lookup: &Vec<String>,
    this_node_type : &String,
    app_uuid : &String
) -> bool {
    let path = if reg {
        node_uri_path::PATH_HTTP_REGISTER.to_string()
    }else{
        node_uri_path::PATH_HTTP_DEREGISTER.to_string()
    };
    for i in host_node_lookup {
        let s = get_register().register(schema_to_be_register.clone(),
                                host_to_be_register.clone(),
                                i.clone(),
                                path.clone(),
                                this_node_type.clone(),
                                app_uuid.clone()).await;
        if s {
            return true;
        }
    }
    false
}

async fn update_ac(
    host_node_lookup: &Vec<String>,
    app_uuid : &String,
)-> bool {
    for i in host_node_lookup {
        if get_register().update(i.clone(), app_uuid.clone()).await {
            return true;
        }
    }
    false
}

async fn future_nodes_update_handle(
    host_node_lookup: Vec<String>,
    app_uuid : String,
) {
    info!("start update register nodes from lookup thread");
    loop {
        get_register().waiting_update_nodes().await;
        info!("update register nodes from lookup, begin\n");
        let update = update_ac(&host_node_lookup, &app_uuid.clone()).await;
        info!("update register nodes from lookup, result {:?}\n", update)
    }
}

async fn future_register_handle(host_node_lookup: Vec<String>,
                                    schema_to_be_register : String,
                                    host_to_be_register : String,
                                    this_node_type : String,
                                    app_uuid : String) {
    loop {
        if get_register().is_exit().await {
            warn!("register procedure exiting due to the system is exiting status\n");
            break;
        }
        let update = update_ac(&host_node_lookup, &app_uuid.clone()).await;
        if update {
            let register = register_ac(true,
                                       &schema_to_be_register,
                                       &host_to_be_register,
                                       &host_node_lookup,
                                       &this_node_type,
                                       &app_uuid
            ).await;
            if !register {
                error!("register self procedure failed\n");
            }
        }else{
            error!("register update procedure failed\n");
        }
        time::sleep(time::Duration::from_millis(1000)).await;
    }

    let de_register = register_ac(false,
                                  &schema_to_be_register,
                                  &host_to_be_register,
                                  &host_node_lookup,
                                  &this_node_type,
                                  &app_uuid
    ).await;
    if !de_register {
        error!("de-register self procedure failed\n");
    }else{
        info!("de-register self procedure succeed\n");
    }
}

pub async fn start_register_loop(
    host_node_lookup: Vec<String>,
    info:(String, String, String, String)
) {
    info!("register info : [{:?}]\n", info);
    let (schema_to_be_register,
        host_to_be_register,
        this_node_type,
        app_uuid) = info;

    tokio::spawn(future_nodes_update_handle(host_node_lookup.clone(),
                                            app_uuid.clone()));

    tokio::spawn(future_register_handle(host_node_lookup.clone(),
                                        schema_to_be_register,
                                        host_to_be_register,
                                        this_node_type,
                                        app_uuid));
}