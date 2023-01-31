pub mod roles;
pub mod auth;
pub mod avatar;

use rusqlite::Result;
pub fn init() -> Result<()> {
    roles::init()?;
    auth::init()?;
    avatar::init()?;
    Ok(())
}
