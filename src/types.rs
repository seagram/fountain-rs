/*
 * The following type modeling differs from the original Objective-C implemenation.
 *
 * The rust-equivalent of the original Objective-C implementation is as follows:
 *
 * enum ElementType {
 *      SceneHeading,
 *      Action
 *      Character,
 *      (etc.)
 * }
 *
 * struct Element {
 *      element_type: ElementType
 *      text: String,
 *      is_centered: bool,
 *      scene_number: Optional<String>
 *      is_dual_dialogue: bool,
 *      section_depth: u32
 * }
 *
 * The problem with design is that most Element fields do not apply to most ElementTypes.
 * For example:
 * - is_centered is only ever true for certain Action elements.
 * - is_dual_dialogue is only ever true for certain Dialogue elements.
 * - scene_number only applies to SceneHeading
 * - section_number only applies to SceneHeading
 *
 * In liue of this apporach, we can used enum variants with named fields.
 * This means each enum variant only contains fields applicable to them.
 */

pub enum Element {
    SceneHeading {
        text: String,
        scene_number: Option<String>,
    },
    Action {
        text: String,
        is_centered: bool,
    },
    Character {
        name: String,
        is_dual_dialogue: bool,
    },
    Dialogue {
        text: String,
    },
    Parenthetical {
        text: String,
    },
    Lyrics {
        text: String,
    },
    Transition {
        text: String,
    },
    PageBreak,
    Note {
        text: String,
    },
    Boneyard {
        text: String,
    },
    SectionHeading {
        text: String,
        depth: u32,
    },
    Synopsis {
        text: String,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct TitlePage {
    // kv pairs
    pub entries: Vec<(String, Vec<String>)>,
}

pub struct Script {
    pub title_page: Option<TitlePage>,
    pub elements: Vec<Element>,
}
