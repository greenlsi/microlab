import React, { useState } from 'react';
import MapData from '../assets/Map.json';
import  "../styles/PinSearcher.css"; 

interface PinSearcherProps {
  modifyFields: (id: string) => void;
}

const PinSearcher: React.FC<PinSearcherProps> = ({ modifyFields }) => {
  const [searchTerm, setSearchTerm] = useState("");

  const normalizeSearch = (term: string) => {
    let normalized = term.toUpperCase().replace(/\s+/g, "");
    normalized = normalized.replace("GPIO", "P");
    normalized = normalized.replace("PIN", "");
    return normalized;
  };

  const handleSearch = () => {
    if (!searchTerm) return;

    const normalized = normalizeSearch(searchTerm);

    // 1. Buscar por formato tipo PA7
    const match = normalized.match(/^P([A-Z])(\d+)$/);

    if (match) {
      const portLetter = match[1];
      const pinNumber = parseInt(match[2], 10);

      const foundId = Object.entries(MapData).find(([key, value]: [string, any]) => {
        return (
          value.type === "gpio" &&
          value.port?.toLowerCase() === `gpio_${portLetter.toLowerCase()}` &&
          value.pin === pinNumber
        );
      })?.[0];

      if (foundId) {
        modifyFields(foundId);
        setSearchTerm(""); // Borra el input al seleccionar
        return;
      }
    }

    // 2. Buscar por alias (RESET, VDD...)
    const foundAliasId = Object.entries(MapData).find(([key, value]: [string, any]) => {
      return value.alias?.toUpperCase() === normalized;
    })?.[0];

    if (foundAliasId) {
      modifyFields(foundAliasId);
      setSearchTerm(""); //Borra el input al seleccionar
      return;
    }

    // 3. No encontrado
    alert("No se encontró ningún pin que coincida.");
  };

  return (
    <div className="search-container">
      <input
        type="text"
        value={searchTerm}
        onChange={(e) => setSearchTerm(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            handleSearch();
          }
        }}
        placeholder="Search pin (e.g., PA7, RESET, VDD...)"
      />
      <button onClick={handleSearch}>Search</button>
    </div>
  );
};

export default PinSearcher;
