//!
//! The Zinc server binary error.
//!

use std::io;

use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "server binding: {}", _0)]
    ServerBinding(io::Error),
    #[fail(display = "server runtime: {}", _0)]
    ServerRuntime(io::Error),
}