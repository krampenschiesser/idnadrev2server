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

#[cfg(debug_assertions)]
//#[options("/repo/<repo_id>")]
pub fn open_repo_ping(req: &mut Request) -> IronResult<Response>{
    let response = Response::with(status::Ok);
    response.headers.set()
    Response::build()
        .raw_header("Access-Control-Allow-Origin", "http://localhost:3000")
        .raw_header("Access-Control-Allow-Methods", "POST")
        .raw_header("Access-Control-Allow-Headers", "content-type")
        .status(Status::Ok)
        .finalize()
}

#[cfg(debug_assertions)]
//#[options("/repo")]
pub fn create_repository(req: &mut Request) -> IronResult<Response> {
    Response::build()
        .raw_header("Access-Control-Allow-Origin", "http://localhost:3000")
        .raw_header("Access-Control-Allow-Methods", "POST")
        .raw_header("Access-Control-Allow-Headers", "content-type")
        .status(Status::Ok)
        .finalize()
}

#[cfg(debug_assertions)]
//#[options("/repo/<repo_id>/file")]
pub fn create_file(req: &mut Request) -> IronResult<Response> {
    Response::build()
        .raw_header("Access-Control-Allow-Origin", "http://localhost:3000")
        .raw_header("Access-Control-Allow-Methods", "POST")
        .raw_header("Access-Control-Allow-Headers", "content-type,token")
        .status(Status::Ok)
        .finalize()
}