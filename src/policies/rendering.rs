//! Policy: the rendering stage, the site writer's: the shell of a site, a version or a preview, with its front page from its Markdown template, and the versions page, each node rendered into its piece of a page through one trait, beside its template; the files of the site's store and the shared assets; what the output folder's marker and store already hold, the plan by which the store is written, and whether the output folder is safe to write.

mod contents;
mod front_page;
mod heading_parts;
mod home_page;
mod node_rendering;
mod output_marker;
mod output_safety;
mod page_frame;
mod page_head;
mod published_store;
mod search_box;
mod shared_assets;
mod shell;
mod store_files;
mod store_format;
mod store_plan;
mod target_files;
mod target_removal;
mod theme_stylesheet;
mod versions_page;

pub(crate) use output_marker::marker_json;
pub(crate) use output_safety::{Change, Moment, output_safety};
pub(crate) use published_store::StoreListing;
pub(crate) use target_files::TargetFiles;
pub(crate) use target_removal::removal_plan;

use shared_assets::shared_assets;
use shell::render_shell;
use store_files::{DescriptorSettings, render_store};
use store_plan::{
    FolderBefore, StoreChange, VersionsPageWrite, WrittenTarget, plan_store, versions_once_changed,
};
use versions_page::versions_page_write;

use output_marker::{folder_by_folder, read_marker};
use store_format::{
    ASSET_FILES, ASSETS, DATA, GHG_VERSION, JSON, OBJECTS, PAGES_INDEX, PREVIEWS, ROOT_PAGE,
    SITE_DESCRIPTOR, STORE_FORMAT, VERSIONS, VERSIONS_LIST, asset_paths, assets_folder,
    descriptor_path, json_bytes, object_path, sha256_hex, shell_path,
};
