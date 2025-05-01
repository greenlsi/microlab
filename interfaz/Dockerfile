# FROM node:18.19.1-alpine

# # Establecer el directorio de trabajo
# WORKDIR /app

# # Copiar todos los archivos del proyecto
# COPY . .

# # Instalar dependencias
# RUN npm install

# # Exponer el puerto en el que se ejecutará la aplicación
# EXPOSE 5173 

# # Asegurar que la variable de entorno se cargue
# ENV HOST=0.0.0.0

# # Comando por defecto para iniciar la aplicación
# CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]


# version mas optimizada: 


FROM node:18.19.1-alpine

# Directorio de trabajo para la app
WORKDIR /app

# Copiamos primero solo los archivos de dependencias (esto permite cachear `npm install`)
COPY package*.json ./

#  Instalamos solo las dependencias necesarias
RUN npm install

#  Copiamos el resto del código fuente
COPY . .

#  Configuramos que escuche en todas las interfaces
ENV HOST=0.0.0.0

#  Puerto donde se expone el servidor de desarrollo
EXPOSE 5173

# 🔧 Comando por defecto al iniciar el contenedor
CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]