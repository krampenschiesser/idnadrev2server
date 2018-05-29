#![feature(try_from)]
#![feature(universal_impl_trait)]

extern crate argon2rs;
extern crate chacha20_poly1305_aead;
#[macro_use]
extern crate failure;
extern crate log;
extern crate quick_protobuf;
extern crate sha1;
extern crate uuid;


mod pb;
mod files;
mod repository;
mod sync;
mod crypt;
mod error;