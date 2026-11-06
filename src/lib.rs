use leptos::prelude::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use web_sys::wasm_bindgen::JsCast;


#[cfg(feature = "ssr")]
use sqlx::PgPool;
#[cfg(feature = "ssr")]
use std::sync::OnceLock;

#[cfg(feature = "ssr")]
pub static DB_POOL: OnceLock<PgPool> = OnceLock::new();

// Структура пользователя (id теперь i64 под bigint из базы)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[server(GetUsers, "/api")]
pub async fn get_users() -> Result<Vec<User>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = crate::DB_POOL.get().ok_or_else(|| {
            ServerFnError::new("База данных не инициализирована")
        })?;

        let users = sqlx::query_as!(
            User,
            "SELECT id, name, email FROM users ORDER BY id DESC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(users)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Только для сервера"))
    }
}

#[server(AddUser, "/api")]
pub async fn add_user(name: String, email: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = crate::DB_POOL.get().ok_or_else(|| {
            ServerFnError::new("База данных не инициализирована")
        })?;

        sqlx::query!(
            "INSERT INTO users (name, email) VALUES ($1, $2)",
            name,
            email
        )
        .execute(pool)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Только для сервера"))
    }
}

#[server(DeleteUser, "/api")]
pub async fn delete_user(id: i64) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let pool = crate::DB_POOL.get().ok_or_else(|| {
            ServerFnError::new("База данных не инициализирована")
        })?;

        sqlx::query!("DELETE FROM users WHERE id = $1", id)
            .execute(pool)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("Только для сервера"))
    }
}

#[component]
pub fn App() -> impl IntoView {
    let users_resource = Resource::new(|| (), |_| async move { get_users().await });

    // Используем ServerAction, созданный автоматически макросом #[server(AddUser, ...)]
    let add_user_action = ServerAction::<AddUser>::new();

    // Автоматически перезагружаем список пользователей, когда экшен завершается успешно
    Effect::new(move |_| {
        if let Some(_) = add_user_action.value().get() {
            users_resource.refetch();
        }
    });

    let delete_user_action = ServerAction::<DeleteUser>::new();

    Effect::new(move |_| {
     if let Some(_) = delete_user_action.value().get() {
        users_resource.refetch();
     }
    }); 
   
    view! {
        <main class="w-full max-w-2xl mx-auto p-6 space-y-8 bg-slate-900 text-slate-100 min-h-screen">
            <div class="text-center space-y-2">
                <h1 class="text-3xl font-extrabold tracking-tight text-indigo-400">"Управление пользователями"</h1>
                <p class="text-slate-400">"Full-stack приложение на Leptos 0.7, Axum и PostgreSQL с Tailwind CSS"</p>
            </div>

            <section class="bg-slate-800 p-6 rounded-xl shadow-lg border border-slate-700">
                <h2 class="text-xl font-semibold mb-4 text-slate-200">"Добавить нового пользователя"</h2>
                
            // Убрали class отсюда
                <ActionForm action=add_user_action>
                    <div class="space-y-4"> // Перенесли класс сюда
                        <div>
                            <label class="block text-sm font-medium text-slate-300 mb-1">"Имя"</label>
                            <input
                                type="text"
                                name="name"
                                required
                                placeholder="Введите имя..."
                                class="w-full px-4 py-2 bg-slate-900 border border-slate-700 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:outline-none text-slate-100 placeholder-slate-500"
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-slate-300 mb-1">"Email"</label>
                            <input
                                type="email"
                                name="email"
                                required
                                placeholder="example@domain.com"
                                class="w-full px-4 py-2 bg-slate-900 border border-slate-700 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:outline-none text-slate-100 placeholder-slate-500"
                            />
                        </div>
                        <button
                            type="submit"
                            class="w-full py-2.5 px-4 bg-indigo-600 hover:bg-indigo-500 font-medium rounded-lg transition-colors shadow-md cursor-pointer text-white"
                        >
                            "Сохранить пользователя"
                        </button>
                    </div>
                </ActionForm>                

            </section>

            <section class="bg-slate-800 p-6 rounded-xl shadow-lg border border-slate-700">
                <h2 class="text-xl font-semibold mb-4 text-slate-200">"Список пользователей"</h2>
                <Suspense fallback=move || view! { <p class="text-slate-400 animate-pulse">"Загрузка пользователей..."</p> }>
                    {move || {
                        let users = users_resource.get();
                        match users {
                            Some(Ok(list)) if list.is_empty() => view! {
                                <p class="text-slate-500 italic">"База данных пока пуста."</p>
                            }.into_any(),
                            Some(Ok(list)) => view! {
                                <ul class="divide-y divide-slate-700">
                                  {list.into_iter().map(|user| {
                                    let user_id = user.id;
                                     view! {
                                         <li class="py-3 flex justify-between items-center px-2 hover:bg-slate-700/50 rounded-lg transition-colors">
                                            <div class="space-y-0.5">
                                               <p class="font-medium text-slate-200">{user.name}</p>
                                                  <p class="text-sm text-slate-400">{user.email}</p>
                                             </div>
                                             <div class="flex items-center space-x-3">
                                               <span class="text-xs px-2.5 py-1 bg-slate-700 text-indigo-300 rounded-full font-mono">
                                                 "ID: "{user.id}
                                               </span>
                                               <ActionForm action=delete_user_action>
                                                 <input type="hidden" name="id" value=user_id.to_string() />
                                                 <button
                                                  type="submit"
                                                     class="px-3 py-1 bg-rose-600/80 hover:bg-rose-600 text-white text-xs font-medium rounded-lg transition-colors shadow cursor-pointer"
                                                 >
                                                 "Удалить"
                                                 </button>
                                                </ActionForm>
                                              </div>
                                            </li>
                                  }
                               }).collect::<Vec<_>>()}
                              </ul>
                            }.into_any(),
                            Some(Err(e)) => view! {
                                <p class="text-red-400">"Ошибка загрузки: "{e.to_string()}</p>
                            }.into_any(),
                            None => view! {
                                <p class="text-slate-400">"Загрузка..."</p>
                            }.into_any(),
                        }
                    }}
                </Suspense>
            </section>
        </main>
    }
}
    

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(crate::App);
}
