/*
 * Terminology:
 * RFP -> RustFountainParser (this parser)
 * FFP -> FastFountanParser (newest, standard Objective-C parser)
 * OFP -> OriginalFountanParser (origina, deprecated Objective-C parser)
 *
 * Approach:
 * Single-pass, line-by-line parser.
 *
 * Differences in RFP compared to FFP:
 * - FFP is regex-heavy. It uses regex for simple single character matching. RFP opts for using std::str for
 * these types of checks, removing the overhead of compiling regexes.
 * - FFP mutates previous elements in-place. For certain element types, the specific type is unknown on it's first line.
 *  FFP will continue to parse each line until identifies the correct type and will then remove the
 *  last element, mutate it, and re-add it to the global element array. RFP instead uses
 *  continuation lines before identfying the element, meaning the global element array is not
 *  mutated unneccessarily. For example, when RFP detects the opening of a multiline element (ex,
 *  dialogue, boneyard, note, etc.), it will append lines to a seperate buffer and once it detects
 *  the end of the element, it will save the contents of the buffer to the global element array.
 *  - FFP has implicit priority as a result of it's element parsing order.
 *  - FFP Has duplicated logic for SceneNumbers. It has the same parsing code twice, once for forced
 *  scene headings and once for regular scene headings. RFP removes the duplication.
 *  - In FFP, Element::Dialogue appending has two different implemenations depending on when
 *  dialogue is being appended. RFP removes this; there is a single implementation for appending
 *  dialogue.
 */

use crate::types::{Element, Script, TitlePage};

fn normalize_input(mut input: String) -> String {
    // Trim leading newlines from input
    input = input.trim_start().to_string();
    // Normalize lines endings to newline characters
    input = input.replace("\r\n", "\n").replace("\r", "\n");
    // Add two trailing newlines to input
    input.push_str("\n\n");

    input
}

fn is_title_page_line(line: &str) -> bool {
    // Title page lines must follow one of two formats:
    // "Key: Value" (inline)
    // "Key:" (directive)
    // Rules:
    // 1. Line starts with a non-whitespace character
    // 2. Contains a ':' character after one or more non-':' characters
    // 3. For inline: has non-whitespace content after ':' and before newline
    // 4. For directive: has only whitespace/nothing after ':'
    let first_char = match line.chars().next() {
        Some(c) => c,
        None => return false,
    };

    if first_char.is_whitespace() {
        return false;
    };

    let colon_position = match line.find(':') {
        Some(position) => position,
        None => return false,
    };

    // There must be at least one character before the colon character
    colon_position > 0
}

fn parse_title_page(input: String) -> Option<TitlePage> {
    let mut title_page = TitlePage {
        entries: Vec::new(),
    };

    // Title pages must be followed by two newline characters
    // We parse from the first line of the input to the last line before the double newline
    let title_page_range = match input.find("\n\n") {
        Some(position) => &input[..position],
        None => return None,
    };

    for line in title_page_range.lines() {
        if is_title_page_line(line) {
            let colon_position = line.find(':').unwrap();
            let after_colon = &line[colon_position + 1..];

            let key = line[..colon_position].to_string();

            if !after_colon.trim().is_empty() {
                // Line is inline ("Key: Value")
                let value = after_colon.trim().to_string();
                title_page.entries.push((key, vec![value]));
            } else {
                // Line is directive ("Key:")
                title_page.entries.push((key, Vec::new()));
            }
        } else if !line.trim().is_empty() {
            break; // not a title page lines or a continuation line
        }
    }
    Some(title_page)
}

pub fn parse_script(mut input: String) -> Script {
    let mut script = Script {
        title_page: None,
        elements: Vec::new(),
    };

    input = normalize_input(input);

    script.title_page = parse_title_page(input);

    todo!()
}
