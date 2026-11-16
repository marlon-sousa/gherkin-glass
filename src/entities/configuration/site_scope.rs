//! Entity: whether a site shows every feature file of its input or only those `--only` lists.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SiteScope {
    EveryFeature,
    OnlyListed,
}
