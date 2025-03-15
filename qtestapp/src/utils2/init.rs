use dotenv::dotenv;
use qtest::session::Session;
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

pub async fn init_session(
    qemu_url: &str,
) -> Result<(Arc<Mutex<Session>>, mpsc::Receiver<Irq>, mpsc::Sender<()>), Box<dyn std::error::Error>>
{
    
    // let (mut session, rx_irq) = Session::<SocketTcp>::new(qemu_url).await.unwrap();
    // debug!("session inicializado esperando a attach_connection");
    // session.attach_connection().await.unwrap();

    // let res = session.irq_intercept_in("/machine/soc").await.unwrap();
    // debug!("IRQ Intercept In: {:?}", res);

    // let session_arc = Arc::new(Mutex::new(session));
    // let (reconnect_tx, reconnect_rx) = mpsc::channel::<()>(1);
    // info!("[Session] Device connected successfully");

    info!("Waiting for new session...");
    let mut session = listener.new_session().await.unwrap();
    
    info!("[Session] Device connected successfully");
    let irq_receiver = session.irq_intercept_in("/machine/soc").await.unwrap();
    debug!("IRQ Intercept In enabled");
    

    tokio::spawn(reconnect_loop(session_arc.clone(), reconnect_rx));

    Ok((session, irq_receiver))
}


//TO DOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO
async fn reconnect_loop(session_arc: Arc<Mutex<Session>>, mut reconnect_rx: mpsc::Receiver<()>) {
    loop {
        reconnect_rx.recv().await;
        info!("Reconectando...");
        let mut locked_session = session_arc.lock().await;
        match locked_session.attach_connection().await {
            Ok(_) => {
                info!("[Session] Conectado correctamente a QEMU");
                if let Err(e) = locked_session.irq_intercept_in("/machine/soc").await {
                    error!("[Session] Error al interceptar IRQ: {:?}", e);
                }
            }
            Err(e) => {
                error!("[Session] Fallo al conectar: {:?}", e);
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}

pub async fn handle_irqs(
    mut rx_irq: mpsc::Receiver<Irq>,
    ws_tx: tokio::sync::watch::Sender<String>,
    json_data: Arc<Mutex<Value>>,
    session_arc: Arc<Mutex<Session>>,
    periferico: Peripheral,
) {
        info!("[Session] Received IRQ: {:?}", irq);
        handle_irq_update(
            json_data.clone(),
            &irq,
            ws_tx.clone(),
            periferico.clone(),
            session_arc.clone(),
        )
        .await;
    }

