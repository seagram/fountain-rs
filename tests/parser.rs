use fountain_rs::{parser, types::TitlePage};

fn entry(key: &str, values: &[&str]) -> (String, Vec<String>) {
    (
        key.to_string(),
        values.iter().map(|s| s.to_string()).collect(),
    )
}

#[test]
fn test_title_page_parsing() {
    let input = include_str!("data/brick_and_steel.fountain").to_string();
    let result = parser::parse_title_page(input);
    let expected = TitlePage {
        entries: vec![
            entry("Title", &["_**BRICK & STEEL**_", "_**FULL RETIRED**_"]),
            entry("Credit", &["Written by"]),
            entry("Author", &["Stu Maschwitz"]),
            entry("Source", &["Story by KTM"]),
            entry("Draft date", &["1/27/2012"]),
            entry(
                "Contact",
                &[
                    "Next Level Productions",
                    "1588 Mission Dr.",
                    "Solvang, CA 93463",
                ],
            ),
        ],
    };

    assert_eq!(result, Some(expected));
}
