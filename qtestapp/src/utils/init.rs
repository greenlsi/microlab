use dotenv::dotenv;
use qtest::parser::Parser;
use qtest::socket::tcp::SocketTcp;
use qtest::{Irq, IrqState};
use qtest_stm32f4nucleo::Peripheral;

use serde_json::Value;
use std::env;

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info};


use crate::utils::fields::handle_irq_update;

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    info!("Servidor iniciado");
}

pub fn load_env() -> (String, String, String) {
    dotenv().ok();
    let api_url = env::var("API_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let ws_url = env::var("WS_URL").unwrap_or_else(|_| "127.0.0.1:8081".to_string());
    let qemu_url = env::var("QEMU_URL").unwrap_or_else(|_| "localhost:3000".to_string());

    (api_url, ws_url, qemu_url)
}


pub async fn init_parser(
    qemu_url: &str,
) -> Result<
    (
        Arc<Mutex<Parser<SocketTcp>>>,
        mpsc::Receiver<Irq>,
        mpsc::Sender<()>,
    ),
    Box<dyn std::error::Error>,
> {
    let (mut parser, rx_irq) = Parser::<SocketTcp>::new(qemu_url).await.unwrap();
    debug!("parser inicializado esperando a attach_connection");
    parser.attach_connection().await.unwrap();

    let res = parser.irq_intercept_in("/machine/soc").await.unwrap();
    debug!("IRQ Intercept In: {:?}", res);

    let parser_arc = Arc::new(Mutex::new(parser));
    let (reconnect_tx, reconnect_rx) = mpsc::channel::<()>(1);
    info!("[Parser] Device connected successfully");

    tokio::spawn(reconnect_loop(parser_arc.clone(), reconnect_rx));

    Ok((parser_arc, rx_irq, reconnect_tx))
}

async fn reconnect_loop(
    parser_arc: Arc<Mutex<Parser<SocketTcp>>>,
    mut reconnect_rx: mpsc::Receiver<()>,
) {
    loop {
        reconnect_rx.recv().await;
        info!("Reconectando...");
        let mut locked_parser = parser_arc.lock().await;
        match locked_parser.attach_connection().await {
            Ok(_) => {
                info!("[Parser] Conectado correctamente a QEMU");
                if let Err(e) = locked_parser.irq_intercept_in("/machine/soc").await {
                    error!("[Parser] Error al interceptar IRQ: {:?}", e);
                }
            }
            Err(e) => {
                error!("[Parser] Fallo al conectar: {:?}", e);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

pub async fn handle_irqs(
    mut rx_irq: mpsc::Receiver<Irq>,
    reconnect_tx: mpsc::Sender<()>,
    ws_tx: tokio::sync::watch::Sender::<String>,
    json_data: Arc<Mutex<Value>>,
    parser_arc: Arc<Mutex<Parser<SocketTcp>>>,
    periferico: Peripheral,
) {
    loop {
        let irq = rx_irq.recv().await.unwrap();
        if irq.state == IrqState::Disconnected {
            info!("Reconectando...");
            reconnect_tx.send(()).await.unwrap();
            continue;
        }
        info!("[Parser] Received IRQ: {:?}", irq);
        handle_irq_update(
            json_data.clone(),
            &irq,
            ws_tx.clone(),
            periferico.clone(),
            parser_arc.clone(),
        )
        .await;
    }
}
