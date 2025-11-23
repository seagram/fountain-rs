// Fountain Syntax Documentation: https://fountain.io/syntax/

pub enum Element {
    SceneHeading {
        text: String,
        scene_number: Option<String>,
    },
    Action(String),
    Character {
        name: String,
        extension: Option<String>,
        is_dual: bool,
    },
    Dialogue(String),
    Parenthetical(String),
    Lyric(String),
    Transition(String),
    CenteredText(String),
    PageBreak,
    Section {
        level: u8,
        text: String,
    },
    Synopsis(String),
    Note(String),
    TitlePageEntry {
        key: String,
        value: String,
    },
}

pub struct Document {
    pub elements: Vec<Element>,
}
