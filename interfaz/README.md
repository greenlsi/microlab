
Map.json: 
tipos y sus atributos correspondientes

| Type    | Attributes    |
|---------|-------------|
| gpio    | port, pin   |
| power   | value, alias |
| control | alias        |
| NC      | N/A         |

En los arduino, se añade el atributo alias tambien
	"CN9_8": {
		"type": "gpio",
		"port": "gpio_a",
		"pin": 8
        "alias": "D7",
	},

El boton de usuario es CN7_23 (a la altura de VIN)