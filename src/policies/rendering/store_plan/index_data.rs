//! Policy: the list of versions and the index of pages across versions as JSON, from every version's descriptor.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::entities::{PublishedFeature, VersionList, VersionName, VersionsPageRecord};
use crate::policies::rendering::{STORE_FORMAT, json_bytes};
use crate::policies::{order_versions, precedence_groups};

#[derive(Serialize)]
struct VersionsData<'a> {
    format: u32,
    development: Vec<&'a str>,
    releases: Vec<&'a str>,
    pre_releases: Vec<&'a str>,
    precedence: Vec<Vec<&'a str>>,
    versions_page: VersionsPageData<'a>,
}

#[derive(Serialize)]
struct VersionsPageData<'a> {
    ghg: &'a str,
    title: &'a str,
    accent_light: String,
    accent_dark: String,
    front_page: Option<&'a str>,
}

#[derive(Serialize)]
struct PagesData<'a> {
    format: u32,
    pages: Vec<PageData<'a>>,
}

#[derive(Serialize)]
struct PageData<'a> {
    page: &'a str,
    objects: Vec<ObjectVersions<'a>>,
}

#[derive(Serialize)]
struct ObjectVersions<'a> {
    object: &'a str,
    versions: Vec<&'a str>,
}

/// The compact JSON of `data/versions.json` of `versions`, the versions page written with `page`.
pub(super) fn versions_json(
    versions: &BTreeMap<VersionName, Vec<PublishedFeature>>,
    page: &VersionsPageRecord,
) -> Vec<u8> {
    let list = order_versions(versions.keys().cloned());
    let precedence = precedence_groups(versions.keys().cloned());
    json_bytes(&VersionsData {
        format: STORE_FORMAT,
        development: names(&list.development),
        releases: names(&list.releases),
        pre_releases: names(&list.pre_releases),
        precedence: precedence.iter().map(Vec::as_slice).map(names).collect(),
        versions_page: VersionsPageData {
            ghg: &page.ghg,
            title: &page.title,
            accent_light: page.accents.light.to_string(),
            accent_dark: page.accents.dark.to_string(),
            front_page: page.front_page.as_deref(),
        },
    })
}

/// The compact JSON of `data/pages.json` of `versions`: each page, sorted by its name folder by
/// folder, with each object a version has for it, sorted by name, and the versions that have
/// it, in the order of the versions page.
pub(super) fn pages_json(versions: &BTreeMap<VersionName, Vec<PublishedFeature>>) -> Vec<u8> {
    let list = order_versions(versions.keys().cloned());
    let pages = listed(&list)
        .flat_map(|name| uses_of(versions, name))
        .fold(Pages::new(), add_use);
    json_bytes(&PagesData {
        format: STORE_FORMAT,
        pages: pages.iter().map(page_data).collect(),
    })
}

/// Each page, by its names folder by folder and then its name, with each of its objects and
/// the versions that have it.
type Pages<'a> = BTreeMap<PageKey<'a>, ObjectUses<'a>>;

/// A page's names folder by folder, then its name.
type PageKey<'a> = (Vec<&'a str>, &'a str);

/// Each object of a page, with the versions that have it.
type ObjectUses<'a> = BTreeMap<&'a str, Vec<&'a str>>;

/// A version's feature: its page, its object and the version.
struct PageUse<'a> {
    page: &'a str,
    object: &'a str,
    version: &'a str,
}

fn uses_of<'a>(
    versions: &'a BTreeMap<VersionName, Vec<PublishedFeature>>,
    name: &'a VersionName,
) -> impl Iterator<Item = PageUse<'a>> {
    versions
        .get(name)
        .into_iter()
        .flatten()
        .map(move |feature| page_use(feature, name))
}

fn page_use<'a>(feature: &'a PublishedFeature, name: &'a VersionName) -> PageUse<'a> {
    PageUse {
        page: &feature.page,
        object: &feature.object,
        version: name.as_str(),
    }
}

fn add_use<'a>(mut pages: Pages<'a>, found: PageUse<'a>) -> Pages<'a> {
    pages
        .entry((found.page.split('/').collect(), found.page))
        .or_default()
        .entry(found.object)
        .or_default()
        .push(found.version);
    pages
}

/// Every version of `list`, in its order.
fn listed(list: &VersionList) -> impl Iterator<Item = &VersionName> {
    list.development
        .iter()
        .chain(&list.releases)
        .chain(&list.pre_releases)
}

fn page_data<'a>(((_, page), objects): (&PageKey<'a>, &ObjectUses<'a>)) -> PageData<'a> {
    PageData {
        page,
        objects: objects.iter().map(object_versions).collect(),
    }
}

fn object_versions<'a>((object, versions): (&&'a str, &Vec<&'a str>)) -> ObjectVersions<'a> {
    ObjectVersions {
        object,
        versions: versions.clone(),
    }
}

fn names(versions: &[VersionName]) -> Vec<&str> {
    versions.iter().map(VersionName::as_str).collect()
}

#[cfg(test)]
mod tests;
