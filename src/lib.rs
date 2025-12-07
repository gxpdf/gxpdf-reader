#![warn(clippy::dbg_macro)]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]
#![warn(clippy::todo)]
#[macro_use]
extern crate rust_i18n;
i18n!("locales", fallback = "en");

mod components;
mod utils;
mod gxview;
mod gxdocument;
pub use components::app::App;
