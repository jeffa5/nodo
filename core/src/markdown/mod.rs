use crate::{Block, Inline, ListItem, ListType, Nodo, Parse, Render};
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::{io, iter::Peekable};
use thiserror::Error;

#[cfg(not(test))]
use log::trace;

#[cfg(test)]
use std::println as trace;

pub struct Markdown;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("found an unexpected event: {event}")]
    UnexpectedElement { event: String },

    #[error("received non-text event while parsing plaintext: {event}")]
    NoText { event: String },

    #[error("found non-end event while parsing plaintext: {event}")]
    ExpectedEnd { event: String },
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("failed to write content: {0}")]
    WriteFailure(#[from] io::Error),
}

const INDENT: &str = "    ";

fn parse_blocks(p: &mut Peekable<Parser>) -> Result<Vec<Block>, ParseError> {
    let mut blocks = Vec::new();

    while let Some(e) = p.next() {
        trace!("parse_blocks: {:?}", e);

        match e {
            Event::Start(ref tag) => match tag {
                Tag::Heading(level) => blocks.push(Block::Heading(*level, parse_inlines(p)?)),
                Tag::Paragraph => blocks.push(Block::Paragraph(parse_inlines(p)?)),
                Tag::BlockQuote => blocks.push(Block::Quote(parse_blocks(p)?)),
                Tag::CodeBlock(kind) => {
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                    };
                    blocks.push(Block::Code(lang, parse_text(p)?))
                }
                Tag::List(kind) => {
                    if kind.is_some() {
                        blocks.push(Block::List(ListType::Numbered, parse_list_items(p)?))
                    } else {
                        blocks.push(Block::List(ListType::Plain, parse_list_items(p)?))
                    }
                }
                Tag::Item
                | Tag::FootnoteDefinition(_)
                | Tag::TableHead
                | Tag::TableRow
                | Tag::TableCell
                | Tag::Table(_)
                | Tag::Emphasis
                | Tag::Strikethrough
                | Tag::Strong
                | Tag::Link(_, _, _)
                | Tag::Image(_, _, _) => {
                    return Err(ParseError::UnexpectedElement {
                        event: format!("{:?}", e),
                    })
                }
            },
            Event::End(_) => break,
            Event::Text(s) => {
                let mut text = vec![Inline::Plain(s.to_string())];
                let mut inlines = parse_tight_paragraph(p)?;
                text.append(&mut inlines);
                blocks.push(Block::Paragraph(text))
            }
            Event::Code(_)
            | Event::Html(_)
            | Event::FootnoteReference(_)
            | Event::HardBreak
            | Event::SoftBreak
            | Event::TaskListMarker(_) => {
                return Err(ParseError::UnexpectedElement {
                    event: format!("{:?}", e),
                })
            }
            Event::Rule => blocks.push(Block::Rule),
        }
    }

    Ok(blocks)
}

fn parse_list_items(p: &mut Peekable<Parser>) -> Result<Vec<ListItem>, ParseError> {
    let mut items = Vec::new();

    while let Some(e) = p.next() {
        trace!("parse_list_items: {:?}", e);

        match e {
            Event::Start(ref tag) => match tag {
                Tag::Paragraph
                | Tag::Heading(_)
                | Tag::BlockQuote
                | Tag::CodeBlock(_)
                | Tag::List(_)
                | Tag::FootnoteDefinition(_)
                | Tag::Table(_)
                | Tag::TableHead
                | Tag::TableRow
                | Tag::TableCell
                | Tag::Emphasis
                | Tag::Strong
                | Tag::Strikethrough
                | Tag::Link(_, _, _)
                | Tag::Image(_, _, _) => {
                    return Err(ParseError::UnexpectedElement {
                        event: format!("{:?}", e),
                    })
                }
                Tag::Item => match p.peek() {
                    None => break,
                    Some(Event::TaskListMarker(b)) => {
                        let b = *b;
                        p.next().unwrap();
                        items.push(ListItem {
                            task: Some(b),
                            blocks: parse_blocks(p)?,
                        })
                    }
                    Some(_) => items.push(ListItem {
                        task: None,
                        blocks: parse_blocks(p)?,
                    }),
                },
            },
            Event::End(_) => break,
            Event::Text(_)
            | Event::Code(_)
            | Event::Html(_)
            | Event::SoftBreak
            | Event::HardBreak => items.push(ListItem {
                task: None,
                blocks: parse_blocks(p)?,
            }),
            Event::FootnoteReference(_) => {
                return Err(ParseError::UnexpectedElement {
                    event: format!("{:?}", e),
                })
            }
            Event::Rule => continue,
            Event::TaskListMarker(b) => items.push(ListItem {
                task: Some(b),
                blocks: parse_blocks(p)?,
            }),
        }
    }

    Ok(items)
}

fn parse_tight_paragraph(p: &mut Peekable<Parser>) -> Result<Vec<Inline>, ParseError> {
    let mut inlines = Vec::new();

    while let Some(e) = p.peek() {
        trace!("parse_tight_paragraph: {:?}", e);

        match e {
            Event::Start(tag) => match tag {
                Tag::Paragraph
                | Tag::Heading(_)
                | Tag::BlockQuote
                | Tag::CodeBlock(_)
                | Tag::List(_)
                | Tag::FootnoteDefinition(_)
                | Tag::Table(_)
                | Tag::TableHead
                | Tag::TableRow
                | Tag::TableCell
                | Tag::Item => break,
                Tag::Emphasis => {
                    p.next().unwrap();
                    inlines.push(Inline::Emph(parse_tight_paragraph(p)?))
                }
                Tag::Strong => {
                    p.next().unwrap();
                    inlines.push(Inline::Strong(parse_tight_paragraph(p)?))
                }
                Tag::Strikethrough => {
                    p.next().unwrap();
                    inlines.push(Inline::Strikethrough(parse_tight_paragraph(p)?))
                }
                Tag::Link(_type, s, l) => {
                    let (s, l) = (s.to_string(), l.to_string());
                    p.next().unwrap();
                    inlines.push(Inline::Link(s, l))
                }
                Tag::Image(_type, s, l) => {
                    let (s, l) = (s.to_string(), l.to_string());
                    p.next().unwrap();
                    inlines.push(Inline::Image(s, l))
                }
            },
            Event::End(_) => break,
            Event::Text(s) => {
                let s = s.to_string();
                p.next().unwrap();
                inlines.push(Inline::Plain(s))
            }
            Event::Code(s) => {
                let s = s.to_string();
                p.next().unwrap();
                inlines.push(Inline::Code(s))
            }
            Event::Html(s) => {
                let s = s.to_string();
                p.next().unwrap();
                inlines.push(Inline::Html(s))
            }
            Event::SoftBreak => {
                p.next().unwrap();
                inlines.push(Inline::SoftBreak)
            }
            Event::HardBreak => {
                p.next().unwrap();
                inlines.push(Inline::HardBreak)
            }
            Event::Rule => {
                p.next().unwrap();
                continue;
            }
            Event::FootnoteReference(_) | Event::TaskListMarker(_) => {
                return Err(ParseError::UnexpectedElement {
                    event: format!("{:?}", e),
                })
            }
        }
    }

    Ok(inlines)
}

fn parse_inlines(p: &mut Peekable<Parser>) -> Result<Vec<Inline>, ParseError> {
    let mut inlines = Vec::new();

    while let Some(e) = p.next() {
        trace!("parse_inlines: {:?}", e);

        match e {
            Event::Start(ref tag) => match tag {
                Tag::Paragraph
                | Tag::Heading(_)
                | Tag::BlockQuote
                | Tag::CodeBlock(_)
                | Tag::List(_)
                | Tag::FootnoteDefinition(_)
                | Tag::Table(_)
                | Tag::TableHead
                | Tag::TableRow
                | Tag::TableCell
                | Tag::Item => {
                    return Err(ParseError::UnexpectedElement {
                        event: format!("{:?}", e),
                    })
                }
                Tag::Emphasis => inlines.push(Inline::Emph(parse_inlines(p)?)),
                Tag::Strong => inlines.push(Inline::Strong(parse_inlines(p)?)),
                Tag::Strikethrough => inlines.push(Inline::Strikethrough(parse_inlines(p)?)),
                Tag::Link(_type, s, l) => inlines.push(Inline::Link(s.to_string(), l.to_string())),
                Tag::Image(_type, s, l) => {
                    inlines.push(Inline::Image(s.to_string(), l.to_string()))
                }
            },
            Event::End(_) => break,
            Event::Text(s) => inlines.push(Inline::Plain(s.to_string())),
            Event::Code(s) => inlines.push(Inline::Code(s.to_string())),
            Event::Html(s) => inlines.push(Inline::Html(s.to_string())),
            Event::SoftBreak => inlines.push(Inline::SoftBreak),
            Event::HardBreak => inlines.push(Inline::HardBreak),
            Event::Rule => continue,
            Event::FootnoteReference(_) | Event::TaskListMarker(_) => {
                return Err(ParseError::UnexpectedElement {
                    event: format!("{:?}", e),
                })
            }
        }
    }

    Ok(inlines)
}

fn parse_text(p: &mut Peekable<Parser>) -> Result<String, ParseError> {
    let e = match p.next() {
        None => return Ok("".to_string()),
        Some(e) => e,
    };

    trace!("parse_text: {:?}", e);

    let ret = match e {
        Event::Text(s) => s.to_string(),
        _ => {
            return Err(ParseError::NoText {
                event: format!("{:?}", e),
            })
        }
    };

    match p.next() {
        None | Some(Event::End(_)) => Ok(ret),
        Some(e) => Err(ParseError::ExpectedEnd {
            event: format!("{:?}", e),
        }),
    }
}

impl Parse for Markdown {
    type ParseError = ParseError;

    fn parse(s: &str) -> Result<Nodo, Self::ParseError> {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TASKLISTS);
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        let blocks = parse_blocks(&mut (Parser::new_ext(s, opts)).peekable())?;
        Ok(Nodo { blocks })
    }
}

fn render_blocks<W: std::io::Write>(
    bs: &[Block],
    prefix: &str,
    w: &mut W,
) -> Result<(), std::io::Error> {
    for (i, b) in bs.iter().enumerate() {
        trace!("render_blocks: {:?}", b);

        match b {
            Block::Rule => write!(w, "{}---", prefix)?,
            Block::Paragraph(inlines) => render_inlines(inlines, prefix, w)?,
            Block::Heading(level, inlines) => {
                write!(w, "{}{} ", prefix, "#".repeat(*level as usize))?;
                render_inlines(inlines, prefix, w)?
            }
            Block::Code(lang, content) => write!(w, "{}```{}\n{}```", prefix, lang, content)?,
            Block::Quote(blocks) => {
                write!(w, "{}> ", prefix)?;
                render_blocks(blocks, prefix, w)?
            }
            Block::List(ty, items) => render_list_items(*ty, items, prefix, w)?,
        }

        if i != bs.len() - 1 {
            write!(w, "\n\n{}", prefix)?
        }
    }
    Ok(())
}

fn render_list_items<W: std::io::Write>(
    list_type: ListType,
    is: &[ListItem],
    prefix: &str,
    w: &mut W,
) -> Result<(), std::io::Error> {
    for (i, item) in is.iter().enumerate() {
        trace!("render_list_items: {:?}", item);

        match list_type {
            ListType::Numbered => write!(w, "{}{}. ", if i == 0 { "" } else { prefix }, i + 1)?,
            ListType::Plain => write!(w, "{}- ", if i == 0 { "" } else { prefix })?,
        }

        if let Some(b) = item.task {
            if b {
                write!(w, "[x] ")?
            } else {
                write!(w, "[ ] ")?
            }
        }

        render_blocks(&item.blocks, &format!("{}{}", prefix, INDENT), w)?;

        if i != is.len() - 1 {
            writeln!(w)?
        }
    }

    Ok(())
}

fn render_inlines<W: std::io::Write>(
    is: &[Inline],
    prefix: &str,
    w: &mut W,
) -> Result<(), std::io::Error> {
    for item in is.iter() {
        trace!("render_inlines: {:?}", item);

        match item {
            Inline::Plain(s) | Inline::Html(s) => write!(w, "{}", s)?,
            Inline::Emph(i) => {
                write!(w, "*")?;
                render_inlines(i, "", w)?;
                write!(w, "*")?
            }
            Inline::Strong(i) => {
                write!(w, "**")?;
                render_inlines(i, "", w)?;
                write!(w, "**")?
            }
            Inline::Code(s) => write!(w, "`{}`", s)?,
            Inline::Strikethrough(i) => {
                write!(w, "~~")?;
                render_inlines(i, "", w)?;
                write!(w, "~~")?
            }
            Inline::Link(n, l) => write!(w, "[{}]({})", n, l)?,
            Inline::Image(n, l) => write!(w, "![{}]({})", n, l)?,
            Inline::SoftBreak => {
                writeln!(w)?;
                write!(w, "{}", prefix)?
            }
            Inline::HardBreak => {
                write!(w, "\n\n")?;
                write!(w, "{}", prefix)?
            }
        }
    }
    Ok(())
}

impl Render for Markdown {
    type RenderError = RenderError;

    fn render<W: std::io::Write>(n: &Nodo, w: &mut W) -> Result<(), Self::RenderError> {
        render_blocks(&n.blocks, "", w)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_and_write() {
        let md = "# a *header* with **bold** and ~~strikethrough~~

a + b

- a list
- more list

    - nested list item

a split
paragraph

1. a numbered list

    1. a sub numbered list
2. a second number

- [] a task looking item
- [ ] an incomplete task list
- [x] an complete task list

    paragraph

    - [ ] a sub task

        another paragraph

```rust
some code {
    echo
}
```";
        let nodo = Markdown::parse(md).unwrap();

        let mut out = Vec::new();
        Markdown::render(&nodo, &mut out).unwrap();

        assert_eq!(md, String::from_utf8(out).unwrap(), "{:?}", nodo)
    }
}
