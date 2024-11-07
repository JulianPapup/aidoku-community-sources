# Compila el proyecto en modo nightly para WASM
cargo +nightly build --release

# Crea un directorio para almacenar los archivos empaquetados (Payload)
mkdir -p target/wasm32-unknown-unknown/release/Payload

# Copia archivos adicionales (si los tienes, como imágenes, configuraciones, etc.) a la carpeta Payload
cp res/* target/wasm32-unknown-unknown/release/Payload

# Copia el archivo .wasm generado a la carpeta Payload y lo renombra como main.wasm
cp target/wasm32-unknown-unknown/release/*.wasm target/wasm32-unknown-unknown/release/Payload/main.wasm

# Cambia al directorio donde están los archivos listos para empaquetar
cd target/wasm32-unknown-unknown/release

# Crea un archivo ZIP con el contenido de la carpeta Payload
zip -r package.aix Payload

# Mueve el archivo ZIP empaquetado a la ubicación final
mv package.aix ../../../package.aix
