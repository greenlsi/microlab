import { useEffect, useState } from 'react';
import WebSocketComponent from './WebSocketComponent.tsx';
import './App.css';
import Board from './board.tsx';
import importedMapData from './assets/Map2.json';
import initialFieldsData from './assets/Fields.json';
import SelectedPins from './selectedPins.tsx';
//import 'bootstrap/dist/css/bootstrap.min.css';

// interface Field {
//     peripheral: string;
//     pin: string;
//     data?: boolean;
//     [key: string]: any; // Permite atributos adicionales
// }

interface Field {
    type: "gpio" | "power" | "control" | "NC" | "analog" | "digital"| string;
    port?: string; // Indica el puerto como "gpio_a", "gpio_b", etc.
    pin?: number | null; // Puede ser un número o null en caso de no aplicar
    value?: number | null; // Para los pines de power
    name?: string; // Para identificar nombres como "VDD", "RESET", etc.
    data?: boolean | number; // Opcional, si es necesario para alguna funcionalidad
    [key: string]: any; // Permite atributos adicionales
}

interface FieldsData {
    fields: Record<string, Field>;
}

interface MapData {
    [key: string]: Field;
}


function App() {
    const [isPressed, setIsPressed] = useState<boolean>(false);
    const [ledState, setLedState] = useState<boolean>(false);
    const [fieldsData, setFieldsData] = useState<FieldsData>(initialFieldsData);
    const [resultado, setResultado] = useState<typeof fieldsData.fields>(initialFieldsData.fields);
    const [mapData] = useState<MapData>(importedMapData);

    const changeButtonState = () => {
        setIsPressed((prevValue) => !prevValue);
    };

    const modifyFields = (id: string) => {
        if (!mapData[id]) {
            console.error(`ID ${id} no encontrado en mapData.`);
            console.log("fieldsData:", fieldsData);
            console.log("mapData:", mapData);
            return fieldsData;
        }

        const newFieldsData = { ...fieldsData };
        
        // Si no hay 'fields', lo inicializamos
        if (!newFieldsData.fields) {
            newFieldsData.fields = {};
        }

        if (!newFieldsData.fields[id]) {
            newFieldsData.fields[id] = { ...mapData[id] };
            console.log(`🟢 ID ${id} agregado a fieldsData:`, newFieldsData);
        } else {
            delete newFieldsData.fields[id];
            console.log(`🔴 ID ${id} eliminado de fieldsData:`, newFieldsData);
        }

        setFieldsData(newFieldsData);
    };

    useEffect(() => {
        const fetchData = async () => {
            try {
                console.log("API URL:", import.meta.env.VITE_APP_API_URL);
                const response = await fetch(`${import.meta.env.VITE_APP_API_URL}/gpio_c/pulsar_boton/13/${isPressed ? 0 : 1}`, {
                //const response = await fetch(`http://127.0.0.1:8080/gpio_c/pulsar_boton/13/${isPressed ? 0 : 1}`, {
                  
                //const response = await fetch(`http://localhost:8080/gpio_c/pulsar_boton/13/${isPressed ? 0 : 1}`, {
                    method: 'GET',
                });

                if (!response.ok) throw new Error('Failed to set pin state');
                const data = await response.json();
                console.log('Data tras pulsar botón:', data);
            } catch (error) {
                console.error('Error:', error);
            }
        };
        fetchData();
    }, [isPressed]);

    const handleWebSocketMessage = (webSocketMessage: string | object) => {
        try {
            const data: { fields: Field } = typeof webSocketMessage === "string" ? JSON.parse(webSocketMessage) : webSocketMessage;
            setResultado(data.fields);

            if (data.fields["led2"]) {
                setLedState(data.fields["led2"].data || false);
            }
        } catch (error) {
            console.error("Error procesando el mensaje del WebSocket:", error);
        }
    };

    return (
        <div className="container">
            <div className="board-container">
                <Board ledState={ledState} changeButtonState={changeButtonState} modifyFields={modifyFields} fieldsData={fieldsData}/>
            </div>
            <div className="resultado">
                <div className="websocket-container">
                    <div className="websocket-view">
                        <WebSocketComponent onMessage={handleWebSocketMessage} fieldsData={fieldsData} />
                    </div>
                </div>
                {resultado && <SelectedPins pins={resultado} />}
            </div>
        </div>
    );
}

export default App;

