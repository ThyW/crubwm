#[derive(Debug)]
pub enum Error {
    Generic(String),
    Io(std::io::Error),
    ParseBool(std::str::ParseBoolError),
    ParseInt(std::num::ParseIntError),
    X11Connect(x11rb::errors::ConnectError),
    X11Connection(x11rb::errors::ConnectionError),
    X11Reply(x11rb::errors::ReplyError),
    Borrow(std::cell::BorrowError),
    BorrowMut(std::cell::BorrowMutError),
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

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{}", e),
            Self::Generic(e) => write!(f, "{}", e),
            Self::Borrow(e) => write!(f, "{}", e),
            Self::ParseInt(e) => write!(f, "{}", e),
            Self::ParseBool(e) => write!(f, "{}", e),
            Self::BorrowMut(e) => write!(f, "{}", e),
            Self::X11Connect(e) => write!(f, "{}", e),
            Self::X11Reply(e) => write!(f, "{}", e),
            Self::X11Connection(e) => write!(f, "{}", e)
        }
    }
}

impl std::error::Error for Error {}

pub type WmResult<T = ()> = Result<T, Error>;
