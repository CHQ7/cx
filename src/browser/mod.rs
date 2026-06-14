// Browser module - browser automation driver and session management
pub mod driver;
pub mod session;

pub use driver::{TMWebDriver, JsResult};
pub use session::{Session, SessionType};
