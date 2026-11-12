//! Policy: the versions of a site, whether a build may write into an output folder, and the order of the versions.

mod output_safety;
mod version_order;

pub(crate) use output_safety::{Moment, output_safety};
pub(crate) use version_order::order_versions;
