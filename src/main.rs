use serde::Deserialize;
use figment::{Figment, providers::{Format, Yaml}};

#[derive(Deserialize)]
struct Settings {
    port: usize
}

fn main() {
    let config: Settings = Figment::new()
        .merge(Yaml::file("example/node1/config.yml"))
        .extract()
        .unwrap();
    println!("{}", config.port);
}
