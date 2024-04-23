use std::time::Duration;
use etcd_rs;
use etcd_rs::{Client, ClientConfig, Endpoint, LeaseId, LeaseGrantRequest, LeaseOp, PutRequest, KeyValueOp, LeaseKeepAlive, LeaseRevokeRequest, TxnRequest, TxnCmp, KeyRange, RangeRequest, TxnOp};
use log::{error, info};
use crate::libs::app::app_inst::{AppStatus, get_app_instance};
use crate::libs::types;

#[derive(Default)]
pub struct EtcdInst {}

impl EtcdInst {
  pub const TTL_LEASE : i64 = 10;
  pub async fn create_etcd_client(endpoints : Vec<String>) -> types::Result<Client> {
    let etcd_endpoints = endpoints
        .iter()
        .map(|x| Endpoint::new(x))
        .collect::<Vec<Endpoint>>();
    let client_res = Client::connect(ClientConfig::new(etcd_endpoints)).await;
    let cli = match client_res {
      Ok(v) => { v }
      Err(e) => { return Err(e.to_string())?; }
    };
    Ok(cli)
  }

  async fn create_lease(client: &Client, ttl: i64) -> Result<LeaseId, Box<dyn std::error::Error>> {
    let lease_request = LeaseGrantRequest::new(Duration::from_secs(ttl as u64));
    let lease_response = match client.grant_lease(lease_request).await {
      Ok(v) => { v }
      Err(e) => {
        return Err(e.to_string())?;
      }
    };
    Ok(lease_response.id)
  }

  pub async fn create_lease_id(client: &Client,) -> types::Result<LeaseId> {
    let lease_id = match Self::create_lease(&client, Self::TTL_LEASE).await {
      Ok(v) => { v }
      Err(e) => {
        return Err(e.to_string())?;
      }
    };
    Ok(lease_id)
  }

  pub async fn acquire_lock(
    client: &Client,
    key: &str,
    lease_id: LeaseId,
  ) -> types::Result<()> {
    let key_range = KeyRange::range(key, "");
    let put_request = PutRequest::new(key, "lock_holder_value")
        .lease(lease_id)
        .prev_kv(true);
    let range_request = RangeRequest::new(key_range.clone());
    let txn_request = TxnRequest::new()
        .when_create_revision(key_range.clone(), TxnCmp::Equal, 0)
        .and_then(TxnOp::Put(put_request))
        .or_else(TxnOp::Range(range_request));
    match client.txn(txn_request).await {
      Ok(v) => {
        info!("acquire_lock# pull response : {:?}\n", v);
        if !v.succeeded {
          return Err("acquire_lock# txn response with unsuccessful".to_string())?;
        }
      }
      Err(e) => {
        return Err(e.to_string())?;
      }
    }
    Ok(())
  }

  pub async fn release_lock(
    client: &Client,
    key: &str,
    lease_id: LeaseId,
  ) -> types::Result<()> {
    client.delete(key).await?;
    let revoke_req = LeaseRevokeRequest::new(lease_id);
    match client.revoke(revoke_req).await {
      Ok(v) => {
        info!("release_lock# pull response : {:?}\n", v);
      }
      Err(e) => {
        return Err(e.to_string())?
      }
    }
    Ok(())
  }

  async fn send_keep_alive(lease_alive : &mut LeaseKeepAlive) -> Option<etcd_rs::Error> {
      match lease_alive.keep_alive().await {
          Ok(v) => {
              info!("lease keep alive response : {:?}\n", v);
          }
          Err(e) => {
            error!("lease keep alive error : {:?}\n", e);
            return Some(e);
          }
      }
    None
  }

  async fn create_lease_alive_client(client: &Client, lease_id: LeaseId) -> LeaseKeepAlive {
    loop {
      match client.keep_alive_for(lease_id).await {
        Ok(v) => { return v; }
        Err(e) => {
          error!("create keep_alive_for failed, err {}\n", e);
          tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
          continue
        }
      };
    }
  }

  pub async fn keep_lease_alive(client: &Client, lease_id: LeaseId, secs : u64) {
    info!("start keep lease alive, lease_id {}\n", lease_id);
    let mut lease_alive = Self::create_lease_alive_client(client, lease_id).await;

    loop {
      tokio::time::sleep(tokio::time::Duration::from_secs(secs)).await;
      let exit_status = match get_app_instance().get_app_status().await {
        AppStatus::EXITING(s) => {
          info!("exiting status : cause {}\n", s);
          true
        }
        AppStatus::EXITED => {
          true
        }
        _ => {
          false
        }
      };
      if exit_status {
        info!("system exit status, stop keep lease alive, lease_id {}\n", lease_id);
        break;
      }
      // Send keep alive request
      let er = match Self::send_keep_alive(&mut lease_alive).await {
        Some(v) => { v }
        None => {
          continue
        }
      };
      match er {
        etcd_rs::Error::ChannelClosed => {
          lease_alive = Self::create_lease_alive_client(client, lease_id).await;
        }
        _ => {}
      }
    }
  }

}
