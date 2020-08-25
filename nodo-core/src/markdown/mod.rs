use crate::{Block, Inline, ListItem, ListType, Nodo, Read, Write};
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::iter::Peekable;

#[cfg(not(test))]
use log::debug;

#[cfg(test)]
use std::println as debug;

pub struct Markdown;

fn read_blocks(p: &mut Peekable<Parser>) -> Vec<Block> {
    let mut blocks = Vec::new();

    loop {
        let e = match p.next() {
            None => return blocks,
            Some(e) => e,
        };

        debug!("read_blocks: {:?}", e);

        match e {
            Event::Start(tag) => match tag {
                Tag::Heading(level) => blocks.push(Block::Heading(level, read_inlines(p))),
                Tag::Paragraph => blocks.push(Block::Paragraph(read_inlines(p))),
                Tag::BlockQuote => blocks.push(Block::Quote(read_blocks(p))),
                Tag::CodeBlock(kind) => {
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                    };
                    blocks.push(Block::Code(lang, read_text(p)))
                }
                Tag::List(kind) => {
                    if kind.is_some() {
                        blocks.push(Block::List(ListType::Numbered, read_list_items(p)))
                    } else {
                        blocks.push(Block::List(ListType::Plain, read_list_items(p)))
                    }
                }
                Tag::Item => unimplemented!(),
                Tag::FootnoteDefinition(_) => unimplemented!(),
                Tag::TableHead | Tag::TableRow | Tag::TableCell => unimplemented!(),
                Tag::Table(_) => unimplemented!(),
                Tag::Emphasis => unimplemented!(),
                Tag::Strikethrough | Tag::Strong | Tag::Link(_, _, _) | Tag::Image(_, _, _) => {
                    unimplemented!()
                }
            },
            Event::End(_) => break,
            Event::Text(s) => {
                let mut text = vec![Inline::Plain(s.to_string())];
                let mut inlines = read_tight_paragraph(p);
                text.append(&mut inlines);
                blocks.push(Block::Paragraph(text))
            }
            Event::Code(_)
            | Event::Html(_)
            | Event::FootnoteReference(_)
            | Event::HardBreak
            | Event::SoftBreak => unimplemented!(),
            Event::Rule => blocks.push(Block::Rule),
            Event::TaskListMarker(_) => unimplemented!(),
        }
    }

    blocks
}

fn read_list_items(p: &mut Peekable<Parser>) -> Vec<ListItem> {
    let mut items = Vec::new();

    loop {
        let e = match p.next() {
            None => break,
            Some(e) => e,
        };

        debug!("read_list_items: {:?}", e);

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
                | Tag::Emphasis
                | Tag::Strong
                | Tag::Strikethrough
                | Tag::Link(_, _, _)
                | Tag::Image(_, _, _) => unimplemented!(),
                Tag::Item => match p.peek() {
                    None => break,
                    Some(Event::TaskListMarker(b)) => {
                        let b = *b;
                        p.next().unwrap();
                        items.push(ListItem(Some(b), read_blocks(p)))
                    }
                    _ => items.push(ListItem(None, read_blocks(p))),
                },
            },
            Event::End(_) => break,
            Event::Text(_) | Event::Code(_) | Event::Html(_) => {
                items.push(ListItem(None, read_blocks(p)))
            }
            Event::FootnoteReference(_) => todo!(),
            Event::SoftBreak | Event::HardBreak => items.push(ListItem(None, read_blocks(p))),
            Event::Rule => continue,
            Event::TaskListMarker(b) => items.push(ListItem(Some(b), read_blocks(p))),
        }
    }

    items
}

fn read_tight_paragraph(p: &mut Peekable<Parser>) -> Vec<Inline> {
    let mut inlines = Vec::new();

    loop {
        let e = match p.peek() {
            None => break,
            Some(e) => e,
        };

        debug!("read_tight_paragraph: {:?}", e);

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
                    inlines.push(Inline::Emph(read_tight_paragraph(p)))
                }
                Tag::Strong => {
                    p.next().unwrap();
                    inlines.push(Inline::Strong(read_tight_paragraph(p)))
                }
                Tag::Strikethrough => {
                    p.next().unwrap();
                    inlines.push(Inline::Strikethrough(read_tight_paragraph(p)))
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
            Event::FootnoteReference(_) => todo!(),
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
            Event::TaskListMarker(_) => unimplemented!(),
        }
    }

    inlines
}

fn read_inlines(p: &mut Peekable<Parser>) -> Vec<Inline> {
    let mut inlines = Vec::new();

    loop {
        let e = match p.next() {
            None => break,
            Some(e) => e,
        };

        debug!("read_inlines: {:?}", e);

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
                | Tag::Item => unimplemented!(),
                Tag::Emphasis => inlines.push(Inline::Emph(read_inlines(p))),
                Tag::Strong => inlines.push(Inline::Strong(read_inlines(p))),
                Tag::Strikethrough => inlines.push(Inline::Strikethrough(read_inlines(p))),
                Tag::Link(_type, s, l) => inlines.push(Inline::Link(s.to_string(), l.to_string())),
                Tag::Image(_type, s, l) => {
                    inlines.push(Inline::Image(s.to_string(), l.to_string()))
                }
            },
            Event::End(_) => break,
            Event::Text(s) => inlines.push(Inline::Plain(s.to_string())),
            Event::Code(s) => inlines.push(Inline::Code(s.to_string())),
            Event::Html(s) => inlines.push(Inline::Html(s.to_string())),
            Event::FootnoteReference(_) => todo!(),
            Event::SoftBreak => inlines.push(Inline::SoftBreak),
            Event::HardBreak => inlines.push(Inline::HardBreak),
            Event::Rule => continue,
            Event::TaskListMarker(_) => unimplemented!(),
        }
    }

    inlines
}

fn read_text(p: &mut Peekable<Parser>) -> String {
    let e = match p.next() {
        None => return "".to_string(),
        Some(e) => e,
    };

    debug!("read_text: {:?}", e);

    let ret = match e {
        Event::Text(s) => s.to_string(),
        _ => unimplemented!(),
    };

    match p.next() {
        None => return ret,
        Some(Event::End(_)) => return ret,
        Some(e) => panic!("{:?}", e),
    }
}

impl Read for Markdown {
    fn read(s: &str) -> Nodo {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TASKLISTS);
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        let blocks = read_blocks(&mut (Parser::new_ext(s, opts)).peekable());
        Nodo { blocks }
    }
}

fn write_blocks<W: std::io::Write>(
    bs: &[Block],
    prefix: &str,
    w: &mut W,
) -> Result<(), std::io::Error> {
    let mut prev = None;
    for (i, b) in bs.iter().enumerate() {
        match b {
            Block::Rule => write!(w, "{}---", prefix)?,
            Block::Paragraph(inlines) => write_inlines(inlines, w)?,
            Block::Heading(level, inlines) => {
                write!(w, "{}{} ", prefix, "#".repeat(*level as usize))?;
                write_inlines(inlines, w)?
            }
            Block::Code(lang, content) => write!(w, "{}```{}\n{}```", prefix, lang, content)?,
            Block::Quote(blocks) => {
                write!(w, "{}> ", prefix)?;
                write_blocks(blocks, prefix, w)?
            }
            Block::List(ty, items) => write_list_items(*ty, items, prefix, w)?,
        }

        if i != bs.len() - 1 {
            let mut skip = false;
            if prefix != "" {
                if let Some(&Block::Paragraph(_)) = prev {
                    if let Block::List(_, _) = b {
                        skip = true
                    }
                }
            }

            if !skip {
                write!(w, "\n\n")?
            }
        }

        prev = Some(b)
    }
    Ok(())
}

fn write_list_items<W: std::io::Write>(
    list_type: ListType,
    is: &[ListItem],
    prefix: &str,
    w: &mut W,
) -> Result<(), std::io::Error> {
    for (i, item) in is.iter().enumerate() {
        match list_type {
            ListType::Numbered => write!(w, "{}{}. ", prefix, i + 1)?,
            ListType::Plain => write!(w, "{}- ", prefix)?,
        }
        if let Some(b) = item.0 {
            if b {
                write!(w, "[x] ")?
            } else {
                write!(w, "[ ] ")?
            }
        }

        write_blocks(&item.1, &format!("{}  ", prefix), w)?;

        if i != is.len() - 1 {
            writeln!(w)?
        }
    }

    Ok(())
}

fn write_inlines<W: std::io::Write>(is: &[Inline], w: &mut W) -> Result<(), std::io::Error> {
    for i in is.iter() {
        match i {
            Inline::Plain(s) => write!(w, "{}", s)?,
            Inline::Emph(i) => {
                write!(w, "*")?;
                write_inlines(i, w)?;
                write!(w, "*")?
            }
            Inline::Strong(i) => {
                write!(w, "**")?;
                write_inlines(i, w)?;
                write!(w, "**")?
            }
            Inline::Code(s) => write!(w, "`{}`", s)?,
            Inline::Strikethrough(i) => {
                write!(w, "~~")?;
                write_inlines(i, w)?;
                write!(w, "~~")?
            }
            Inline::Link(n, l) => write!(w, "[{}]({})", n, l)?,
            Inline::Image(n, l) => write!(w, "![{}]({})", n, l)?,
            Inline::Html(s) => write!(w, "{}", s)?,
            Inline::SoftBreak => writeln!(w)?,
            Inline::HardBreak => write!(w, "\n\n")?,
        }
    }
    Ok(())
}

impl Write for Markdown {
    fn write<W: std::io::Write>(n: &Nodo, w: &mut W) -> Result<(), std::io::Error> {
        write_blocks(&n.blocks, "", w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn read_and_write() {
        let md = "# a *header* with **bold** and ~~strikethrough~~

a + b

- a list
- more list

  - nested list item

1. a numbered list
2. a second number

- [] a task looking item
- [ ] an incomplete task list
- [x] an complete task list

```rust
some code {
    echo
}
```";
        let nodo = Markdown::read(md);

        let mut out = Vec::new();
        Markdown::write(&nodo, &mut out).unwrap();

        assert_eq!(md, String::from_utf8(out).unwrap(), "{:?}", nodo)
    }
}
