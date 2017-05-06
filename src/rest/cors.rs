// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use rocket::{Route, Data, Outcome, Request, Response};
use rocket::response::Body;
use rocket::handler;
use rocket::http::{Header, Status, Method};
use state::GlobalState;
use crypt::CryptoActor;
use crypt::actor::dto::{RepositoryDescriptor, EncTypeDto, RepositoryDto, AccessToken};
use serde_json::to_string;
use rocket_contrib::UUID;
use rocket::State;
use rocket::response::Stream;
use rocket_contrib::{JSON};
use std::io::Cursor;
use std::sync::{RwLock, Arc};
use uuid::Uuid;


#[cfg(debug_assertions)]
#[options("/repo/<repo_id>")]
pub fn open_repo_ping(repo_id: UUID) -> Response<'static> {
    Response::build()
        .raw_header("Access-Control-Allow-Origin", "http://localhost:3000")
        .raw_header("Access-Control-Allow-Methods", "POST")
        .raw_header("Access-Control-Allow-Headers", "content-type")
        .status(Status::Ok)
        .finalize()
}

#[cfg(debug_assertions)]
#[options("/repo")]
pub fn create_repository() -> Response<'static> {
    Response::build()
        .raw_header("Access-Control-Allow-Origin", "http://localhost:3000")
        .raw_header("Access-Control-Allow-Methods", "POST")
        .raw_header("Access-Control-Allow-Headers", "content-type")
        .status(Status::Ok)
        .finalize()
}

#[cfg(debug_assertions)]
#[options("/repo/<repo_id>/file")]
pub fn create_file(repo_id: UUID) -> Response<'static> {
    Response::build()
        .raw_header("Access-Control-Allow-Origin", "http://localhost:3000")
        .raw_header("Access-Control-Allow-Methods", "POST")
        .raw_header("Access-Control-Allow-Headers", "content-type,token")
        .status(Status::Ok)
        .finalize()
}