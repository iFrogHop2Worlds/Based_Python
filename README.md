# Based_Python
A transpiler that makes python better. Written in Rust. Inspired by bython (bracket python).


# Just transpile to Python and print to stdout
cargo run -- -i example.bython

# Transpile and save to file
cargo run -- -i example.bython -o output.py

# Transpile, save to file, and run immediately
cargo run -- -i example.bython -o output.py --run

# Transpile and run directly (without saving to file)
cargo run -- -i example.bython --run

# Use a specific Python interpreter
cargo run -- -i example.bython --run --python-interpreter python3

A work in progress. Feel free to contribute or raise issues :)