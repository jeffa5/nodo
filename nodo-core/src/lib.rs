mod markdown;

pub use markdown::Markdown;

pub trait Read {
    fn read(s: &str) -> Nodo;
}

pub trait Write {
    fn write<W: std::io::Write>(n: &Nodo, w: &mut W) -> Result<(), std::io::Error>;
}

#[derive(Debug, Eq, PartialEq)]
enum Inline {
    Plain(String),
    Emph(Vec<Inline>),
    Strong(Vec<Inline>),
    Code(String),
    Strikethrough(Vec<Inline>),
    Link(String, String),
    Image(String, String),
    Html(String),
    SoftBreak,
    HardBreak,
}

#[derive(Debug, Eq, PartialEq)]
struct ListItem(Option<bool>, Vec<Block>);

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum ListType {
    Numbered,
    Plain,
}

#[derive(Debug, Eq, PartialEq)]
enum Block {
    Paragraph(Vec<Inline>),
    Heading(u32, Vec<Inline>),
    Code(String, String),
    Quote(Vec<Block>),
    List(ListType, Vec<ListItem>),
    Rule,
}

#[derive(Default, Debug, Eq, PartialEq)]
pub struct Nodo {
    blocks: Vec<Block>,
}
