# Suite de Seguridad y Gestión de Credenciales — Módulo `User`

Base inicial del módulo de usuarios, construida con **Rust + Axum + SQLx (PostgreSQL)**
bajo **Arquitectura Hexagonal (Puertos y Adaptadores)**, organizada **por objeto
de negocio** (feature-first) en vez de por capa.

## Estructura del proyecto

```
src/
├── shared/                       # "Shared kernel": genérico, de nadie en particular
│   ├── mod.rs
│   └── pagination.rs             # Pagination + PaginatedResult<T>
├── users/                        # Objeto "User": arquitectura hexagonal COMPLETA
│   │                              # y autocontenida para este objeto
│   ├── mod.rs
│   ├── domain/                   # Núcleo: no depende de Axum ni de SQLx
│   │   ├── mod.rs                # Reexporta la API pública del módulo
│   │   ├── entity.rs             # struct User
│   │   ├── error.rs              # enum DomainError
│   │   ├── repository.rs         # trait UserRepository (puerto de salida)
│   │   └── service.rs            # trait UserServicePort (puerto de entrada)
│   ├── application/
│   │   ├── mod.rs
│   │   └── service.rs            # UserService: hasheo Argon2, validaciones,
│   │                              # paginación, implementa UserServicePort
│   └── infrastructure/
│       ├── mod.rs
│       ├── postgres_repository.rs  # Adaptador de salida (SQLx)
│       └── http/
│           ├── mod.rs
│           ├── dto.rs            # Request/response DTOs (incluye paginación)
│           ├── error_response.rs # DomainError -> respuesta HTTP
│           ├── handlers.rs       # Handlers de Axum (solo orquestación)
│           └── routes.rs         # Router + TraceLayer (logging)
├── lib.rs                        # Expone shared/users como librería
│                                  # (necesario para tests/)
└── main.rs                       # Binario: pool, migraciones, arranque
tests/
└── user_service_tests.rs         # Pruebas de integración de UserService
migrations/
└── 0001_create_users_table.sql
```

### Por qué feature-first (`users/domain/`, `users/application/`, `users/infrastructure/`) y no layer-first (`domain/user/`, `application/`, `infrastructure/`)

Con un solo objeto, ambas formas son válidas. La ventaja de esta organización
aparece cuando el proyecto crece: el enunciado original habla de una "Suite de
Seguridad y Gestión de Credenciales", así que en algún momento van a existir
más objetos (`credentials/`, `sessions/`, `roles/`, etc.). Con la estructura
por objeto, agregar uno nuevo es crear `src/credentials/` con su propio
`domain/application/infrastructure` adentro, sin tocar ni un archivo de
`users/`. Cada carpeta de objeto es un módulo autocontenido con su propia
arquitectura hexagonal completa — se puede entender, testear y hasta extraer
a su propio crate sin desenredarlo del resto.

La capa (`domain`/`application`/`infrastructure`) sigue existiendo y sigue
significando exactamente lo mismo que antes (mismas reglas de dependencia:
`domain` no conoce a nadie, `application` solo conoce a `domain`,
`infrastructure` implementa los puertos de `domain`) — lo único que cambió es
el nivel en el que se agrupa primero: por objeto, y dentro de cada objeto, por
capa.

### Dónde quedó la paginación, y por qué

Preguntaste si `Pagination`/`PaginatedResult<T>` deberían ir dentro de
`users/` o si cada objeto futuro debería tener la suya. Mi recomendación —y lo
que implementé— es un tercer lugar: **`src/shared/`**, fuera de cualquier
objeto.

Motivo: `Pagination` y `PaginatedResult<T>` no tienen ninguna regla de negocio
de `User` — no saben qué es un usuario, un email o una contraseña. Son un
concepto genérico (una página, un límite, un offset) que **cualquier** objeto
que liste resultados va a necesitar. Ponerlas dentro de `users/domain/`
obligaría a `credentials/` o `sessions/` a: (a) duplicar el mismo código, o
(b) depender de `users` solo para pedirle prestada la paginación — ambas
opciones rompen la idea de que cada objeto es autocontenido. `shared/` es lo
que en Diseño Orientado a Dominio se llama **shared kernel**: el lugar para lo
que de verdad es transversal, sin volverse una carpeta "de todo un poco". Si
en el futuro un objeto puntual necesitara una variante especial de paginación
(por ejemplo, paginación por cursor en vez de por página), esa variante sí
iría dentro de su propio `domain/`, conviviendo con la genérica de `shared/`
sin conflicto.

### `src/lib.rs`: por qué existe

Rust exige que las pruebas de integración (carpeta `tests/`, fuera de `src/`)
enlacen contra una **librería**, no contra un binario. Por eso el crate expone
`src/lib.rs` (con `pub mod shared; pub mod users;`) y `src/main.rs` quedó como
un binario delgado que usa `security_suite::...`.

## Seguridad implementada

- Contraseñas hasheadas con **Argon2id** (sal aleatoria por usuario, vía `OsRng`).
  El hash nunca se compara ni se expone en texto plano.
- Los DTOs de respuesta (`UserResponse`) **omiten explícitamente** `password_hash`.
- Validación de entrada (username ≥ 3 caracteres, email con formato básico válido,
  password ≥ 8 caracteres) antes de tocar la base de datos.
- Email `UNIQUE` a nivel de base de datos **y** verificación a nivel de aplicación
  (devuelve `409 Conflict` en vez de un error 500 genérico).
- El usuario del contenedor de runtime (`appuser`) no es root.

## Endpoints

| Método | Ruta                              | Descripción                                   |
|--------|-----------------------------------|------------------------------------------------|
| POST   | `/api/v1/users`                   | Registra un usuario (hashea la contraseña)     |
| GET    | `/api/v1/users/?page=&page_size=` | Lista usuarios paginados (sin `password_hash`) |
| GET    | `/api/v1/users/:id`               | Obtiene un usuario por id                      |
| PUT    | `/api/v1/users/:id`               | Actualiza `username` y/o `email`               |
| DELETE | `/api/v1/users/:id`               | Elimina la cuenta                              |

Ejemplo de body para `POST /api/v1/users`:

```json
{
  "username": "johndoe",
  "email": "john@example.com",
  "password": "supersecret123"
}
```

### Paginación

`GET /api/v1/users/` acepta `page` (default `1`) y `page_size` (default `20`,
máximo `100`; valores fuera de rango se ajustan automáticamente, nunca dan
error). Ejemplo: `GET /api/v1/users/?page=2&page_size=10`.

Respuesta:

```json
{
  "items": [ { "id": "...", "username": "...", "email": "...", "created_at": "...", "updated_at": "..." } ],
  "page": 2,
  "page_size": 10,
  "total_items": 47,
  "total_pages": 5
}
```

La lógica vive en `shared/pagination.rs` (`Pagination`, `PaginatedResult<T>`),
es genérica y no depende de `User`, así que sirve para paginar cualquier otra
entidad que se agregue más adelante. El repositorio de PostgreSQL la traduce a
`LIMIT`/`OFFSET` + `SELECT COUNT(*)`.

### Logging (`TraceLayer`)

Cada request deja **una sola línea**, con lo esencial (método, path, status,
latencia) — no el par de líneas "started processing request" / "finished
processing request" que trae `TraceLayer` por defecto:

```
2026-09-08T12:00:00.123456Z  INFO http{method=POST path=/api/v1/users}: security_suite::users::infrastructure::http::routes: status=201 latency_ms=4 request
```

Se logra desactivando el callback `on_request` (que es el que emite el
mensaje de "inicio") y dejando que `on_response` sea el único punto que loguea,
ya con el status y la latencia. Configurado en `users/infrastructure/http/routes.rs`.

## Cómo correrlo

### Opción A: Docker Compose — producción

```bash
cp .env.example .env
docker compose up --build
```

Levanta `db` (PostgreSQL 15) y `app` (binario Rust compilado en release), en
la red `security_net`. La app espera a que la base esté saludable
(`healthcheck` con `pg_isready`) y corre las migraciones automáticamente.

La API queda disponible en `http://localhost:8080/api/v1/users`.

### Opción B: Docker Compose — desarrollo con recarga automática

```bash
cp .env.example .env
docker compose -f docker-compose.yml -f docker-compose.dev.yml up --build
```

Cada cambio guardado en tu editor reinicia el servidor solo, sin que vuelvas
a correr `docker compose up --build` a mano. Ver la sección
[Recarga automática en desarrollo](#recarga-automática-en-desarrollo-hot-reload)
para el detalle de cómo funciona y por qué no es tan lento como parece.

### Opción C: local (sin Docker)

Necesitás un PostgreSQL corriendo y accesible vía `DATABASE_URL`.

```bash
cp .env.example .env
# ajustá DATABASE_URL si tu Postgres no corre en localhost:5432
cargo install cargo-watch   # opcional, para recarga automática
cargo watch -x run          # o simplemente `cargo run`
```

Las migraciones (`migrations/0001_create_users_table.sql`) se aplican solas
al arrancar (`sqlx::migrate!`).

## Recarga automática en desarrollo (hot-reload)

**El problema que resuelve**: antes, cualquier cambio de código exigía
`docker compose up --build`, que reconstruye la imagen completa (recompila
absolutamente todo, dependencias incluidas) cada vez — lento y tedioso.

**La solución** (`docker-compose.dev.yml`, stage `dev` del `Dockerfile`):

1. **`cargo-watch`** corre dentro del contenedor y recompila + reinicia el
   servidor automáticamente al detectar un cambio.
2. El código fuente se monta como **bind mount** (`.:/app`), así los cambios
   llegan al contenedor al instante, sin reconstruir la imagen de Docker.
3. **Acá está la clave de rendimiento** (lo que investigaste y es cierto: sin
   esto, sería lento): `target/` y el registro de Cargo se montan como
   **volúmenes nombrados** (`target_cache`, `cargo_registry_cache`), no como
   parte del bind mount. Eso significa que:
   - Los binarios ya compilados de cada dependencia **persisten** entre
     reinicios del contenedor.
   - Cuando cambiás un archivo, `cargo-watch` dispara una compilación
     **incremental**: recompila solo tu código (y lo que dependa de él), no
     las ~150 dependencias de terceros. En la práctica, después del primer
     arranque, cada recarga tarda unos pocos segundos, no minutos.

El costo real es únicamente el **primer arranque** (compila todo desde cero,
como cualquier `cargo build` limpio) — después, la cache en los volúmenes
nombrados hace que las recargas sean rápidas. Esta imagen `dev` **no** se usa
en producción: `docker-compose.yml` (sin el override) sigue construyendo el
binario optimizado de `release` de siempre.

## Comandos que usaras mucho

Producción en segundo plano:
docker compose up --build -d

Desarrollo + hot reload en segundo plano:
docker compose -f docker-compose.yml -f docker-compose.dev.yml up --build -d

Ver estado:
docker compose ps

Ver logs de producción:
docker compose logs -f app

Ver logs de desarrollo:
docker compose -f docker-compose.yml -f docker-compose.dev.yml logs -f app

Detener producción:
docker compose down

Detener desarrollo:
docker compose -f docker-compose.yml -f docker-compose.dev.yml down

## Tests

Viven en `tests/user_service_tests.rs` (pruebas de integración, fuera de
`src/`, el lugar idiomático en Rust). Corren contra la librería
`security_suite` expuesta en `src/lib.rs`.

```bash
cargo test
```

Salida esperada (resumida):

```
     Running tests/user_service_tests.rs (target/debug/deps/user_service_tests-...)

running 12 tests
test delete_user_elimina_correctamente ... ok
test delete_user_falla_si_no_existe ... ok
test falla_si_el_email_es_invalido ... ok
test get_all_users_pagina_los_resultados ... ok
test get_all_users_usa_valores_por_defecto_si_no_se_especifican ... ok
...
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Usan un **mock manual** de `UserRepository` (en memoria, con `HashMap` +
`Mutex`, definido en el propio archivo de test), sin necesidad de una base de
datos real. Cubren:

- Que la contraseña se hashea (nunca queda en texto plano) y que el hash
  resultante verifica correctamente contra la contraseña original.
- Rechazo de contraseña corta, email inválido y username corto.
- Conflicto por email duplicado (en registro y en actualización).
- `NotFound` al consultar/actualizar/eliminar un id inexistente.
- Actualización parcial de perfil (username y/o email) sin tocar el hash.
- Paginación: tamaño de página, cálculo de `total_pages`, y valores por
  defecto cuando no se especifican `page`/`page_size`.

> Nota de verificación: este proyecto fue compilado y sus tests ejecutados
> exitosamente (`cargo check`, `cargo build`, `cargo test` → 12/12 passed)
> antes de la entrega.

## `Cargo.lock`

Este repositorio se entrega **sin** `Cargo.lock` (ver `.gitignore`) para que,
al compilar por primera vez, Cargo resuelva las versiones más recientes
compatibles de cada dependencia según los rangos declarados en `Cargo.toml`.
Si preferís reproducibilidad estricta de builds (recomendado para un
binario en producción), corré `cargo build` una vez, comiteá el
`Cargo.lock` resultante y quitá la línea `Cargo.lock` del `.gitignore`.

## Próximos pasos sugeridos (fuera del alcance de esta etapa)

- **Autenticación**: endpoint `POST /api/v1/auth/login` que verifique la
  contraseña (ya está `UserService::verify_password`, solo falta emitir un
  JWT o crear una sesión) y middleware de autorización para proteger el
  resto de las rutas.
- **Rate limiting** en `POST /api/v1/users` y en el futuro `/login`, para
  mitigar fuerza bruta y scraping de emails vía el `409 Conflict`.
- **`sqlx::query_as!` en modo offline** (`cargo sqlx prepare`) si en algún
  momento se quiere volver a las macros verificadas en tiempo de compilación
  en vez de las versiones "runtime-checked" usadas acá.
