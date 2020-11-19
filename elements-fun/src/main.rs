use elements_fun::wally::{confidential_addr_from_addr, bip39_mnemonic_to_seed, asset_blinding_key_from_seed, asset_blinding_key_to_ec_private_key, ec_public_key_from_private_key};
use elements_fun::bitcoin::Script;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn make_address() -> String {
    let mnemonic = "all all all all all all all all all all all all";
    let passphrase = "";
    let seed = bip39_mnemonic_to_seed(mnemonic, passphrase);
    assert!(seed.is_some());
    let seed = seed.unwrap();
    assert_eq!(seed.len(), 64);
    assert_eq!(hex::encode(&seed[..]), "c76c4ac4f4e4a00d6b274d5c39c700bb4a7ddc04fbc6f78e85ca75007b5b495f74a9043eeb77bdd53aa6fc3a0e31462270316fa04b8c19114c8798706cd02ac8");
    let master_blinding_key = asset_blinding_key_from_seed(&seed);
    assert_eq!(
        hex::encode(&master_blinding_key.0[32..]),
        "6c2de18eabeff3f7822bc724ad482bef0557f3e1c1e1c75b7a393a5ced4de616"
    );

    let unconfidential_addr = "2dpWh6jbhAowNsQ5agtFzi7j6nKscj6UnEr";
    let script: Script = hex::decode("76a914a579388225827d9f2fe9014add644487808c695d88ac")
        .unwrap()
        .into();
    let blinding_key = asset_blinding_key_to_ec_private_key(&master_blinding_key, &script);
    let public_key = ec_public_key_from_private_key(blinding_key);
    let conf_addr =
        confidential_addr_from_addr(unconfidential_addr, 0x04, public_key);

    // "CTEkf75DFff5ReB7juTg2oehrj41aMj21kvvJaQdWsEAQohz1EDhu7Ayh6goxpz3GZRVKidTtaXaXYEJ"
    conf_addr
}

fn main() {}
