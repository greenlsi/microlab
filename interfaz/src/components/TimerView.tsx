import { useState } from "react";
import "../styles/TimerView.css";
import { TimerProps } from "../types/fieldTypes";

const TimerView: React.FC<TimerProps> = ({ timerInfo }) => {
    const [showAllChannels, setShowAllChannels] = useState(false);

    if (!timerInfo) {
        return <div className="timer-container"><p>Este pin no tiene un Timer asignado.</p></div>;
    }

    const mainChannel = timerInfo.channels.find((ch: any) => ch.channel === timerInfo.channel);

    return (
        <div className="timer-container">
            <h2>{timerInfo.timer}</h2>
            <p>Canal seleccionado: {timerInfo.channel}</p>
            <p>ARR (Auto-Reload Register): {timerInfo.auto_reload}</p>
            <p>PSC (Prescaler): {timerInfo.prescaler}</p>

            {mainChannel ? (
                <div className="main-channel-info">
                    <h3>Información del Canal {mainChannel.channel}</h3>
                    <p>Estado: {mainChannel.enabled ? "Habilitado" : "Deshabilitado"}</p>
                    <p>Modo: {mainChannel.mode}</p>
                    <p>Polaridad: {mainChannel.polarity}</p>
                    {mainChannel.duty_cycle !== null && (
                        <p>Duty Cycle: {mainChannel.duty_cycle}%</p>
                    )}
                    {mainChannel.frequency !== null && (
                        <p>Frecuencia: {mainChannel.frequency} Hz</p>
                    )}
                </div>
            ) : (
                <p>No se encontró información para el canal seleccionado.</p>
            )}

            {/* 🔥 Botón para desplegar el resto de canales */}
            <button 
                className="show-channels-button" 
                onClick={() => setShowAllChannels(!showAllChannels)}
            >
                {showAllChannels ? "Ocultar otros canales" : "Mostrar otros canales"}
            </button>

            {/* 🔥 Resto de canales */}
            {showAllChannels && (
                <div className="other-channels-info">
                    <h3>Otros Canales:</h3>
                    {timerInfo.channels
                        .filter((ch: any) => ch.channel !== timerInfo.channel)
                        .map((ch: any) => (
                            <div key={ch.channel} className="channel-info">
                                <h4>Canal {ch.channel}</h4>
                                <p>Estado: {ch.enabled ? "Habilitado" : "Deshabilitado"}</p>
                                <p>Modo: {ch.mode}</p>
                                <p>Polaridad: {ch.polarity}</p>
                                {ch.duty_cycle !== null && (
                                    <p>Duty Cycle: {ch.duty_cycle}%</p>
                                )}
                                {ch.frequency !== null && (
                                    <p>Frecuencia: {ch.frequency} Hz</p>
                                )}
                            </div>
                        ))}
                </div>
            )}
        </div>
    );
};

export default TimerView;