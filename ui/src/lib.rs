//! Common UI logics
//!
//! This module contains common UI logics for both platforms(web/desktop)
//!
//! The UI crate is responsible to control all the visible things in application UI. This crate
//! was built using the [egui](https://docs.rs/egui) immediate mode UI library. And accepting
//! two external interfaces as the infrastructure. Because this crate was built as a platform
//! agnostic module and we can not keep platform specific code in this module. So developers have
//! to pass platform specific functionalities as parameters.
//!
//! The entry-point to this library is `ui::Ui` struct.

pub mod state;
pub mod ui;
pub mod components;
pub mod commands;
pub mod scopes;
pub mod tab;
pub mod graphics;

// Infrastructure
pub mod external;
pub mod client;
