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

use crate::types::{Element, ParserState, Script, TitlePage};

// Helper functions

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

// Element parsers

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

fn parse_scene_number(input: String) -> Option<String> {
    // Helper function for parsing SceneHeadings
    // Used for parse_scene_heading and parse_forced
    // Checks if a scene_heading contains a scene_number
    // Defintion:
    // Any alphanumerics (including dashes and periods), wrapped in '#'.
    // Must be appended at the end of the scene heading
    let last_word = input.split_whitespace().last()?;
    let first_char = last_word.chars().next()?;
    let last_char = last_word.chars().last()?;

    match (first_char, last_char) {
        ('#', '#') => {
            let scene_number = last_word.trim_matches('#').to_string();
            Some(scene_number)
        }
        _ => None,
    }
}

fn parse_forced(input: String) -> Option<Element> {
    // Defintion:
    // Fountain elements can be "forced" by starting a line with a corresponding character.
    // While uncommon in practice and is more often utilized by power users, it is still fully
    // supported in the official fountain spec.
    // Instead of checking for each of these characters in each seperate element parsing logic, we
    // can check one, here.

    let first_char = input.chars().next();
    // Forced character must be stripped from output
    let text = input[1..].to_string();

    match first_char {
        Some('.') => Some(Element::SceneHeading {
            scene_number: parse_scene_number(text.clone()),
            text,
        }),
        Some('!') => Some(Element::Action {
            text,
            is_centered: false, // TODO
        }),
        Some('@') => Some(Element::Character {
            name: text,
            is_dual_dialogue: false, // TODO
        }),
        Some('~') => Some(Element::Lyrics { text }),
        Some('>') => Some(Element::Transition { text }),
        Some('=') => Some(Element::Synopsis { text }),
        _ => None,
    }
}

fn parse_scene_heading(input: String) -> Option<Element> {
    // Definition:
    // Any line that is follwed by a blank line
    // Must begin with valid scene heading (see below)
    // Note: valid_scene_headings are case-insensitive (ex. 'ext' and 'int' are valid)
    const VALID_SCENE_HEADINGS: &[&str] =
        &["INT.", "EXT.", "EST.", "INT./EXT.", "INT/EXT.", "I/E."];
    let first_word_uppercase = input.split_whitespace().next()?.to_uppercase();

    if !VALID_SCENE_HEADINGS.contains(&first_word_uppercase.as_str()) {
        return None;
    }

    Some(Element::SceneHeading {
        text: input,
        scene_number: None,
    })

    // TODO: Check for scene numbers wrapped in # at end of line
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

fn parse_transition(input: String) -> Option<Element> {
    // Definition:
    // (a) Must be uppercase
    // (b) Ends in 'TO:' (Ex. "CUT TO:", "FLASHBACK TO:", etc.)
    // (c) Preceded and followed by an empty line

    let all_uppercase = input
        .chars()
        .all(|c| !c.is_alphanumeric() || c.is_uppercase()); // (a)
    let ends_in_to = input.split_whitespace().last().unwrap() == "TO:"; // (b)
    // let empty_line_before_and_after = todo!(); // (c)

    match (all_uppercase, ends_in_to) {
        (true, true) => Some(Element::Transition { text: input }),
        _ => None,
    }
}

fn parse_parenthetical(input: String) -> Option<Element> {
    // Definition:
    // (a) Follows a Character or Dialogue element
    // (b) Wrapped in parentheses

    // let follows_character_or_dialogue = todo!(); // (a)
    let wrapped_in_parentheses = (input.chars().next()?, input.chars().last()?) == ('(', ')'); // (b)

    match wrapped_in_parentheses {
        true => {
            let inside_paren = input
                .strip_prefix('(')
                .and_then(|s| s.strip_suffix(')'))?
                .to_string();
            Some(Element::Parenthetical { text: inside_paren })
        }
        _ => None,
    }
}

fn parse_dialogue(input: String, state: &ParserState) -> Option<Element> {
    // Definition:
    // (a) Any text following a Character or Parenthetical element
    // NOTE: manual line breaks are allowed. See Line Breaks for more.

    match state.last_element {
        Some(Element::Character { .. } | Element::Parenthetical { .. }) => {
            Some(Element::Dialogue { text: input })
        }
        _ => None,
    }
}

fn parse_character(input: String, state: &ParserState) -> Option<Element> {
    // Definition:
    // (a) Any line that is entirely uppercase
    // (b) has one empty line before it
    // (c) has a non-empty line after it
    // (d) Includes at least one alphabetical character
    let all_uppercase: bool = input
        .chars()
        .all(|c| !c.is_alphanumeric() || c.is_uppercase()); // (a)
    let has_empty_line_before: bool = state.prev_line.is_empty(); // (b)
    let has_non_empty_line_after: bool = !state.next_line.is_empty(); // (c)
    let contains_alphabetical_char: bool = input.contains(|c: char| c.is_alphabetic());

    match (
        all_uppercase,
        has_empty_line_before,
        has_non_empty_line_after,
        contains_alphabetical_char,
    ) {
        (true, true, true, true) => Some(Element::Character {
            name: input,
            is_dual_dialogue: false, // TODO
        }),
        _ => None,
    }

    // TODO: Add support for Character Extensions
}

fn parse_boneyard(input: String) -> Option<Vec<Element>> {
    // NOTE: This returns all the boneyard elements in a screenplay at once.
    // Unlike the other element parsers, all boneyards at parsed at the start of parsing.
    // Boneyards are ignored in formatted ouput but a still parsed for flexibility
    // Here, input refers to the entire screenplay, passed as a string
    // Definition:
    // (a) Any text wrapped in '/*' and '*/'
    todo!();
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

    let mut state = ParserState {
        last_element: None,
        prev_line: String::new(),
        curr_line: String::new(),
        next_line: String::new(),
        script_lines: input.lines().map(|s| s.to_string()).collect(),
    };

    for line in &state.script_lines {
        // Skip empty lines
        if line.trim().is_empty() {
            state.prev_line = line.to_string();
            continue;
        }

        // Define parsers in priority order
        let parsers: &[fn(String) -> Option<Element>] = &[
            parse_scene_heading,
            parse_transition,
            parse_page_break,
            parse_parenthetical,
            parse_section,
            parse_centered_action,
            parse_forced,
        ];

        let stateful_parsers: &[fn(String, &ParserState) -> Option<Element>] =
            &[parse_dialogue, parse_character];

        // Try each parser in priority order, return first successul result
        let element = parsers.iter().find_map(|f| f(line.clone())).or_else(|| {
            stateful_parsers
                .iter()
                .find_map(|f| f(line.clone(), &state))
        });

        if let Some(e) = element {
            state.last_element = Some(e.clone());
            script.elements.push(e);
        }
    }
    script
}

// TODO:
//
// - [x] Scene Headings
// - [ ] Action
// - [x] Character
// - [x] Dialogue
// - [x] Parenthetical
// - [ ] Dual-Dialogue
// - [x] Lyrics
// - [x] Transition
// - [x] Centered Text
// - [ ] Emphasis
// - [x] Title Page
// - [x] Page Breaks
// - [ ] Line Breaks
// - [ ] Indenting
// - [ ] Notes
// - [ ] Boneyard
// - [x] Sections
// - [x] Synopses
// - [ ] Error Handling
