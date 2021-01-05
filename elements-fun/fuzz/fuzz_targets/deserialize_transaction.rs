extern crate elements_fun;

#[cfg(any(feature = "afl", feature = "honggfuzz", test))]
fn do_test(data: &[u8]) {
    match elements_fun::encode::deserialize::<elements_fun::Transaction>(data) {
        Err(_) => {}
        Ok(tx) => {
            for output in &tx.output {
                output.is_null_data();
                output.is_pegout();
                output.pegout_data();
                output.is_fee();
                output.minimum_value();
            }
        }
    }
}

#[cfg(feature = "afl")]
extern crate afl;

#[cfg(feature = "honggfuzz")]
#[macro_use]
extern crate honggfuzz;

fn main() {
    #[cfg(feature = "afl")]
    afl::read_stdio_bytes(|data| {
        do_test(&data);
    });

    #[cfg(feature = "honggfuzz")]
    loop {
        fuzz!(|data| {
            do_test(data);
        });
    }
}

#[cfg(test)]
mod tests {
    fn extend_vec_from_hex(hex: &str, out: &mut Vec<u8>) {
        let mut b = 0;
        for (idx, c) in hex.as_bytes().iter().enumerate() {
            b <<= 4;
            match *c {
                b'A'..=b'F' => b |= c - b'A' + 10,
                b'a'..=b'f' => b |= c - b'a' + 10,
                b'0'..=b'9' => b |= c - b'0',
                _ => panic!("Bad hex"),
            }
            if (idx & 1) == 1 {
                out.push(b);
                b = 0;
            }
        }
    }

    #[test]
    fn duplicate_crash() {
        let mut a = Vec::new();
        extend_vec_from_hex("00", &mut a);
        super::do_test(&a);
    }
}
