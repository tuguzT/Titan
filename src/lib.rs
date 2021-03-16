mod graphics;

pub fn run(_config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let _renderer = graphics::Renderer::new();
    return Ok(());
}

pub struct Config {
}
