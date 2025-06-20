use warp:: Filter;
use tracing:: error;
use std::sync::Arc;
use tokio::sync::Mutex;

use qtest::parser::Parser;
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

// Función set_irq_in para crear una interrupción 
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

// Ruta para obtener información del Timer
pub fn get_timer_info(
    parser: Arc<Mutex<Parser<SocketTcp>>>,
    periferico: Peripheral,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path!("timer_info" / String / usize)
        .and(warp::any().map(move || Arc::clone(&parser)))
        .and(warp::any().map(move || periferico.clone()))
        .and_then(
            |timer_name: String,
             selected_channel: usize,
             parser: Arc<Mutex<Parser<SocketTcp>>>,
             periferico: Peripheral| async move {
                let mut p = parser.lock().await;

                if let Some(timer) = periferico.get_timer(&timer_name) {
                    // Leemos diagnosis completa
                    let psc_result = timer.psc().read_register(&mut p).await;
                    let arr_result = timer.arr().read_register(&mut p).await;
                    let diagnosis_result = timer.full_channel_diagnosis(&mut p).await;

                    match (psc_result, arr_result, diagnosis_result) {
                        (Ok(psc), Ok(arr), Ok(diagnosis)) => {
                            // Construimos respuesta JSON limpia
                            let response = json!({
                                "timer": timer_name,
                                "channel": selected_channel,
                                "prescaler": psc,
                                "auto_reload": arr,
                                "channels": diagnosis.into_iter().map(|ch| {
                                    json!({
                                        "channel": ch.channel,
                                        "enabled": ch.enabled,
                                        "mode": ch.mode,
                                        "polarity": ch.polarity,
                                        "duty_cycle": ch.duty_cycle,
                                        "frequency": ch.frequency
                                    })
                                }).collect::<Vec<_>>()
                            });

                            Ok(warp::reply::json(&response))
                        }
                        _ => {
                            error!("Error al leer información del Timer {:?}", timer_name);
                            Err(warp::reject::custom(CustomError))
                        }
                    }
                } else {
                    error!("Timer no encontrado: {:?}", timer_name);
                    Err(warp::reject::custom(CustomError))
                }
            },
        )
}
