pub mod roles;
pub mod auth;

use rusqlite::Result;
pub fn init() -> Result<()> {
  roles::init()?;
  auth::init()?;
  Ok(())
}