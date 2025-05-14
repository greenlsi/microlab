import { useEffect, useState } from 'react';
import WebSocketComponent from './components/WebSocketComponent.tsx';
import './styles/App.css';
import Board from './components/Board.tsx';
import importedMapData from './assets/Map.json';
import initialFieldsData from './assets/Fields.json';
import SelectedPins from './components/selectedPins.tsx';
import PinSearcher from './components/PinSearcher.tsx';
import Layout from "./components/Layout";
import { Field, FieldsData, MapData, LedState  } from './types/fieldTypes';



function App() {
    const [isPressed, setIsPressed] = useState<boolean>(false);
    const [ledState, setLedState] = useState<Field>({ data: false, type: '' });
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
           //Hacemos un patch de data.fields en fieldsData.fields
           const result = {...fieldsData.fields, ...data.fields};

            setResultado(result);


            // if (data.fields["led2"]) {
            //     setLedState(data.fields["led2"].data || false);
            // }
        } catch (error) {
            console.error("Error procesando el mensaje del WebSocket:", error);
        }
    };

    const handleLedStateChange = (field: Field) => {
        console.log("Nuevo ledState:", field);
        setLedState(field);
      };


    return (
        <Layout>
                <div className="board-container">
                    <Board ledState={ledState} changeButtonState={changeButtonState} modifyFields={modifyFields} fieldsData={fieldsData}/>
                </div>
                {/* <Board ledState={ledState} changeButtonState={changeButtonState} modifyFields={modifyFields} fieldsData={fieldsData}/> */}

                <div className="resultado">
                    
                    <WebSocketComponent onMessage={handleWebSocketMessage} fieldsData={fieldsData} />
                    <PinSearcher modifyFields={modifyFields} />
                    {resultado && <SelectedPins fields={resultado} handleLedStateChange={handleLedStateChange} modifyFields={modifyFields}/>}
               
                </div>
        </Layout>
    );
}

export default App;

