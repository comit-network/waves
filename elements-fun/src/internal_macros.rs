// Rust Bitcoin Library
// Written in 2014 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

macro_rules! impl_consensus_encoding {
    ($thing:ident, $($field:ident),+) => (
        impl $crate::encode::Encodable for $thing {
            #[inline]
            fn consensus_encode<S: std::io::Write>(&self, mut s: S) -> Result<usize, $crate::encode::Error> {
                let mut ret = 0;
                $( ret += self.$field.consensus_encode(&mut s)?; )+
                Ok(ret)
            }
        }

        impl $crate::encode::Decodable for $thing {
            #[inline]
            fn consensus_decode<D: std::io::BufRead>(mut d: D) -> Result<$thing, $crate::encode::Error> {
                Ok($thing {
                    $( $field: $crate::encode::Decodable::consensus_decode(&mut d)?, )+
                })
            }
        }
    );
}

#[cfg(test)]
macro_rules! hex_deserialize(
    ($e:expr) => ({
        use $crate::encode::deserialize;

        fn hex_char(c: char) -> u8 {
            match c {
                '0' => 0,
                '1' => 1,
                '2' => 2,
                '3' => 3,
                '4' => 4,
                '5' => 5,
                '6' => 6,
                '7' => 7,
                '8' => 8,
                '9' => 9,
                'a' | 'A' => 10,
                'b' | 'B' => 11,
                'c' | 'C' => 12,
                'd' | 'D' => 13,
                'e' | 'E' => 14,
                'f' | 'F' => 15,
                x => panic!("Invalid character {} in hex string", x),
            }
        }

        let mut ret = Vec::with_capacity($e.len() / 2);
        let mut byte = 0;
        for (ch, store) in $e.chars().zip([false, true].iter().cycle()) {
            byte = (byte << 4) + hex_char(ch);
            if *store {
                ret.push(byte);
                byte = 0;
            }
        }
        deserialize(&ret).expect("deserialize object")
    });
);

#[cfg(test)]
macro_rules! hex_script(
    ($e:expr) => ({
        let v: Vec<u8> = ::bitcoin::hashes::hex::FromHex::from_hex($e)
            .expect("hex decoding");
        crate::Script::from(v)
    })
);
