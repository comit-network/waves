use anyhow::Result;
use bitcoin_hashes::Hash;
use elements::bitcoin::PublicKey;
use elements::opcodes::all::*;
use elements::secp256k1::{Message, Signature, SECP256K1};
use elements::Script;

pub fn simulate(script: Script, witness_stack: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
    let mut stack = witness_stack.clone();
    let scritp_asm = script.asm();
    let script = scritp_asm.split(" ");

    let mut alt_stack = vec![];

    // remove script
    stack.pop();

    log::debug!("Before: {}", &format(&stack));
    for item in script {
        log::debug!("Current: {}", &format(&stack));

        match item {
            "OP_CAT" => {
                let mut item1 = stack.pop().unwrap();
                let mut item0 = stack.pop().unwrap();
                item0.append(&mut item1);
                stack.push(item0);
            }
            "OP_IF" => {
                stack.pop();
            }
            "OP_PUSHDATA1" | "OP_PUSHBYTES_33" | "OP_ELSE" | "OP_ENDIF" | "OP_CLTV" => {
                // ignore
            }
            "OP_SWAP" => {
                let first = stack.pop().unwrap();
                let second = stack.pop().unwrap();
                stack.push(first);
                stack.push(second);
            }
            "OP_HASH256" => {
                let un_hashed = stack.pop().unwrap();
                let hashed = bitcoin_hashes::sha256d::Hash::hash(&un_hashed).to_vec();
                stack.push(hashed);
            }
            "OP_SHA256" => {
                let un_hashed = stack.pop().unwrap();
                let hashed = bitcoin_hashes::sha256::Hash::hash(&un_hashed).to_vec();
                stack.push(hashed);
            }
            "OP_TOALTSTACK" => {
                let item = stack.pop().unwrap();
                alt_stack.push(item);
            }
            "OP_FROMALTSTACK" => {
                stack.push(alt_stack.pop().unwrap());
            }
            "OP_DEPTH" => {
                let depth = stack.len();
                stack.push(vec![depth as u8])
            }
            "OP_1SUB" => {
                // we assume that it's max 1 byte
                let item = stack.pop().unwrap()[0];
                stack.push(vec![item - 1]);
            }
            "OP_PICK" => {
                let index = stack.pop().unwrap()[0];
                let picked = stack[stack.clone().len() - index as usize - 1].clone();
                stack.push(picked);
            }
            "OP_PUSHNUM_1" => {
                stack.push(vec![1]);
            }
            "OP_CHECKSIGVERIFY" => {
                log::warn!("OP_CHECKSIGVERIFY is ignored");
                stack.pop();
                stack.pop();
            }
            "OP_CHECKSIGFROMSTACK" => {
                let pk = PublicKey::from_slice(&stack.pop().unwrap()).unwrap();
                let message = Message::from_slice(&stack.pop().unwrap()).unwrap();
                let signature = Signature::from_der(&stack.pop().unwrap()).unwrap();

                SECP256K1.verify(&message, &signature, &pk.key).unwrap();
                return Ok(stack);
            }
            everything_else => {
                let byte_array = hex::decode(everything_else).unwrap();
                stack.push(byte_array);
            }
        }
    }
    log::debug!("After: {}", &format(&stack));
    Ok(stack)
}

fn format(stack: &Vec<Vec<u8>>) -> String {
    let mut message = "".to_string();
    for i in stack {
        let item = hex::encode(i);
        message.push_str(&format!("[{}], ", item))
    }
    message
}

#[cfg(all(test))]
mod test {
    use super::*;
    use elements::bitcoin::util::psbt::serialize::Serialize;
    use elements::bitcoin::{Network, PrivateKey};
    use elements::script::Builder;
    use elements::secp256k1::rand::thread_rng;
    use elements::secp256k1::SecretKey;
    use env_logger;
    use std::env;

    const LAST_ITEM: u8 = 0xFF;
    const ITEM_0: u8 = 0x0a;
    const ITEM_1: u8 = 0x0b;
    const TRUE: u8 = 0x01;

    fn init() {
        env::set_var("RUST_LOG", "DEBUG");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn op_cat() {
        init();
        let script = Builder::new().push_opcode(OP_CAT).into_script();

        let witness = vec![
            vec![LAST_ITEM],
            vec![ITEM_0],
            vec![ITEM_1],
            script.to_bytes(),
        ];
        let result = simulate(script, witness).unwrap();
        assert_eq!(result, vec![vec![LAST_ITEM], vec![ITEM_0, ITEM_1]]);
    }

    #[test]
    fn op_swap() {
        init();
        let script = Builder::new().push_opcode(OP_SWAP).into_script();
        let witness = vec![
            vec![LAST_ITEM],
            vec![ITEM_0],
            vec![ITEM_1],
            script.to_bytes(),
        ];
        let result = simulate(script, witness).unwrap();
        assert_eq!(result, vec![vec![LAST_ITEM], vec![ITEM_1], vec![ITEM_0]]);
    }

    #[test]
    fn op_if() {
        init();
        let script = Builder::new().push_opcode(OP_IF).into_script();
        let witness = vec![
            vec![LAST_ITEM],
            vec![ITEM_0],
            vec![ITEM_1],
            vec![TRUE],
            script.to_bytes(),
        ];
        let result = simulate(script, witness).unwrap();
        assert_eq!(result, vec![vec![LAST_ITEM], vec![ITEM_0], vec![ITEM_1]]);
    }

    #[test]
    fn op_hash256() {
        init();
        let script = Builder::new().push_opcode(OP_HASH256).into_script();
        let witness = vec![
            vec![LAST_ITEM],
            vec![ITEM_0],
            vec![ITEM_1],
            script.to_bytes(),
        ];
        let un_hashed = vec![ITEM_1];
        let hashed = bitcoin_hashes::sha256d::Hash::hash(&un_hashed).to_vec();
        let result = simulate(script, witness).unwrap();
        assert_eq!(result, vec![vec![LAST_ITEM], vec![ITEM_0], hashed]);
    }

    #[test]
    fn op_sha256() {
        init();
        let script = Builder::new().push_opcode(OP_SHA256).into_script();
        let witness = vec![
            vec![LAST_ITEM],
            vec![ITEM_0],
            vec![ITEM_1],
            script.to_bytes(),
        ];
        let un_hashed = vec![ITEM_1];
        let hashed = bitcoin_hashes::sha256::Hash::hash(&un_hashed).to_vec();
        let result = simulate(script, witness).unwrap();
        assert_eq!(result, vec![vec![LAST_ITEM], vec![ITEM_0], hashed]);
    }

    #[test]
    fn op_to_and_from_alt_stack() {
        init();
        let script = Builder::new()
            .push_opcode(OP_TOALTSTACK)
            .push_opcode(OP_FROMALTSTACK)
            .into_script();
        let witness = vec![
            vec![LAST_ITEM],
            vec![ITEM_0],
            vec![ITEM_1],
            script.to_bytes(),
        ];
        let result = simulate(script, witness).unwrap();
        assert_eq!(result, vec![vec![LAST_ITEM], vec![ITEM_0], vec![ITEM_1]]);
    }

    #[test]
    fn op_depth() {
        init();
        let script = Builder::new().push_opcode(OP_DEPTH).into_script();
        let witness = vec![
            vec![LAST_ITEM],
            vec![ITEM_0],
            vec![ITEM_1],
            script.to_bytes(),
        ];
        let result = simulate(script, witness).unwrap();
        assert_eq!(
            result,
            vec![vec![LAST_ITEM], vec![ITEM_0], vec![ITEM_1], vec![3]]
        );
    }

    #[test]
    fn op_1sub() {
        init();
        let script = Builder::new().push_opcode(OP_1SUB).into_script();
        let witness = vec![
            vec![LAST_ITEM],
            vec![ITEM_0],
            vec![ITEM_1],
            script.to_bytes(),
        ];
        let result = simulate(script, witness).unwrap();
        assert_eq!(
            result,
            vec![vec![LAST_ITEM], vec![ITEM_0], vec![ITEM_1 - 1]]
        );
    }

    #[test]
    fn op_pick() {
        init();
        let script = Builder::new().push_opcode(OP_PICK).into_script();
        let witness = vec![
            vec![LAST_ITEM],
            vec![ITEM_0],
            vec![ITEM_1],
            vec![2],
            script.to_bytes(),
        ];
        let result = simulate(script, witness).unwrap();
        assert_eq!(
            result,
            vec![vec![LAST_ITEM], vec![ITEM_0], vec![ITEM_1], vec![LAST_ITEM]]
        );
    }

    #[test]
    fn op_pushnum_1() {
        init();
        let script = Builder::new().push_opcode(OP_PUSHNUM_1).into_script();
        let witness = vec![
            vec![LAST_ITEM],
            vec![ITEM_0],
            vec![ITEM_1],
            script.to_bytes(),
        ];
        let result = simulate(script, witness).unwrap();
        assert_eq!(
            result,
            vec![vec![LAST_ITEM], vec![ITEM_0], vec![ITEM_1], vec![1]]
        );
    }

    #[test]
    fn op_checksigfromstack() {
        init();
        let msg = Message::from_slice(&b"Yoda: btc, I trust. HODL I must!"[..]).expect("32 bytes");
        let sk = SecretKey::new(&mut thread_rng());
        let pk = PublicKey::from_private_key(
            &SECP256K1,
            &PrivateKey {
                compressed: true,
                network: Network::Regtest,
                key: sk,
            },
        );
        let signature = SECP256K1.sign(&msg, &sk);

        let script = Builder::new()
            .push_opcode(OP_CHECKSIGFROMSTACK)
            .into_script();
        let witness = vec![
            vec![LAST_ITEM],
            signature.serialize_der().to_vec(),
            msg.as_ref().to_vec(),
            pk.serialize().to_vec(),
            script.to_bytes(),
        ];
        let result = simulate(script, witness).unwrap();
        assert_eq!(result, vec![vec![LAST_ITEM]]);
    }
}
