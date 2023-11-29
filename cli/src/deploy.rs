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

fn generate_fixtures(wasm_code: Vec<u8>) -> Vec<Fixture> {
    use wasmtime::*;

    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_code).unwrap();

    let mut store = Store::new(&engine, ());

    let mem = Memory::new(&mut store, MemoryType::new(256, None)).unwrap();

    let alloc = Func::wrap(
        &mut store,
        move |mut caller: Caller<'_, ()>, pages: i32| -> i32 {
            mem.grow(&mut caller, pages as u64).unwrap();
            mem.size(&mut caller) as i32
        },
    );

    let free = Func::wrap(&mut store, |_page: i32| -> i32 { 0 });

    let gr_panic = Func::wrap(&mut store, |a: i32, b: i32| {
        println!("panic at {}:{}", a, b);
    });

    let imports = [mem.into(), alloc.into(), free.into(), gr_panic.into()];

    let instance = Instance::new(&mut store, &module, &imports).unwrap();

    let test_fn = instance
        .get_typed_func::<(), i64>(&mut store, "test")
        .unwrap();

    let ptr_len = test_fn.call(&mut store, ()).unwrap() as u64;
    let ptr = ((ptr_len & 0xffffffff00000000u64) >> 32) as usize;
    let len = (ptr_len & 0x00000000ffffffffu64) as usize;

    let extracted_vec = mem.data(&mut store)[ptr..ptr + len].to_vec();

    Vec::<Fixture>::decode(&mut &extracted_vec[..]).unwrap()
}

impl Deploy {
    /// Exec command deploy
    pub async fn exec(&self, signer: Signer) -> gcli::result::Result<()> {
        let code = fs::read(&self.code)?;

        println!("Parsing fixtures...");

        let fixtures = generate_fixtures(code);

        println!("Found fixtures: {}", fixtures.len());
        println!("Uploading service program...");

        let code = onchain_test_service::WASM_BINARY.to_vec();

        let gas = signer
            .rpc
            .calculate_upload_gas(None, code.clone(), vec![], 0, false, None)
            .await?
            .min_limit;

        // Estimate gas and upload program.
        let gas_limit = signer.api().cmp_gas_limit(gas)?;
        signer
            .calls
            .upload_program(code, vec![], vec![], gas_limit, 0)
            .await?;

        Ok(())
    }
}
