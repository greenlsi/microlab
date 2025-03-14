FROM node:18.19.1-alpine

# Establecer el directorio de trabajo
WORKDIR /app

# Copiar todos los archivos del proyecto
COPY . .

# Instalar dependencias
RUN npm install

# Exponer el puerto en el que se ejecutará la aplicación
EXPOSE 5173 

# Asegurar que la variable de entorno se cargue
ENV HOST=0.0.0.0

# Comando por defecto para iniciar la aplicación
CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]
