//! Policy: the versions of the site, the versions page and the order it lists them in.

mod version_order;
mod versions_page;

pub(super) use versions_page::build_versions_page;

use version_order::order_versions;
