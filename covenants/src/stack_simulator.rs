use anyhow::{anyhow, Result};
use bitcoin_hashes::Hash;
use elements::{
    bitcoin::PublicKey,
    secp256k1::{Message, Signature, SECP256K1},
    Script,
};

pub fn simulate(script: Script, witness_stack: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
    let mut stack = witness_stack;
    let scritp_asm = script.asm();
    let script = scritp_asm.split(' ');

    let mut alt_stack = vec![];

    // remove script
    stack.pop();

    log::debug!("Before: {}", &format(&stack));
    for item in script {
        log::debug!("Current: {}", &format(&stack));
        log::debug!("opcode: {}", item);

        match item {
            "OP_CAT" => {
                let mut item1 = pop(&mut stack)?;
                let mut item0 = pop(&mut stack)?;
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
                let first = pop(&mut stack)?;
                let second = pop(&mut stack)?;
                stack.push(first);
                stack.push(second);
            }
            "OP_HASH256" => {
                let un_hashed = pop(&mut stack)?;
                let hashed = bitcoin_hashes::sha256d::Hash::hash(&un_hashed).to_vec();
                stack.push(hashed);
            }
            "OP_SHA256" => {
                let un_hashed = pop(&mut stack)?;
                let hashed = bitcoin_hashes::sha256::Hash::hash(&un_hashed).to_vec();
                stack.push(hashed);
            }
            "OP_TOALTSTACK" => {
                let item = pop(&mut stack)?;
                alt_stack.push(item);
            }
            "OP_FROMALTSTACK" => {
                let item = pop(&mut alt_stack)?;
                stack.push(item);
            }
            "OP_DEPTH" => {
                let depth = stack.len();
                stack.push(vec![depth as u8])
            }
            "OP_1SUB" => {
                // we assume that it's max 1 byte
                let item = pop(&mut stack)?[0];
                stack.push(vec![item - 1]);
            }
            "OP_PICK" => {
                let index = pop(&mut stack)?[0];
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
                let pk = PublicKey::from_slice(&pop(&mut stack)?)?;
                let message_unhashed = &pop(&mut stack)?;
                let hashed = bitcoin_hashes::sha256::Hash::hash(&message_unhashed).to_vec();
                let message = Message::from_slice(&hashed)?;
                let signature = Signature::from_der(&pop(&mut stack)?)?;

                SECP256K1.verify(&message, &signature, &pk.key)?;
                return Ok(stack);
            }
            everything_else => {
                let byte_array = hex::decode(everything_else)?;
                stack.push(byte_array);
            }
        }
    }
    log::debug!("After: {}", &format(&stack));
    Ok(stack)
}

fn pop(stack: &mut Vec<Vec<u8>>) -> Result<Vec<u8>> {
    let item = stack.pop().ok_or_else(|| anyhow!("Could not pop item."))?;
    Ok(item)
}

fn format(stack: &[Vec<u8>]) -> String {
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
    use elements::{
        bitcoin::{util::psbt::serialize::Serialize, Network, PrivateKey},
        opcodes::all::*,
        script::Builder,
        secp256k1::{rand::thread_rng, SecretKey},
    };
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

        let msg_unhashed = b"Yoda: btc, I trust. HODL I must!".to_vec();
        let sk = SecretKey::new(&mut thread_rng());
        let pk = PublicKey::from_private_key(
            &SECP256K1,
            &PrivateKey {
                compressed: true,
                network: Network::Regtest,
                key: sk,
            },
        );

        //OP_CHECKSIGFROMSTACK will hash the message before verifying against the signature.
        // hence we need to hash ig first before we signing it.
        let msg_hashed = bitcoin_hashes::sha256::Hash::hash(&msg_unhashed).to_vec();
        let msg_hashed = Message::from_slice(&msg_hashed).unwrap();
        let signature = SECP256K1.sign(&msg_hashed, &sk);

        let script = Builder::new()
            .push_opcode(OP_CHECKSIGFROMSTACK)
            .into_script();
        let witness = vec![
            vec![LAST_ITEM],
            signature.serialize_der().to_vec(),
            msg_unhashed,
            pk.serialize().to_vec(),
            script.to_bytes(),
        ];
        let result = simulate(script, witness).unwrap();
        assert_eq!(result, vec![vec![LAST_ITEM]]);
    }
}
