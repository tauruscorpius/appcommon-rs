use log::error;
use backtrace::Backtrace;

pub async fn set_panic_hook() {
    // panic hook
    std::panic::set_hook(Box::new(|x| {
        eprint!("{:?}\n", x);
        let bt = Backtrace::new();
        error!("panic hook invoked, panic info : {:?}, backtrace: {:?}\n", x, bt);
        std::process::exit(-1);
    }));
}
