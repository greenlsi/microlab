use futures_util::{SinkExt, StreamExt};
use qtest::Irq;
use qtest::{parser::Parser, socket::tcp::SocketTcp};
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

    // Inicializa el parser y el receptor de interrupciones
    let (mut parser, mut rx_irq): (Parser<SocketTcp>, Receiver<_>) =
        Parser::<SocketTcp>::new("localhost:3000").await.unwrap();

    // Inicia QEMU con los parámetros adecuados
    //start_qemu().unwrap();

    parser.attach_connection().await.unwrap();

    //}
    info!("[Parser] Device connected successfully");

    let res = parser.irq_intercept_in("/machine/soc").await.unwrap();
    info!("IRQ Intercept In: {:?}", res);

    let parser = Arc::new(Mutex::new(parser));

    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Inicializa el periférico
    let periferico = Peripheral::new();

    // Manejar las conexiones WebSocket en un hilo separado
    let json_data_clone = json_data.clone();
    let parser_clone = parser.clone();
    let periferico_clone = periferico.clone();
    let ws_tx_clone = ws_tx.clone();

    tokio::spawn(async move {
        let listener = TcpListener::bind(websocket_addr)
            .await
            .expect("Error al enlazar el listener");
        while let Ok((stream, _)) = listener.accept().await {
            let ws_stream = accept_async(stream)
                .await
                .expect("Error al aceptar conexión WebSocket");
            info!("Nuevo cliente conectado");

            // Gestionar la conexión WebSocket
            let ws_rx_clone = ws_rx.clone();
            tokio::spawn(handle_connection(
                ws_stream,
                ws_rx_clone,
                json_data_clone.clone(),
                periferico_clone.clone(),
                parser_clone.clone(),
            ));
        }
    });

    let json_data_clone2 = json_data.clone();
    let periferico_clone = periferico.clone();
    let parser_clone = parser.clone();
    //let ws_tx_clone_clone = ws_tx_clone.clone();

    tokio::spawn(async move {
        loop {
            let irq = rx_irq.recv().await.unwrap();
            info!("[Parser] Received IRQ: {:?}", irq);
            handle_irq_update(
                json_data_clone2.clone(),
                &irq,
                ws_tx_clone.clone(),
                periferico_clone.clone(),
                parser_clone.clone(),
            )
            .await;
            // if let Err(e) = ws_tx.send(format!("{:?}", irq)) {
            //     error!("Error al enviar IRQ: {:?}", e);
            //     break; // Detener el bucle si el WebSocket se cierra
            // }
        }
    });

    // Configurar las rutas de la API REST

    // Configuración CORS
    let cors = warp::cors()
        .allow_any_origin() // Permitir cualquier origen
        .allow_methods(vec!["GET", "POST", "OPTIONS"]) // Permitir métodos específicos
        .allow_headers(vec!["Content-Type", "Authorization"]); // Permitir cabeceras

    // Aplicar CORS para permitir solicitudes de cualquier origen
    let routes = pulsar_boton(parser.clone(), periferico.clone())
        .with(cors) // Permitir cualquier origen
        .with(warp::log("api")); // Asegurarte de que esté aplicado para todas las rutas

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;

    // Dejar que el servidor REST corra sin terminar el proceso principal
    debug!("Servidor REST escuchando en http://127.0.0.1:8080");
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    }
}

// Manejar la conexión WebSocket
async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    mut ws_rx: tokio::sync::watch::Receiver<String>, // Receptor del canal watch
    arc_mutex_json_data: Arc<Mutex<Value>>, // Variable compartida para almacenar los campos
    peripheral: Peripheral,
    parser: Arc<Mutex<Parser<SocketTcp>>>,
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
                            match prepare_fields(arc_mutex_json_data.clone(), peripheral.clone(), parser.clone()).await {
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

// Función set_irq_in que manipula el estado de los pines GPIO
async fn create_interruption(
    nombre_gpio: String,
    pin: usize,
    value: isize,
    parser: &mut Parser<SocketTcp>,
) -> Result<(), warp::Rejection> {
    let gpio = match nombre_gpio.as_str() {
        "gpio_a" => "0",
        "gpio_b" => "1",
        "gpio_c" => "2",
        _ => return Err(warp::reject::custom(InvalidGpioName)),
    };

    parser
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
    parser: Arc<Mutex<Parser<SocketTcp>>>,
    periferico: Peripheral,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!(String / "pulsar_boton" / usize / isize)
        .and(warp::any().map(move || Arc::clone(&parser)))
        .and(warp::any().map(move || periferico.clone()))
        .and_then(
            |nombre_gpio: String,
             pin: usize,
             value: isize,
             parser: Arc<Mutex<Parser<SocketTcp>>>,
             periferico: Peripheral| async move {
                let mut p = parser.lock().await;
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

//funcion para inicializar qemu:

// fn start_qemu() -> Result<(), Box<dyn std::error::Error>> {
//     let file = File::create("./qtest-stm32f4nucleo/src/bin/output_qemu.txt").map_err(|e| {
//         info!("Failed to create output file: {}", e);
//         e
//     })?;

//     let stdio = Stdio::from(file);

//     Command::new("../qemu_new/build/qemu-system-arm")
//         .args([
//             "-cpu",
//             "cortex-m4",
//             "-machine",
//             "netduinoplus2",
//             "-nographic",
//             "-semihosting-config",
//             "enable=on,target=native",
//             "-monitor",
//             "stdio",
//             "-qtest",
//             "tcp:localhost:3000",
//             "-kernel",
//             "../test_v1.elf",
//         ])
//         .stdout(stdio)
//         .stderr(Stdio::piped()) // Captura stderr para monitorear errores
//         .spawn()
//         .expect("Failed to start QEMU");

//     Ok(())
// }

//RELATED TO JSON MESSAGE

async fn handle_irq_update(
    json_data: Arc<Mutex<Value>>,
    irq: &Irq,
    ws_tx: tokio::sync::watch::Sender<String>,
    peripheral: Peripheral,
    parser: Arc<Mutex<Parser<SocketTcp>>>,
) {
    debug!("Iniciando actualización del JSON con IRQ...");

    // Intentamos actualizar los campos
    match update_fields(json_data.clone(), irq, peripheral, parser).await {
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

async fn update_fields(
    json_data: Arc<Mutex<Value>>,
    irq: &Irq,
    peripheral: Peripheral,
    parser: Arc<Mutex<Parser<SocketTcp>>>,
) -> Result<(), InvalidGpioName> {
    let mut p = parser.lock().await;

    // Obtener un bloqueo mutable del JSON
    //esto seria fields
    let mut json_data = json_data.lock().await;
    if let Some(fields) = json_data.get_mut("fields").and_then(|v| v.as_object_mut()) {
        for (field_name, field_data) in fields.iter_mut() {
            let peripheral_n;
            let pin_n;

            if let Some(peripheral_name) = field_data.get("peripheral").and_then(|v| v.as_str()) {
                peripheral_n = peripheral_name;
                let gpio_line = peripheral_to_number(peripheral_name)?;
                debug!(
                    "Procesando periférico: {} (GPIO line: {})",
                    peripheral_name, gpio_line
                );
                if irq.line.to_string() != gpio_line {
                    debug!(
                        "IRQ line ({}) no coincide con GPIO line ({}). Ignorando...",
                        irq.line, gpio_line
                    );
                    continue; //pasa al siguiente elemento de fields si no coincide con la interrupcion
                }
            } else {
                error!("Campo 'peripheral' no encontrado en {}", field_name);
                return Err(InvalidGpioName);
            }
            if let Some(pin_str) = field_data.get("pin").and_then(|v| v.as_str()) {
                pin_n = pin_str;
            } else {
                error!("Campo 'pin' no encontrado en {}", field_name);
                return Err(InvalidGpioName);
            }

            let pin = pin_n.parse::<usize>().map_err(|_| InvalidGpioName)?;
            info!(pin);
            // Conseguir el MODER y actualizar los campos correspondientes
            if let Some(gpio) = peripheral.get(peripheral_n) {
                match gpio.moder().get_mode(pin, &mut p).await {
                    Ok(mode) => {
                        let mode_clone = mode.clone();
                        //field_data["mode"] = Value::String(mode.clone());
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
                error!("Al parecer no encuentra peripheral?");
            }
        }
    } else {
        error!("El JSON no contiene un objeto válido en 'fields'.");
    }
    Ok(())
}

async fn prepare_fields(
    json_data: Arc<Mutex<Value>>,
    peripheral: Peripheral,
    parser: Arc<Mutex<Parser<SocketTcp>>>,
) -> Result<(), InvalidGpioName> {
    let mut p = parser.lock().await;

    // Obtener un bloqueo mutable del JSON
    let mut json_data = json_data.lock().await;

    // Verificar que el JSON tiene un objeto "fields"
    if let Some(fields) = json_data.get_mut("fields").and_then(|v| v.as_object_mut()) {
        for (field_name, field_data) in fields.iter_mut() {
            let peripheral_n;
            let pin_n;
            if let Some(peripheral_name) = field_data.get("peripheral").and_then(|v| v.as_str()) {
                peripheral_n = peripheral_name;
            } else {
                error!("Campo 'peripheral' no encontrado en {}", field_name);
                return Err(InvalidGpioName);
            }
            //info!("{}", field_data);
            if let Some(pin_str) = field_data.get("pin").and_then(|v| v.as_str()) {
                pin_n = pin_str;
            } else {
                error!("Campo 'pin' no encontrado en {}", field_name);
                return Err(InvalidGpioName);
            }

            let pin = pin_n.parse::<usize>().map_err(|_| InvalidGpioName)?;

            // Conseguir el MODER y actualizar los campos correspondientes
            if let Some(gpio) = peripheral.get(peripheral_n) {
                match gpio.moder().get_mode(pin, &mut p).await {
                    Ok(mode) => {
                        let mode_clone = mode.clone();
                        //field_data["mode"] = Value::String(mode.clone());
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
            }
        }
    } else {
        error!("El JSON no contiene un objeto válido en 'fields'.");
    }

    // TO DO: mandar un update_fields inicial para actualizar los valores de los pines? (antes de que se produzca interrupción)

    Ok(())
}

//dependiendo del modo en el que esté
async fn update_data(
    gpio: &Gpio,
    pin: usize,
    mode_str: &str,
    parser: &mut Parser<SocketTcp>,
) -> Result<Value, std::io::Error> {
    match mode_str {
        "Input" => gpio.idr().is_high(pin, parser).await.map(Value::Bool),
        "Output" => gpio.odr().is_high(pin, parser).await.map(Value::Bool),
        "Alternate Function" => Ok(Value::String("AF Config".to_string())),
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
