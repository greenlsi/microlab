import React from "react";
import "../styles/SelectedPins.css"; // Archivo CSS para estilos
import alternateTable from "../assets/alternate_function2.json"; // Archivo JSON con datos de alternate function

interface Field {
    type: "gpio" | "power" | "control" | "NC" | string;
    port?: string; // Indica el puerto como "gpio_a", "gpio_b", etc.
    pin?: number | null; // Puede ser un número o null en caso de no aplicar
    value?: number | null; // Para los pines de power
    name?: string; // Para identificar nombres como "VDD", "RESET", etc.
    data?: boolean | number; // Opcional, si es necesario para alguna funcionalidad
    [key: string]: any; // Permite atributos adicionales
}

interface AlternateFunction {
    [key: string]: string | undefined; // Estructura de funciones alternativas por clave AF
}
interface PinMapping {
    [pin: number]: AlternateFunction;
}

interface AlternateTable {
    [port: string]: PinMapping;
}


interface SelectedPinsProps {
    pins: Record<string, Field>;
}



const SelectedPins: React.FC<SelectedPinsProps> = ({ pins }) => {
    // return (
    //     <div className="selected-pins-container">
    //         <h3 className="title">PINS SELECCIONADOS</h3>
    //         <div className="fields-grid-overflow-y-auto">
    //             {Object.keys(pins).map((fieldKey, fieldIndex) => {
    //                 const field = pins[fieldKey]; // Acceder al campo específico


    //                 // Obtener la función alternativa si el modo es "Alternate Function" y data es un número
    //                 const alternateFunctionKey = field.mode === "Alternate Function" && typeof field.data === "number"
    //                     ? `AF${field.data}`
    //                     : undefined;

    //                 const alternateFunction = alternateFunctionKey && field.port && field.pin !== undefined
    //                     ? (alternateTable as AlternateTable)[field.port]?.[field.pin as number]?.[alternateFunctionKey]
    //                     : undefined;


    //                 return (
    //                     <div key={fieldIndex} className="field-card">
    //                         <div className="field-body">
    //                             <div className="pins-list">
    //                                 <div className="pin-item">
    //                                     <h5>
    //                                         {fieldKey.toUpperCase()} {field.alias ? `(${field.alias})` : ""}
    //                                     </h5>

    //                                     {/* Renderizar según el tipo de pin */}
    //                                     {field.type === "gpio" && (
    //                                         <>
    //                                             <strong>Port:</strong> {field.port} <br />
    //                                             <strong>Pin:</strong> {field.pin} <br />
    //                                             {field.alias && (
    //                                                 <>
    //                                                     <strong>Alias:</strong> {field.alias} <br />
    //                                                 </>
    //                                             )}
    //                                             {field.mode && <strong>Mode:</strong>}{field.mode} <br />
    //                                             {field.data !== undefined && (
    //                                                 alternateFunction ? (
    //                                                     <div>
    //                                                         <strong>Alternate Function:</strong> {alternateFunction} <br />
    //                                                     </div>
    //                                                 ) : (
    //                                                     <div>
    //                                                         <strong>Estado:</strong>
    //                                                         <span className={`status ${field.data ? "high" : "low"}`}>
    //                                                             {field.data ? "Alto" : "Bajo"}
    //                                                         </span>
    //                                                     </div>
    //                                                 )
    //                                             )}
    //                                         </>
    //                                     )}

    //                                     {field.type === "power" && (
    //                                         <>
    //                                             <strong>Name:</strong> {field.alias} <br />
    //                                         </>
    //                                     )}

    //                                     {field.type === "control" && (
    //                                         <>
    //                                             <strong>Name:</strong> {field.alias} <br />
    //                                         </>
    //                                     )}

    //                                     {field.type === "NC" && (
    //                                         <strong>No Connection</strong>
    //                                     )}
    //                                 </div>
    //                             </div>
    //                         </div>
    //                     </div>
    //                 );
    //             })}
    //         </div>
    //     </div>
    // );
    return (
  <div className="selected-pins-container">
    <h3 className="title">PINS SELECCIONADOS</h3>
        
    {/* 🔹 Contenedor con Grid para distribuir las tarjetas */}
    <div className="field-grid">
      {Object.keys(pins).map((fieldKey, fieldIndex) => {
        const field = pins[fieldKey];
                            // Obtener la función alternativa si el modo es "Alternate Function" y data es un número
                    const alternateFunctionKey = field.mode === "Alternate Function" && typeof field.data === "number"
                        ? `AF${field.data}`
                        : undefined;

                    const alternateFunction = alternateFunctionKey && field.port && field.pin !== undefined
                        ? (alternateTable as AlternateTable)[field.port]?.[field.pin as number]?.[alternateFunctionKey]
                        : undefined;

        return (
          <div key={fieldIndex} className="field-card">
            <div className="field-body">
              <div className="pins-list">
                <div className="pin-item">
                  <h3>
                    {fieldKey.toUpperCase()} {field.alias ? `(${field.alias})` : ""}
                  </h3>

                  {/* 🔹 Renderizar según el tipo de pin */}
                  {field.type === "gpio" && (
                    <>
                      <strong>Port:</strong> {field.port} <br />
                      <strong>Pin:</strong> {field.pin} <br />
                      {field.alias && (
                        <>
                          <strong>Alias:</strong> {field.alias} <br />
                        </>
                      )}
                      {field.mode && <strong>Mode:</strong>}{field.mode} <br />
                      {field.data !== undefined && (
                        alternateFunction ? (
                          <div>
                            <strong>Alternate Function:</strong> {alternateFunction} <br />
                          </div>
                        ) : (
                          <div>
                            <strong>Estado:</strong>
                            <span className={`status ${field.data ? "high" : "low"}`}>
                              {field.data ? "Alto" : "Bajo"}
                            </span>
                          </div>
                        )
                      )}
                    </>
                  )}

                  {field.type === "power" && (
                    <>
                      <strong>Name:</strong> {field.alias} <br />
                    </>
                  )}

                  {field.type === "control" && (
                    <>
                      <strong>Name:</strong> {field.alias} <br />
                    </>
                  )}

                  {field.type === "NC" && <strong>No Connection</strong>}
                </div>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  </div>
);

};




export default SelectedPins;
