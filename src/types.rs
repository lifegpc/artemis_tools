use std::collections::BTreeMap;

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
}

#[derive(Debug)]
pub struct AstFile {
    pub astver: f64,
    pub astname: String,
    pub ast: Value,
}

impl AstFile {
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
