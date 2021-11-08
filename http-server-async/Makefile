heap := 0x10000000
stack := 0x50000
threads := 32
# override `mode` with command line args, e.g. `make run mode=release`
mode := debug
release_flags = $(subst release,--release,$(subst debug,,$(mode)))

all:
	export CFLAGS_x86_64_fortanix_unknown_sgx="-isystem/usr/include/x86_64-linux-gnu"
	cd enclave && cargo build --target x86_64-fortanix-unknown-sgx $(release_flags)
	cd runner && cargo build $(release_flags)

run: all
	ftxsgx-elf2sgxs --heap-size $(heap) --stack-size $(stack) --threads $(threads) target/x86_64-fortanix-unknown-sgx/$(mode)/http-server-async
	./target/$(mode)/runner target/x86_64-fortanix-unknown-sgx/$(mode)/http-server-async.sgxs

non-sgx:
	cd enclave && cargo run $(release_flags)

clean:
	cd enclave && cargo clean
	cd runner && cargo clean
