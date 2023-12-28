use core::Logger;

use log::{debug, error, info, log, warn};

fn main() {
    Logger::initialize();
    info!("hi");
    debug!("bye");
    error!("sigh");
    let no_target: String = "nononono".to_string();
    info!(target: "nono", "yeehaw");
    // info!(target: no_target, "yeehaw");
    info!(component = no_target; "yeehaw");
}
