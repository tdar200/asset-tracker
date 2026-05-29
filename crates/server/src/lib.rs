//! Library surface for tests + the binary. Re-exports the router builder.
mod store;
mod api;

pub use api::build_router;
pub use store::Store;
