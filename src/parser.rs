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

use crate::types::Script;

pub fn parse(input: String) -> Script {
    !todo!()
}
