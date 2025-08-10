pub fn hello() -> String {
    "Hello, world!".to_string()
}

pub fn run() {
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
