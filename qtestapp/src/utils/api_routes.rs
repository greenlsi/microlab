use warp:: Filter;
use tracing:: error;
use std::sync::Arc;
use tokio::sync::Mutex;

use qtest::session::Session;
use qtest::socket::tcp::SocketTcp;

use qtest_stm32f4nucleo::Peripheral;
// Define errores personalizados
use crate::utils::error::CustomError;
use crate::utils::fields::peripheral_to_number;
use serde_json::json;
use warp::Rejection;
use warp::Reply;

// Define una ruta REST con Warp
 // Filtro pulsar_boton que establece la ruta y recibe los parámetros
 // TO DO: Aquí debería quitar lo de despues de create_interruption en un futuro
 pub fn pulsar_boton(
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

// Función set_irq_in que manipula el estado de los pines GPIO
pub async fn create_interruption(
    nombre_gpio: String,
    pin: usize,
    value: isize,
    parser: &mut Parser<SocketTcp>,
) -> Result<(), warp::Rejection> {
    let gpio=peripheral_to_number(nombre_gpio.as_str())?;
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

pub fn get_timer_info(
    parser: Arc<Mutex<Parser<SocketTcp>>>,
    periferico: Peripheral,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("timer_info" / String / usize) // 🔥 Se corrige la estructura de la ruta
        .and(warp::any().map(move || Arc::clone(&parser)))
        .and(warp::any().map(move || periferico.clone()))
        .and_then(
            |timer_name: String,
             channel: usize,
             parser: Arc<Mutex<Parser<SocketTcp>>>,
             periferico: Peripheral| async move {
                let mut p = parser.lock().await;

                // 🔹 Obtener el Timer
                if let Some(timer) = periferico.get_timer(&timer_name) {
                    let psc_result = timer.psc().get_prescaler(&mut p).await;
                    let arr_result = timer.arr().get_auto_reload(&mut p).await;

                    match (psc_result, arr_result) {
                        (Ok(psc), Ok(arr)) => {
                            let response = json!({
                                "timer": timer_name, // 🔥 Ahora `timer_name` es un `String`
                                "channel": channel,
                                "ARR": arr,
                                "PSC": psc
                            });
                            Ok(warp::reply::json(&response))
                        }
                        _ => {
                            error!("Error al leer los valores del Timer {:?}", timer_name); // 🔥 Usa {:?} para Debug
                            Err(warp::reject::custom(CustomError))
                        }
                    }
                } else {
                    // 🔹 Si el Timer no existe
                    error!("Timer no encontrado: {:?}", timer_name);
                    Err(warp::reject::custom(CustomError))
                }
            },
        )
}