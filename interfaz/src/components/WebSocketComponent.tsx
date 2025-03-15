import React, { useEffect } from "react";
import useWebSocket from "react-use-websocket";
import "../styles/Websocket.css";
import { WebSocketComponentProps } from "../types/fieldTypes";
import merge from "lodash.merge";
import { useState } from "react";
import { FieldsData } from "../types/fieldTypes";



//LO QUE HACE EL COMPONENTE ES ENVIAR UN MENSAJE AL SERVIDOR CADA VEZ QUE SE ACTUALIZA EL ESTADO DE LOS CAMPOS. 
//EL MENSAJE QUE ENVÍA SOLO CONTIENE LOS CAMPOS DE TIPO "GPIO" PARA QUE EL SERVIDOR LOS PROCESE.
//CUANDO RECIBE UN MENSAJE DEL SERVIDOR, LLAMA A LA FUNCIÓN onMessage CON LOS DATOS ACTUALIZADOS.
//  onMessage ES UNA FUNCIÓN QUE SE PASA COMO PROPS AL COMPONENTE WebSocketComponent Y SE EJECUTA EN EL COMPONENTE PADRE (App.tsx)
//  PARA ACTUALIZAR LOS DATOS DE LOS CAMPOS E INTEGRARLOS CON EL RESTO QUE NO SON DE TIPO "GPIO" Y RENDERIZARLOS EN LA VISTA SelectedPins.tsx


const WebSocketComponent: React.FC<WebSocketComponentProps> = ({ onMessage, fieldsData }) => {
    


    const { sendMessage, readyState } = useWebSocket(import.meta.env.VITE_APP_WS_URL || "ws://127.0.0.1:8081", {
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
        if (!fieldsData || !fieldsData.fields) {
            console.error("Error: fieldsData no contiene la estructura esperada.");
            return;
        }
    
        //  Filtramos solo los elementos con type "gpio" dentro de "fields"
        const filteredFields = Object.fromEntries(
            Object.entries(fieldsData.fields)
                .filter(([_, field]) => (field as any).type === "gpio") //  Solo los de tipo "gpio"
                .map(([key, field]) => {
                    const { type, pin, port } = field as any;
                    return [key, { type, pin, port }];
                }) //  Mantiene estructura
        );
    
        // Creamos el objeto final con la misma estructura
        const finalJson = { fields: filteredFields };
    
        const jsonString = JSON.stringify(finalJson);
        console.log("Enviando fields.json al servidor:", jsonString);
        sendMessage(jsonString);
    };
    

    //Enviar mensaje cada vez que fieldsData cambie
    useEffect(() => {

        if (readyState === 1) {
            sendFieldsMessage();
        }
    }, [fieldsData, readyState]);
    

    // Determinar la clase CSS para la línea según el estado
    const lineClass: string = {
        0: "line connecting",
        1: "line open",
        2: "line closing",
        3: "line closed",
    }[readyState as 0 | 1 | 2 | 3] || "unknown";

    return (
        <div className="websocket-container">
            <div className={lineClass}></div>
        </div>
    );
};

export default WebSocketComponent;


//tengo aqui un problema con el tipo de fieldsData, que no se puede asignar a newFieldsData, por lo que he tenido que hacer un cast a FieldsData | null
//para que no me de error en la asignación de setNewFieldsData(fieldsData) en el useEffect.
//NO se si es al hacer el merge o que. Pero necesito que newFieldsData se actualice con los datos de fieldsData para que se envíen al servidor y con lo que le llega tmb....?????