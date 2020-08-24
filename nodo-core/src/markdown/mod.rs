use crate::{Block, Inline, ListItem, ListType, Nodo, Read, Write};
use pulldown_cmark::{Event, Options, Parser, Tag};

pub struct Markdown;

fn read_blocks(p: &mut Parser) -> Vec<Block> {
    let mut blocks = Vec::new();

    loop {
        let e = match p.next() {
            None => break,
            Some(e) => e,
        };

        dbg!(&e);

        match e {
            Event::Start(tag) => match tag {
                Tag::Heading(level) => blocks.push(Block::Heading(level, read_inlines(p))),
                Tag::Paragraph => blocks.push(Block::Paragraph(read_inlines(p))),
                Tag::BlockQuote => blocks.push(Block::Quote(read_text(p))),
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
            Event::Text(_)
            | Event::Code(_)
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

fn read_list_items(p: &mut Parser) -> Vec<ListItem> {
    let mut items = Vec::new();

    loop {
        let mut peekable = p.peekable();
        let e = match peekable.peek() {
            None => break,
            Some(e) => e,
        };

        dbg!(&e);

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
                | Tag::TableCell => unimplemented!(),
                Tag::Item
                | Tag::Emphasis
                | Tag::Strong
                | Tag::Strikethrough
                | Tag::Link(_, _, _)
                | Tag::Image(_, _, _) => items.push(ListItem(None, read_inlines(p))),
            },
            Event::End(_) => break,
            Event::Text(_) | Event::Code(_) | Event::Html(_) => {
                items.push(ListItem(None, read_inlines(p)))
            }
            Event::FootnoteReference(_) => todo!(),
            Event::SoftBreak | Event::HardBreak => items.push(ListItem(None, read_inlines(p))),
            Event::Rule => continue,
            Event::TaskListMarker(b) => items.push(ListItem(Some(*b), read_inlines(p))),
        }
    }

    items
}

fn read_inlines(p: &mut Parser) -> Vec<Inline> {
    let mut inlines = Vec::new();

    loop {
        let e = match p.next() {
            None => break,
            Some(e) => e,
        };

        dbg!(&e);

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
                | Tag::TableCell => unimplemented!(),
                Tag::Item => inlines.push(Inline::ListItem(read_inlines(p))),
                Tag::Emphasis => inlines.push(Inline::Emph(read_text(p))),
                Tag::Strong => inlines.push(Inline::Strong(read_text(p))),
                Tag::Strikethrough => inlines.push(Inline::Strikethrough(read_text(p))),
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
            Event::TaskListMarker(_) => continue,
        }
    }

    inlines
}

fn read_text(p: &mut Parser) -> String {
    let e = match p.next() {
        None => return "".to_string(),
        Some(e) => e,
    };

    dbg!(&e);

    let ret = match e {
        Event::Text(s) => s.to_string(),
        _ => unimplemented!(),
    };

    if let Some(Event::End(_)) = p.next() {
        return ret;
    }

    panic!("nope")
}

impl Read for Markdown {
    fn read(s: &str) -> Nodo {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TASKLISTS);
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        let blocks = read_blocks(&mut Parser::new_ext(s, opts));
        Nodo { blocks }
    }
}

fn write_blocks<W: std::fmt::Write>(bs: &[Block], w: &mut W) -> Result<(), std::fmt::Error> {
    for (i, b) in bs.iter().enumerate() {
        match b {
            Block::Rule => write!(w, "---")?,
            Block::Paragraph(inlines) => write_inlines(inlines, w)?,
            Block::Heading(level, inlines) => {
                write!(w, "{} ", "#".repeat(*level as usize))?;
                write_inlines(inlines, w)?
            }
            Block::Code(lang, content) => write!(w, "```{}\n{}```", lang, content)?,
            Block::Quote(content) => {
                for l in content.lines() {
                    write!(w, "> {}", l)?
                }
            }
            Block::List(ty, items) => write_list_items(*ty, items, w)?,
        }
        if i != bs.len() - 1 {
            write!(w, "\n\n")?
        }
    }
    Ok(())
}

fn write_list_items<W: std::fmt::Write>(
    list_type: ListType,
    is: &[ListItem],
    w: &mut W,
) -> Result<(), std::fmt::Error> {
    for (i, item) in is.iter().enumerate() {
        match list_type {
            ListType::Numbered => write!(w, "{}. ", i + 1)?,
            ListType::Plain => write!(w, "- ")?,
        }
        if let Some(b) = item.0 {
            if b {
                write!(w, "[x] ")?
            } else {
                write!(w, "[ ] ")?
            }
        }

        write_inlines(&item.1, w)?
    }

    Ok(())
}

fn write_inlines<W: std::fmt::Write>(is: &[Inline], w: &mut W) -> Result<(), std::fmt::Error> {
    for i in is.iter() {
        match i {
            Inline::Plain(s) => write!(w, "{}", s)?,
            Inline::Emph(s) => write!(w, "*{}*", s)?,
            Inline::Strong(s) => write!(w, "**{}**", s)?,
            Inline::Code(s) => write!(w, "`{}`", s)?,
            Inline::Strikethrough(s) => write!(w, "~~{}~~", s)?,
            Inline::Link(n, l) => write!(w, "[{}]({})", n, l)?,
            Inline::Image(n, l) => write!(w, "![{}]({})", n, l)?,
            Inline::Html(s) => write!(w, "{}", s)?,
            Inline::SoftBreak => write!(w, "\n")?,
            Inline::HardBreak => write!(w, "\n\n")?,
            Inline::ListItem(inlines) => write_inlines(inlines, w)?,
        }
    }
    Ok(())
}

impl Write for Markdown {
    fn write<W: std::fmt::Write>(n: &Nodo, w: &mut W) -> Result<(), std::fmt::Error> {
        write_blocks(&n.blocks, w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_and_write() {
        let md = "# a *header* with **bold** and ~~strikethrough~~\n\na + b\n\n- a list\n\n1. a numbered list\n\n```rust\nsome code\n```";
        let nodo = Markdown::read(md);

        let mut out = String::new();
        Markdown::write(&nodo, &mut out).unwrap();

        assert_eq!(md, out)
    }
}
