#!/bin/bash

# The intention of this is to exit with a non-zero status code if any of
# the tests fail, but to allow all of them the opportunity to run, since
# they're all independent of one another.
exit=0

# The tests parse output from the CLI process and they are expecting it to be in a certain form, so
# any additional logging needs to be disabled.
unset RUST_LOG
# The default timeout value is 120 seconds, which causes NRS to run extremely slow.
export SN_QUERY_TIMEOUT=10
export RUST_BACKTRACE=full

cd sn_cli
cargo run --release -- keys create --for-cli || ((exit++))
cargo test --release --test cli_node || ((exit++))
cargo test --release --test cli_xorurl || ((exit++))
cargo test --release --test cli_cat -- --test-threads=1 || ((exit++))
cargo test --release --test cli_dog -- --test-threads=1 || ((exit++))
cargo test --release --test cli_files -- --test-threads=1 || ((exit++))
cargo test --release --test cli_files_get -- --test-threads=1 || ((exit++))
cargo test --release --test cli_keys || ((exit++))
cargo test --release --test cli_nrs -- --test-threads=1 || ((exit++))

exit $exit
