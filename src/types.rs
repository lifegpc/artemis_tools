use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug)]
pub enum Value {
    Float(f64),
    Int(i64),
    Str(String),
    KeyVal((String, Box<Value>)),
    Array(Vec<Value>),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    pub fn find_array_attrs(&self, key: &str) -> Vec<&Value> {
        match self {
            Value::Array(arr) => {
                let mut result = Vec::new();
                for v in arr {
                    match v {
                        Value::Array(arr) => {
                            if arr.len() > 0 {
                                if let Value::Str(s) = &arr[0] {
                                    if s == key {
                                        result.push(v);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                result
            }
            _ => Vec::new(),
        }
    }

    pub fn find_keyval(&self, key: &str) -> Option<&Value> {
        match self {
            Value::KeyVal((k, v)) => {
                if k == key {
                    Some(v)
                } else {
                    None
                }
            }
            Value::Array(arr) => {
                for v in arr {
                    match v {
                        Value::KeyVal((k, v)) => {
                            if k == key {
                                return Some(v);
                            }
                        }
                        _ => {}
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn get_member(&self, index: usize) -> Option<&Value> {
        match self {
            Value::Array(arr) => arr.get(index),
            _ => None,
        }
    }

    pub fn last(&self) -> Option<&Value> {
        match self {
            Value::Array(arr) => arr.last(),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct AstFile {
    pub astver: f64,
    pub astname: String,
    pub ast: Value,
}

impl AstFile {
    pub fn get_blocks(&self) -> HashMap<String, Box<Value>> {
        let mut blocks = HashMap::<String, Box<Value>>::new();
        match &self.ast {
            Value::Array(arr) => {
                for v in arr {
                    match v {
                        Value::KeyVal((k, v)) => {
                            if let Some(ori) = blocks.get(k) {
                                let ori_line = ori.find_keyval("line").map_or(None, |v| v.as_int());
                                let line = v.find_keyval("line").map_or(None, |v| v.as_int());
                                if let (Some(ori_line), Some(line)) = (ori_line, line) {
                                    if line > ori_line {
                                        blocks.insert(k.clone(), v.clone());
                                    }
                                } else if let Some(_) = line {
                                    blocks.insert(k.clone(), v.clone());
                                }
                            } else {
                                blocks.insert(k.clone(), v.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        blocks
    }

    pub fn get_messages(&self) -> anyhow::Result<Messages> {
        let mut result = Messages::default();
        let blocks = self.get_blocks();
        let label = blocks
            .get("label")
            .ok_or(anyhow::anyhow!("label block not found"))?;
        let mut label = label
            .find_keyval("top")
            .map_or(None, |v| v.find_keyval("block"))
            .map_or(None, |v| v.as_str())
            .ok_or(anyhow::anyhow!("Can not get top block from label"))?;
        loop {
            let block = match blocks.get(label) {
                Some(b) => b,
                None => break,
            };
            if result.savetitle.is_none() {
                if let Some(v) = block.find_array_attrs("savetitle").first() {
                    if let Some(v) = v.find_keyval("text").map_or(None, |v| v.as_str()) {
                        result.savetitle = Some(v.to_string());
                    }
                }
            }
            if let Some(v) = block.find_array_attrs("excall").first() {
                let file = v
                    .find_keyval("file")
                    .map_or(None, |v| v.as_str())
                    .ok_or(anyhow::anyhow!("Can not get file from excall"))?;
                result.messages.push(Message::ExCall(ExCall {
                    file: file.to_string(),
                }));
            } else {
                let selects = block.find_array_attrs("select");
                if selects.is_empty() {
                    let text = block.find_keyval("text").ok_or(anyhow::anyhow!(
                        "Can not get text from dialogue block {}",
                        label
                    ))?;
                    match text {
                        Value::Array(v) => {
                            let mut tmp = BTreeMap::new();
                            for t in v {
                                match t {
                                    Value::KeyVal((k, v)) => {
                                        if k == "vo" {
                                            continue;
                                        }
                                        let vec = if tmp.contains_key(k) {
                                            tmp.get_mut(k).unwrap()
                                        } else {
                                            tmp.insert(k.to_string(), Vec::new());
                                            tmp.get_mut(k).unwrap()
                                        };
                                        match v.as_ref() {
                                            Value::Array(v) => {
                                                for v in v {
                                                    let name = v
                                                        .find_keyval("name")
                                                        .map_or(None, |v| v.last())
                                                        .map_or(None, |v| v.as_str())
                                                        .map(|v| v.to_string());
                                                    let mut text = String::new();
                                                    match v {
                                                        Value::Array(v) => {
                                                            for v in v {
                                                                match v {
                                                                    Value::Str(s) => {
                                                                        text.push_str(s)
                                                                    }
                                                                    Value::Array(s) => {
                                                                        let ok = if s.len() == 1 {
                                                                            if let Value::Str(s) =
                                                                                &s[0]
                                                                            {
                                                                                if s == "rt2"
                                                                                    || s == "ret2"
                                                                                {
                                                                                    text.push_str(
                                                                                        "\n",
                                                                                    );
                                                                                    true
                                                                                } else {
                                                                                    false
                                                                                }
                                                                            } else {
                                                                                false
                                                                            }
                                                                        } else {
                                                                            false
                                                                        };
                                                                        if !ok {
                                                                            return Err(
                                                                                anyhow::anyhow!(
                                                                                    "Invalid text in dialogue block {}: {:?}",
                                                                                    label,
                                                                                    v
                                                                                ),
                                                                            );
                                                                        }
                                                                    }
                                                                    Value::KeyVal(_) => {}
                                                                    _ => {
                                                                        return Err(
                                                                            anyhow::anyhow!(
                                                                                "Invalid text in dialogue block {}: {:?}",
                                                                                label,
                                                                                v
                                                                            ),
                                                                        );
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                    vec.push(Dialogue { text, name });
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            loop {
                                let is_empty = tmp.iter().all(|(_, v)| v.is_empty());
                                if is_empty {
                                    break;
                                }
                                let mut tmp2 = BTreeMap::new();
                                for (k, v) in tmp.iter_mut() {
                                    if let Some(v) = v.pop() {
                                        tmp2.insert(k.clone(), v);
                                    }
                                }
                                result.messages.push(Message::Dialogue(tmp2));
                            }
                        }
                        _ => {}
                    }
                } else {
                    let mut tmp = BTreeMap::new();
                    let mut used = BTreeMap::new();
                    for select in selects {
                        let text = match select.find_keyval("text").map_or(None, |key| key.as_str())
                        {
                            Some(v) => v,
                            None => continue,
                        };
                        let file = select
                            .find_keyval("file")
                            .map_or(None, |v| v.as_str())
                            .ok_or(anyhow::anyhow!(
                                "Can not get file from select block {}",
                                label
                            ))?;
                        if !used.contains_key(text) {
                            used.insert(text, 0);
                        }
                        let count = used.get_mut(text).unwrap();
                        let text_block = block.find_keyval(text).ok_or(anyhow::anyhow!(
                            "Can not get text block {} from select block {}",
                            text,
                            label
                        ))?;
                        match text_block {
                            Value::Array(v) => {
                                for v in v {
                                    match v {
                                        Value::KeyVal((k, v)) => {
                                            let vec = if tmp.contains_key(k) {
                                                tmp.get_mut(k).unwrap()
                                            } else {
                                                tmp.insert(k.to_string(), Vec::new());
                                                tmp.get_mut(k).unwrap()
                                            };
                                            let text = v
                                                .get_member(*count)
                                                .map_or(None, |v| v.as_str())
                                                .ok_or(anyhow::anyhow!(
                                                    "Can not get text from select block {}",
                                                    label
                                                ))?;
                                            *count += 1;
                                            vec.push(Select {
                                                text: text.to_string(),
                                                file: file.to_string(),
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    result.messages.push(Message::Select(tmp));
                }
            }
            label = match block.find_keyval("linknext").map_or(None, |v| v.as_str()) {
                Some(v) => v,
                None => break,
            };
        }
        Ok(result)
    }

    pub fn sort_blocks(&mut self) {
        match &mut self.ast {
            Value::Array(arr) => {
                let mut maps = BTreeMap::new();
                let mut others = Vec::new();
                while let Some(o) = arr.pop() {
                    match o {
                        Value::KeyVal((k, v)) => {
                            let line = v.find_keyval("line").map_or(None, |v| v.as_int());
                            if let Some(line) = line {
                                maps.insert(line, Value::KeyVal((k, v)));
                            } else {
                                others.push(Value::KeyVal((k, v)));
                            }
                        }
                        _ => others.push(o),
                    }
                }
                for (_, v) in maps {
                    arr.push(v);
                }
                for o in others {
                    arr.push(o);
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct Dialogue {
    pub text: String,
    pub name: Option<String>,
}

#[derive(Debug)]
pub struct ExCall {
    pub file: String,
}

#[derive(Debug)]
pub struct Select {
    pub text: String,
    pub file: String,
}

#[derive(Debug)]
pub enum Message {
    Dialogue(BTreeMap<String, Dialogue>),
    ExCall(ExCall),
    Select(BTreeMap<String, Vec<Select>>),
}

#[derive(Debug, Default)]
pub struct Messages {
    pub savetitle: Option<String>,
    pub messages: Vec<Message>,
}
