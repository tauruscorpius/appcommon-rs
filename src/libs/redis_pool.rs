use std::time::Duration;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use r2d2_redis_cluster::r2d2::{Pool, PooledConnection};
use r2d2_redis_cluster::RedisClusterConnectionManager;
use crate::libs::types;

fn build_pool(uri : Vec<String>, pool_size : u32) -> types::Result<Pool<RedisClusterConnectionManager>>{
    let redis_uri = uri
        .iter()
        .map(|x| x.as_str())
        .collect::<Vec<&str>>();
    let manager = match RedisClusterConnectionManager::new(redis_uri) {
        Ok(v) => { v }
        Err(e) => {
            return Err(e.to_string())?;
        }
    };
    let pool = match Pool::builder().max_size(pool_size)
        .connection_timeout(Duration::from_secs(15))
        .build(manager) {
        Ok(v) => {v }
        Err(e) => {
            return Err(e.to_string())?;
        }
    };
    Ok(pool)
}

pub struct RedisPool {
    pool : Mutex<Option<Pool<RedisClusterConnectionManager>>>,
}

impl RedisPool {
    const MAX_REDIS_POOL : u32 = 128;
    pub fn new() -> Self{
        Self{
            pool: Default::default(),
        }
    }
    pub async fn init_pool(&self, uri : Vec<String>, pool_size : u32) -> types::Result<()> {
        let mut size = pool_size;
        if size <= 0 {
            size = 1;
        }
        if size > Self::MAX_REDIS_POOL {
            size = Self::MAX_REDIS_POOL
        }
        let x = build_pool(uri, size)?;
        let mut inner = self.pool.lock().await;
        *inner = Some(x);
        Ok(())
    }
    pub async fn get(&self) -> types::Result<PooledConnection<RedisClusterConnectionManager>> {
        let inner = self.pool.lock().await;
        let conn = match inner.clone().unwrap().get() {
            Ok(v) => { v }
            Err(e) => {
                return Err(e.to_string())?;
            }
        };
        Ok(conn)
    }
}

lazy_static!(
  static ref REDIS_POOL_INSTANCE : RedisPool = RedisPool::new();
);

pub fn get_redis_pool() -> &'static RedisPool { &*REDIS_POOL_INSTANCE }
