use qtest::session::Session;
use qtest::socket::tcp::SocketTcp;
use qtest::Irq;
use qtest_stm32f4nucleo::gpio::Gpio;
use qtest_stm32f4nucleo::Peripheral;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};
//use qtest::IrqState;
// Define errores personalizados
use crate::utils::error::InvalidPeripheralName;

pub async fn process_fields(
    json_data: Arc<Mutex<Value>>,
    irq: Option<&Irq>, // Solo se pasa si es para la actualización de los campos debido a una interrupción
    peripheral: Peripheral,
    session: Arc<Mutex<Session>>,
    is_update: bool, // Si es actualización o solo preparación
) -> Result<(), InvalidPeripheralName> {
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
                return Err(InvalidPeripheralName);
            }

            // Obtener el nombre del periférico y pin
            if let Some(peripheral_name) = field_data.get("port").and_then(|v| v.as_str()) {
                peripheral_n = peripheral_name;
            } else {
                error!("Campo 'port' no encontrado en {}", field_name);
                return Err(InvalidPeripheralName);
            }

            // if let Some(pin_str) = field_data.get("pin").and_then(|v| v.as_str()) {
            //     pin_n = pin_str;

            if let Some(pin) = field_data.get("pin").and_then(|v| v.as_u64()) {
                pin_n = pin;
            } else {
                error!("Campo 'pin' no encontrado en {}", field_name);
                return Err(InvalidPeripheralName);
            }
            let pin = pin_n as usize;

            //let pin = pin_n.parse::<usize>().map_err(|_| InvalidPeripheralName)?;

            // Conseguir el MODER y actualizar los campos correspondientes
            if let Some(gpio) = peripheral.get_gpio(peripheral_n) {
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

pub async fn handle_receive_fields(
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

//dependiendo del modo en el que esté
pub async fn update_data(
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
                gpio.afrl()
                    .get_alternate_function(pin, session)
                    .await
                    .map(|v| Value::Number(v.into()))
            } else {
                gpio.afrh()
                    .get_alternate_function(pin, session)
                    .await
                    .map(|v| Value::Number(v.into()))
            }
        }
        "Analog" => Ok(Value::String("Analog Data".to_string())),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid mode",
        )),
    }
}

//RELATED TO JSON MESSAGE
pub async fn handle_irq_update(
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
            info!(json_str);
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

pub fn peripheral_to_number(nombre_gpio: &str) -> Result<&'static str, InvalidPeripheralName> {
    match nombre_gpio {
        "gpio_a" => Ok("0"),
        "gpio_b" => Ok("1"),
        "gpio_c" => Ok("2"),
        "gpio_d" => Ok("3"),
        "gpio_e" => Ok("4"),
        "gpio_f" => Ok("5"),
        "gpio_g" => Ok("6"),
        "gpio_h" => Ok("7"),
        _ => Err(InvalidPeripheralName), // Retorna un error si el nombre no es válido
    }
}
