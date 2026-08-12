use pacord::capture::CaptureConfig;
use pacord::module2::{run_wayland_host, run_x11_host};
use std::env;
use std::net::SocketAddr;

fn usage() {
    eprintln!(
        "Uso: PACORD_SECRET='<segredo>' cargo run --bin pacord-host -- <wayland|x11> <zerotier-ip:porta>\n\nExemplo:\n  PACORD_SECRET='troque-por-um-segredo-longo' cargo run --bin pacord-host -- x11 10.147.20.5:7777"
    );
}

#[tokio::main]
async fn main() {
    let mut args = env::args().skip(1);
    let Some(backend) = args.next() else {
        usage();
        std::process::exit(2);
    };
    let Some(bind_addr) = args.next() else {
        usage();
        std::process::exit(2);
    };

    let secret = match env::var("PACORD_SECRET") {
        Ok(value) if value.len() >= 16 => value.into_bytes(),
        _ => {
            eprintln!("PACORD_SECRET deve existir e ter pelo menos 16 bytes; nenhum segredo padrão é aceito.");
            std::process::exit(2);
        }
    };
    let bind_addr: SocketAddr = match bind_addr.parse() {
        Ok(value) => value,
        Err(error) => {
            eprintln!("endereço de bind inválido: {error}");
            std::process::exit(2);
        }
    };
    let config = CaptureConfig::default();

    let result = match backend.as_str() {
        "wayland" => run_wayland_host(bind_addr, secret, config).await,
        "x11" => run_x11_host(bind_addr, secret, config).await,
        _ => {
            usage();
            std::process::exit(2);
        }
    };

    if let Err(error) = result {
        eprintln!("PACORD host encerrou com erro: {error}");
        std::process::exit(1);
    }
}
