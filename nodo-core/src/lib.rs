mod markdown;

pub use markdown::Markdown;

pub trait Read {
    fn read(s: &str) -> Nodo;
}

pub trait Write {
    fn write<W: std::fmt::Write>(n: &Nodo, w: &mut W) -> Result<(), std::fmt::Error>;
}

#[derive(Debug, Eq, PartialEq)]
enum Inline {
    Plain(String),
    Emph(String),
    Strong(String),
    Code(String),
    Strikethrough(String),
    Link(String, String),
    Image(String, String),
    Html(String),
    SoftBreak,
    HardBreak,
    ListItem(Vec<Inline>),
}

#[derive(Debug, Eq, PartialEq)]
struct ListItem(Option<bool>, Vec<Inline>);

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum ListType {
    Numbered,
    Plain,
}

#[derive(Debug, Eq, PartialEq)]
enum Block {
    Rule,
    Paragraph(Vec<Inline>),
    Heading(u32, Vec<Inline>),
    Code(String, String),
    Quote(String),
    List(ListType, Vec<ListItem>),
}

#[derive(Default, Debug, Eq, PartialEq)]
pub struct Nodo {
    blocks: Vec<Block>,
}
