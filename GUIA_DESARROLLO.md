# Guía de desarrollo — Suite de Seguridad y Gestión de Credenciales

Esta guía asume que ya tenés el módulo `users/` funcionando (ver el
`README.md` principal para cómo correr el proyecto). Acá se explica **cómo
está armado por dentro** y **cómo extenderlo** sin romper nada, usando como
ejemplos reales el modelo que planeamos: `Usuario`, `Configuracion2FA`,
`Carpeta`, `ItemBoveda`, `HistorialContrasena`, `LogAuditoria`.

---

## 1. Cómo funciona el proyecto

### 1.1 La regla de oro: tres capas, una sola dirección de dependencia

Cada objeto (`users/`, y cada objeto nuevo que agregues) tiene siempre esta
forma:

```
src/<objeto>/
├── domain/          # Capa 1: reglas de negocio puras
├── application/      # Capa 2: casos de uso
└── infrastructure/   # Capa 3: conexión con el mundo real (DB, HTTP)
```

La regla que no se rompe nunca: **las flechas de dependencia apuntan hacia
adentro**.

```
infrastructure  ──depende de──>  application  ──depende de──>  domain
     (sabe de Axum, SQLx)         (sabe de domain)              (no sabe de nada)
```

- `domain/` no importa `axum`, ni `sqlx`, ni sabe que existe una base de
  datos. Solo define **qué es** un `User` (`entity.rs`), **qué puede salir
  mal** (`error.rs`) y **qué operaciones existen**, como dos contratos
  (traits): `UserRepository` (qué necesita de una base de datos) y
  `UserServicePort` (qué casos de uso ofrece el módulo).
- `application/` (`service.rs`) implementa `UserServicePort`. Acá vive la
  lógica: validar, hashear la contraseña, chequear que el email no esté
  repetido. Para hablar con la base de datos, **no conoce PostgreSQL**: solo
  conoce el trait `UserRepository` (recibe `Arc<dyn UserRepository>`).
- `infrastructure/` tiene las implementaciones concretas de esos traits:
  `postgres_repository.rs` implementa `UserRepository` con SQL de verdad, y
  `http/` (handlers + routes) es la puerta de entrada HTTP que llama a
  `UserServicePort`.

**Por qué importa**: gracias a que `application` solo depende de un trait
(no de PostgreSQL), los tests (`tests/user_service_tests.rs`) pueden
reemplazar la base de datos real por un mock en memoria, sin tocar una sola
línea de `UserService`.

### 1.2 El recorrido de un request, de punta a punta

Ejemplo: `POST /api/v1/users`.

```
1. main.rs           arma el Router y lo pone a escuchar en un puerto
2. routes.rs          matchea la URL/método y llama al handler
3. handlers.rs         create_user(): deserializa el JSON a CreateUserRequest
4. dto.rs              (la forma del JSON de entrada/salida vive acá)
5. UserServicePort     el handler llama a service.register_user(...)
                       (no sabe si "service" es un UserService real o un mock)
6. application/service.rs   valida, hashea la contraseña con Argon2
7. UserRepository      pide al repositorio: repository.create(&user)
                       (no sabe si el repositorio es Postgres o un mock)
8. postgres_repository.rs   ejecuta el INSERT real contra la base
9. de vuelta:          User (entidad) -> UserResponse (dto, sin password_hash) -> JSON
```

Ningún paso "salta" capas: el handler nunca llama a SQLx directamente, ni el
`domain` sabe que existe Axum. Eso es lo que te permite, por ejemplo, cambiar
Axum por otro framework HTTP, o PostgreSQL por otra base, tocando **solo**
`infrastructure/`, sin tocar `domain/` ni `application/`.

### 1.3 `shared/`: lo que no es de ningún objeto en particular

`src/shared/pagination.rs` (`Pagination`, `PaginatedResult<T>`) es genérico:
no sabe qué es un `User`. Cualquier objeto nuevo (`Carpeta`, `ItemBoveda`,
`LogAuditoria`) reusa esto en vez de reinventarlo. La regla para decidir si
algo va en `shared/` o dentro de un objeto: **¿esta lógica conoce reglas de
negocio de un objeto puntual?** Si no, va en `shared/`. Ejemplos que
**deberían** ir en `shared/` más adelante: el middleware de JWT (no le
pertenece a `users`, aunque valide usuarios — lo va a usar `folders`,
`vault`, todo lo que requiera auth).

### 1.4 `lib.rs` / `main.rs`: dónde se conecta todo

- `src/lib.rs` solo declara `pub mod shared; pub mod users; ...` — existe
  para que `tests/` pueda importar el código como si fuera una librería.
- `src/main.rs` es el **composition root**: el único lugar del proyecto
  donde se crean las implementaciones concretas y se "inyectan" en los
  traits (`Arc<PostgresUserRepository>` guardado como
  `Arc<dyn UserRepository>`). Ningún otro archivo hace esto — es intencional,
  así siempre sabés dónde mirar para ver "qué implementación real se está
  usando".

### 1.5 `tests/` y `scripts/`

- `tests/user_service_tests.rs`: pruebas de integración (fuera de `src/`,
  como exige Rust) que usan un `MockUserRepository` en memoria en vez de
  Postgres — corren rápido y no necesitan Docker.
- `scripts/scaffold_object.py`: genera el andamiaje (`domain/application/
  infrastructure/` + tests) de un objeto nuevo con la plantilla genérica
  (`id`, `name`, `created_at`, `updated_at`). Ahorra el boilerplate repetido;
  no reemplaza pensar el modelo (ver sección 3).

### 1.6 Mapa rápido: "quiero cambiar X, ¿qué archivo toco?"

| Querés...                                             | Archivo(s)                                              |
|--------------------------------------------------------|-----------------------------------------------------------|
| Agregar una regla de validación                        | `<objeto>/application/service.rs`                         |
| Cambiar qué campos tiene la entidad                     | `<objeto>/domain/entity.rs`                                |
| Cambiar qué devuelve la API (JSON de salida)            | `<objeto>/infrastructure/http/dto.rs`                      |
| Agregar/cambiar una consulta SQL                        | `<objeto>/infrastructure/postgres_repository.rs`            |
| Agregar un endpoint nuevo                               | `handlers.rs` + `routes.rs` (del mismo objeto)              |
| Cambiar un código de error HTTP                         | `<objeto>/infrastructure/http/error_response.rs`            |
| Conectar todo (wiring)                                   | `src/main.rs`                                              |

---

## 2. Agregar o quitar un atributo sin romper nada

La ventaja de Rust acá: **el compilador es tu checklist**. Si cambiás la
forma de una `struct`, todo lugar que la construye deja de compilar hasta
que lo arregles — no hay forma de "olvidarte" de un lugar silenciosamente.

### 2.1 Agregar un atributo — ejemplo: `phone_number` en `User`

**Paso 1 — Migración nueva** (nunca edites una migración vieja que ya
corrió; el historial de migraciones es un registro, no un borrador):

```sql
-- migrations/0002_add_phone_number_to_users.sql
ALTER TABLE users ADD COLUMN phone_number TEXT NULL;
```

`NULL` (opcional) es la opción segura para agregar una columna a una tabla
que ya tiene filas — si la pusieras `NOT NULL` sin `DEFAULT`, la migración
fallaría contra datos existentes.

**Paso 2 — Entidad de dominio** (`users/domain/entity.rs`):

```rust
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub phone_number: Option<String>,   // <- nuevo
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

En este punto, `cargo check` ya te va a marcar en rojo **todos** los lugares
que arman un `User` (ej. `User::new(...)` en `entity.rs`, si el constructor
no recibe el campo nuevo) y todo el que arma un `UserResponse` a mano — seguí
esos errores uno por uno, son tu checklist.

**Paso 3 — Repositorio Postgres** (`postgres_repository.rs`): agregar la
columna a **las tres** consultas que la tocan (fácil de olvidar una):

```rust
// create(): agregar a INSERT, VALUES y RETURNING
// find_by_id / find_by_email / find_all: agregar a SELECT
// update(): agregar a SET si es editable por el usuario
```

**Paso 4 — DTOs** (`dto.rs`), solo si el campo debe viajar por la API:

```rust
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub phone_number: Option<String>,   // <- nuevo, si se puede editar
}

pub struct UserResponse {
    // ...
    pub phone_number: Option<String>,   // <- nuevo, si se debe mostrar
}
```

Y actualizar el `impl From<User> for UserResponse`.

**Paso 5 — Lógica de negocio** (`application/service.rs`), si hace falta
validar el campo nuevo (ej. formato de teléfono).

**Paso 6 — `cargo build`** hasta que no queden errores de compilación (esa
es la señal de que no te olvidaste ningún lugar).

**Paso 7 — Tests**: agregar un caso en `tests/user_service_tests.rs` que
cubra el campo nuevo (y correr `cargo test` para confirmar que nada viejo se
rompió).

### 2.2 Quitar un atributo — mismo truco, al revés

1. Borrá el campo de `entity.rs` **primero**.
2. `cargo build` va a listar, con línea y columna exacta, cada archivo que
   todavía lo usa (constructores, DTOs, queries SQL con `.bind()` de más,
   tests). Andá arreglando esa lista.
3. Recién al final, escribí la migración de baja:
   ```sql
   -- migrations/0003_drop_phone_number_from_users.sql
   ALTER TABLE users DROP COLUMN phone_number;
   ```
4. `cargo test`.

**Por qué en ese orden** (código primero, migración al final): si borrás la
columna de la base antes que el código, tu servidor actual (todavía corriendo
con el binario viejo) va a empezar a tirar errores 500 en cada request hasta
que despliegues el código nuevo. Haciéndolo en el orden de arriba, el código
sigue funcionando contra la base vieja hasta el último paso.

---

## 3. Agregar un objeto nuevo

Dos caminos, según qué tan parecido sea el objeto a la plantilla genérica
(`id`, `name`, `created_at`, `updated_at`).

### 3.1 Camino rápido: objeto simple con el scaffold — ejemplo `Carpeta`

`Carpeta` es casi la plantilla genérica (tiene `nombre`) más una relación con
`Usuario` (ver sección 4). Pasos:

**Paso 1 — Generar el andamiaje:**

```bash
python3 scripts/scaffold_object.py folder
```

Esto crea `src/folders/{domain,application,infrastructure}/...` y
`tests/folders_tests.rs`, y al final imprime en pantalla los pasos manuales
(no los hace solo, a propósito):

**Paso 2 — Registrar el módulo** en `src/lib.rs`:

```rust
pub mod folders;
```

**Paso 3 — Migración** (el script te da un ejemplo base; para `Carpeta`
hace falta agregar la relación con `Usuario`, ver sección 4):

```sql
-- migrations/000X_create_folders_table.sql
CREATE TABLE IF NOT EXISTS folders (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Paso 4 — Ajustar la entidad y el resto de los archivos generados** para
la relación (agregar `user_id: Uuid` en `entity.rs`, en el repositorio, en el
servicio — ver el ejemplo completo en la sección 4.2, es el mismo caso).

**Paso 5 — Cablear en `main.rs`** (repositorio -> servicio -> rutas, igual
que `users`) y **mergear el router**:

```rust
let app = user_routes.merge(folder_routes);
```

**Paso 6 — `cargo test`.**

### 3.2 Camino manual: objeto que no calza con la plantilla — ejemplo `ItemBoveda`

`ItemBoveda` tiene un campo `tipo` (Login / Tarjeta / Nota) que no es texto
libre, sino un **enum cerrado** — el scaffold no genera esto, pero podés
usarlo como punto de partida y reemplazar `entity.rs` a mano:

```rust
// vault/domain/entity.rs
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum ItemTipo {
    Login,
    TarjetaCredito,
    NotaSegura,
}

pub struct ItemBoveda {
    pub id: Uuid,
    pub user_id: Uuid,
    pub folder_id: Option<Uuid>,   // ver sección 4.3 (relación opcional)
    pub tipo: ItemTipo,
    pub titulo: String,
    pub contenido_cifrado: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

A partir de acá, el resto del trabajo (repositorio, DTOs, handlers, routes,
tests) sigue **exactamente la misma forma** que `users/` — copiá la
estructura de carpetas de `users/` (o de un objeto generado con el script) y
adaptá los tipos.

**Nota de seguridad, importante para este proyecto en particular**: en
`ItemBoveda`, `Carpeta`, y cualquier objeto que pertenezca a un usuario, el
repositorio **nunca** debe tener un `find_all()` sin filtrar — siempre
`find_all_by_user_id(user_id, pagination)`, para que un usuario no pueda
listar los ítems de otro. Esto se vuelve exigible recién cuando exista el
middleware de JWT (que le da al handler el `user_id` del token, no del body
del request — nunca confíes en un `user_id` que venga en el JSON).

---

## 4. Relacionar dos objetos (1:1, 1:N, N:M)

### Principio general: las entidades de dominio se relacionan **por ID**, no anidadas

Ni `Carpeta` debe contener un `Usuario` completo adentro, ni `ItemBoveda` debe
contener una `Carpeta` completa. Cada entidad solo guarda el **id** del
objeto relacionado (`user_id: Uuid`, `folder_id: Option<Uuid>`). Motivos:

- Evita que dos objetos (dos carpetas de `src/`) tengan que importarse mutuamente.
- Evita traer de la base más datos de los que hacen falta en cada consulta.
- Si un handler necesita mostrar el nombre de la carpeta junto con el ítem,
  eso se resuelve con un `JOIN` puntual en el repositorio (o una consulta
  aparte), no anidando la entidad completa en el dominio.

### 4.1 Relación 1:1 — `Usuario` ↔ `Configuracion2FA`

La tabla del lado "1" opcional lleva la FK, con `UNIQUE` para forzar que no
haya más de una fila por usuario:

```sql
CREATE TABLE IF NOT EXISTS configuraciones_2fa (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    secreto_totp TEXT NOT NULL,
    activado BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

```rust
// two_fa_configs/domain/entity.rs
pub struct Configuracion2FA {
    pub id: Uuid,
    pub user_id: Uuid,       // el UNIQUE en la DB garantiza el 1:1
    pub secreto_totp: String,
    pub activado: bool,
    pub created_at: DateTime<Utc>,
}
```

El repositorio, en vez de `find_by_id` como método principal, expone
`find_by_user_id`, porque así es como se va a consultar siempre en la
práctica ("dame el 2FA *de este* usuario"):

```rust
async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<Configuracion2FA>, DomainError>;
```

### 4.2 Relación 1:N — `Usuario` ↔ `Carpeta`

El lado "muchos" (`Carpeta`) lleva la FK, sin `UNIQUE` (un usuario puede
tener muchas carpetas):

```sql
CREATE TABLE IF NOT EXISTS folders (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

`ON DELETE CASCADE` es una decisión de negocio: acá significa "si se borra
el usuario, se borran sus carpetas". La alternativa sería `ON DELETE
RESTRICT` (no dejar borrar el usuario si tiene carpetas).

```rust
// folders/domain/entity.rs
pub struct Folder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

```rust
// folders/domain/repository.rs
// En vez de (o adem\u00e1s de) find_all genérico, la consulta real de esta
// app siempre necesita el scope por usuario:
async fn find_all_by_user_id(
    &self,
    user_id: Uuid,
    pagination: Pagination,
) -> Result<Vec<Folder>, DomainError>;
```

```rust
// folders/application/service.rs
// El user_id NUNCA viene del body del request (eso sería que cualquiera
// pueda crear una carpeta "a nombre de" otro usuario) — viene del JWT,
// como parámetro explícito del caso de uso:
async fn create_folder(&self, user_id: Uuid, name: String) -> Result<Folder, DomainError> {
    Self::validate_name(&name)?;
    let folder = Folder::new(user_id, name);
    self.repository.create(&folder).await
}
```

```rust
// folders/infrastructure/postgres_repository.rs
async fn find_all_by_user_id(&self, user_id: Uuid, pagination: Pagination) -> Result<Vec<Folder>, DomainError> {
    sqlx::query_as::<_, Folder>(
        r#"
        SELECT id, user_id, name, created_at, updated_at
        FROM folders
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(&self.pool)
    .await
    .map_err(|e| DomainError::Repository(e.to_string()))
}
```

### 4.3 Relación N:1 opcional — `ItemBoveda` ↔ `Carpeta`

Un ítem puede o no estar en una carpeta. La FK va en el lado "muchos"
(`ItemBoveda`), y es **nullable**:

```sql
CREATE TABLE IF NOT EXISTS vault_items (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    folder_id UUID NULL REFERENCES folders(id) ON DELETE SET NULL,
    tipo TEXT NOT NULL,
    titulo VARCHAR(255) NOT NULL,
    contenido_cifrado TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

Notá `ON DELETE SET NULL` en vez de `CASCADE`: si se borra una carpeta, los
ítems no se borran, quedan "sin carpeta" — decisión de negocio distinta a la
del 1:N de arriba, y es exactamente lo que representa `Option<Uuid>` del
lado Rust:

```rust
pub struct ItemBoveda {
    pub id: Uuid,
    pub user_id: Uuid,
    pub folder_id: Option<Uuid>,   // <- nullable = relación opcional
    // ...
}
```

### 4.4 Relación N:M — ejemplo (no está en el modelo original): `ItemBoveda` ↔ `Etiqueta`

Ninguna relación del modelo que planeamos es N:M, así que este es un ejemplo
hipotético para el día que quieras agregar etiquetas ("Trabajo", "Urgente")
que un ítem puede tener varias y una etiqueta puede estar en varios ítems.

**Ninguna de las dos tablas lleva la FK de la otra** — hace falta una
**tabla pivote**:

```sql
CREATE TABLE IF NOT EXISTS tags (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    CONSTRAINT tags_user_name_unique UNIQUE (user_id, name)
);

CREATE TABLE IF NOT EXISTS vault_item_tags (
    vault_item_id UUID NOT NULL REFERENCES vault_items(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (vault_item_id, tag_id)   -- evita duplicados, no hace falta un id propio
);
```

Del lado Rust, **ninguna de las dos entidades** (`ItemBoveda`, `Tag`) lleva
una lista de la otra adentro (eso volvería el `struct` inconsistente con la
base y forzaría cargar todo siempre). En cambio, el repositorio de la tabla
pivote expone operaciones puntuales:

```rust
// vault/domain/repository.rs (o un módulo aparte para la relación)
async fn add_tag(&self, vault_item_id: Uuid, tag_id: Uuid) -> Result<(), DomainError>;
async fn remove_tag(&self, vault_item_id: Uuid, tag_id: Uuid) -> Result<(), DomainError>;
async fn find_tags_for_item(&self, vault_item_id: Uuid) -> Result<Vec<Tag>, DomainError>;
```

```rust
// implementación: find_tags_for_item hace el JOIN
async fn find_tags_for_item(&self, vault_item_id: Uuid) -> Result<Vec<Tag>, DomainError> {
    sqlx::query_as::<_, Tag>(
        r#"
        SELECT t.id, t.user_id, t.name
        FROM tags t
        INNER JOIN vault_item_tags vit ON vit.tag_id = t.id
        WHERE vit.vault_item_id = $1
        "#,
    )
    .bind(vault_item_id)
    .fetch_all(&self.pool)
    .await
    .map_err(|e| DomainError::Repository(e.to_string()))
}
```

Y a nivel HTTP, la relación se maneja con endpoints propios, no metiendo la
lista de tags dentro del body de `PUT /vault/:id`:

```
POST   /api/v1/vault/:id/tags/:tag_id     -> agrega una etiqueta al ítem
DELETE /api/v1/vault/:id/tags/:tag_id     -> la quita
GET    /api/v1/vault/:id/tags              -> lista las etiquetas del ítem
```

### 4.5 Resumen: dónde va la FK según el tipo de relación

| Relación        | Dónde va la FK                              | Ejemplo del proyecto              |
|------------------|------------------------------------------------|--------------------------------------|
| 1:1              | En la tabla "opcional", con `UNIQUE`            | `Usuario` ↔ `Configuracion2FA`      |
| 1:N              | En la tabla del lado "muchos"                   | `Usuario` ↔ `Carpeta`               |
| N:1 opcional     | En la tabla del lado "muchos", **nullable**      | `ItemBoveda` ↔ `Carpeta`            |
| N:M              | Tabla pivote aparte, sin FK en ninguna de las dos | `ItemBoveda` ↔ `Etiqueta` (ejemplo) |

Y la misma pregunta llevada al lado Rust: **¿la entidad necesita conocer al
otro lado?** Si es un solo id (1:1, 1:N, N:1) va como campo (`Uuid` o
`Option<Uuid>`) en `entity.rs`. Si son varios (N:M), no va como campo — va
como método de repositorio que hace el `JOIN` cuando hace falta.
