#[derive(Debug, Clone)]
pub enum Line {
    Blank,
    Preprocessor(String),
    CommentOnly {
        indent: usize,
        text: String,
    },
    Statement {
        label: Option<Label>,
        body: Option<Body>,
        comment: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct Label {
    pub name: String,
    pub has_colon: bool,
}

#[derive(Debug, Clone)]
pub struct Body {
    pub mnemonic: String,
    pub operands: Vec<String>,
}

pub const SECTION_DIRECTIVES: &[&str] = &[
    "section", "segment", "global", "extern", "common", "default", "bits", "use16", "use32",
    "use64", "cpu", "org", "absolute", "struc", "endstruc", "istruc", "iend", "align", "alignb",
];

impl Body {
    pub fn is_section_level(&self) -> bool {
        SECTION_DIRECTIVES.contains(&self.mnemonic.as_str())
    }
}
