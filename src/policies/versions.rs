//! Policy: the versions of a site, whether a build may write into an output folder, the order of the versions, and the versions page.

mod output_safety;
mod version_order;
mod versions_page;

pub(crate) use output_safety::{Moment, output_safety};
pub(crate) use version_order::order_versions;
pub(crate) use versions_page::{VERSIONS_PAGE, render_versions_page};
