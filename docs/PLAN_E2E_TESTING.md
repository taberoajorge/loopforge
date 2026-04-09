# LoopForge E2E Testing Plan v2

## Objetivo

Esta version v2 redefine el gate E2E inicial para el producto que existe hoy. El objetivo no es recuperar un contrato historico de pruebas ni cubrir superficies aspiracionales, sino validar los flujos que la aplicacion expone de forma visible y activa en la build actual.

## Decision Principal

El gate E2E inicial solo cubre recorridos soportados por la aplicacion vigente:

- Home con listado de proyectos y entrypoints visibles
- Wizard en sus pasos activos
- Monitor con paneles y controles actualmente conectados
- Persistencia observable desde la UI en artefactos y estado de proyecto

Todo modulo no visible, congelado, desconectado o sin entrypoint activo queda fuera de este gate hasta que vuelva a formar parte de la superficie real del producto.

## Que Cambia Respecto Al Plan Anterior

La estrategia previa mezclaba infraestructura candidata, cobertura futura y contratos amplios de producto. Esa version ya no sirve como fuente de verdad para aprobar la aplicacion actual.

En v2:

- el alcance se define por superficie visible, no por modulos existentes en el repo
- las pruebas iniciales funcionan como gate de humo y regresion de flujos activos
- cualquier cobertura futura se agrega solo despues de que la UI o el comando correspondiente este expuesto y soportado

## Superficie Activa A Cubrir

### 1. Home

Validar que la aplicacion abre en el home y permite:

- ver el estado vacio
- ver proyectos existentes por estado cuando existan datos
- iniciar un proyecto nuevo
- reanudar un flujo visible desde una tarjeta o accion soportada

### 2. Wizard

Validar el recorrido soportado de punta a punta:

- Describe
- Plan
- Atomize
- Configure
- Launch

El gate inicial comprueba que cada paso visible puede cargarse, aceptar interaccion basica y avanzar o bloquearse con feedback coherente cuando falte informacion requerida.

### 3. Monitor

Validar la superficie que ya existe en la aplicacion:

- carga del monitor para un proyecto valido
- render de progreso o estado de sesion
- paneles y tabs activos
- acciones visibles como pausa, reanudacion o stop cuando el estado lo permita

## Criterio Del Gate Inicial

El gate pasa si la suite confirma estos comportamientos:

- la app puede arrancar y llegar al home sin errores fatales
- el usuario puede iniciar el flujo principal visible hasta launch
- el monitor puede abrir un proyecto existente y renderizar estado util
- la persistencia observable por la UI no se rompe entre pasos soportados

El gate no exige cobertura exhaustiva de ramas, combinatorias ni modulos internos sin superficie publica actual.

## Contrato De Directorio De Datos Para E2E

Existe una discrepancia entre el uso de `app_data_dir()` en la implementacion actual y el contrato de artefactos documentado en `AGENTS.md` bajo `~/.config/loopforge/projects/<project-id>/`.

Para las pruebas E2E, la unica regla operativa es esta: los artefactos del proyecto se validan en `app_data_dir()/projects/<project-id>`, usando la ruta de datos que resuelve la aplicacion en runtime para la plataforma donde corre la suite.

En consecuencia:

- `app_data_dir()` define la ubicacion base real que debe inspeccionar la prueba
- el contrato de `AGENTS.md` se interpreta como la forma logica del arbol `projects/<project-id>/` y del set de archivos esperados, no como un path absoluto portable entre plataformas
- las assertions E2E deben comprobar `draft.json`, `plan.md`, `prd.json`, `config.json`, `prompt.md` y `guardrails.md` dentro de esa ubicacion resuelta en runtime

Con esta regla, el documento mantiene una sola fuente de verdad operativa para ubicar artefactos: la suite sigue el directorio de datos efectivo de la app y dentro de el valida el contrato de archivos por proyecto.

## Reglas De Alcance

- Si un flujo no tiene entrypoint visible en la UI actual, no entra en v2.
- Si un comando existe pero no esta conectado a una superficie activa, no define cobertura E2E inicial.
- Si una expectativa documental contradice el comportamiento visible del producto, prevalece el comportamiento actual hasta que el producto cambie y la documentacion se actualice.

## No Objetivos De V2

Quedan fuera de esta version:

- contratos de prueba heredados que asumian cobertura total del workspace
- suites definidas para modulos no expuestos al usuario
- matrices de compatibilidad o automatizacion avanzada que no bloquean el flujo principal actual
- cobertura de features futuras o reconexiones pendientes

## Evolucion

Cuando una nueva superficie pase a estar visible y soportada, se agrega al plan con su propio criterio de aprobacion. Hasta entonces, este documento mantiene una regla simple: el gate E2E inicial protege solo la experiencia real que hoy puede recorrer un usuario en LoopForge.
