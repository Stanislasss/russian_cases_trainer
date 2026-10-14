//! Use cases and the ports (traits) they need from the outside world.
//! Depends only on `domain`; adapters live in `infrastructure` and `api`.

pub mod auth;
pub mod game;
pub mod ports;
pub mod speech;
