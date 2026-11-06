#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use simple_rust_api::*;
    use sqlx::PgPool;

    // Загружаем переменные окружения из .env
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("Переменная окружения DATABASE_URL не задана");

    // Инициализируем пул соединений с PostgreSQL
    let pool = PgPool::connect(&database_url)
    .await
    .expect("Ne udalos podklyuchitsya k baze dannyh");

    // Sohranyaem pool v globalnuyu peremennuju
    simple_rust_api::DB_POOL.set(pool.clone()).map_err(|_| ()).unwrap();

    // Конфигурация Leptos
    let conf = get_configuration(Some("Cargo.toml")).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Собираем роутер Axum
    // Собираем роутер Axum
    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let pool = pool.clone();
            move || {
                provide_context(pool.clone());
                view! {
                    <!DOCTYPE html>
                    <html>
                        <head>
                            <meta charset="utf-8"/>
                            <meta name="viewport" content="width=device-width, initial-scale=1"/>
                            <leptos_config/>
                            <link rel="stylesheet" href="/pkg/output.css"/>
                        </head>
                        <body class="bg-slate-900 text-slate-100 min-h-screen flex flex-col items-center justify-center">
                            <App/>
                        </body>
                    </html>
                }
            }
        })
        // Раздаем статику из папки pkg (для WASM, JS и Tailwind CSS)
        .nest_service("/pkg", tower_http::services::ServeDir::new("target/site/pkg"))
        .fallback(leptos_axum::file_and_error_handler(|_options| view! {
            <!DOCTYPE html>
            <html>
                <head>
                    <meta charset="utf-8"/>
                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                    <leptos_config/>
                    <link rel="stylesheet" href="/pkg/output.css"/>
                </head>
                <body class="bg-slate-900 text-slate-100 min-h-screen flex flex-col items-center justify-center">
                    <App/>
                </body>
            </html>
        }))
        .with_state(leptos_options);

    println!("Full-stack сервер запущен на http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

#[cfg(not(feature = "ssr"))]
fn main() {
    // Пустой main для клиентской сборки (компиляции в WASM)
}
