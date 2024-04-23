use signal_hook::{iterator::Signals};
use signal_hook::consts::{SIGINT, SIGTERM};
use log::{warn};
use tokio::time;
use crate::libs::app::app_inst;
use crate::libs::app::app_inst::AppStatus;
use crate::libs::register;

// signaling hook function
pub async fn waiting_signal_term() {
    let mut signals = Signals::new(&[SIGTERM, SIGINT]).unwrap();
    for signal in signals.forever() {
        match signal {
            SIGINT | SIGTERM  => {
                println!("received signal {:?}\n", signal);
                warn!("received sig {:?} , system exiting now\n", signal);
                app_inst::get_app_instance().set_app_status(AppStatus::EXITING(signal.to_string())).await;
                register::node_register_impl::get_register().set_exit().await;
                warn!("system exited status, waiting post procedure\n");
                //todo : do others
                app_inst::get_app_instance().set_app_status(AppStatus::EXITED).await;
                time::sleep(time::Duration::from_secs(1)).await;
                std::process::exit(0);
            }
            _ => unreachable!(),
        }
    }
}
