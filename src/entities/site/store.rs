//! Entity: the site's store, the object of each feature file's content and what a descriptor holds besides, as a build makes them, the files a build renders of it and how it writes them, and what the output folder's store already holds.

mod feature_object;
mod object_block;
mod object_step;
mod published_store;
mod site_store;
mod store_files;

pub(crate) use feature_object::FeatureObject;
pub(crate) use object_block::{ObjectBlock, ObjectRun};
pub(crate) use object_step::ObjectStep;
pub(crate) use published_store::{PublishedFeature, PublishedStore};
pub(crate) use site_store::{DescriptorFeature, DescriptorSentences, DescriptorTag, SiteStore};
pub(crate) use store_files::{StoreFiles, StorePlan};
