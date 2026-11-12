//! Policy: the links from tags to other sites.

use regex_automata::util::interpolate;
use regex_automata::{Input, PatternID};

use crate::entities::{OutsideLink, TagLink, percent_encoded};

/// The link of the first of `links` whose pattern matches `tag`, written with its `@`.
pub(crate) fn outside_link(tag: &str, links: &[TagLink]) -> Option<OutsideLink> {
    links.iter().find_map(|link| {
        let mut captures = link.pattern.create_captures();
        link.pattern
            .search_captures(&Input::new(tag), &mut captures);
        if !captures.is_match() {
            return None;
        }
        let names = link.pattern.group_info();
        let fill = |template: &str, encode: bool| {
            let mut filled = String::new();
            interpolate::string(
                template,
                |index, out| {
                    let text = captures.get_group(index).map_or("", |span| &tag[span]);
                    if encode {
                        out.push_str(&percent_encoded(text));
                    } else {
                        out.push_str(text);
                    }
                },
                |name| names.to_index(PatternID::ZERO, name),
                &mut filled,
            );
            filled
        };
        let address = fill(&link.address, true);
        let text = fill(&link.text, false);
        // A link with no text, or nowhere to go, would be heard as nothing.
        (!address.is_empty() && !text.trim().is_empty()).then_some(OutsideLink { address, text })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::tag_link as link;

    #[test]
    fn the_first_pattern_that_matches_the_whole_tag_gives_the_link() {
        let links = [
            link("@issue-(\\d+)", "https://x/issues/$1", "Issue $1 on GitHub"),
            link("@issue-(?<n>.+)", "https://y/${n}", "Other $n"),
        ];
        assert_eq!(
            outside_link("@issue-12", &links),
            Some(OutsideLink {
                address: "https://x/issues/12".into(),
                text: "Issue 12 on GitHub".into(),
            })
        );
        assert_eq!(
            outside_link("@issue-1 2#ç", &links),
            Some(OutsideLink {
                address: "https://y/1%202%23%C3%A7".into(),
                text: "Other 1 2#ç".into(),
            })
        );
        assert_eq!(outside_link("@my-issue-12", &links[..1]), None);
        assert_eq!(outside_link("@issue-12a", &links[..1]), None);
    }

    #[test]
    fn a_link_whose_text_or_address_comes_out_empty_is_not_shown() {
        let links = [
            link("@x(.*)", "https://x/$1", "$1"),
            link("@y(.*)", "$1", "Y $1"),
        ];
        assert_eq!(outside_link("@x", &links), None);
        assert!(outside_link("@xa", &links).is_some());
        assert_eq!(outside_link("@y", &links), None);
    }

    #[test]
    fn a_whole_match_is_found_even_when_a_shorter_alternative_comes_first() {
        let links = [link("@a|@ab", "x", "$0 $$ $ end $")];
        assert_eq!(
            outside_link("@ab", &links).expect("a match").text,
            "@ab $ $ end $"
        );
    }
}
