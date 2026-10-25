//! Production adapter + unchanged pinned beta common/SDK/NativeSessions.
//! Tauri event emitter and installation policy are explicit OS/policy stubs.
extern crate self as tauri;
#[derive(Clone)]
pub struct AppHandle;
pub trait Emitter {
    fn emit(&self, channel: &str, payload: serde_json::Value) -> Result<(), String>;
}
impl Emitter for AppHandle {
    fn emit(&self, _channel: &str, _payload: serde_json::Value) -> Result<(), String> {
        Ok(())
    }
}
#[cfg(not(test))]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
#[cfg(not(test))]
pub use native_core::common::{CoreContext, CorePaths, CoreResult};
#[cfg(not(test))]
pub use native_core::extensions::{direct, dispatch_supports, Sessions};
