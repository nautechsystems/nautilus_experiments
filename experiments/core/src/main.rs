use core::Logger;

use log::{debug, error, info};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    let _handle = Logger::initialize();
    info!("hi");
    debug!("bye");
    error!("sigh");
    let no_target: String = "nononono".to_string();
    info!(target: "nono", "yeehaw");
    // info!(target: no_target, "yeehaw");
    info!(component = no_target; "yeehaw");
}
