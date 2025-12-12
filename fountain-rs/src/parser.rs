use crate::types::{Document, Element};

struct ParserState {
    in_title_page: bool,
    in_dialogue_block: bool,
    in_boneyard: bool,
    last_element: Option<Element>,
}

impl ParserState {
    fn new() -> Self {
        ParserState {
            in_title_page: false,
            in_dialogue_block: false,
            in_boneyard: false,
            last_element: None,
        }
    }
}

fn parse(input: &str) -> Document {
    todo!();
}

fn parse_line(line: &str, state: &mut ParserState) -> Option<Element> {
    todo!();
}

fn parse_action(line: &str, state: &mut ParserState) -> Option<Element> {
    // EXAMPLE: This is an action.
    // TODO: Check if line starts with '!' which forces an action.
    // TODO: Respect whitespace formatting including newlines, tabs, & spaces.
    // TODO: Convert tab characters to four spaces.
    // TODO: Ensure any line that doesn't match any other element by default matches action.
    // TODO: Ensure when action element is centered, leading spaces are trimmed.
    // TODO: Ensure all empty lines are parsed as empty action lines to render vertical whitespace.
    todo!();
}

fn parse_scene_heading(line: &str, state: &mut ParserState) -> Option<Element> {
    // EXAMPLE: INT. HOUSE - DAY #I-1-A.a#
    // TODO: Check if there is a blank line preceding & following line to be parsed.
    // TODO: Check if line starts with '.' which forces a scene heading.
    // TODO: Check if line ends with a scene number (alphanumeric characters including dashes
    //       and periods wrapped inside '#')

    // Split line on first occurrence of either a space or a period
    let prefix = line.split_once(".");
    // Fountain supports lower-case scene headings. Convert to uppercase for parsing.

    // Check if parsed scene heading exists in allowed list
    let valid_options = vec!["INT", "EXT", "EST", "INT./EXIT", "INT/EXT", "I/E"];
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action() {
        let input =
            "This is an example action element.\nIt supports newline characters.    And spaces.";
        let mut state = ParserState::new();
        let result = parse_line(input, &mut state);
        assert!(matches!(result, Some(Element::Action(_))));
    }
}
