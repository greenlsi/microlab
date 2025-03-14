use futures_util::{SinkExt, StreamExt};
use qtest::Irq;
//use qtest::{session::Session, socket::tcp::SocketTcp};
use qtest::{session::Session, socket::SocketListener};
use qtest_stm32f4nucleo::gpio::Gpio;
use qtest_stm32f4nucleo::Peripheral;
use serde_json::{json, Value};
use std::fmt::Debug;
//use std::fs::File;
//use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{debug, error, info};
//use tracing_subscriber;
use warp::{reject::Reject, Filter};

// Define errores personalizados
#[derive(Debug)]
struct InvalidGpioName;
impl Reject for InvalidGpioName {}

#[derive(Debug)]
struct CustomError;
impl Reject for CustomError {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializa tracing con un formato de salida básico
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG) // Muestra logs de nivel DEBUG o superior
        .init();

    let json_data: Arc<Mutex<Value>> = Arc::new(Mutex::new(json!([])));

    // Configurar el servidor WebSocket
    let websocket_addr = "127.0.0.1:8081"; // Puerto para WebSocket
    info!("Servidor WebSocket escuchando en {}", websocket_addr);

    let (ws_tx, ws_rx) = tokio::sync::watch::channel::<String>("".to_string());

    //INICIALIZAMOS PARSER(CONEXION CON QEMU)

    // Inicializa el session y el receptor de interrupciones
    // let (mut session, mut rx_irq): (Session, Receiver<_>) =
    //     Session::<SocketTcp>::new("localhost:3000").await.unwrap();

    let address = "localhost:3000";

    let mut listener = SocketListener::new(address).await.unwrap();
    tracing::info!("Socket listener ready. listening on {}", listener.address());
    let mut listener = Arc::new(Mutex::new(listener));
    // Inicia QEMU con los parámetros adecuados
    //start_qemu().unwrap();

    let (session, irq_receiver) = {
        let mut listener_guard = listener.lock().await;
        setup_session(&mut listener_guard).await
    };
    let session_arc = Arc::new(Mutex::new(session));
    let irq_receiver = Arc::new(Mutex::new(irq_receiver));
    let periferico = Peripheral::new();

    let session_clone = session_arc.clone();
    let listener_clone = listener.clone();
    let irq_receiver_clone = irq_receiver.clone();
    let json_data_clone2 = json_data.clone();
    let ws_tx_clone = ws_tx.clone();
    let periferico_clone = periferico.clone();

    tokio::spawn(async move {
        loop {
            {
                let mut session_guard = session_clone.lock().await;
                let mut listener_guard = listener_clone.lock().await;
                let mut irq_receiver_guard = irq_receiver_clone.lock().await;
                if !session_guard.is_alive() {
                    info!("[Session] Connection lost. Attempting to reconnect...");
                    let (new_session, new_irq_receiver) = setup_session(&mut *listener_guard).await;
                    *session_guard = new_session;
                    *irq_receiver_guard = new_irq_receiver;
                }
            }
        }
    });   
     /////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Inicializa el periférico
 

    tokio::spawn(async move {
        loop {
            let mut irq_receiver_guard = irq_receiver.lock().await;
            let irq = irq_receiver_guard.recv().await.unwrap();
            info!("[Session] Received IRQ: {:?}", irq);
            handle_irq_update(
                json_data_clone2.clone(),
                &irq,
                ws_tx_clone.clone(),
                periferico_clone.clone(),
                session_clone.clone(),
            )
            .await;
        }
        tracing::error!("IRQ receiver terminated");
    });


    // let json_data_clone2 = json_data.clone();
    // let periferico_clone = periferico.clone();
    // let session_clone = session_arc.clone();
    //let ws_tx_clone_clone = ws_tx_clone.clone();


    // tokio::spawn(async move {
    //     loop {
    //         let irq = rx_irq.recv().await.unwrap();
    //         info!("[Session] Received IRQ: {:?}", irq);
    //         handle_irq_update(
    //             json_data_clone2.clone(),
    //             &irq,
    //             ws_tx_clone.clone(),
    //             periferico_clone.clone(),
    //             session_clone.clone(),
    //         )
    //         .await;
    //         // if let Err(e) = ws_tx.send(format!("{:?}", irq)) {
    //         //     error!("Error al enviar IRQ: {:?}", e);
    //         //     break; // Detener el bucle si el WebSocket se cierra
    //         // }
    //     }
    // });

    // Configurar las rutas de la API REST

    // Configuración CORS
    let cors = warp::cors()
        .allow_any_origin() // Permitir cualquier origen
        .allow_methods(vec!["GET", "POST", "OPTIONS"]) // Permitir métodos específicos
        .allow_headers(vec!["Content-Type", "Authorization"]); // Permitir cabeceras

    // Aplicar CORS para permitir solicitudes de cualquier origen
    let routes = pulsar_boton(session_arc.clone(), periferico.clone())
        .with(cors) // Permitir cualquier origen
        .with(warp::log("api")); // Asegurarte de que esté aplicado para todas las rutas

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;

    // Dejar que el servidor REST corra sin terminar el proceso principal
    debug!("Servidor REST escuchando en http://127.0.0.1:8080");
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    }
}

// Función set_irq_in que manipula el estado de los pines GPIO
async fn create_interruption(
    nombre_gpio: String,
    pin: usize,
    value: isize,
    session: &mut Session,
) -> Result<(), warp::Rejection> {
    let gpio = match nombre_gpio.as_str() {
        "gpio_a" => "0",
        "gpio_b" => "1",
        "gpio_c" => "2",
        _ => return Err(warp::reject::custom(InvalidGpioName)),
    };

    session
        .set_irq_in(
            &format!("/machine/soc/gpio[{}]", gpio),
            "input-in",
            pin,
            value,
        )
        .await
        .map_err(|_| warp::reject::custom(CustomError))?;
    Ok(())
}

// Filtro pulsar_boton que establece la ruta y recibe los parámetros
fn pulsar_boton(
    session: Arc<Mutex<Session>>,
    periferico: Peripheral,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!(String / "pulsar_boton" / usize / isize)
        .and(warp::any().map(move || Arc::clone(&session)))
        .and(warp::any().map(move || periferico.clone()))
        .and_then(
            |nombre_gpio: String,
             pin: usize,
             value: isize,
             session: Arc<Mutex<Session>>,
             periferico: Peripheral| async move {
                let mut p = session.lock().await;
                create_interruption(nombre_gpio, pin, value, &mut p).await?;
                let register_value = periferico.gpio_c().idr().is_high(pin, &mut p).await;
                match register_value {
                    Ok(value) => Ok(warp::reply::json(&format!(
                        "Pin {} configurado a {}. Valor del registro: {}",
                        pin, value, value
                    ))),
                    Err(e) => {
                        error!(
                            "Error al leer el valor del registro en pulsar_boton: {:?}",
                            e
                        );
                        Err(warp::reject::custom(CustomError))
                    }
                }
            },
        )
}

//RELATED TO JSON MESSAGE
async fn handle_irq_update(
    json_data: Arc<Mutex<Value>>,
    irq: &Irq,
    ws_tx: tokio::sync::watch::Sender<String>,
    peripheral: Peripheral,
    session: Arc<Mutex<Session>>,
) {
    debug!("Iniciando actualización del JSON con IRQ...");

    // Intentamos actualizar los campos
    match process_fields(json_data.clone(), Some(irq), peripheral, session, true).await {
        Ok(()) => {
            let updated_json = {
                let json_locked = json_data.lock().await;
                json_locked.clone()
            };

            // Enviar el JSON actualizado al canal
            let json_str = serde_json::to_string(&updated_json).unwrap_or_else(|e| {
                error!("Error al serializar el JSON actualizado: {}", e);
                "{}".to_string()
            });

            if let Err(e) = ws_tx.send(json_str) {
                error!("Error al enviar el JSON actualizado: {}", e);
            } else {
                info!("JSON actualizado enviado exitosamente.");
            }
        }
        Err(e) => {
            error!("Error durante la actualización de los campos: {:?}", e);
        }
    }
}

// Manejar la conexión WebSocket
async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    mut ws_rx: tokio::sync::watch::Receiver<String>, // Receptor del canal watch
    arc_mutex_json_data: Arc<Mutex<Value>>, // Variable compartida para almacenar los campos
    peripheral: Peripheral,
    session: Arc<Mutex<Session>>,
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

//dependiendo del modo en el que esté
async fn update_data(
    gpio: &Gpio,
    pin: usize,
    mode_str: &str,
    session: &mut Session,
) -> Result<Value, std::io::Error> {
    match mode_str {
        "Input" => gpio.idr().is_high(pin, session).await.map(Value::Bool),
        "Output" => gpio.odr().is_high(pin, session).await.map(Value::Bool),
        "Alternate Function" => {
            if pin < 8 {
                //gpio.afrl().get_alternate_function(pin, session).await.map(|v| Value::Number(v.into()))
                gpio.idr().is_high(pin, session).await.map(Value::Bool)
            } else {
                //gpio.afrh().get_alternate_function(pin, session).await.map(|v| Value::Number(v.into()))
                gpio.odr().is_high(pin, session).await.map(Value::Bool)
            }
        }
        "Analog" => Ok(Value::String("Analog Data".to_string())),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid mode",
        )),
    }
}

// Función que convierte el nombre del GPIO en un número de línea (String)
fn peripheral_to_number(nombre_gpio: &str) -> Result<&'static str, InvalidGpioName> {
    match nombre_gpio {
        "gpio_a" => Ok("0"),
        "gpio_b" => Ok("1"),
        "gpio_c" => Ok("2"),
        "gpio_d" => Ok("3"),
        "gpio_e" => Ok("4"),
        "gpio_f" => Ok("5"),
        "gpio_g" => Ok("6"),
        "gpio_h" => Ok("7"),
        "gpio_i" => Ok("8"),
        _ => Err(InvalidGpioName), // Retorna un error si el nombre no es válido
    }
}

async fn handle_receive_fields(
    json_data: String,
    arc_mutex_json_data: Arc<Mutex<Value>>, // Variable compartida para almacenar los campos
) -> Result<(), Box<dyn std::error::Error>> {
    // Parsear el JSON recibido
    let received_json: Value = serde_json::from_str(&json_data)?;

    // Extraer los campos y almacenarlos en arc_mutex_json_data
    let mut stored_data = arc_mutex_json_data.lock().await;
    *stored_data = received_json;
    debug!("Campos iniciales guardados: {}", stored_data);

    Ok(())
}

async fn process_fields(
    json_data: Arc<Mutex<Value>>,
    irq: Option<&Irq>, // Solo se pasa si es para la actualización de los campos debido a una interrupción
    peripheral: Peripheral,
    session: Arc<Mutex<Session>>,
    is_update: bool, // Si es actualización o solo preparación
) -> Result<(), InvalidGpioName> {
    let mut p = session.lock().await;
    let mut json_data = json_data.lock().await;

    if let Some(fields) = json_data.get_mut("fields").and_then(|v| v.as_object_mut()) {
        for (field_name, field_data) in fields.iter_mut() {
            let peripheral_n;
            let pin_n;

            // Filtrar solo los pines de tipo "gpio"
            if let Some(field_type) = field_data.get("type").and_then(|v| v.as_str()) {
                if field_type != "gpio" {
                    continue; // Ignorar cualquier campo que no sea "gpio"
                }
            } else {
                error!("Campo 'type' no encontrado en {}", field_name);
                return Err(InvalidGpioName);
            }

            // Obtener el nombre del periférico y pin
            if let Some(peripheral_name) = field_data.get("port").and_then(|v| v.as_str()) {
                peripheral_n = peripheral_name;
            } else {
                error!("Campo 'port' no encontrado en {}", field_name);
                return Err(InvalidGpioName);
            }

            // if let Some(pin_str) = field_data.get("pin").and_then(|v| v.as_str()) {
            //     pin_n = pin_str;

            if let Some(pin) = field_data.get("pin").and_then(|v| v.as_u64()) {
                pin_n = pin;
            } else {
                error!("Campo 'pin' no encontrado en {}", field_name);
                return Err(InvalidGpioName);
            }
            let pin = pin_n as usize;

            //let pin = pin_n.parse::<usize>().map_err(|_| InvalidGpioName)?;

            // Conseguir el MODER y actualizar los campos correspondientes
            if let Some(gpio) = peripheral.get(peripheral_n) {
                match gpio.moder().get_mode(pin, &mut p).await {
                    Ok(mode) => {
                        let mode_clone = mode.clone();

                        // Solo actualizar si es un "update"
                        if is_update {
                            // Verificar si la línea IRQ coincide con el GPIO
                            if let Some(irq_ref) = irq {
                                let gpio_line = peripheral_to_number(peripheral_n)?;
                                if irq_ref.line.to_string() != gpio_line {
                                    continue;
                                }
                            }

                            match update_data(gpio, pin, &mode_clone, &mut p).await {
                                Ok(data) => {
                                    field_data["mode"] = Value::String(mode.clone());
                                    field_data["data"] = data;
                                }
                                Err(e) => {
                                    error!(
                                        "Error leyendo datos para el pin {} del periférico {}: {:?}",
                                        pin, peripheral_n, e
                                    );
                                    field_data["mode"] = Value::String("Error".to_string());
                                    field_data["data"] = Value::Null;
                                }
                            }
                        } else {
                            // Preparación de campos antes de la interrupción
                            field_data["mode"] = Value::String(mode.clone());
                            match update_data(gpio, pin, &mode_clone, &mut p).await {
                                Ok(data) => {
                                    field_data["data"] = data;
                                }
                                Err(_) => {
                                    field_data["data"] = Value::Null;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            "Error obteniendo el modo para el pin {} del periférico {}: {:?}",
                            pin, peripheral_n, e
                        );
                        field_data["mode"] = Value::String("Error".to_string());
                        field_data["data"] = Value::Null;
                    }
                }
            } else {
                error!("No se encontró el periférico {}", peripheral_n);
            }
        }
    } else {
        error!("El JSON no contiene un objeto válido en 'fields', puede que no se haya recibido mensaje del websocket");
    }

    Ok(())
}


async fn setup_session(listener: &mut SocketListener) -> (Session, tokio::sync::mpsc::Receiver<Irq>) {
    tracing::info!("Waiting for new session...");
    let mut session = listener.new_session().await.unwrap();
    tracing::info!("New session created");
    
    info!("[Session] Device connected successfully");
    let irq_receiver = session.irq_intercept_in("/machine/soc").await.unwrap();
    tracing::debug!("IRQ Intercept In enabled");
    
    (session, irq_receiver)
}