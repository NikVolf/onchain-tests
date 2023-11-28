use std::{fs, path::PathBuf};
use clap::Parser;
use gsdk::signer::Signer;
use onchain_test_types::Fixture;

#[derive(Debug, Parser)]
struct Deploy {
    code: PathBuf,

    #[arg(short, long, default_value = "0x")]
    service: String,

    #[arg(short, long, default_value = "0x")]
    salt: String,
}

fn generate_fixtures(wasm_code: Vec<u8>) -> Vec<Fixture> {
    unimplemented!()
}

impl Deploy {
    /// Exec command deploy
    pub async fn exec(&self, signer: Signer) -> gcli::result::Result<()> {
        let code = fs::read(&self.code)?;

        let fixtures = generate_fixtures(code);

        Ok(())
    }
}
