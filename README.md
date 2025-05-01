# microlab
MICROLAB: a virtual lab for learning about microcontrollers



sudo chown -R inesvj:inesvj /home/inesvj/TFG/microlab/interfaz


desde TFG: ./qemu_new/build/qemu-system-arm     -cpu cortex-m4     -machine netduinoplus2     -nographic     -semihosting-config enable=on,target=native     -monitor stdio     -serial null     -qtest tcp:localhost:3000     -kernel ./prueba/test_v1.elf

---
---

# 🐳 Optimización de Dockerfiles para `rust-server` y `interfaz-grafica`

Este repositorio contiene dos servicios dockerizados: un servidor en Rust y una interfaz gráfica en Node.js. Ambos Dockerfiles han sido optimizados para reducir el tamaño de imagen, mejorar los tiempos de construcción y seguir buenas prácticas de despliegue.

---

## 🦀 `rust-server`: Multi-stage Build

### Antes:
- Imagen extremadamente pesada (**1.32 GB**)
- Contenía el compilador de Rust, dependencias de desarrollo y el código fuente
- Riesgos de seguridad y mayor superficie de ataque

### Ahora:
- **Multi-stage**: se compila en una imagen con Rust, pero se ejecuta en una imagen base mínima (`alpine`)
- Solo se copia el **binario final**
- Se usan capas de cache con `cargo fetch`
- Se utiliza `strip` para reducir aún más el tamaño

### Resultado:
✅ Imagen final ligera, segura y rápida  
✅ Ideal para producción o despliegue en CI/CD  
✅ **Tamaño final: 13.8 MB**  
🧨 **Ahorro total: ~99%** respecto a la imagen original de 1.32 GB

---

## 🌐 `interfaz-grafica` (Node + Vite)

### Antes:
- Imagen con dependencias copiadas desde fuera, no reproducible
- Tamaño de imagen: **232 MB**
- `COPY . .` antes de `npm install`: invalida el cache al más mínimo cambio

### Ahora:
- Primero se copian solo `package.json` y `package-lock.json`
- Luego se hace `npm install` y **se cachea la capa**
- Finalmente se copia el resto del código fuente
- Se construye todo desde cero, lo que garantiza reproducibilidad total

### Resultado:
✅ Builds más rápidos  
✅ Imagen más estable para desarrollo  
✅ Mejores prácticas reproducibles  
✅ **Tamaño final: 249 MB**  
📈 **+17 MB respecto a la anterior**, pero con una imagen más segura y coherente

---

## 📦 `.dockerignore` recomendado

Asegúrate de tener un archivo `.dockerignore` para evitar subir archivos innecesarios a las imágenes:

```dockerignore
.git
target/
node_modules/
*.log
.env

--

## 🔧 Recomendaciones extra

- Si haces `git clone` dentro de Docker (no ocurre aquí), usa `--depth=1` para evitar historial innecesario.