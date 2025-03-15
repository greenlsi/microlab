use futures_util::{SinkExt, StreamExt};

use qtest::session::Session;
use qtest::socket::tcp::SocketTcp;
use qtest_stm32f4nucleo::Peripheral;
use tokio::net::TcpListener;

use serde_json::{json, Value};

use std::sync::Arc;

use tokio::sync::{watch, Mutex};
use tokio_tungstenite::accept_async;

use crate::utils::fields::{handle_receive_fields, process_fields};
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{error, info};

// Manejar la conexión WebSocket
pub async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    mut ws_rx: tokio::sync::watch::Receiver<String>, // Receptor del canal watch
    arc_mutex_json_data: Arc<Mutex<Value>>, // Variable compartida para almacenar los campos
    session: Arc<Mutex<Session>>,
    peripheral: Peripheral,
) {
    let (mut write, mut read) = ws_stream.split();

    // Manejo concurrente de mensajes del cliente y del canal
    loop {
        tokio::select! {
            // Procesar mensajes recibidos del cliente
            Some(message) = read.next() => {
                match message {
                    Ok(msg) => {
                        if let Message::Text(text) = msg {
                            if let Err(e) = handle_receive_fields(text, arc_mutex_json_data.clone()).await {
                                error!("Error al procesar mensaje del cliente: {}", e);
                            }
                            // Intentamos actualizar los campos
                            match process_fields(arc_mutex_json_data.clone(), None,peripheral.clone(), session.clone(), false).await {
                                Ok(()) => {
                                    let updated_json = {
                                        let json_locked = arc_mutex_json_data.lock().await;
                                        json_locked.clone()
                                    };

                                    // Enviar el JSON actualizado al canal
                                    let json_str = serde_json::to_string(&updated_json).unwrap_or_else(|e| {
                                        error!("Error al serializar el JSON actualizado: {}", e);
                                        "{}".to_string()
                                    });

                                    if let Err(e) = write.send(Message::Text(json_str)).await {
                                        error!("Error al enviar el JSON actualizado: {}", e);
                                    }
                                }
                                Err(e) => {
                                    error!("Error al preparar los campos: {:?}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error en la conexión WebSocket: {:?}", e);
                        break;
                    }
                }
            }
            // Procesar mensajes desde el canal `mpsc`
            _ = ws_rx.changed() => {
                let message = ws_rx.borrow().clone();
                let mensaje = json!(message);
                let mensaje_serializado = match serde_json::to_string(&mensaje) {
                    Ok(json_str) => json_str,
                    Err(e) => {
                        eprintln!("Error al serializar mensaje de mpsc: {}", e);
                        continue;
                    }
                };
                if write.send(Message::Text(mensaje_serializado)).await.is_err() {
                    error!("[WebSocket] Error al enviar mensaje desde mpsc.");
                    break;
                }
            }
        }
    }
}

pub async fn start_websocket_server(
    ws_url: String,
    ws_rx: watch::Receiver<String>,
    json_data: Arc<Mutex<Value>>,
    session_arc: Arc<Mutex<Session>>,
    peripheral: Peripheral,
) {
    let listener = TcpListener::bind(ws_url)
        .await
        .expect("Error al enlazar el listener WebSocket");

    while let Ok((stream, _)) = listener.accept().await {
        let ws_stream = accept_async(stream)
            .await
            .expect("Error al aceptar conexión WebSocket");
        info!("Nuevo cliente WebSocket conectado");

        tokio::spawn(handle_connection(
            ws_stream,
            ws_rx.clone(),
            json_data.clone(),
            session_arc.clone(),
            peripheral.clone(),
        ));
    }
}
