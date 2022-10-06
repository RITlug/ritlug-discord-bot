mod roles;
pub use roles::get_page;

pub fn init() {
  roles::init();
}