use crate::types::{AstFile, Message};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalTranslMessage {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl AstFile {
    pub fn to_galtransl_json(&self, lang: Option<String>) -> anyhow::Result<String> {
        let mut messages = Vec::new();
        let mes = self.get_messages()?;
        let mut lang: Option<String> = lang.clone();
        match &mes.savetitle {
            Some(title) => {
                messages.push(GalTranslMessage {
                    message: title.clone(),
                    name: None,
                });
            }
            None => {}
        };
        for mes in mes.messages.iter() {
            match mes {
                Message::Dialogue(d) => {
                    let d = if let Some(lang) = &lang {
                        d.dialogues.get(lang)
                    } else {
                        d.dialogues.first_key_value().map(|(k, v)| {
                            lang = Some(k.to_string());
                            v
                        })
                    };
                    match d {
                        Some(d) => {
                            for d in d {
                                messages.push(GalTranslMessage {
                                    message: d.text.clone(),
                                    name: d.name.clone(),
                                });
                            }
                        }
                        None => {}
                    }
                }
                Message::Select(sel) => {
                    let select = if let Some(lang) = &lang {
                        sel.sels.get(lang)
                    } else {
                        sel.sels.first_key_value().map(|(k, v)| {
                            lang = Some(k.to_string());
                            v
                        })
                    };
                    match select {
                        Some(sel) => {
                            for sel in sel {
                                messages.push(GalTranslMessage {
                                    message: sel.text.clone(),
                                    name: None,
                                });
                            }
                        }
                        None => {}
                    }
                }
                _ => {}
            }
        }
        if messages.is_empty() {
            return Ok(String::new());
        }
        let json = serde_json::to_string_pretty(&messages)?;
        Ok(json)
    }
}
