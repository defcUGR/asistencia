# Asistencia DEFC

> Aplicación para pasar la asistencia en las sesiones de la DEFC en base a el censo

## Funcionalidades

- Soporte para prácticamente cualquier escáner por USB
- Alternativa de entrada por teclado
- Lectura automática de TUIs
- Creación de CSV para encuestas de LimeSurvey
- Si la TUI no se reconoce, selecciona a una persona a la que asignarla

## Uso

1. Descarga el CSV con los datos completos y en el formato especificado
2. En caso de tener un lector/escáner externo, conéctalo al ordenador y asegúrate de que esté en serial con una baud rate preferiblemente de 115.200
3. Selecciona el archivo CSV y el lector/escáner que hayas conectado (o teclado en caso de no disponer de lector/escáner)
4. Haz click en _Empezar_ y comienza a registrar TUIs
5. Exporta el registro a CSV o al formato de LimeSurvey

## Instalación

Las diferentes versiones de la app están disponibles en las [Releases](https://github.com/defcUGR/asistencia/releases) del repositorio de GitHub. Actualmente está disponible para Linux y Windows.

## Desarrollo

Antes de empezar, asegúrate de tener [Rust](https://rust-lang.org) y [NodeJS](https://nodejs.org) así como seguir la [guía de preparación del entorno para Tauri](https://tauri.app/v1/guides/getting-started/prerequisites). Antes de continuar, asegúrate de haber instalado los packetes de node con `npm`, `yarn`, `pnpm` o el gestor que utilices.

Para poder compilar la aplicación, debido al uso de varios archivos generados automáticamente, tienes que ejecutar los siguientes comandos:

```sh
cd src-tauri && cargo test # Genera los typos de TypeScript desde las structs de Rust
cd .. && npm dev # Ctrl+C en cuanto cargue, se asegura así que se generan las definiciones de los elementos auto-importados
```

Para ejecutarla en modo de desarrollo (con HMR y sin optimizar), ejecuta el script `dev` del paquete:

```sh
npm run dev
```

Para compilar la aplicación y crear el ejecutable así como los diferentes paquetes de instalación, ejecuta el script `build` del paquete:

```sh
npm run dev
```
