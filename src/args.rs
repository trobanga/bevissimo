use clap::Parser;
use serde::Deserialize;
use std::ffi::OsString;

#[derive(Parser, Debug, Clone, Deserialize)]
#[serde(default)]
#[clap(
    name = "bevissimo",
    rename_all = "kebab-case",
    rename_all_env = "screaming-snake"
)]
pub struct Args {
    #[clap(short, long, default_value = "127.0.0.1")]
    pub server_address: String,

    #[clap(short, long, default_value = "3536")]
    pub port: u16,

    #[clap(short, long, value_parser)]
    pub username: String,

    #[clap(short = 'P', long, value_parser)]
    pub password: String,
}

impl Default for Args {
    fn default() -> Self {
        let args = Vec::<OsString>::new();
        Args::parse_from(args)
    }
}

impl Args {
    pub fn get() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let qs = web_sys::window()
                .unwrap()
                .location()
                .search()
                .unwrap()
                .trim_start_matches("?")
                .to_owned();

            Args::from_query(&qs)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Args::parse()
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn from_query(query: &str) -> Self {
        serde_qs::from_str(query).unwrap()
    }
}
