mod markdown;

pub use markdown::Markdown;

pub trait Parse {
    type ParseError;

    fn parse(s: &str) -> Result<Nodo, Self::ParseError>;
}

pub trait Render {
    type RenderError;

    fn render<W: std::io::Write>(n: &Nodo, w: &mut W) -> Result<(), Self::RenderError>;
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
