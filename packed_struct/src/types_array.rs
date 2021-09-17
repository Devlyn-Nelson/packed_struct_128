use super::packing::*;

macro_rules! packable_u8_array {
    ($N: expr) => {
        impl PackedStruct for [u8; $N] {
            type ByteArray = [u8; $N];

            #[inline]
            fn pack(&self) -> PackingResult<Self::ByteArray> {
                Ok(*self)
            }

            #[inline]
            fn unpack(src: &Self::ByteArray) -> Result<Self::ByteArray, PackingError> {
                Ok(*src)
            }
        }

        impl PackedStructInfo for [u8; $N] {
            #[inline]
            fn packed_bits() -> usize {
                $N * 8
            }
        }
    };
}

packable_u8_array!(0);
packable_u8_array!(1);
packable_u8_array!(2);
packable_u8_array!(3);
packable_u8_array!(4);
packable_u8_array!(5);
packable_u8_array!(6);
packable_u8_array!(7);
packable_u8_array!(8);
packable_u8_array!(9);
packable_u8_array!(10);
packable_u8_array!(11);
packable_u8_array!(12);
packable_u8_array!(13);
packable_u8_array!(14);
packable_u8_array!(15);
packable_u8_array!(16);
packable_u8_array!(17);
packable_u8_array!(18);
packable_u8_array!(19);
packable_u8_array!(20);
packable_u8_array!(21);
packable_u8_array!(22);
packable_u8_array!(23);
packable_u8_array!(24);
packable_u8_array!(25);
packable_u8_array!(26);
packable_u8_array!(27);
packable_u8_array!(28);
packable_u8_array!(29);
packable_u8_array!(30);
packable_u8_array!(31);
packable_u8_array!(32);
packable_u8_array!(33);
packable_u8_array!(34);
packable_u8_array!(35);
packable_u8_array!(36);
packable_u8_array!(37);
packable_u8_array!(38);
packable_u8_array!(39);
packable_u8_array!(40);
packable_u8_array!(41);
packable_u8_array!(42);
packable_u8_array!(43);
packable_u8_array!(44);
packable_u8_array!(45);
packable_u8_array!(46);
packable_u8_array!(47);
packable_u8_array!(48);
packable_u8_array!(49);
packable_u8_array!(50);
packable_u8_array!(51);
packable_u8_array!(52);
packable_u8_array!(53);
packable_u8_array!(54);
packable_u8_array!(55);
packable_u8_array!(56);
packable_u8_array!(57);
packable_u8_array!(58);
packable_u8_array!(59);
packable_u8_array!(60);
packable_u8_array!(61);
packable_u8_array!(62);
packable_u8_array!(63);
packable_u8_array!(64);
packable_u8_array!(128);
packable_u8_array!(256);
packable_u8_array!(512);
packable_u8_array!(1024);
packable_u8_array!(2048);
