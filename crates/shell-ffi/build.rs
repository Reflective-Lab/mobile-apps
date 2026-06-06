fn main() {
    uniffi::generate_scaffolding("src/shell_ffi.udl").expect("generate UniFFI scaffolding");
}
