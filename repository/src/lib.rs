#![feature(try_from)]

extern crate log;
extern crate quick_protobuf;
extern crate failure;
extern crate uuid;
extern crate chacha20_poly1305_aead;
extern crate sha1;

mod pb;
mod files;
mod repository;
mod sync;
mod crypt;