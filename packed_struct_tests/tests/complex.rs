use packed_struct::prelude::*;

#[test]
fn complex_serialization_endianess_big() {
    #[derive(PackedStruct, PartialEq, Debug)]
    #[packed_struct(endian = "msb")]
    pub struct TestStruct {
        val1: u16,
        val2: Integer<u8, packed_bits::Bits4>,
        val3: Integer<u16, packed_bits::Bits12>,
        val4: u16,
    }

    let b = TestStruct {
        val1: 123,
        val2: 3.into(),
        val3: 258.into(),
        val4: 321,
    };

    let packed = b.pack().unwrap();
    //assert_eq!(&packed, &[0b10000000, 0b10000000]);

    let unpacked = TestStruct::unpack(&packed).unwrap();
    assert_eq!(&unpacked, &b);
}

#[test]
fn complex_serialization_endianess_little() {
    #[derive(PackedStruct, PartialEq, Debug)]
    #[packed_struct(endian = "lsb")]
    pub struct TestStruct {
        val1: u16,
        val2: Integer<u8, packed_bits::Bits4>,
        val3: Integer<u16, packed_bits::Bits12>,
        val4: u16,
    }

    let b = TestStruct {
        val1: 123,
        val2: 3.into(),
        val3: 258.into(),
        val4: 321,
    };

    let packed = b.pack().unwrap();
    //assert_eq!(&packed, &[0b10000000, 0b10000000]);

    let unpacked = TestStruct::unpack(&packed).unwrap();
    assert_eq!(&unpacked, &b);
}
