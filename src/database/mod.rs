mod roles;
pub use roles::get_page;
pub use roles::set_page;
pub use roles::delete_page;
pub use roles::get_page_amount;

use rusqlite::Result;

pub fn init() -> Result<()> {
  roles::init()?;
  Ok(())
}