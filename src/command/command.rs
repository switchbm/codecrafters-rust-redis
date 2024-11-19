use super::super::resp::RespValue;

pub enum Command {
    Ping(String),
    Echo(String),
    Get(String),
    Set(String, String),
}

impl Command {
    pub fn from_resp_value(resp_value: &RespValue) -> Option<Self> {
        if let RespValue::Array(elements) = resp_value {
            let mut values_iter = elements.iter();
            if let Some(RespValue::BulkString(command_bytes)) = values_iter.next() {
                let command = String::from_utf8_lossy(command_bytes).to_lowercase();

                match command.as_str() {
                    "ping" => Some(Command::Ping(Self::extract_argument(&mut values_iter).unwrap_or_else(|| "PONG".to_string()))),
                    "echo" => Self::extract_argument(&mut values_iter).map(Command::Echo),
                    "get" => Self::extract_argument(&mut values_iter).map(Command::Get),
                    "set" => {
                        let key = Self::extract_argument(&mut values_iter)?;
                        let value = Self::extract_argument(&mut values_iter)?;
                        Some(Command::Set(key, value))
                    }
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Helper function to extract a single argument from the iterator.
    fn extract_argument<'a>(iter: &mut impl Iterator<Item = &'a RespValue>) -> Option<String> {
        iter.next().and_then(|value| match value {
            RespValue::BulkString(data) => Some(String::from_utf8_lossy(data).to_string()),
            _ => None,
        })
    }
}
