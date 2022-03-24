#[cfg(all(test, feature = "evm"))]
#[macro_use]
extern crate quickcheck;

pub mod qcli_lib;
pub use crate::qcli_lib::*;
