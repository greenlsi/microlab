use dotenv::dotenv;
use qtest_stm32f4nucleo::Peripheral;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};
use warp::Filter;

mod utils;
use utils::api_routes::{get_timer_info, pulsar_boton};
use utils::init::*;
use utils::websocket::start_websocket_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Inicializa tracing y variables de entorno desde el archivo .env
    init_tracing();
    dotenv().ok();

    //Obtiene las variables de entorno
    let (api_url, ws_url, qemu_url) = load_env();

    // let (host, port) = parse_host_port(&api_url);
    info!("API_URL: {}, WS_URL : {}", api_url, ws_url);

    // Inicializa el mensaje que utilizará el WebSocket y el periferico
    let json_data: Arc<Mutex<Value>> = Arc::new(Mutex::new(json!([])));
    let periferico = Peripheral::new();

    // Inicializar Parser (Conexión con QEMU)
    let (parser_arc, rx_irq, reconnect_tx) = init_parser(&qemu_url).await?;

    //Iniciamos websocket
    let (ws_tx, ws_rx) = tokio::sync::watch::channel::<String>("".to_string());
    tokio::spawn(start_websocket_server(
        ws_url.clone(),
        ws_rx,
        json_data.clone(),
        parser_arc.clone(),
        periferico.clone(),
    ));
    info!("Servidor WebSocket escuchando en {}", ws_url);

    //Iniciamos el hilo para manejar interrupciones
    tokio::spawn(handle_irqs(
        rx_irq,
        reconnect_tx,
        ws_tx.clone(),
        json_data.clone(),
        parser_arc.clone(),
        periferico.clone(),
    ));

    // Configurar servidor REST con rutas
    // Configuración CORS
    let cors = warp::cors()
        .allow_any_origin() // Permitir cualquier origen
        .allow_methods(vec!["GET", "POST", "OPTIONS"]) // Permitir métodos específicos
        .allow_headers(vec!["Content-Type", "Authorization"]); // Permitir cabeceras

    // Aplicar CORS para permitir solicitudes de cualquier origen
    let routes = pulsar_boton(parser_arc.clone(), periferico.clone())
        .or(get_timer_info(parser_arc.clone(), periferico.clone()))
        .with(cors) // Permitir cualquier origen
        .with(warp::log("api")); // Asegurarte de que esté aplicado para todas las rutas

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;

    // Dejar que el servidor REST corra sin terminar el proceso principal
    debug!("Servidor REST escuchando en {}", api_url);

    Ok(())
}
