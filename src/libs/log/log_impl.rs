use std::str::FromStr;
use lazy_static::lazy_static;
use log4rs::{
    Handle,
    config::{
        Appender, Config, Root,
    },
    encode::{
        pattern::{
            PatternEncoder,
        }
    }
};
use log::{error, info, LevelFilter};
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::{FixedWindowRoller};
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::filter::threshold::ThresholdFilter;
use tokio::sync::Mutex;
use crate::libs::utility;

pub struct SysLogger {
    handle : Mutex<Option<Handle>>,
}

impl SysLogger {
    pub fn new() -> Self {
        Self{
            handle : Default::default()
        }
    }
    pub fn set_log_level(arg : String) {
        let level = i32::from_str(&arg);
        match level {
            Ok(v) => {
                let mut f = true;
                match v {
                    0 =>  { log::set_max_level(log::LevelFilter::Off); }
                    1 =>  { log::set_max_level(log::LevelFilter::Error); }
                    2 =>  { log::set_max_level(log::LevelFilter::Warn); }
                    3 =>  { log::set_max_level(log::LevelFilter::Info); }
                    4 =>  { log::set_max_level(log::LevelFilter::Debug); }
                    5 =>  { log::set_max_level(log::LevelFilter::Trace); }
                    _ => {
                        f = false;
                    }
                }
                if f {
                    info!("succeed set loglevel : level {} \n", v);
                }else{
                    error!("failed set loglevel : level {} \n", v);
                }
            }
            Err(e) => {
                error!("set log level failed, invalid loglevel {}, err {}\n", arg, e);
            }
        }
    }

    pub async fn init_log(&self, log_name : &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let config = self.build_config(log_name, LevelFilter::Trace).unwrap();
        return match log4rs::init_config(config) {
            Ok(v) => {
                let mut x = self.handle.lock().await;
                *x = Some(v);
                Ok(())
            }
            Err(e) => {
                Err(format!("{}\n", e.to_string()))?
            }
        }
    }

    fn build_config(&self, log_name : &str, log_level : LevelFilter) -> Result<Config, Box<dyn std::error::Error + Send + Sync>> {
        let home = utility::get_home();
        let file_path = format!("{}/log/{}.log", home, log_name);
        let file_path_roll = format!("{}/log/{}{{}}.log", home, log_name);

        let window_size = 20;
        let fixed_window_roller = FixedWindowRoller::builder().build(&file_path_roll,window_size).unwrap();

        let size_limit = 500 * 1024 * 1024;
        let size_trigger = SizeTrigger::new(size_limit);
        let compound_policy = CompoundPolicy::new(Box::new(size_trigger),Box::new(fixed_window_roller));

        match Config::builder()
            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(log_level)))
                    .build(
                        "logfile",
                        Box::new(
                            RollingFileAppender::builder()
                                .encoder(Box::new(PatternEncoder::new("{d} {l} {M}:{L} - {m}{n}")))
                                .build(file_path, Box::new(compound_policy))?,
                        ),
                    ),
            )
            .build(
                Root::builder()
                    .appender("logfile")
                    .build(log_level),
            ) {
            Ok(v) => {
                Ok(v)
            }
            Err(e) => {
                Err(format!("{}\n", e.to_string()))?
            }
        }
    }
}

lazy_static!(
  static ref SINGLETON_INSTANCE : SysLogger = SysLogger::new();
);

pub fn get_logger() -> & 'static SysLogger {
    &*SINGLETON_INSTANCE
}
