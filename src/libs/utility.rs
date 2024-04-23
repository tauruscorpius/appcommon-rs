use std::env;
use lazy_static::lazy_static;
use rand;
use rand::Rng;
use tokio::sync::Mutex;

fn create_session_u64_func() -> Box<dyn FnMut() -> u64 + Sync + Send>
{
    let mut rng = rand::thread_rng();
    let x : u32 = rng.gen::<u32>() & 0x7FFFffff;
    let mut y : u32 = 0;
    Box::new(move || -> u64{
        y += 1;
        ((x as u64) << 32) + y as u64
    })
}

pub fn get_home() -> String {
    let home = match env::var("HOME") {
        Ok(v) =>  v,
        Err(e) => {
            println!("cant find HOME env, err : {}\n", e);
            return "".to_string();
        }
    };
    home
}

lazy_static!(
  static ref SESSION_ID_INSTANCE : Mutex<Box<dyn FnMut() -> u64 + Send + Sync>> = Mutex::new(create_session_u64_func());
);

pub async fn get_session_id() -> u64 {
    let mut x = SESSION_ID_INSTANCE.lock().await;
    x()
}