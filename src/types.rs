// Copyright 2019 Stichting Organism
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//Taken from Parity libs & changed as needed

construct_fixed_hash! { 
    /// Fixed-size uninterpreted hash type with 16 bytes (128 bits) size.
    pub struct H128(16); 
}

construct_fixed_hash! {
	/// Fixed-size uninterpreted hash type with 20 bytes (160 bits) size.
	pub struct H160(20);
}

construct_fixed_hash! {
    /// Fixed-size uninterpreted hash type with 32 bytes (256 bits) size.
    pub struct H256(32);
}

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

construct_fixed_hash! {
    /// Fixed-size uninterpreted hash type with 48 bytes (384 bits) size.
    pub struct H384(48);
}


/// Add Serde serialization support to an integer created by `construct_uint!`.
#[macro_export]
macro_rules! impl_uint_serde {
    ($name: ident, $len: expr) => {
        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut bytes = [0u8; $len * 8];
                self.to_big_endian(&mut bytes);
                $crate::mserde::serialize_uint(&bytes, serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                $crate::mserde::deserialize_check_len(
                    deserializer,
                    $crate::mserde::ExpectedLen::Between(0, $len * 8),
                )
                .map(|x| (&*x).into())
            }
        }
    };
}

/// Add Serde serialization support to a fixed-sized hash type created by `construct_fixed_hash!`.
#[macro_export]
macro_rules! impl_fixed_hash_serde {
    ($name: ident, $len: expr) => {
        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                $crate::mserde::serialize(&self.0, serializer)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                $crate::mserde::deserialize_check_len(
                    deserializer,
                    $crate::mserde::ExpectedLen::Exact($len),
                )
                .map(|x| $name::from_slice(&x))
            }
        }
    };
}

/// Add Mohan serialization support to a fixed-sized hash type created by `construct_fixed_hash!`.
#[macro_export]
macro_rules! impl_fixed_hash_mohan_ser {
    ($name: ident, $len: expr) => {

        impl crate::ser::Readable for $name {
            fn read(reader: &mut dyn crate::ser::Reader) -> Result<Self, crate::ser::Error> {
                let v = reader.read_fixed_bytes($len)?;
                let mut a = [0; $len];
                a.copy_from_slice(&v[..]);
                Ok($name(a))
            }
        }

        impl crate::ser::Writeable for $name {
            fn write<W: crate::ser::Writer>(&self, writer: &mut W) -> Result<(), crate::ser::Error> {
                writer.write_fixed_bytes(&self.0)
            }
        }
    };
}



/// Add Mohan serialization support to a fixed-sized Uint type created by `construct_uint!`.
#[macro_export]
macro_rules! impl_uint_mohan_ser {
    ($name: ident, $len: expr) => {

        impl crate::ser::Readable for $name {
            fn read(reader: &mut dyn crate::ser::Reader) -> Result<Self, crate::ser::Error> {
                let v = reader.read_fixed_bytes($len * 8)?;
                Ok($name::from_big_endian(&v))
            }
        }

        impl crate::ser::Writeable for $name {
            fn write<W: crate::ser::Writer>(&self, writer: &mut W) -> Result<(), crate::ser::Error> {
                let mut bytes = [0u8; $len * 8];
                self.to_big_endian(&mut bytes);

                writer.write_fixed_bytes(&bytes)
            }
        }
    };
}



macro_rules! impl_uint_conversions {
    ($hash: ident, $uint: ident) => {
        impl From<$uint> for $hash {
            fn from(value: $uint) -> Self {
                let mut ret = $hash::zero();
                value.to_big_endian(ret.as_bytes_mut());
                ret
            }
        }

        impl<'a> From<&'a $uint> for $hash {
            fn from(value: &'a $uint) -> Self {
                let mut ret = $hash::zero();
                value.to_big_endian(ret.as_bytes_mut());
                ret
            }
        }

        impl From<$hash> for $uint {
            fn from(value: $hash) -> Self {
                Self::from(&value)
            }
        }

        impl<'a> From<&'a $hash> for $uint {
            fn from(value: &'a $hash) -> Self {
                Self::from(value.as_ref() as &[u8])
            }
        }
    };
}


//
// Serde Encoding
//
impl_fixed_hash_serde!(H128, 16);
impl_fixed_hash_serde!(H160, 20);
impl_fixed_hash_serde!(H256, 32);
impl_fixed_hash_serde!(H384, 48);
impl_uint_serde!(U256, 4);

//
// Raw Byte Big Endian Encoding
//
impl_fixed_hash_mohan_ser!(H128, 16);
impl_fixed_hash_mohan_ser!(H160, 20);
impl_fixed_hash_mohan_ser!(H256, 32);
impl_uint_mohan_ser!(U256, 4);

impl_uint_conversions!(H256, U256);


impl From<u64> for H256 {
    fn from(val: u64) -> Self {
        H256::from_low_u64_be(val)
    }
}

#[cfg(test)]
mod tests {
    use super::{H256, H160};
    use serde_json as ser;

    impl From<u64> for H160 {
        fn from(val: u64) -> Self {
            H160::from_low_u64_be(val)
        }
    }

    #[test]
    fn test_serialize_h160() {
        let tests = vec![
            (H160::from(0), "0x0000000000000000000000000000000000000000"),
            (H160::from(2), "0x0000000000000000000000000000000000000002"),
            (H160::from(15), "0x000000000000000000000000000000000000000f"),
            (H160::from(16), "0x0000000000000000000000000000000000000010"),
            (
                H160::from(1_000),
                "0x00000000000000000000000000000000000003e8",
            ),
            (
                H160::from(100_000),
                "0x00000000000000000000000000000000000186a0",
            ),
            (
                H160::from(u64::max_value()),
                "0x000000000000000000000000ffffffffffffffff",
            ),
        ];

        for (number, expected) in tests {
            assert_eq!(
                format!("{:?}", expected),
                ser::to_string_pretty(&number).unwrap()
            );
            assert_eq!(number, ser::from_str(&format!("{:?}", expected)).unwrap());
        }
    }

    #[test]
    fn test_serialize_h256() {
        let tests = vec![
            (
                H256::from(0),
                "0x0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                H256::from(2),
                "0x0000000000000000000000000000000000000000000000000000000000000002",
            ),
            (
                H256::from(15),
                "0x000000000000000000000000000000000000000000000000000000000000000f",
            ),
            (
                H256::from(16),
                "0x0000000000000000000000000000000000000000000000000000000000000010",
            ),
            (
                H256::from(1_000),
                "0x00000000000000000000000000000000000000000000000000000000000003e8",
            ),
            (
                H256::from(100_000),
                "0x00000000000000000000000000000000000000000000000000000000000186a0",
            ),
            (
                H256::from(u64::max_value()),
                "0x000000000000000000000000000000000000000000000000ffffffffffffffff",
            ),
        ];

        for (number, expected) in tests {
            assert_eq!(
                format!("{:?}", expected),
                ser::to_string_pretty(&number).unwrap()
            );
            assert_eq!(number, ser::from_str(&format!("{:?}", expected)).unwrap());
        }
    }

    #[test]
    fn test_serialize_invalid() {
        assert!(ser::from_str::<H256>(
            "\"0x000000000000000000000000000000000000000000000000000000000000000\""
        )
        .unwrap_err()
        .is_data());
        assert!(ser::from_str::<H256>(
            "\"0x000000000000000000000000000000000000000000000000000000000000000g\""
        )
        .unwrap_err()
        .is_data());
        assert!(ser::from_str::<H256>(
            "\"0x00000000000000000000000000000000000000000000000000000000000000000\""
        )
        .unwrap_err()
        .is_data());
        assert!(ser::from_str::<H256>("\"\"").unwrap_err().is_data());
        assert!(ser::from_str::<H256>("\"0\"").unwrap_err().is_data());
        assert!(ser::from_str::<H256>("\"10\"").unwrap_err().is_data());
    }
}
