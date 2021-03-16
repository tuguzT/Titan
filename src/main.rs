fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = titan::Config {
        // ...
    };
    return titan::run(config);
}
