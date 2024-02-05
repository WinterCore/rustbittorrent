use std::collections::HashMap;


#[derive(Debug)]
pub enum Value {
    Integer(i64),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Dict(HashMap<Vec<u8>, Value>),
}

impl TryFrom<&[u8]> for Value {
    type Error = BencodeParserError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        todo!()
    }
}

pub struct BencodeParserError {
    error: String,
    pos: usize,
}

struct BencodeParser<'data> {
    data: &'data [u8],
    ptr: usize,
}

impl<'data> BencodeParser<'data> {
    fn new(data: &'data [u8]) -> Self {
        Self { data, ptr: 0 }
    }

    fn parse(&mut self) -> Result<Value, BencodeParserError> {
        match self.data[self.ptr] as char {
            'i' => self.consume_integer(),
            _ => unimplemented!(),
        }
    }

    fn consume_integer(&mut self) -> Result<Value, BencodeParserError> {
        let Self { data, ptr } = self;

        *ptr += 1; // Skip the 'i'

        let integer_length: usize = data[*ptr..]
            .into_iter()
            .take_while(|b| **b as char != 'e')
            .count();

        let number_str = String::from_utf8(data[*ptr..(*ptr + integer_length)].to_owned());

        let integer = number_str
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .ok_or(BencodeParserError {
                error: format!("Failed to parse integer"),
                pos: *ptr,
            })?;

        *ptr += integer_length;

        Ok(Value::Integer(integer))
    }
}
