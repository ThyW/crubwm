use std::fs::{File, OpenOptions};
use std::io::{stderr, stdout, Write};
use std::os::unix::prelude::AsFd;
use std::os::unix::prelude::OwnedFd;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Mutex,
};

use crate::config::WmResult;

pub const LL_OFF: u8 = 0;
pub const LL_NORMAL: u8 = 1;
pub const LL_ALL: u8 = 2;

pub const LF_STDOUT: &str = "STDOUT";
pub const LF_STDERR: &str = "STDERR";

static mut LOG_LEVEL: AtomicU8 = AtomicU8::new(LL_OFF);
static mut WRITER: Mutex<Option<OwnedFd>> = Mutex::new(None);

pub fn prepare_logger(file: &impl AsRef<str>, level: u8) -> WmResult {
    if level >= 3 {
        return Err("Invalid log level: {level}".into());
    }
    unsafe {
        LOG_LEVEL.store(level, Ordering::Relaxed);
    }
    let writer = Mutex::new(None);
    let fname = file.as_ref();
    if fname == LF_STDOUT {
        if let Ok(mut guard) = writer.lock() {
            let fd = stdout().as_fd().try_clone_to_owned()?;
            *guard = Some(fd);
            println!("{guard:#?}");
            drop(guard)
        }
    } else if fname == LF_STDERR {
        if let Ok(mut guard) = writer.lock() {
            let fd = stderr().as_fd().try_clone_to_owned()?;
            let _ = guard.insert(fd);
        }
    } else if let Ok(file) = OpenOptions::new().write(true).create(true).open(fname) {
        {
            if let Ok(mut guard) = writer.lock() {
                let fd = file.into();
                let _ = guard.insert(fd);
            }
        }
    }

    unsafe {
        WRITER = writer;
    }

    Ok(())
}

pub fn log<T: AsRef<str> + ?Sized>(msg: &T, level: u8) -> bool {
    unsafe {
        if level >= LOG_LEVEL.load(Ordering::Relaxed) && level != LL_OFF {
            if let Ok(guard) = WRITER.lock() {
                if guard.is_some() {
                    let fd = guard.as_ref().unwrap();
                    if let Ok(cloned) = fd.try_clone() {
                        let mut file = File::from(cloned);
                        if writeln!(&mut file, "[LOG] {}", msg.as_ref()).is_ok() {
                            return file.flush().is_ok();
                        } else {
                            return false;
                        }
                    }
                }
            }
        }
    }

    false
}
