//! Entity: whether features, rules and scenarios may have no name, the `allow_unnamed` setting.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Unnamed {
    #[default]
    Allowed,
    Refused,
}
