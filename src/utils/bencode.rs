use std::{borrow::Borrow, collections::HashMap, fmt::{Debug, Formatter, Write}, str};

#[derive(PartialEq, Eq)]
pub enum BencodeValue {
    Integer(i64),
    Bytes(Vec<u8>),
    List(Vec<BencodeValue>),
    Dict(HashMap<Vec<u8>, BencodeValue>),
}

impl Debug for BencodeValue {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(n) => {
                print!("hi");
                fmt.write_str(&n.to_string())
            },
            Self::Bytes(bytes) => {
                let mut list_fmt = fmt.debug_list();

                for byte in bytes {
                    list_fmt.entry(&format_args!("{:#04X}", byte));
                }

                list_fmt.finish()
            },
            Self::List(list) => {
                let mut list_fmt = fmt.debug_list();

                for item in list {
                    list_fmt.entry(item);
                }

                list_fmt.finish()
            },
            Self::Dict(dict) => {
                let mut dict_fmt = fmt.debug_struct("BencodeValue::Dict");

                for (key, value) in dict.iter() {

                    let key = {
                        if let Ok(string) = str::from_utf8(key) {
                            format!("{}", string)
                        } else {
                            format!("{:04X?}", key)
                        }
                    };

                    dict_fmt.field(&key, value);
                }

                dict_fmt.finish()
            },
        }
    }
}

impl BencodeValue {
    pub fn serialize(&self) -> Vec<u8> {
        let mut data: Vec<u8> = vec![];

        match self {
            Self::Integer(num) =>
                Self::write_serialized_integer(*num, &mut data),
            Self::Bytes(bytes) =>
                Self::write_serialized_bytes(bytes, &mut data),
            Self::List(list) =>
                Self::write_serialized_list(list, &mut data),
            Self::Dict(dict) =>
                Self::write_serialized_dict(dict, &mut data),
        };

        data
    }

    fn write_serialized_integer(num: i64, buff: &mut Vec<u8>) {
        buff.extend_from_slice(
            format!("i{}e", num).as_bytes()
        );
    }

    fn write_serialized_bytes(bytes: &[u8], buff: &mut Vec<u8>) {
        buff.extend_from_slice(bytes.len().to_string().as_bytes());
        buff.push(':' as u8);
        buff.extend_from_slice(bytes);
    }

    fn write_serialized_list(list: &Vec<BencodeValue>, buff: &mut Vec<u8>) {
        buff.push('l' as u8);

        for value in list {
            buff.extend_from_slice(&value.serialize());
        }

        buff.push('e' as u8)
    }

    fn write_serialized_dict(dict: &HashMap<Vec<u8>, BencodeValue>, buff: &mut Vec<u8>) {
        let mut entries = dict
            .iter()
            .collect::<Vec<(&Vec<u8>, &BencodeValue)>>();

        // TODO: Does the sorting here actually work?
        entries.sort_by(|a, b| a.0.cmp(b.0));

        buff.push('d' as u8);

        for (key, value) in entries {
            // Write the key
            Self::write_serialized_bytes(key, buff);

            // TODO: This could be optimized by making serialize receive the buffer that it writes
            // to instead of creating it
            // Write the value
            buff.extend(value.serialize());
        }

        buff.push('e' as u8);
    }
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

        // println!("PTR: {:?}", self.ptr);

        while self.data[self.ptr] as char != 'e' {
            let key = self.consume_bytes()?;
            // println!("DICT Consumed key");
            // println!("PTR: {:?}", self.ptr);
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
        // println!("Consuming bytes: {:?}", self.ptr);
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



#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::utils::bencode::BencodeParser;

    use super::BencodeValue;

    #[test]
    fn serializes_and_deserializes_list() {
        let list = BencodeValue::List(Vec::from([
            BencodeValue::Integer(15),
            BencodeValue::Bytes("hello world".as_bytes().to_vec()),
        ]));

        let serialized = list.serialize();

        let parsed = BencodeParser::new(&serialized)
            .parse_value()
            .unwrap();

        assert_eq!(list, parsed);
    }

    #[test]
    fn serializes_and_deserializes_complex_nested_values() {
        let list = BencodeValue::List(Vec::from([
            BencodeValue::Integer(15),
            BencodeValue::Bytes("hello world".as_bytes().to_vec()),
            BencodeValue::Dict(HashMap::from([
                (
                    "dict".as_bytes().to_vec(),
                    BencodeValue::Dict(
                        HashMap::from([
                            (
                                "nested_num".as_bytes().to_vec(),
                                BencodeValue::Integer(11111),
                            ),
                        ])
                    ),
                ),
                (
                    "num".as_bytes().to_vec(),
                    BencodeValue::Integer(55),
                ),
                (
                    "bytes".as_bytes().to_vec(),
                    BencodeValue::Bytes("bytes".as_bytes().to_vec()),
                ),
            ])),
        ]));

        let serialized = list.serialize();
        println!("Serialized {:?}", std::str::from_utf8(&serialized));

        let parsed = BencodeParser::new(&serialized)
            .parse_value()
            .unwrap();

        assert_eq!(list, parsed);
    }
}
