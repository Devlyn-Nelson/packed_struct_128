extern crate quote;
extern crate syn;

use crate::pack::*;
use crate::pack_parse_attributes::*;

use syn::spanned::Spanned;

use std::ops::Range;

use crate::utils_syn::{get_expr_int_val, get_single_segment, tokens_to_string};

pub fn parse_sub_attributes(
    attributes: &Vec<syn::Attribute>,
    main_attribute: &str,
    wrong_attribute: &str,
) -> syn::Result<Vec<(String, String)>> {
    let mut r = vec![];

    for attr in attributes {
        let meta = attr.parse_meta()?;
        match &meta {
            &syn::Meta::List(ref metalist) => {
                if let Some(path) = metalist.path.get_ident() {
                    if path == wrong_attribute {
                        return Err(syn::Error::new(
                            path.span(),
                            format!(
                                "This attribute is not supported here, did you mean {:?}?",
                                main_attribute
                            ),
                        ));
                    }
                    if path == main_attribute {
                        for nv in &metalist.nested {
                            match nv {
                                syn::NestedMeta::Meta(m) => match m {
                                    syn::Meta::Path(_) => {}
                                    syn::Meta::List(_) => {}
                                    syn::Meta::NameValue(nv) => {
                                        match (nv.path.get_ident(), &nv.lit) {
                                            (Some(key), syn::Lit::Str(lit)) => {
                                                r.push((key.to_string(), lit.value()));
                                            }
                                            (_, _) => (),
                                        }
                                    }
                                },
                                syn::NestedMeta::Lit(_) => {}
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }

    Ok(r)
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// https://en.wikipedia.org/wiki/Bit_numbering
pub enum BitNumbering {
    Lsb0,
    Msb0,
}

impl BitNumbering {
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.to_lowercase();
        match s.as_str() {
            "lsb0" => Some(BitNumbering::Lsb0),
            "msb0" => Some(BitNumbering::Msb0),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
/// https://en.wikipedia.org/wiki/Endianness
pub enum IntegerEndianness {
    Msb,
    Lsb,
}

impl IntegerEndianness {
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.to_lowercase();
        match s.as_str() {
            "lsb" | "le" => Some(IntegerEndianness::Lsb),
            "msb" | "be" => Some(IntegerEndianness::Msb),
            _ => None,
        }
    }
}

fn get_builtin_type_bit_width(p: &syn::PathSegment) -> syn::Result<Option<usize>> {
    match p.ident.to_string().as_str() {
        "bool" => Ok(Some(1)),
        "u8" | "i8" => Ok(Some(8)),
        "u16" | "i16" => Ok(Some(16)),
        "u32" | "i32" => Ok(Some(32)),
        "u64" | "i64" => Ok(Some(64)),
        "u128" | "i128" => Ok(Some(128)),
        "ReservedZero" | "ReservedZeroes" | "ReservedOne" | "ReservedOnes" | "Integer" => {
            match p.arguments {
                ::syn::PathArguments::AngleBracketed(ref args) => {
                    for t in &args.args {
                        if let syn::GenericArgument::Type(ty) = t {
                            let ty_str = tokens_to_string(ty);
                            if let Some(bits_pos) = ty_str.find("Bits") {
                                let possible_int = &ty_str[(bits_pos + 4)..];
                                if let Ok(bits) = possible_int.parse::<usize>() {
                                    return Ok(Some(bits));
                                }
                            }
                        }
                    }

                    Ok(None)
                }
                _ => Ok(None),
            }
        }
        _ => Ok(None),
    }
}

fn get_field_mid_positioning(field: &syn::Field) -> syn::Result<FieldMidPositioning> {
    let mut array_size = 1;
    let bit_width_builtin: Option<usize>;

    let _ty = match &field.ty {
        syn::Type::Path(type_path) => {
            let segment = get_single_segment(type_path)?;

            bit_width_builtin = get_builtin_type_bit_width(segment)?;
            segment.clone()
        }
        syn::Type::Array(type_array) => {
            let path = match *type_array.elem {
                syn::Type::Path(ref p) => p,
                _ => {
                    return Err(syn::Error::new(
                        type_array.elem.span(),
                        "Unknown array path type",
                    ))
                }
            };

            let segment = get_single_segment(path)?;
            bit_width_builtin = get_builtin_type_bit_width(segment)?;
            let size = get_expr_int_val(&type_array.len)?;

            if size == 0 {
                return Err(syn::Error::new(
                    type_array.len.span(),
                    "Arrays sized 0 are not supported.",
                ));
            }
            array_size = size;

            segment.clone()
        }
        _ => {
            return Err(syn::Error::new(field.ty.span(), "Unsupported type"));
        }
    };

    let field_attributes = PackFieldAttribute::parse_all(&parse_sub_attributes(
        &field.attrs,
        "packed_field",
        "packed_struct",
    )?);

    let bits_position = field_attributes
        .iter()
        .filter_map(|a| match a {
            &PackFieldAttribute::BitPosition(b) | &PackFieldAttribute::BytePosition(b) => Some(b),
            _ => None,
        })
        .next()
        .unwrap_or(BitsPositionParsed::Next);

    let bit_width = if let Some(bits) = field_attributes
        .iter()
        .filter_map(|a| {
            if let &PackFieldAttribute::SizeBits(bits) = a {
                Some(bits)
            } else {
                None
            }
        })
        .next()
    {
        if array_size > 1 {
            return Err(syn::Error::new(
                field.span(),
                "Please use the 'element_size_bits' or 'element_size_bytes' for arrays.",
            ));
        }
        bits
    } else if let Some(bits) = field_attributes
        .iter()
        .filter_map(|a| {
            if let &PackFieldAttribute::ElementSizeBits(bits) = a {
                Some(bits)
            } else {
                None
            }
        })
        .next()
    {
        bits * array_size
    } else if let BitsPositionParsed::Range(a, b) = bits_position {
        (b as isize - a as isize).abs() as usize + 1
    } else if let Some(bit_width_builtin) = bit_width_builtin {
        // todo: is it even possible to hit this branch?
        bit_width_builtin * array_size
    } else {
        return Err(syn::Error::new(
            field.span(),
            "Couldn't determine the bit/byte width for this field.",
        ));
    };

    Ok(FieldMidPositioning {
        bit_width: bit_width,
        bits_position: bits_position,
    })
}

fn parse_field(
    field: &syn::Field,
    mp: &FieldMidPositioning,
    bit_range: &Range<usize>,
    default_endianness: Option<IntegerEndianness>,
) -> syn::Result<FieldKind> {
    match &field.ty {
        syn::Type::Path(_) => {
            return Ok(FieldKind::Regular {
                field: parse_reg_field(field, &field.ty, bit_range, default_endianness)?,
                ident: field
                    .ident
                    .clone()
                    .ok_or(syn::Error::new(field.span(), "Missing ident!"))?,
            });
        }
        syn::Type::Array(type_array) => {
            let size = get_expr_int_val(&type_array.len)?;

            let element_size_bits: usize = mp.bit_width as usize / size as usize;
            if (mp.bit_width % element_size_bits) != 0 {
                return Err(syn::Error::new(
                    type_array.span(),
                    "Element and array size mismatch!",
                ));
            }

            let mut elements = vec![];
            for i in 0..size as usize {
                let s = bit_range.start + (i * element_size_bits);
                let element_bit_range = s..(s + element_size_bits - 1);
                elements.push(parse_reg_field(
                    field,
                    &type_array.elem,
                    &element_bit_range,
                    default_endianness,
                )?);
            }
            return Ok(FieldKind::Array {
                ident: field
                    .ident
                    .clone()
                    .ok_or(syn::Error::new(field.span(), "Missing ident!"))?,
                size,
                elements,
            });
        }
        _ => (),
    };

    Err(syn::Error::new(field.span(), "Field not supported."))
}

fn parse_reg_field(
    field: &syn::Field,
    ty: &syn::Type,
    bit_range: &Range<usize>,
    default_endianness: Option<IntegerEndianness>,
) -> syn::Result<FieldRegular> {
    let mut wrappers = vec![];

    let bit_width = (bit_range.end - bit_range.start) + 1;
    let ty_str = tokens_to_string(ty);
    let field_attributes = PackFieldAttribute::parse_all(&parse_sub_attributes(
        &field.attrs,
        "packed_field",
        "packed_struct",
    )?);

    let is_enum_ty = field_attributes
        .iter()
        .filter_map(|a| match a {
            &PackFieldAttribute::Ty(TyKind::Enum) => Some(()),
            _ => None,
        })
        .next()
        .is_some();

    let needs_int_wrap = {
        let int_types = [
            "u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64", "u128", "i128",
        ];
        is_enum_ty || int_types.iter().any(|t| t == &ty_str)
    };

    let needs_endiannes_wrap = {
        let our_int_ty = ty_str.starts_with("Integer < ") && ty_str.contains("Bits");
        our_int_ty || needs_int_wrap
    };

    if is_enum_ty {
        wrappers.push(SerializationWrapper::PrimitiveEnumWrapper);
    }

    if needs_int_wrap {
        let ty = if is_enum_ty {
            format!("<{} as PrimitiveEnum>::Primitive", tokens_to_string(ty))
        } else {
            ty_str.clone()
        };
        let integer_wrap_ty = syn::parse_str(&format!("Integer<{}, Bits{}>", ty, bit_width))?;
        wrappers.push(SerializationWrapper::IntegerWrapper {
            integer: integer_wrap_ty,
        });
    }

    if needs_endiannes_wrap {
        let mut endiannes = if let Some(endiannes) = field_attributes
            .iter()
            .filter_map(|a| {
                if let &PackFieldAttribute::IntEndiannes(endiannes) = a {
                    Some(endiannes)
                } else {
                    None
                }
            })
            .next()
        {
            Some(endiannes)
        } else {
            default_endianness
        };

        if bit_width <= 8 {
            endiannes = Some(IntegerEndianness::Msb);
        }

        if endiannes.is_none() {
            panic!("Missing serialization wrapper for simple type {:?} - did you specify the integer endiannes on the field or a default for the struct?", ty_str);
        }

        let ty_prefix = match endiannes.unwrap() {
            IntegerEndianness::Msb => "Msb",
            IntegerEndianness::Lsb => "Lsb",
        };

        let endiannes_wrap_ty = syn::parse_str(&format!("{}Integer", ty_prefix)).unwrap();
        wrappers.push(SerializationWrapper::EndiannesWrapper {
            endian: endiannes_wrap_ty,
        });
    }

    Ok(FieldRegular {
        ty: ty.clone(),
        serialization_wrappers: wrappers,
        bit_width: bit_width,
        bit_range: bit_range.clone(),
        bit_range_rust: bit_range.start..(bit_range.end + 1),
    })
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BitsPositionParsed {
    Next,
    // total bit size of struct
    NextRev(usize),
    /// starting index of the field
    Start(usize),
    /// starting index of the field, total bit size of struct
    StartRev(usize, usize),
    /// starting index of the field, ending index of the field
    Range(usize, usize),
    /// starting index of the field, ending index of the field, total bit size of struct
    RangeRev(usize, usize, usize),
}

impl Default for BitsPositionParsed {
    fn default() -> Self {
        Self::Next
    }
}

impl BitsPositionParsed {
    /*fn to_bits_position(&self) -> Box<dyn BitsRange> {
        match *self {
            BitsPositionParsed::Next => Box::new(NextBits),
            BitsPositionParsed::NextRev => Box::new(NextRevBits),
            BitsPositionParsed::Start(s) => Box::new(s),
            BitsPositionParsed::Range(a, b) => Box::new(a..b),
        }
    }*/

    pub fn get_bits_range(
        &self,
        packed_bit_width: usize,
        prev_range: &Option<Range<usize>>,
    ) -> Range<usize> {
        match self {
            Self::Next => {
                if let &Some(ref prev_range) = prev_range {
                    (prev_range.end + 1)..((prev_range.end) + (packed_bit_width as usize))
                } else {
                    0..((packed_bit_width as usize) - 1)
                }
            }
            Self::NextRev(struct_size_bits) => {
                if let &Some(ref prev_range) = prev_range {
                    let prev_start = (struct_size_bits) - 1 - prev_range.end;
                    ((prev_start) - (packed_bit_width as usize))..(prev_start - 1)
                } else {
                    ((struct_size_bits) - (packed_bit_width as usize))..*struct_size_bits - 1
                }
            }
            Self::Start(start) => *start..(start + packed_bit_width as usize - 1),
            Self::StartRev(start, struct_size_bits) => {
                let mut end = start + packed_bit_width as usize - 1;
                end = (struct_size_bits) - 1 - end;
                let start = (struct_size_bits) - 1 - start;
                end..start
            }
            Self::Range(start, end) => *start..*end,
            Self::RangeRev(start, end, struct_size_bits) => {
                let start = (struct_size_bits) - 1 - start;
                let end = (struct_size_bits) - 1 - end;
                end..start
            }
        }
    }

    pub fn is_rev(&self) -> Option<usize> {
        match self {
            Self::NextRev(a) | Self::StartRev(_, a) | Self::RangeRev(_, _, a) => Some(*a),
            _ => None,
        }
    }

    pub fn rev(&mut self, struct_size_bytes: usize) {
        let mut val = match std::mem::take(self) {
            Self::Next => Self::NextRev(struct_size_bytes * 8),
            Self::NextRev(_) => Self::Next,
            Self::Start(start) => Self::StartRev(start, struct_size_bytes * 8),
            Self::StartRev(start, _) => Self::Start(start),
            Self::Range(start, end) => Self::RangeRev(start, end, struct_size_bytes * 8),
            Self::RangeRev(start, end, _) => Self::Range(start, end),
        };
        std::mem::swap(self, &mut val);
    }

    pub fn range_in_order(a: usize, b: usize) -> Self {
        BitsPositionParsed::Range(::std::cmp::min(a, b), ::std::cmp::max(a, b))
    }
}

pub fn parse_num(s: &str) -> usize {
    let s = s.trim();

    if s.starts_with("0x") || s.starts_with("0X") {
        usize::from_str_radix(&s[2..], 16).expect(&format!("Invalid hex number: {:?}", s))
    } else {
        s.parse()
            .expect(&format!("Invalid decimal number: {:?}", s))
    }
}

pub fn parse_struct(ast: &syn::DeriveInput) -> syn::Result<PackStruct> {
    let attributes = PackStructAttribute::parse_all(&parse_sub_attributes(
        &ast.attrs,
        "packed_struct",
        "packed_field",
    )?);

    let data_struct = match &ast.data {
        syn::Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new(
                ast.span(),
                "#[derive(PackedStruct)] can only be used with braced structs",
            ))
        }
    };
    let fields: Vec<_> = data_struct.fields.iter().collect();

    if ast.generics.params.len() > 0 {
        return Err(syn::Error::new(
            ast.span(),
            "Structures with generic fields currently aren't supported.",
        ));
    }

    let bit_positioning = {
        attributes
            .iter()
            .filter_map(|a| match a {
                &PackStructAttribute::BitNumbering(b) => Some(b),
                _ => None,
            })
            .next()
    };

    let default_int_endianness = attributes
        .iter()
        .filter_map(|a| match a {
            &PackStructAttribute::DefaultIntEndianness(i) => Some(i),
            _ => None,
        })
        .next();

    let struct_size_bytes = attributes
        .iter()
        .filter_map(|a| {
            if let &PackStructAttribute::SizeBytes(size_bytes) = a {
                Some(size_bytes)
            } else {
                None
            }
        })
        .next();

    let first_field_is_auto_positioned = {
        if let Some(ref field) = fields.first() {
            let mp = get_field_mid_positioning(field)?;
            mp.bits_position == BitsPositionParsed::Next
        } else {
            false
        }
    };

    let mut fields_parsed: Vec<FieldKind> = vec![];
    {
        let mut prev_bit_range = None;
        for field in &fields {
            let mp = get_field_mid_positioning(field)?;
            let bits_position = match (bit_positioning, mp.bits_position) {
                /*(Some(BitNumbering::Lsb0), BitsPositionParsed::Start(_)) => {
                    return Err(syn::Error::new(
                        field.span(),
                        "LSB0 field positioning currently requires explicit, full field positions.",
                    ));
                }
                (Some(BitNumbering::Lsb0), BitsPositionParsed::Next) => {
                    if let Some(struct_size_bytes) = struct_size_bytes {
                        BitsPositionParsed::Previous
                    } else {
                        return Err(syn::Error::new(
                            field.span(),
                            "LSB0 field positioning currently requires explicit struct byte size.",
                        ));
                    }
                }
                (Some(BitNumbering::Lsb0), BitsPositionParsed::Range(start, end)) => {
                    if let Some(struct_size_bytes) = struct_size_bytes {
                        BitsPositionParsed::range_in_order(
                            (struct_size_bytes * 8) - 1 - start,
                            (struct_size_bytes * 8) - 1 - end,
                        )
                    } else {
                        return Err(syn::Error::new(
                            field.span(),
                            "LSB0 field positioning currently requires explicit struct byte size.",
                        ));
                    }
                }*/
                (None, p @ BitsPositionParsed::Next) => p,
                (Some(BitNumbering::Msb0), p) => p,
                (Some(BitNumbering::Lsb0), mut p) => {
                    if let Some(struct_size_bytes) = struct_size_bytes {
                        p.rev(struct_size_bytes);
                        p
                    } else {
                        return Err(syn::Error::new(
                            field.span(),
                            "LSB0 field positioning currently requires explicit struct byte size.",
                        ));
                    }
                }

                (None, _) => {
                    return Err(syn::Error::new(field.span(), "Please explicitly specify the bit numbering mode on the struct with an attribute: #[packed_struct(bit_numbering=\"msb0\")] or \"lsb0\"."));
                }
            };
            let bit_range = bits_position.get_bits_range(mp.bit_width, &prev_bit_range);

            fields_parsed.push(parse_field(field, &mp, &bit_range, default_int_endianness)?);

            if let Some(byte_width) = bits_position.is_rev() {
                let mut temp = bits_position.clone();
                temp.rev(byte_width);
                prev_bit_range = Some(temp.get_bits_range(mp.bit_width, &prev_bit_range));
            } else {
                prev_bit_range = Some(bit_range);
            }
        }
    }

    let num_bits: usize = {
        if let Some(struct_size_bytes) = struct_size_bytes {
            struct_size_bytes * 8
        } else {
            let last_bit = fields_parsed
                .iter()
                .map(|f| match f {
                    &FieldKind::Regular { ref field, .. } => field.bit_range_rust.end,
                    &FieldKind::Array { ref elements, .. } => {
                        elements.last().unwrap().bit_range_rust.end
                    }
                })
                .max()
                .unwrap();
            last_bit
        }
    };

    let num_bytes = (num_bits as f32 / 8.0).ceil() as usize;

    if first_field_is_auto_positioned && (num_bits % 8) != 0 && struct_size_bytes == None {
        return Err(syn::Error::new(fields[0].span(), "Please explicitly position the bits of the first field of this structure, as the alignment isn't obvious to the end user."));
    }

    // check for overlaps
    {
        let mut bits = vec![None; num_bytes * 8];
        for field in &fields_parsed {
            let mut find_overlaps = |name: String, range: &Range<usize>| {
                for i in range.start..(range.end + 1) {
                    if let Some(&Some(ref n)) = bits.get(i) {
                        return Err(syn::Error::new(
                            name.span(),
                            format!(
                                "Overlap in bits between fields {} and {}",
                                n,
                                name.to_string()
                            ),
                        ));
                    }

                    bits[i] = Some(name.clone());
                }

                Ok(())
            };

            match field {
                &FieldKind::Regular {
                    ref field,
                    ref ident,
                } => {
                    find_overlaps(ident.to_string(), &field.bit_range)?;
                }
                &FieldKind::Array {
                    ref ident,
                    ref elements,
                    ..
                } => {
                    for (i, field) in elements.iter().enumerate() {
                        find_overlaps(format!("{}[{}]", ident.to_string(), i), &field.bit_range)?;
                    }
                }
            }
        }
    }
    Ok(PackStruct {
        derive_input: ast,
        data_struct,
        fields: fields_parsed,
        num_bytes,
        num_bits,
    })
}
