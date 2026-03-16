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

fn is_title_page_key_line(line: &str) -> bool {
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

pub fn parse_title_page(input: String) -> Option<TitlePage> {
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
        if is_title_page_key_line(line) {
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
        } else if line.starts_with('\t') || line.starts_with(' ') {
            // If the line starts with a whitespace character and does not contain a ':'
            // It is considered a continuation line whose contents are appended to the last key's value vector
            if let Some((_, values)) = title_page.entries.last_mut() {
                values.push(line.trim().to_string())
            }
        } else if !line.trim().is_empty() {
            break; // not a title page line or a continuation line
        }
    }
    Some(title_page)
}

fn parse_forced(input: String) -> Option<Element> {
    // Defintion:
    // Fountain elements can be "forced" by starting a line with a corresponding character.
    // While uncommon in practice and is more often utilized by power users, it is still fully
    // supported in the official fountain spec.
    // Instead of checking for each of these characters in each seperate element parsing logic, we
    // can check one, here.
    //
    // TODO: Strip forced character from output

    let first_char = input.chars().next();
    match first_char {
        Some('.') => Some(Element::SceneHeading {
            text: input,
            scene_number: None, // TODO
        }),
        Some('!') => Some(Element::Action {
            text: input,
            is_centered: false, // TODO
        }),
        Some('@') => Some(Element::Character {
            name: input,
            is_dual_dialogue: false, // TODO
        }),
        Some('~') => Some(Element::Lyrics { text: input }),
        Some('>') => Some(Element::Transition { text: input }),
        Some('=') => Some(Element::Synopsis { text: input }),
        _ => None,
    }
}

fn parse_scene_heading(input: String) -> Option<Element> {
    // Definition:
    // Any line that is follwed by a blank line
    // Must begin with valid scene heading (see below)
    // Can be forced by starting the line with '.'
    // Note: '.' character must be stripped from output
    // Note: valid_scene_headings are case-insensitive (ex. 'ext' and 'int' are valid)
    let valid_scene_headings = vec!["INT", "EXT", "EST", "INT./EXT", "INT/EXT", "I/E"];
    todo!();
}

fn parse_centered_action(input: String) -> Option<Element> {
    // Defintion:
    // Defined as any line that starts with a '>' character and ends with a '<' character
    // Example: >THE END<
    // Leading spaces are stripped when parsing but are allowed syntatically.
    // Example: >   THE END   < is parsed to >THE END<
    let first_char = input.chars().next();
    let last_char = input.chars().last();

    match (first_char, last_char) {
        (Some('>'), Some('<')) => {
            // Strip arrow characters, leading and trailing whitespace
            let stripped_input = input
                .trim_start_matches('>')
                .trim_start()
                .trim_end_matches('<')
                .trim_end()
                .to_string();
            Some(Element::Action {
                text: stripped_input,
                is_centered: true,
            })
        }
        _ => None,
    }
}

fn parse_page_break(input: String) -> Option<Element> {
    // Definition:
    // Any line containing three or more consectutive equals signs and nothing else.
    // Example: ===, =====, =======
    match input.chars().all(|c| c == '=') {
        true => Some(Element::PageBreak),
        false => None,
    }
}

fn parse_section(input: String) -> Option<Element> {
    // Definition:
    // Any line starting with one or more consectutive '#' characters
    // The number of '#' characters denotes the section depth
    // Example: # Act (Depth = 1)
    //          ## Sequence (Depth = 2)
    //          ### Scene (Depth = 3)

    let first_word = input.split_whitespace().next()?;
    if !first_word.chars().all(|c| c == '#') {
        return None;
    }
    let depth = first_word.len() as u32;
    Some(Element::SectionHeading { text: input, depth })
}

pub fn parse_script(mut input: String) -> Script {
    let mut script = Script {
        title_page: None,
        elements: Vec::new(),
    };

    input = normalize_input(input);

    script.title_page = parse_title_page(input.clone());

    // Discard title page lines, start parsing at beggining of screenplay
    input = match input.find("\n\n") {
        Some(position) => input[..position].to_string(),
        None => input,
    };

    let input_lines: Vec<String> = input.lines().map(|s| s.to_string()).collect();

    for line in input_lines {
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Define parsers in priority order
        let parsers: &[fn(String) -> Option<Element>] = &[
            parse_page_break,
            parse_section,
            parse_centered_action,
            parse_forced,
        ];

        // Try each parser in priority order, return first successul result
        let element = parsers.iter().find_map(|f| f(line.clone()));
        if let Some(e) = element {
            script.elements.push(e);
        }
    }
    script
}

// TODO:
//
// - [ ] Scene Headings
// - [ ] Action
// - [ ] Character
// - [ ] Dialogue
// - [ ] Parenthetical
// - [ ] Dual-Dialogue
// - [ ] Lyrics
// - [ ] Transition
// - [x] Centered Text
// - [ ] Emphasis
// - [x] Title Page
// - [x] Page Breaks
// - [ ] Punctuation
// - [ ] Line Breaks
// - [ ] Indenting
// - [ ] Notes
// - [ ] Boneyard
// - [x] Sections
// - [x] Synopses
// - [ ] Error Handling
