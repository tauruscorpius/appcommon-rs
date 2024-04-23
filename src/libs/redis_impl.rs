extern crate redis_cluster_rs;
use std::fmt::{Debug};
use chrono::{DateTime, Duration, Local};
use redis_cluster_rs::{Client, Commands, Connection, RedisResult};
//use redis::AsyncCommands;
use serde::{Serialize, Deserialize};
use crate::libs::types;
use log::{error, info, debug};
use redis::cluster::ClusterClient;
use redis::cluster_async::ClusterConnection;
use redis_cluster_rs::redis::{ErrorKind};
use crate::libs::config::get_config;
use crate::libs::json::json_impl;

pub trait RedisKeyMaker {
    fn key(&self) -> String;
}

pub trait RedisKeyPattern {
    fn pattern() -> String;
}

pub trait RedisScoreMemberMaker {
    fn member(&self) -> String;
    fn set_member(&mut self, m : &str);
    fn score(&self) -> i64;
}

pub trait RedisKeyIndex {
    fn index(&self) -> i64;
}

pub struct RedisOp {}

impl RedisOp {
    // todo : async cluster client pool use crate bb8-redis-cluster
    pub async fn _connect_async() -> types::Result<ClusterConnection> {
        let vec = get_config().lock().await.get_redis_config();
        info!("redis nodes : {:?}\n", vec);
        let init_nodes = vec.iter().map(|a| a.as_str()).collect();
        match ClusterClient::new(init_nodes) {
            Ok(v) => {
                match v.get_async_connection().await {
                    Ok(v1) => {
                        return Ok(v1);
                    }
                    Err(e) => {
                        Err(e.to_string())?
                    }
                }
            }
            Err(e) => {
                Err(e.to_string())?
            }
        }
    }

    pub async fn connect() -> types::Result<Connection> {
        let vec = get_config().lock().await.get_redis_config();
        info!("redis nodes : {:?}\n", vec);
        let init_nodes = vec.iter().map(|a| a.as_str()).collect();
        match Client::open(init_nodes) {
            Ok(v) => {
                match v.get_connection() {
                    Ok(v1) => {
                        return Ok(v1);
                    }
                    Err(e) => {
                        Err(e.to_string())?
                    }
                }
            }
            Err(e) => {
                Err(e.to_string())?
            }
        }
    }

    pub async fn reconnect(c : &mut Connection, last : &mut DateTime<Local>, ty : &str) -> bool {
        let now = Local::now();
        if now.signed_duration_since(*last) <= Duration::seconds(1) {
            return true;
        }
        if c.check_connection() {
            return true
        }
        let f = match Self::connect().await {
            Ok(v) => {
                v
            }
            Err(e) => {
                error!("type[{}] error re-connect redis op : {:?}\n", ty, e);
                return false;
            }
        };
        *c = f;
        info!("type[{}] redis re-connection succeed\n", ty);
        true
    }

    pub async fn get<'de, T>(redis_op : &mut Connection, stx: &'de mut String, data: &mut T) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
        where T : Debug + Deserialize<'de> + RedisKeyMaker
    {
        match redis_op.get(data.key()) {
            Ok(v) => {
                *stx = v;
            }
            Err(x) => {
                Err(format!("get key failed {}\n", x.to_string()))?
            }
        }
        match json_impl::unmarshal(stx, data) {
            Ok(_) => {
                info!("get : {:?}\n", &data);
                return Ok(())
            }
            _ => {
                Err("get data failed\n")?
            }
        }
        Ok(())
    }

    pub async fn set<T>(redis_op : &mut Connection, data : &T) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
        where T : Debug + Serialize + RedisKeyMaker
    {
        info!("set : {:?}\n", &data);
        let j = json_impl::marshal(data).unwrap();
        let _:() = redis_op.set(data.key(), j).unwrap();
        Ok(())
    }

    pub async fn del<T>(redis_op : &mut Connection, data : &T) -> Result<(), Box<dyn std::error::Error>>
        where T : Debug + RedisKeyMaker
    {
        info!("del : {:?}\n", &data);
        let _:() = redis_op.del(data.key()).unwrap();
        Ok(())
    }

    pub async fn l_push<T>(redis_op: &mut Connection, data : &T) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
        where T : Debug + Serialize + RedisKeyMaker
    {
        let key = data.key();
        info!("l_push#{} : {:?}\n", key.clone(), &data);
        let j = json_impl::marshal(data).unwrap();
        let _:() = redis_op.lpush(key, j).unwrap();
        Ok(())
    }

    pub async fn r_pop<'de, T>(redis_op : &mut redis_cluster_rs::Connection, stx: &'de mut String, data : &mut T) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>
        where T : Debug + Deserialize<'de> + RedisKeyMaker
    {
        //let mut c = redis_pool::get_redis_pool().connection().await;
        let key = data.key();
        match redis_op.rpop(key.clone()) {
            Ok(v) => {
                *stx = v;
            }
            Err(x) => {
                if x.kind() != ErrorKind::TypeError {
                    Err(format!("r_pop#{} resp failed {}\n", key, x.to_string()))?
                }
                return Ok(false);
            }
        }
        match json_impl::unmarshal(stx, data) {
            Ok(_) => {
                info!("r_pop#{} : {:?}\n", key, &data);
                return Ok(true);
            }
            _ => {
                Err(format!("r_pop#{} unmarshal data failed\n", key))?
            }
        }
        Ok(false)
    }

    pub async fn br_pop<'de, T>(redis_op : &mut Connection,
                                time_out : usize,
                                stx: &'de mut String,
                                data : &mut T
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
        where T : Debug + Deserialize<'de> + RedisKeyMaker
    {
        let key = data.key();
        let result: RedisResult<Option<(String, String)>> = redis_op.brpop(key.clone(), time_out);
        match result {
            Ok(Some((_, element))) => {
                debug!("br_pop #{} ok value {}\n", key, element);
                *stx = element;
            }
            Ok(None) => {
                Err(format!("br_pop#{} key none\n", key))?
            }
            Err(x) => {
                Err(format!("br_pop#{} key failed {}\n", key, x.to_string()))?
            }
        }
        match json_impl::unmarshal(stx, data) {
            Ok(_) => {
                info!("br_pop#{} : {:?}\n", key, &data);
                return Ok(())
            }
            _ => {
                Err(format!("br_pop#{} data failed\n", key))?
            }
        }
        Ok(())
    }

    pub async fn l_len<T>(redis_op : &mut Connection, data : &T) -> Result<i64, Box<dyn std::error::Error + Send + Sync>>
        where T : Debug + RedisKeyMaker
    {
        let key = data.key();
        info!("len : key={}\n", key);
        let l : i64 = redis_op.llen(key).unwrap();
        Ok(l)
    }

    pub async fn z_add<T>(redis_op : &mut Connection, data : &T) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
        where T : Debug + RedisKeyMaker + RedisScoreMemberMaker
    {
        let key = data.key();
        info!("z_add : key={} member={:?}\n", key, &data);
        let _:() = redis_op.zadd(key, data.member(), data.score()).unwrap();
        Ok(())
    }

    pub async fn z_rem<T>(redis_op : &mut Connection, data : &T) -> Result<(), Box<dyn std::error::Error>>
        where T : Debug + RedisKeyMaker + RedisScoreMemberMaker
    {
        let key = data.key();
        info!("z_rem : key={} member={:?}\n", key, &data);
        let _:() = redis_op.zrem(key, data.member()).unwrap();
        Ok(())
    }

    pub async fn z_range_by_score<T>(redis_op : &mut Connection,
                                     data : &T,
                                     min : String,
                                     max : String,
                                     offset : isize,
                                     page : isize
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>
        where T : Debug + Serialize + RedisKeyMaker
    {
        let key = data.key();
        debug!("z_range_by_score :  key={} min = {} max = {} filter={:?}\n", key, min, max, &data);
        let ret = redis_op.zrangebyscore_limit(key, min, max, offset, page);
        match ret  {
            Ok(v) => {
                return Ok(v)
            }
            Err(x) => {
                Err(format!("z_range_by_score key failed {}\n", x.to_string()))?
            }
        }
        Ok(vec![])
    }

    pub async fn z_count<T>(redis_op : &mut Connection,
                            data : &T,
                            min : String,
                            max : String
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>
        where T : Debug + Serialize + RedisKeyMaker
    {
        let key = data.key();
        debug!("z_count :  key={} filter={:?}\n", key, &data);
        let  ret = redis_op.zcount(key, min, max);
        match ret  {
            Ok(v) => {
                return Ok(v);
            }
            Err(x) => {
                Err(format!("z_counter key failed {}\n", x.to_string()))?
            }
        }
        Ok(0)
    }
}