use core::Logger;

use log::{debug, error, info, log, warn};

fn main() {
    Logger::initialize();
    info!("hi");
    debug!("bye");
    error!("sigh");
}
