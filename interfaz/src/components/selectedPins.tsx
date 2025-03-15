import React from "react";
import "../styles/SelectedPins.css"; // Archivo CSS para estilos
import alternateTable from "../assets/alternate_function2.json"; // Archivo JSON con datos de alternate function
import { AlternateTable, FieldsData, Field } from "../types/fieldTypes"; // Importar tipos de datos
import { fetchTimerInfo } from "../hooks/useApi"; // Importar función para obtener información de temporizadores
import { useEffect, useState } from "react";
import ToggleDescription from "./ToggleDescription";

const SelectedPins: React.FC<FieldsData> = ({ fields }) => {

  //lógica para preparar toda la información fields y mostrarla en la interfaz:

  const [processedFields, setProcessedFields] = useState<Field[]>([]);

  useEffect(() => {
    // 🔥 Procesar los datos cada vez que `fields` cambie
    const processFields = async () => {
      const newFields: Field[] = await Promise.all(
        Object.keys(fields).map(async (fieldKey) => {
          const field = fields[fieldKey];

          // 🔹 Obtener `alternateFunction` si el modo es "Alternate Function"
          const alternateFunctionKey =
            field.mode === "Alternate Function" && typeof field.data === "number"
              ? `AF${field.data}`
              : undefined;

          const alternateFunction =
            alternateFunctionKey && field.port && field.pin !== undefined
              ? (alternateTable as AlternateTable)[field.port]?.[field.pin as number]?.[alternateFunctionKey]
              : undefined;

          // 🔹 Si `alternateFunction` es un Timer, obtener más información de la API
          let timerInfo = null;
          if (alternateFunction && alternateFunction.startsWith("TIM")) {


            const match = alternateFunction.match(/TIM(\d+)_CH(\d+)/);

            if (match) {
              const timer = `Timer${match[1]}`;  // Convierte "TIM1" a "Timer1"
              const channel = match[2]; //  Extrae solo el número, sin "CH"

              // Llamada a la API para obtener más detalles del Timer
              timerInfo = await fetchTimerInfo(timer, channel);
            }
          }

          return {
            key: fieldKey,
            ...field,
            alternateFunction,
            timerInfo, // Se agrega la info del Timer si está disponible
          };
        })
      );
      console.log("newFields:", newFields);
      setProcessedFields(newFields);
    };

    processFields();
  }, [fields]);


  return (
    <div className="selected-pins-container">
      <h3 className="title">PINS SELECCIONADOS</h3>

      {/* 🔹 Renderizado final sin cálculos */}
      <div className="field-grid">
        {processedFields.map((field, index) => (
          <div key={index} className="field-card">
            <div className="field-body">
              <div className="pins-list">
                <div className="pin-item">
                  <h3>{field.key.toUpperCase()} {field.alias ? `(${field.alias})` : ""}</h3>

                  {/* 🔹 Mostrar los datos procesados */}
                  {field.port && <><strong>Port:</strong> {field.port} <br /></>}
                  {field.pin !== null && field.pin !== undefined && (
                    <><strong>Pin:</strong> {field.pin} <br /></>
                  )}
                  {field.mode && <><strong>Mode:</strong> {field.mode} <br /></>}

                  {field.alternateFunction && (
                    <><strong>Alternate Function:</strong> {field.alternateFunction} <br /></>
                  )}

                  {field.timerInfo && (
                    <div>
                      <strong>Timer Info:</strong>
                      <p>Timer: {field.timerInfo.timer}</p>
                      <p>Channel: {field.timerInfo.channel}</p>
                      <p>ARR: {field.timerInfo.ARR}</p>
                      <p>PSC: {field.timerInfo.PSC}</p>
                    </div>
                  )}

                  {field.data !== undefined && !field.alternateFunction && (
                    <div>
                      <strong>Estado:</strong>
                      <span className={`status ${field.data ? "high" : "low"}`}>
                        {field.data ? "Alto" : "Bajo"}
                      </span>
                    </div>
                  )}
                  {field.type === "NC" && (
                    <strong>No Connection</strong>
                  )}

                  {field.description && <ToggleDescription description={field.description} />}

                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};




export default SelectedPins;
