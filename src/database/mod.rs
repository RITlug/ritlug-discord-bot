pub mod roles;

use rusqlite::Result;
pub fn init() -> Result<()> {
  roles::init()?;
  Ok(())
}