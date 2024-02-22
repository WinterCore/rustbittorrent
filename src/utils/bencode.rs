use std::collections::HashMap;

#[derive(Debug)]
pub enum BencodeValue {
    Integer(i64),
    Bytes(Vec<u8>),
    List(Vec<BencodeValue>),
    Dict(HashMap<Vec<u8>, BencodeValue>),
}

impl TryFrom<&[u8]> for BencodeValue {
    type Error = BencodeParserError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(Debug)]
pub struct BencodeParserError {
    error: String,
    pos: usize,
}

#[derive(Debug)]
pub struct BencodeParser<'data> {
    data: &'data [u8],
    ptr: usize,
}

impl<'data> BencodeParser<'data> {
    pub fn new(data: &'data [u8]) -> Self {
        Self { data, ptr: 0 }
    }

    pub fn parse_value(&mut self) -> Result<BencodeValue, BencodeParserError> {
        match self.data[self.ptr] as char {
            'i' => Ok(BencodeValue::Integer(self.consume_integer()?)),
            'l' => Ok(BencodeValue::List(self.consume_list()?)),
            'd' => Ok(BencodeValue::Dict(self.consume_dict()?)),
            c => {
                if c.is_ascii_digit() {
                    Ok(BencodeValue::Bytes(self.consume_bytes()?))
                } else {
                    Err(BencodeParserError {
                        error: format!("Unexpected {}", c),
                        pos: self.ptr,
                    })
                }
            },
        }
    }

    fn consume_dict(
        &mut self,
    ) -> Result<HashMap<Vec<u8>, BencodeValue>, BencodeParserError> {
        let mut dict: HashMap<Vec<u8>, BencodeValue> = HashMap::new();

        // Skip the start marker 'd'
        self.ptr += 1;

        println!("PTR: {:?}", self.ptr);

        while self.data[self.ptr] as char != 'e' {
            let key = self.consume_bytes()?;
            println!("DICT Consumed key");
            println!("PTR: {:?}", self.ptr);
            let value = self.parse_value()?;
            
            dict.insert(key, value);
        }

        // Skip the end marker 'e'
        self.ptr += 1;

        Ok(dict)
    }

    fn consume_list(&mut self) -> Result<Vec<BencodeValue>, BencodeParserError> {
        // Skip the start marker 'l'
        self.ptr += 1;

        let mut list: Vec<BencodeValue> = vec![];

        while self.data[self.ptr] as char != 'e' {
            list.push(self.parse_value()?);
        }

        // Skip the ending marker 'e'
        self.ptr += 1;

        Ok(list)
    }

    fn consume_integer(&mut self) -> Result<i64, BencodeParserError> {
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

        *ptr += integer_length + 1;

        Ok(integer)
    }

    fn consume_bytes(&mut self) -> Result<Vec<u8>, BencodeParserError> {
        println!("Consuming bytes: {:?}", self.ptr);
        let Self { data, ptr } = self;

        let len = {
            let str = data[*ptr..]
                .iter()
                .take_while(|c| **c as char != ':')
                .map(|x| *x as char)
                .collect::<String>();

            let len: usize = str
                .parse()
                .map_err(|_|
                    BencodeParserError {
                        error: format!("Failed to parse bytestring length {}", str),
                        pos: *ptr,
                    }
                )?;

            *ptr += str.len() + 1;

            len
        };
        let bytes = data[(*ptr)..(*ptr + len)].to_owned();

        *ptr += len;

        Ok(bytes)
    }
}
