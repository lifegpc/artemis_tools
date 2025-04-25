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
            let title = if let Some(lang) = &self.language {
                title.get(lang).or_else(|| title.get("text"))
            } else {
                title.first_key_value().map(|(k, v)| {
                    if k != "text" {
                        self.language = Some(k.to_string());
                    }
                    v
                })
            };
            if let Some(title) = title {
                self.render_title(title)?;
            }
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
                self.count += 1;
                write!(self.writer, "{}. ", self.count)?;
                if let Some(labels) = &dialogue.labels {
                    for label in labels.iter() {
                        write!(self.writer, "<a name=\"{}\"></a>", label)?;
                    }
                }
                let d = if let Some(lang) = &self.language {
                    dialogue.dialogues.get(lang)
                } else {
                    dialogue.dialogues.first_key_value().map(|(k, v)| {
                        self.language = Some(k.to_string());
                        v
                    })
                };
                if let Some(ds) = d {
                    if ds.len() == 1 {
                        let d = &ds[0];
                        let text = d.text.trim_end().replace("\n", "  \n");
                        if let Some(name) = &d.name {
                            writeln!(self.writer, "{}: {}", name, text)?;
                        } else {
                            writeln!(self.writer, "{}", text)?;
                        }
                    } else {
                        writeln!(self.writer, "")?;
                        for d in ds {
                            let text = d.text.trim_end().replace("\n", "  \n");
                            if let Some(name) = &d.name {
                                writeln!(self.writer, "  - {}: {}", name, text)?;
                            } else {
                                writeln!(self.writer, "  - {}", text)?;
                            }
                        }
                    }
                } else {
                    writeln!(self.writer, "")?;
                }
            }
            Message::ExCall(excall) => {
                self.count += 1;
                write!(self.writer, "{}. ", self.count)?;
                if let Some(labels) = &excall.labels {
                    for label in labels.iter() {
                        write!(self.writer, "<a name=\"{}\"></a>", label)?;
                    }
                }
                if excall.excalls.len() == 1 {
                    let excall = &excall.excalls[0];
                    if let (Some(file), Some(label)) = (&excall.file, &excall.label) {
                        writeln!(self.writer, "[{}]({}.ast#{})", file, file, label)?;
                    } else if let Some(file) = &excall.file {
                        writeln!(self.writer, "[{}]({}.ast)", file, file)?;
                    } else if let Some(label) = &excall.label {
                        writeln!(self.writer, "[{}](#{})", label, label)?;
                    }
                } else {
                    writeln!(self.writer, "")?;
                    for excall in &excall.excalls {
                        if let (Some(file), Some(label)) = (&excall.file, &excall.label) {
                            writeln!(self.writer, "  - [{}]({}.ast#{})", file, file, label)?;
                        } else if let Some(file) = &excall.file {
                            writeln!(self.writer, "  - [{}]({}.ast)", file, file)?;
                        } else if let Some(label) = &excall.label {
                            writeln!(self.writer, "  - [{}](#{})", label, label)?;
                        }
                    }
                }
            }
            Message::Select(select) => {
                self.count += 1;
                write!(self.writer, "{}. ", self.count)?;
                if let Some(labels) = &select.labels {
                    for label in labels.iter() {
                        write!(self.writer, "<a name=\"{}\"></a>", label)?;
                    }
                }
                let select = if let Some(lang) = &self.language {
                    select.sels.get(lang)
                } else {
                    select.sels.first_key_value().map(|(k, v)| {
                        self.language = Some(k.to_string());
                        v
                    })
                };
                if let Some(select) = select {
                    writeln!(self.writer, "")?;
                    for sel in select {
                        if let (Some(file), Some(label)) = (&sel.file, &sel.label) {
                            writeln!(self.writer, "  - [{}]({}.ast#{})", sel.text, file, label)?;
                        } else if let Some(file) = &sel.file {
                            writeln!(self.writer, "  - [{}]({}.ast)", sel.text, file)?;
                        } else if let Some(label) = &sel.label {
                            writeln!(self.writer, "  - [{}](#{})", sel.text, label)?;
                        } else {
                            writeln!(self.writer, "  - {}", sel.text)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
