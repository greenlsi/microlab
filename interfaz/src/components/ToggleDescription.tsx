import { useState } from "react";
import "../styles/ToggleDescription.css";
import { ToggleDescriptionProps } from "../types/fieldTypes";

const ToggleDescription: React.FC<ToggleDescriptionProps> = ({ description }) => {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="toggle-container">
      {/* 🔥 Cualquier parte del texto será clickeable */}
      <p 
        className="toggle-text" 
        onClick={() => setIsOpen(!isOpen)}
      >
        {isOpen ? " Ocultar descripción" : "Ver descripción"}
      </p>

      {isOpen && (
        <div className="description">
          <p>{description}</p>
        </div>
      )}
    </div>
  );
};

export default ToggleDescription;
