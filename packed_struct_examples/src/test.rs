use packed_struct::prelude::*;

#[derive(PackedStruct, PartialEq, Debug)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "2")]
pub struct LilBools {
    #[packed_field(bits = "0")]
    bool1: bool,
    #[packed_field(bits = "1:8")]
    num1: u8,
    #[packed_field(bits = "9:15")]
    num2: Integer<u8, packed_bits::Bits7>,
}

#[derive(PackedStruct, PartialEq, Debug)]
#[packed_struct(bit_numbering = "lsb0", size_bytes = "2", endian = "le")]
pub struct AutoBools {
    bool1: bool,

    #[packed_field(bits = "1..")]
    num1: u8,

    #[packed_field(bits = "9..")]
    num2: Integer<u8, packed_bits::Bits7>,
}

fn main() {
    let b = LilBools {
        bool1: true,
        num1: 1,
        num2: 0b00000000.into(),
    };
    let a = AutoBools {
        bool1: true,
        num1: 1,
        num2: 0b00000000.into(),
    };
    if let Ok(ref bytes) = b.pack() {
        for (index, byte) in (0..bytes.len()).zip(bytes) {
            println!("{:08b}", byte);
        }
    }
    println!("{:?}", b);

    if let Ok(ref bytes) = a.pack() {
        for (index, byte) in (0..bytes.len()).zip(bytes) {
            println!("{:08b}", byte);
        }
    }
    println!("{:?}", a);
}
