#[cfg(test)]
#[macro_use(lazy_static)]
extern crate lazy_static;

#[cfg(test)]
pub mod qcli;
#[cfg(test)]
pub mod quibitous;
#[cfg(all(test, feature = "network"))]
pub mod networking;
#[cfg(all(test, feature = "non-functional"))]
pub mod non_functional;

pub mod startup;
