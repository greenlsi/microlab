import React, { useEffect } from "react";
import useWebSocket from "react-use-websocket";

interface WebSocketComponentProps {
    onMessage?: (message: any) => void;
    fieldsData: Record<string, any>;
}

const WebSocketComponent: React.FC<WebSocketComponentProps> = ({ onMessage, fieldsData }) => {
    //const { sendMessage, readyState } = useWebSocket(process.env.REACT_APP_WS_URL || "", {
    
    const { sendMessage, readyState } = useWebSocket(import.meta.env.VITE_APP_WS_URL || "ws://127.0.0.1:8081", {
    //const { sendMessage, readyState } = useWebSocket(import.meta.env.VITE_APP_WS_URL || "", {

    //const { sendMessage, readyState } = useWebSocket("ws://127.0.0.1:8081", {
        onOpen: () => {
            console.log("Conexión WebSocket abierta");
            sendFieldsMessage(); // Enviar mensaje inicial
        },
        onMessage: (event) => {
            console.log("Mensaje recibido:", event.data);
            try {
                const parsedMessage = JSON.parse(event.data);
                if (onMessage) {
                    onMessage(parsedMessage);
                }
            } catch (error) {
                console.error("Error al analizar el mensaje WebSocket:", error);
            }
        },
        onError: (event) => {
            console.error("Error en la conexión WebSocket:", event);
        },
        shouldReconnect: () => true,
    });

    // Enviar mensaje 
    const sendFieldsMessage = () => {
        const jsonString = JSON.stringify(fieldsData);
        console.log("Enviando fields.json al servidor:", jsonString);
        sendMessage(jsonString);
    };

    //Enviar mensaje cada vez que fieldsData cambie
    useEffect(() => {
        if (readyState === 1) {
            sendFieldsMessage();
        }
    }, [fieldsData, readyState]);
    

    const connectionStatus: string = {
        0: "Conectando",
        1: "Abierto",
        2: "Cerrando",
        3: "Cerrado",
    }[readyState as 0 | 1 | 2 | 3] || "Desconocido";

    // Determinar la clase CSS para el spinner según el estado
    // Determinar la clase CSS para la línea según el estado
    const lineClass: string = {
        0: "line connecting",
        1: "line open",
        2: "line closing",
        3: "line closed",
    }[readyState as 0 | 1 | 2 | 3] || "Desconocido";

    return (
        <div>
            <div className={lineClass}></div>
            <p>Estado de la conexión: {connectionStatus}</p>
        </div>
    );
};

export default WebSocketComponent;