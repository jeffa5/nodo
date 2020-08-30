use crate::{Block, Nodo};

pub struct TaskCount {
    pub completed: u32,
    pub total: u32,
}

impl Nodo {
    #[must_use]
    pub fn count_tasks(&self) -> TaskCount {
        let mut completed = 0;
        let mut total = 0;
        for block in &self.blocks {
            match block {
                Block::List(_, items) => {
                    for item in items.iter() {
                        if let Some(c) = item.task {
                            total += 1;
                            if c {
                                completed += 1;
                            }
                        }
                    }
                }
                Block::Paragraph(_)
                | Block::Heading(_, _)
                | Block::Code(_, _)
                | Block::Quote(_)
                | Block::Rule => {}
            }
        }
        TaskCount { completed, total }
    }
}
