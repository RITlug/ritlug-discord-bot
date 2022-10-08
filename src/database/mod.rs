pub mod roles;
pub mod auth;
pub mod guild;

use rusqlite::Result;
pub fn init() -> Result<()> {
  roles::init()?;
  auth::init()?;
  guild::init()?;
  Ok(())
}