use clap::Command;

pub fn hello() -> String {
    "Hello, world!".to_string()
}

pub fn run() {
    let app = Command::new("zerv")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Dynamic Versioning CLI");

    let _matches = app.get_matches();
    println!("{}", hello());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello, world!");
    }
}
