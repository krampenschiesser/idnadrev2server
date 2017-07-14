use iron::Request;
use iron::IronResult;
use std::fmt::{Debug, Display, Result, Formatter};
use std::error::Error;

pub trait FromReq<T>: Sized {
    fn from_req(req: &Request) -> IronResult<T>;
}

pub trait FromMutReq<T>: Sized {
    fn from_req(req: &mut Request) -> IronResult<T>;
}

pub trait FromPostReq<T>: Sized {
    fn from_post_req(req: &mut Request) -> IronResult<T>;
}

#[derive(Debug)]
pub struct StringError {
    msg: String
}

impl StringError {
    pub fn new<T>(msg: T) -> Self
        where T: Into<String> {
        StringError { msg: msg.into() }
    }
}

impl Display for StringError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for StringError {
    fn description(&self) -> &str { &*self.msg }
}