use iron::Request;
use iron::IronResult;

pub trait FromReq<T> : Sized {
    fn from_req(req: &Request) -> IronResult<Self> ;
}