use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

use std::error::Error;
use std::io;

pub fn ignore_would_block(e: Box<dyn Error>) -> Result<(), Box<dyn Error>> {
    if let Some(io_err) = e.downcast_ref::<io::Error>() {
        if io_err.kind() == io::ErrorKind::WouldBlock {
            Ok(())
        } else {
            Err(e)
        }
    } else {
        Err(e)
    }
}
