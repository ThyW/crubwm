use std::sync::PoisonError;

#[derive(Debug)]
pub enum Error {
    Generic(String),
    Io(std::io::Error),
    ParseBool(std::str::ParseBoolError),
    ParseInt(std::num::ParseIntError),
    FromUtf8(std::string::FromUtf8Error),
    Utf8(std::str::Utf8Error),
    X11Connect(x11rb::errors::ConnectError),
    X11Connection(x11rb::errors::ConnectionError),
    X11Reply(x11rb::errors::ReplyError),
    X11ReplyOrIdError(x11rb::errors::ReplyOrIdError),
    Borrow(std::cell::BorrowError),
    BorrowMut(std::cell::BorrowMutError),
    Null(std::ffi::NulError),
    Fmt(std::fmt::Error),
    SystemTime(std::time::SystemTimeError),
    Cairo(cairo::Error),
    HpError(hp::HpError),
    MutexPoison,
    NullPtr,
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_: PoisonError<T>) -> Self {
        Self::MutexPoison
    }
}

impl From<hp::HpError> for Error {
    fn from(e: hp::HpError) -> Self {
        Self::HpError(e)
    }
}

impl From<cairo::Error> for Error {
    fn from(e: cairo::Error) -> Self {
        Self::Cairo(e)
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(e: std::time::SystemTimeError) -> Self {
        Self::SystemTime(e)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(e: std::fmt::Error) -> Self {
        Self::Fmt(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<&'static str> for Error {
    fn from(e: &'static str) -> Self {
        Self::Generic(e.to_string())
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Self::Generic(e)
    }
}

impl From<std::str::ParseBoolError> for Error {
    fn from(e: std::str::ParseBoolError) -> Self {
        Self::ParseBool(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Self::ParseInt(e)
    }
}

impl From<x11rb::errors::ConnectError> for Error {
    fn from(e: x11rb::errors::ConnectError) -> Self {
        Self::X11Connect(e)
    }
}

impl From<x11rb::errors::ConnectionError> for Error {
    fn from(e: x11rb::errors::ConnectionError) -> Self {
        Self::X11Connection(e)
    }
}

impl From<x11rb::errors::ReplyError> for Error {
    fn from(e: x11rb::errors::ReplyError) -> Self {
        Self::X11Reply(e)
    }
}

impl From<std::cell::BorrowError> for Error {
    fn from(e: std::cell::BorrowError) -> Self {
        Self::Borrow(e)
    }
}
impl From<std::cell::BorrowMutError> for Error {
    fn from(e: std::cell::BorrowMutError) -> Self {
        Self::BorrowMut(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Self::FromUtf8(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Utf8(e)
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(e: std::ffi::NulError) -> Self {
        Self::Null(e)
    }
}

impl From<x11rb::errors::ReplyOrIdError> for Error {
    fn from(e: x11rb::errors::ReplyOrIdError) -> Self {
        Self::X11ReplyOrIdError(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "[ERR] {}", e),
            Self::Generic(e) => write!(f, "[ERR] {}", e),
            Self::Borrow(e) => write!(f, "[ERR] {}", e),
            Self::ParseInt(e) => write!(f, "[ERR] {}", e),
            Self::ParseBool(e) => write!(f, "[ERR] {}", e),
            Self::BorrowMut(e) => write!(f, "[ERR] {}", e),
            Self::X11Connect(e) => write!(f, "[ERR] {}", e),
            Self::X11Reply(e) => write!(f, "[ERR] {}", e),
            Self::X11Connection(e) => write!(f, "[ERR] {}", e),
            Self::FromUtf8(e) => write!(f, "[ERR] {}", e),
            Self::Utf8(e) => write!(f, "[ERR] {}", e),
            Self::Null(e) => write!(f, "[ERR] {}", e),
            Self::X11ReplyOrIdError(e) => write!(f, "[ERR] {}", e),
            Self::Fmt(e) => write!(f, "[ERR] {}", e),
            Self::SystemTime(e) => write!(f, "[ERR] {}", e),
            Self::Cairo(e) => write!(f, "[ERR] {}", e),
            Self::MutexPoison => write!(f, "[ERR] bar mutex has been poisoned."),
            Self::NullPtr => write!(f, "[ERR] a pointer expected to be not null is null"),
            Self::HpError(e) => write!(f, "[ERR] {}", e),
        }
    }
}

impl std::error::Error for Error {}

pub type WmResult<T = ()> = Result<T, Error>;
