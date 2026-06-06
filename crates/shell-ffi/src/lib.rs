uniffi::include_scaffolding!("shell_ffi");

pub fn greet(name: String) -> String {
    format!("Hello from Rust, {name}!")
}

pub fn core_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_returns_formatted_string() {
        assert_eq!(greet("iOS".to_string()), "Hello from Rust, iOS!");
    }

    #[test]
    fn core_version_matches_crate_version() {
        assert_eq!(core_version(), env!("CARGO_PKG_VERSION"));
    }
}
