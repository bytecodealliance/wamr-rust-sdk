printf "Running 'postCreateCommand' Script\n"

# Install Rust Targets
printf "Installing Rust Targets\n"
#rustup update stable --no-self-update
rustup default stable
rustup target add wasm32-unknown-unknown
rustup target add wasm32-wasip1
rustup component add clippy
rustup component add rustfmt

cargo install cargo-readme
cargo install rustfilt

# Install Python stuff
printf "Installing Python Dependencies"

# Install NPM dependencies
printf "Installing NPM Dependencies"
