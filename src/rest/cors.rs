// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use iron::prelude::*;
use iron::status;
use hyper::header::{AccessControlAllowOrigin, AccessControlAllowHeaders};
use hyper::method::Method;
use unicase::UniCase;
use iron::headers::AccessControlAllowMethods;


#[cfg(debug_assertions)]
pub fn open_repo_ping(req: &mut Request) -> IronResult<Response> {
    create_response()
}

#[cfg(debug_assertions)]
pub fn create_repository(req: &mut Request) -> IronResult<Response> {
    create_response()
}

#[cfg(debug_assertions)]
pub fn create_file(req: &mut Request) -> IronResult<Response> {
    create_response()
}

fn create_response() -> IronResult<Response> {
    let response = Response::with(status::Ok);
    let ct = UniCase::new("content-type".into());
    response.headers.set(AccessControlAllowOrigin::Value("http://localhost:3000".into()));
    response.headers.set(AccessControlAllowMethods(vec![Method::Post]));
    response.headers.set(AccessControlAllowHeaders(vec![ct]));
    Ok(response)
}
