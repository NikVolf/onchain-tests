use clap::Parser;
use codec::Decode;
use gsdk::signer::Signer;
use onchain_test_types::Fixture;
use std::{fs, path::PathBuf};

#[derive(Debug, Parser)]
pub struct Deploy {
    code: PathBuf,

    #[arg(short, long, default_value = "0x")]
    service: String,

    #[arg(short, long, default_value = "0x")]
    salt: String,
}

/// Extract name providing necessary imports for wasm instantiation.
fn extract_vec(wasm_code: Vec<u8>, fn_name: &str) -> gcli::result::Result<Vec<u8>> {
    use wasmtime::*;

    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_code).unwrap();

    let mut store = Store::new(&engine, ());

    let mut linker = Linker::new(&engine);

    let mem = Memory::new(&mut store, MemoryType::new(256, None)).unwrap();

    linker.func_wrap(
        "env",
        "alloc",
        move |mut caller: Caller<'_, ()>, pages: i32| -> i32 {
            let prev_size = mem.size(&mut caller) as i32;
            mem.grow(&mut caller, pages as u64).unwrap();
            prev_size
        },
    )?;

    linker.func_wrap("env", "free", |_page: i32| -> i32 { 0 })?;

    linker.func_wrap("env", "gr_panic", |a: i32, b: i32| {
        println!("panic at {}:{}", a, b);
    })?;

    linker.define(&mut store, "env", "memory", mem)?;

    let instance = linker.instantiate(&mut store, &module)?;

    let call_func = instance.get_typed_func::<(), i64>(&mut store, fn_name)?;

    let ptr_len = call_func.call(&mut store, ()).unwrap() as u64;
    let ptr = ((ptr_len & 0xffffffff00000000u64) >> 32) as usize;
    let len = (ptr_len & 0x00000000ffffffffu64) as usize;

    let extracted_vec = mem.data(&mut store)[ptr..ptr + len].to_vec();

    Ok(extracted_vec)
}

fn generate_fixtures(wasm_code: Vec<u8>) -> gcli::result::Result<Vec<Fixture>> {
    let extracted_vec = extract_vec(wasm_code, "test")?;
    Ok(Vec::<Fixture>::decode(&mut &extracted_vec[..])?)
}

impl Deploy {
    /// Exec command deploy
    pub async fn exec(&self, signer: Signer) -> gcli::result::Result<()> {
        let code = fs::read(&self.code)?;

        println!("Parsing fixtures...");

        let fixtures = generate_fixtures(code)?;

        println!("Found fixtures: {}", fixtures.len());
        println!("Uploading service program...");

        let code = onchain_test_service::WASM_BINARY.to_vec();

        let gas = signer
            .rpc
            .calculate_upload_gas(None, code.clone(), vec![], 0, false, None)
            .await?
            .min_limit;

        let gas_limit = signer.api().cmp_gas_limit(gas)?;

        signer
            .calls
            .upload_program(code, vec![], vec![], gas_limit, 0)
            .await?;

        Ok(())
    }
}
