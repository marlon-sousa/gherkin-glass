//! Test support: the files of a site's store, written as compact JSON, indented as the committed expected sites hold them, so that each member is on a line of its own. The tests in `tests/` and `cargo xtask build` both bring it in with `#[path]`.

/// `json` with each member and each item on a line of its own, indented by two spaces for each
/// level, a space after each colon, and an empty list or object kept as `[]` or `{}`, as
/// serde_json's pretty printer writes it; the members keep their order, and the text inside
/// strings stays as it is. Spaces and line breaks between tokens are dropped first, so that
/// JSON already indented comes out the same.
pub(crate) fn indented(json: &str) -> String {
    let tokens: Vec<char> = compact(json).chars().collect();
    let mut written = String::new();
    let mut depth = 0usize;
    let mut in_text = false;
    let mut escaped = false;
    let mut place = 0;
    while let Some(&character) = tokens.get(place) {
        place += 1;
        if in_text {
            written.push(character);
            match (escaped, character) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => in_text = false,
                _ => {}
            }
            continue;
        }
        match character {
            '"' => {
                in_text = true;
                written.push(character);
            }
            '{' | '[' if matches!(tokens.get(place), Some('}' | ']')) => {
                written.push(character);
                written.extend(tokens.get(place));
                place += 1;
            }
            '{' | '[' => {
                depth += 1;
                written.push(character);
                new_line(&mut written, depth);
            }
            '}' | ']' => {
                depth = depth.saturating_sub(1);
                new_line(&mut written, depth);
                written.push(character);
            }
            ',' => {
                written.push(character);
                new_line(&mut written, depth);
            }
            ':' => written.push_str(": "),
            _ => written.push(character),
        }
    }
    written.push('\n');
    written
}

/// `json` without the spaces and line breaks between its tokens.
fn compact(json: &str) -> String {
    let mut kept = String::new();
    let mut in_text = false;
    let mut escaped = false;
    for character in json.chars() {
        if in_text {
            match (escaped, character) {
                (true, _) => escaped = false,
                (false, '\\') => escaped = true,
                (false, '"') => in_text = false,
                _ => {}
            }
        } else if character == '"' {
            in_text = true;
        } else if character.is_whitespace() {
            continue;
        }
        kept.push(character);
    }
    kept
}

fn new_line(written: &mut String, depth: usize) {
    written.push('\n');
    written.push_str(&"  ".repeat(depth));
}
