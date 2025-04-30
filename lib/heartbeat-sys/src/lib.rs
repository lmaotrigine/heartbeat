#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::redundant_allocation,
    clippy::cargo
)]

#[cfg(feature = "cli")]
pub mod cli;
#[cfg(feature = "fs")]
pub mod fs;
mod home;
#[cfg(feature = "process")]
pub mod process;
pub use home::*;
pub mod build;
pub mod utils;
