use simplicity::bit_machine::ExecTracker;
use simplicity::ffi::ffi::UWORD;
use simplicity::jet::{Elements, Jet};

use crate::jet::{source_type, target_type};
use crate::str::AliasName;
use crate::types::{AliasedType, TypeInner};
use crate::ResolvedType;

pub struct Tracker;

impl ExecTracker<Elements> for Tracker {
    fn track_left(&mut self, _: simplicity::Ihr) {}
    fn track_right(&mut self, _: simplicity::Ihr) {}
    fn track_jet_call(
        &mut self,
        jet: &Elements,
        input_buffer: &[UWORD],
        output_buffer: &[UWORD],
        success: bool,
    ) {
        let args = format_values(input_buffer, source_type(*jet));
        let res = format_values(output_buffer, vec![target_type(*jet)]);
        println!(
            "{:?}({}) = {}",
            jet,
            args.join(", "),
            res.first().unwrap_or(
                &(if success {
                    "ok".to_string()
                } else {
                    "err".to_string()
                })
            )
        );
    }
}

pub struct WordReader<'a> {
    data: &'a [UWORD],
    word_ptr: usize,
    bit_ptr: usize,
    eof: bool,
}

impl<'a> WordReader<'a> {
    fn new(data: &'a [UWORD]) -> Self {
        if data.is_empty() {
            Self {
                data,
                word_ptr: 0,
                bit_ptr: 0,
                eof: true,
            }
        } else {
            Self {
                data,
                word_ptr: data.len() - 1,
                bit_ptr: 8 * std::mem::size_of::<UWORD>() - 1,
                eof: false,
            }
        }
    }

    fn read_bit(&mut self) -> bool {
        assert!(!self.eof, "eof");
        let bit = (self.data[self.word_ptr] >> self.bit_ptr) & 1;
        if self.bit_ptr == 0 {
            if self.word_ptr == 0 {
                self.eof = true;
            } else {
                self.word_ptr -= 1;
                self.bit_ptr = 8 * std::mem::size_of::<UWORD>() - 1;
            }
        } else {
            self.bit_ptr -= 1;
        }
        bit == 1
    }

    fn read_u8(&mut self) -> u8 {
        let mut result = 0;
        for _ in 0..8 {
            result = (result << 1) | self.read_bit() as u8;
        }
        result
    }
}

fn format_values(data: &[UWORD], types: Vec<AliasedType>) -> Vec<String> {
    let mut reader = WordReader::new(data);
    let mut values = Vec::new();

    let get_alias = |_: &AliasName| -> Option<ResolvedType> { None };

    for ty in types {
        let resolved_type = ty.resolve(get_alias).expect("unexpected alias");
        match resolved_type.as_inner() {
            TypeInner::UInt(uint_type) => {
                let mut uint_value = String::with_capacity(uint_type.byte_width() * 2 + 2);
                uint_value.push_str("0x");
                for _ in 0..uint_type.byte_width() {
                    let byte = reader.read_u8();
                    if byte > 0 {
                        uint_value.push_str(&format!("{:x}", byte));
                    }
                }
                values.push(uint_value);
            }
            TypeInner::Boolean => {
                values.push(reader.read_bit().to_string());
            }
            _ => return data.iter().map(|word| format!("{:?}", word)).collect(),
        }
    }

    values
}
