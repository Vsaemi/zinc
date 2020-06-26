//!
//! The `public key` command arguments.
//!

use std::io::Read;

use serde_json::json;
use structopt::StructOpt;

use franklin_crypto::alt_babyjubjub::AltJubjubBn256;
use franklin_crypto::bellman::pairing::bn256::Bn256;
use franklin_crypto::eddsa;

use crate::arguments::Error;

///
/// The `public key` command arguments.
///
#[derive(StructOpt)]
#[structopt(
    name = "pub-key",
    about = "recover the public key from the private key"
)]
pub struct PubKeyCommand {}

impl PubKeyCommand {
    pub fn execute(&self) -> Result<(), Error> {
        let params = AltJubjubBn256::new();

        let mut private_key_hex = vec![0; 64];
        std::io::stdin().read_exact(&mut private_key_hex)?;
        let private_key_hex = String::from_utf8_lossy(&private_key_hex);

        let bytes = hex::decode(private_key_hex.trim())?;
        let private_key = eddsa::PrivateKey::<Bn256>::read(bytes.as_slice())?;

        let public_key = schnorr::recover_public_key(&params, &private_key);
        let (x, y) = {
            let (x, y) = public_key.0.into_xy();
            (schnorr::fr_into_hex(x), schnorr::fr_into_hex(y))
        };

        let public_key_json = json!({ "x": x, "y": y });
        let public_key_text = serde_json::to_string_pretty(&public_key_json).expect("json");
        println!("{}", public_key_text);

        Ok(())
    }
}
