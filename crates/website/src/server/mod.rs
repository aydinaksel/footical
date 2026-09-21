pub mod auth;
pub mod data;
pub mod scraper;
pub mod squad;

#[cfg(feature = "ssr")]
pub mod database;
#[cfg(feature = "ssr")]
pub mod ical;

#[cfg(feature = "ssr")]
mod state;
#[cfg(feature = "ssr")]
pub use state::*;
