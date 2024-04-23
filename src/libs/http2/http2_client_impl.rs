use tokio::sync::Mutex;
use std::sync::{Arc};
use log::{error};
use lazy_static::lazy_static;

pub async fn reqwest_post(url : String, body : String, read_body : bool) -> Result<(u16, String), reqwest::Error> {
    let client = get_client().get().await;
    let response = client
        .post(url)
        .body(body)
        .version(reqwest::Version::HTTP_2)
        .send()
        .await?;
    let status_code = response.status().as_u16();
    if read_body {
        let resp_json: serde_json::Value = response
            .json()
            .await?;
        Ok((status_code, resp_json.to_string()))
    } else {
        Ok((status_code, "{}".to_string()))
    }
}

pub async fn http2_client_post(url : String, body : String, read_body : bool) -> (u16, String) {
    let x = reqwest_post(url.clone(), body.clone(), read_body).await;
    match x {
        Ok((status, body)) => {
            return (status, body);
        }
        Err(e) => {
            error!("reqwest {} failed, err {}\n", url, e);
        }
    }
    return (0, "".to_string());
}

const HTTP2_CLIENTS_POOL_SIZE  : usize = 1;

struct Http2ClientPoolClients {
    pool : Vec<Arc<reqwest::Client>>,
    idx : usize,
}

impl Http2ClientPoolClients {
    pub fn new() -> Self {
        let mut n = vec![];
        let max = HTTP2_CLIENTS_POOL_SIZE;
        for _ in 0..max {
            n.push(Arc::new(Self::new_node()))
        }
        Http2ClientPoolClients{
            pool: n,
            idx : 0,
        }
    }

    fn new_node() -> reqwest::Client {
        match reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .tls_sni(false)
            .http2_prior_knowledge()
            .build() {
            Ok(v) => { v }
            Err(e) => {
                error!("http2 client new now failed, err : {:?}\n", e);
                std::process::exit(-1);
            }
        }
    }

    pub fn get(&mut self) -> Arc<reqwest::Client> {
        if self.idx >= HTTP2_CLIENTS_POOL_SIZE {
            self.idx = 0;
        }
        let ixx = self.idx;
        self.idx += 1;
        self.pool[ixx].clone()
    }
}

pub struct Http2ClientPool {
    clients : Mutex<Http2ClientPoolClients>,
}

impl Http2ClientPool {
    pub fn new() -> Self {
        Self {
            clients : Mutex::new(Http2ClientPoolClients::new()),
        }
    }

    pub async fn get(&self) -> Arc<reqwest::Client> {
        let mut x = self.clients.lock().await;
        x.get()
    }
}

lazy_static!(
  static ref SINGLETON_INSTANCE : Http2ClientPool = Http2ClientPool::new();
);

pub fn get_client() -> & 'static Http2ClientPool {
    &*SINGLETON_INSTANCE
}
