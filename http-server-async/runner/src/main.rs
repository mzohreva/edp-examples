use aesm_client::AesmClient;
use clap::{App, Arg};
use enclave_runner::EnclaveBuilder;

#[cfg(windows)]
use sgxs_loaders::enclaveapi::Sgx as IsgxDevice;
#[cfg(unix)]
use sgxs_loaders::isgx::Device as IsgxDevice;

fn main() {
    let args = App::new("runner")
        .arg(Arg::with_name("file").required(true))
        .arg(
            Arg::with_name("enclave-args")
                .long_help(
                    "Arguments passed to the enclave. \
                    Note that this is not an appropriate channel for passing \
                    secrets or security configurations to the enclave.",
                )
                .multiple(true),
        )
        .get_matches();

    let file = args.value_of("file").unwrap();

    let mut device = IsgxDevice::new()
        .expect("failed to open SGX device")
        .einittoken_provider(AesmClient::new())
        .build();

    let mut enclave_builder = EnclaveBuilder::new(file.as_ref());
    if let Err(_) = enclave_builder.coresident_signature() {
        enclave_builder.dummy_signature();
    }

    if let Some(enclave_args) = args.values_of("enclave-args") {
        enclave_builder.args(enclave_args);
    }

    let enclave = enclave_builder
        .build(&mut device)
        .expect("failed to load SGX enclave");

    match enclave.run() {
        Err(e) => {
            eprintln!("Error while executing SGX enclave.\n{}", e);
            std::process::exit(-1)
        }
        Ok(()) => {}
    }
}
