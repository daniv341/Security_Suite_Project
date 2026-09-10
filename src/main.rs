use std::net::SocketAddr;
use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;

use security_suite::shared::auth::JwtService;

use security_suite::users::application::login_service::LoginService;
use security_suite::users::application::service::UserService;
use security_suite::users::domain::UserServicePort;
use security_suite::users::infrastructure::http::handlers::AppState;
use security_suite::users::infrastructure::http::routes::user_routes;
use security_suite::users::infrastructure::postgres_repository::PostgresUserRepository;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        )
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL debe estar definida en el entorno");

    let jwt_secret = std::env::var("JWT_SECRET")?;
    let jwt_expiration: i64 = std::env::var("JWT_EXPIRATION")?.parse()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("No se pudo conectar a la base de datos PostgreSQL");

    tracing::info!("Ejecutando migraciones pendientes...");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("No se pudieron ejecutar las migraciones de la base de datos");

    // Repositorio PostgreSQL
    let repository = Arc::new(PostgresUserRepository::new(pool));

    // Servicio de usuarios
    let user_service: Arc<dyn UserServicePort> =
        Arc::new(UserService::new(repository.clone()));

    // Servicio JWT
    let jwt_service = Arc::new(JwtService::new(
        &jwt_secret,
        jwt_expiration,
    ));

    // Servicio de login
    let login_service = Arc::new(LoginService::new(
        repository.clone(),
        jwt_service.clone(),
    ));

    // Estado compartido de la aplicación
    let state = Arc::new(AppState {
        user_service,
        login_service,
        jwt_service,
    });

    let app = user_routes(state);

    let server_addr =
        std::env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    let addr: SocketAddr = server_addr
        .parse()
        .unwrap_or_else(|_| panic!("SERVER_ADDR inválida: '{server_addr}'"));

    tracing::info!("Servidor escuchando en http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("No se pudo enlazar {addr}: {e}"));

    axum::serve(listener, app)
        .await
        .expect("Error ejecutando el servidor Axum");

    Ok(())
}