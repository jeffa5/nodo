#![warn(clippy::pedantic, clippy::nursery, clippy::cargo)]
mod markdown;
pub mod query;

pub use markdown::Markdown;

pub trait Parse {
    type ParseError;

    /// Parse a nodo from a string
    ///
    /// # Errors
    ///
    /// Errors associated with this call should be from a malformed string input for the
    /// implemented format.
    fn parse(s: &str) -> Result<Nodo, Self::ParseError>;
}

pub trait Render {
    type RenderError;

    /// Render a nodo to a generic writer
    ///
    /// # Errors
    ///
    /// Errors associated with this call should be from the writer as well as any issues in
    /// converting the nodo to the format due to a potentially limited format.
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
struct ListItem {
    task: Option<bool>,
    blocks: Vec<Block>,
}

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
