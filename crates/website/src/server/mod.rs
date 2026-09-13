pub mod auth;
pub mod data;
#[cfg(feature = "ssr")]
pub mod ical;
pub mod scraper;
pub mod squad;

#[cfg(feature = "ssr")]
mod state;
#[cfg(feature = "ssr")]
pub use state::*;
