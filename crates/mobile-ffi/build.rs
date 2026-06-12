fn main() {
    uniffi::generate_scaffolding("src/quorum_mobile.udl").expect("generate UniFFI scaffolding");
}
