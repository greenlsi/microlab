import { TimerInfo } from "../types/fieldTypes";

export const fetchTimerInfo = async (timer: string, channel: string): Promise<TimerInfo | null> => {
    try {
      const response = await fetch(`${import.meta.env.VITE_APP_API_URL}/timer_info/${timer}/${channel}`);
      if (!response.ok) throw new Error("Error obteniendo datos del timer");
      return await response.json();
    } catch (error) {
      console.error("Error en la API:", error);
      return null;
    }
  };