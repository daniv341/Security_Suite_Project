use std::net::SocketAddr;
use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use security_suite::shared::state::{AppState, UserState, FolderState, LogState, LoginState};

use security_suite::shared::auth::JwtService;

use security_suite::users::application::login_service::LoginService;
use crate::auth::application::AuthService;
use crate::auth::infrastructure::PostgresRevokedTokenRepository;
use security_suite::users::application::service::UserService;
use security_suite::users::domain::UserServicePort;
use security_suite::users::infrastructure::http::routes::{user_routes, login_routes};
use security_suite::users::infrastructure::postgres_repository::PostgresUserRepository;

use security_suite::folders::application::service::FolderService;
use security_suite::folders::domain::FolderServicePort;
use security_suite::folders::infrastructure::http::routes::folders_routes;
use security_suite::folders::infrastructure::postgres_repository::PostgresFolderRepository;

use security_suite::logs::application::service::LogService;
use security_suite::logs::domain::LogServicePort;
use security_suite::logs::infrastructure::http::routes::logs_routes;
use security_suite::logs::infrastructure::postgres_repository::PostgresLogRepository;


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

    // Repositorios PostgreSQL
    let user_repository = Arc::new(PostgresUserRepository::new(pool.clone()));
    let revoked_token_repository = Arc::new(PostgresRevokedTokenRepository::new(pool.clone()));
    let folder_repository = Arc::new(PostgresFolderRepository::new(pool.clone()));
    let log_repository = Arc::new(PostgresLogRepository::new(pool.clone()));

    // Servicio de usuarios
    let user_service: Arc<dyn UserServicePort> =
        Arc::new(UserService::new(user_repository.clone()));

    let folder_service: Arc<dyn FolderServicePort> = Arc::new(FolderService::new(folder_repository.clone()));

    let log_service: Arc<dyn LogServicePort> = Arc::new(LogService::new(log_repository.clone()));

    // Servicio JWT
    let jwt_service = Arc::new(JwtService::new(
        &jwt_secret,
        jwt_expiration,
    ));

    // Servicio de login
    let login_service = Arc::new(LoginService::new(
        user_repository.clone(),
        jwt_service.clone(),
    ));

    let auth_service =
    Arc::new(AuthService::new(revoked_token_repository));

    // Estado compartido de la aplicación
    let user_state = Arc::new(UserState {
        user_service,
    });

    let folder_state = Arc::new(FolderState {
        folder_service,
    });

    let log_state = Arc::new(LogState {
        log_service,
    });

    let login_state = Arc::new(LoginState {
        login_service,
        jwt_service,
    });

    let app_state = AppState {
        user_state,
        login_state,
        folder_state,
        log_state,
    };

    let app = user_routes()
        .merge(login_routes())
        .merge(folders_routes())
        .merge(logs_routes())
        .with_state(app_state);

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