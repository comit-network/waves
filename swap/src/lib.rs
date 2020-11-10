#[cfg(test)]
mod tests {
    use ecdsa_fun::ECDSA;
    use elements::bitcoin::PublicKey;
    use elements::Address;
    use elements::AddressParams;
    use elements::Transaction;
    use elements_harness::Client;
    use elements_harness::Elementsd;
    use rand::rngs::OsRng;
    use testcontainers::clients::Cli;
    use wally::tx_get_elements_signature_hash;

    #[tokio::test]
    async fn sign_transaction() {
        let tc_client = Cli::default();
        let (client, _container) = {
            let blockchain = Elementsd::new(&tc_client, "0.18.1.9").unwrap();

            (Client::new(blockchain.node_url.clone()), blockchain)
        };

        let sk = ecdsa_fun::fun::Scalar::random(&mut OsRng);

        let ecdsa = ECDSA::<()>::default();
        let pk = ecdsa.verification_key_for(&sk);
        let pk = PublicKey::from_slice(&pk.to_bytes()).unwrap();

        let address = Address::p2wpkh(&pk, None, &AddressParams::ELEMENTS);

        client.send_to_address(address, 1.0).await.unwrap();

        let tx = Transaction {
            version: 2,
            lock_time: 0,
            input: Vec::new(),
            output: Vec::new(),
        };

        // tx_get_elements_signature_hash(tx, 0);
    }
}
