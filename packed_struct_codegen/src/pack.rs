extern crate quote;
extern crate syn;

use crate::pack_parse::*;
use std::fmt::Display;
use std::ops::*;

#[derive(Debug)]
pub struct FieldMidPositioning {
    pub bit_width: usize,
    pub bits_position: BitsPositionParsed,
}

pub enum FieldKind {
    Regular {
        ident: syn::Ident,
        field: FieldRegular,
    },
    Array {
        ident: syn::Ident,
        size: usize,
        elements: Vec<FieldRegular>,
    },
}

impl Display for FieldKind {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Array {
                ident,
                size,
                elements,
            } => {
                write!(fmt, "Ident: [{}]\nSize: [{}]\n", ident, size)?;
                for e in elements {
                    write!(fmt, "field: [{}]\n", e)?;
                }
                Ok(())
            }
            Self::Regular { ident, field } => {
                write!(fmt, "Ident: [{}]\nfield: [{}]\n", ident, field)
            }
        }
    }
}

pub struct FieldRegular {
    pub ty: syn::Type,
    pub serialization_wrappers: Vec<SerializationWrapper>,
    pub bit_width: usize,
    /// The range as parsed by our parser. A single byte: 0..7
    pub bit_range: Range<usize>,
    /// The range that can be used by rust's slices. A single byte: 0..8
    pub bit_range_rust: Range<usize>,
}

impl Display for FieldRegular {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            fmt,
            "bit range: [{}..{}]",
            self.bit_range.start, self.bit_range.end
        )
    }
}

#[derive(Clone)]
pub enum SerializationWrapper {
    IntegerWrapper { integer: syn::Type },
    EndiannesWrapper { endian: syn::Type },
    PrimitiveEnumWrapper,
}

pub struct PackStruct<'a> {
    pub fields: Vec<FieldKind>,
    pub num_bytes: usize,
    pub num_bits: usize,
    pub data_struct: &'a syn::DataStruct,
    pub derive_input: &'a syn::DeriveInput,
}
