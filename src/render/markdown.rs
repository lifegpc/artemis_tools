use crate::types::*;
use std::io::Write;

pub struct MarkdownRenderer {
    writer: Box<dyn Write>,
    count: usize,
    language: Option<String>,
}

impl MarkdownRenderer {
    pub fn new<W: Write + 'static>(writer: W) -> Self {
        MarkdownRenderer {
            writer: Box::new(writer),
            count: 0,
            language: None,
        }
    }

    pub fn render(mut self, messages: &Messages) -> anyhow::Result<()> {
        if let Some(title) = &messages.savetitle {
            self.render_title(title)?;
        }
        for message in &messages.messages {
            self.render_message(message)?;
        }
        Ok(())
    }

    fn render_title(&mut self, title: &str) -> std::io::Result<()> {
        writeln!(self.writer, "# {}", title)
    }

    fn render_message(&mut self, message: &Message) -> std::io::Result<()> {
        match message {
            Message::Dialogue(dialogue) => {
                let d = if let Some(lang) = &self.language {
                    dialogue
                        .get(lang)
                        .or_else(|| dialogue.first_key_value().map(|(_, v)| v))
                } else {
                    dialogue.first_key_value().map(|(k, v)| {
                        self.language = Some(k.to_string());
                        v
                    })
                };
                if let Some(d) = d {
                    self.count += 1;
                    let text = d.text.trim_end().replace("\n", "  \n");
                    if let Some(name) = &d.name {
                        writeln!(self.writer, "{}. {}: {}", self.count, name, text)?;
                    } else {
                        writeln!(self.writer, "{}. {}", self.count, text)?;
                    }
                }
            }
            Message::ExCall(excall) => {
                self.count += 1;
                writeln!(
                    self.writer,
                    "{}. [{}]({}.ast)",
                    self.count, excall.file, excall.file
                )?;
            }
            Message::Select(select) => {
                let select = if let Some(lang) = &self.language {
                    select
                        .get(lang)
                        .or_else(|| select.first_key_value().map(|(_, v)| v))
                } else {
                    select.first_key_value().map(|(k, v)| {
                        self.language = Some(k.to_string());
                        v
                    })
                };
                if let Some(select) = select {
                    self.count += 1;
                    writeln!(self.writer, "{}. ", self.count)?;
                    for sel in select {
                        writeln!(self.writer, "  - [{}]({}.ast)", sel.text, sel.file)?;
                    }
                }
            }
        }
        Ok(())
    }
}
