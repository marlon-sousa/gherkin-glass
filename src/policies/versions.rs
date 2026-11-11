//! Policy: the versions of a site, whether a build may write into a folder, the order of the versions, and the versions page.

mod folder_verdict;
mod version_order;
mod versions_page;

pub(crate) use folder_verdict::{Moment, folder_verdict};
pub(crate) use version_order::order_versions;
pub(crate) use versions_page::{VERSIONS_PAGE, render_versions_page};
