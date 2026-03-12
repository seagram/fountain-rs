enum Element {
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
    TitlePage {
        // TODO
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
