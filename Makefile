# Makefile para DCM Tool Kit: tareas útiles para desarrollo, pruebas y despliegue.
#
# Uso principal:
#   make build     # Compila en modo release (binario optimizado)
#   make run       # Ejecuta el binario optimizado
#   make test      # Ejecuta todos los tests
#   make clean     # Limpia archivos de compilación
#   make install   # Instala el binario dcmtk en /usr/local/bin (requiere permisos)

# Definición de macros
BIN_NAME := dcmtk
RELEASE_TARGET := target/release/$(BIN_NAME)
INSTALL_PATH := /usr/local/bin/$(BIN_NAME)

.PHONY: all build run clean test install help

# Por defecto, muestra ayuda
all: help

## Compilar en modo release
build:
	cargo build --release

## Ejecutar el binario recién compilado
run: build
	$(RELEASE_TARGET)

## Ejecutar tests unitarios y de integración
test:
	cargo test

## Limpiar archivos de compilación
clean:
	cargo clean

## Instalar el binario en /usr/local/bin (puede requerir sudo)
install: build
	cp $(RELEASE_TARGET) $(INSTALL_PATH)
	@echo "Binario instalado en $(INSTALL_PATH)"

## Mostrar ayuda con descripción de comandos
help:
	@echo "Comandos disponibles:"
	@echo "  make build      Compila el binario en modo release"
	@echo "  make run        Ejecuta el binario ya compilado (modo release)"
	@echo "  make test       Ejecuta la suite de pruebas"
	@echo "  make clean      Elimina archivos de compilación"
	@echo "  make install    Instala '$(BIN_NAME)' en /usr/local/bin"
	@echo "  make help       Muestra esta ayuda"
