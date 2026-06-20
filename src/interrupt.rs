use std::sync::Once;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::{Error, Result};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);
static INSTALL_HANDLER: Once = Once::new();

pub fn install_handler() -> Result<()> {
    let mut result = Ok(());

    INSTALL_HANDLER.call_once(|| {
        result = ctrlc::set_handler(|| {
            INTERRUPTED.store(true, Ordering::SeqCst);
        })
        .map_err(Error::SignalHandler);
    });

    result?;
    INTERRUPTED.store(false, Ordering::SeqCst);
    Ok(())
}

pub fn is_requested() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}
