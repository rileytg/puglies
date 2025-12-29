// AIDEV-NOTE: API module - REST clients for Polymarket

mod clob;
mod gamma;
pub mod order;

#[cfg(test)]
mod tests;

pub use clob::ClobClient;
pub use gamma::GammaClient;
