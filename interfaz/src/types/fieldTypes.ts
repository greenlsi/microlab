export interface Field {
    type: "gpio" | "power" | "control" | "NC" | string;
    port?: string; // Indica el puerto como "gpio_a", "gpio_b", etc.
    pin?: number | null; // Puede ser un número o null en caso de no aplicar
    value?: number | null; // Para los pines de power
    name?: string; // Para identificar nombres como "VDD", "RESET", etc.
    data?: boolean | number; // Opcional, si es necesario para alguna funcionalidad
    [key: string]: any; // Permite atributos adicionales
}

export interface FieldsData {
    fields: Record<string, Field>;
}

export interface MapData {
    [key: string]: Field;
}
export interface AlternateFunction {
    [key: string]: string | undefined; // Estructura de funciones alternativas por clave AF
}
export interface PinMapping {
    [pin: number]: AlternateFunction;
}

export interface AlternateTable {
    [port: string]: PinMapping;
}



export interface BoardProps {
    ledState: Field;
    changeButtonState: () => void;
    modifyFields: (id: string) => void;
    fieldsData: FieldsData;
}





export interface ToggleDescriptionProps {
    description: string;
  }


export interface WebSocketComponentProps {
    onMessage?: (message: any) => void;
    fieldsData: Record<string, any>;
}


export interface TimerInfo {
    timer: string;
    channel: number;
    prescaler: number;
    auto_reload: number;
    channels: {
        channel: number;
        enabled: boolean;
        mode: string;
        polarity: string;
        duty_cycle: number | null;
        frequency: number | null;
    }[];
    [key: string]: any; // Permite atributos adicionales
}

export interface TimerProps {
    timerInfo: TimerInfo | null;
}

export interface LedState{
    led: boolean;
    dc: number;
}