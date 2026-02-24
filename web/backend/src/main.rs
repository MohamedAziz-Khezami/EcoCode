use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
    response::IntoResponse,
    routing::{get},
    Json, Router,
};
use tower_http::cors::{Any, CorsLayer};
use sqlx::sqlite::SqlitePoolOptions;
use serde::Serialize;
use tokio::{time::Duration};

// use axum_extra::TypedHeader;
use futures::stream::{Stream, StreamExt};
use std::{convert::Infallible, vec};

use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

#[derive(Serialize, Clone, sqlx::FromRow)]
struct Project {
    id: i64,
    name: String,
}
#[derive(Serialize, Clone, sqlx::FromRow)]
struct Run {
    id: i64,
    name: String,
    project_id: i64,
}

#[derive(Serialize, Clone, sqlx::FromRow)]
struct RecordPoint {
    id: i64,
    run_id: i64,
    pid: i64,
    timestamp: String,
    cpu_usage: f64,
    cpu_energy: f64,
    gpu_usage: f64,
    gpu_energy: f64,
    mem_usage: f64,
    mem_energy: f64,
    igpu_usage: f64,
    igpu_energy: f64,
}

#[derive(Serialize, Clone, sqlx::FromRow)]
struct RunSummary {
    run_id: i64,
    total_cpu_energy: f64,
    total_gpu_energy: f64,
    total_mem_energy: f64,
    total_igpu_energy: f64,
}


#[derive(Clone)]
struct AppState {
    db_pool: sqlx::SqlitePool,
       tx: broadcast::Sender<String>,
}

// --- REST API ---

async fn get_projects(State(state): State<AppState>) -> impl IntoResponse {
    let projects: Vec<Project> = sqlx::query_as::<_, Project>("SELECT id, name FROM projects")
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_else(|_| vec![]); // Graceful error handling

    match projects.len() {
        0 => (StatusCode::NOT_FOUND, "No projects found").into_response(),
        _ => (StatusCode::OK, Json(projects)).into_response(),
    }
}


async fn get_runs(Path(project_id): Path<i64>, State(state): State<AppState>) -> impl IntoResponse {
    let runs: Vec<Run> = sqlx::query_as::<_, Run>(
        "SELECT id, name, project_id FROM runs WHERE project_id = ?",
    )
    .bind(project_id)
    .fetch_all(&state.db_pool)
    .await
    .unwrap_or_else(|_| vec![]); // Graceful error handling

    match runs.len() {
        0 => (StatusCode::NOT_FOUND, "No runs found for this project").into_response(),
        _ => (StatusCode::OK, Json(runs)).into_response(),
    }
}


async fn get_record_points(Path(run_id): Path<i64>, State(state): State<AppState>) -> impl IntoResponse {
    let record_points: Vec<RecordPoint> = sqlx::query_as::<_, RecordPoint>(
        "SELECT id, run_id, pid, timestamp, cpu_usage, cpu_energy, gpu_usage, gpu_energy, mem_usage, mem_energy, igpu_usage, igpu_energy FROM records WHERE run_id = ?",
    )
    .bind(run_id)
    .fetch_all(&state.db_pool)
    .await
    .unwrap(); // Graceful error handling

    match record_points.len() {
        0 => (StatusCode::NOT_FOUND, "No record points found for this run").into_response(),
        _ => (StatusCode::OK, Json(record_points)).into_response(),
    }
}

async fn get_run_summary(Path(run_id): Path<i64>, State(state): State<AppState>) -> impl IntoResponse {
    let summary: Option<RunSummary> = sqlx::query_as::<_, RunSummary>(
        "SELECT run_id, SUM(cpu_energy) AS total_cpu_energy, SUM(gpu_energy) AS total_gpu_energy, SUM(mem_energy) AS total_mem_energy, SUM(igpu_energy) AS total_igpu_energy FROM records WHERE run_id = ? GROUP BY run_id",
    )
    .bind(run_id)
    .fetch_optional(&state.db_pool)
    .await
    .unwrap(); // Graceful error handling

    match summary {
        Some(s) => (StatusCode::OK, Json(s)).into_response(),
        None => (StatusCode::NOT_FOUND, "Run not found").into_response(),
    }
}

// --- SSE ---



// --- Routes ---
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the database connection pool
    let db_url = "sqlite:///home/mak/Desktop/ecocode/ecocode/ecocodeDB.db";
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    let (tx, _) = broadcast::channel::<String>(100);

    let state = AppState {
        db_pool: db.clone(),
        tx: tx.clone(),
    };
        // 3. Spawn the background watcher task
    tokio::spawn(watch_db_changes(db, tx));

    // Initialize the application state with the database connection pool
    let app = Router::new()
        .route("/api/hello", get(hello_world))
        .route("/api/projects", get(get_projects))
        .route("/api/project/{project_id}/runs", get(get_runs))
        .route("/api/run/{run_id}/record_points", get(get_record_points))   
        .route("/api/run/{run_id}/summary", get(get_run_summary))
        .route("/api/sse", get(sse_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    // Run the app with axum, listening globally on port 3001
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(listener, app).await?;

    Ok(())
}


/// The SSE handler: Subscribes to the broadcast channel and streams messages to the client
async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();
    
    let stream = BroadcastStream::new(rx).filter_map(|res| async move {
        match res {
            Ok(msg) => Some(Ok(Event::default().data(msg))),
            Err(_) => None, // Ignore lagged/closed errors
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}



/// Background Task: Polls SQLite for version changes
async fn watch_db_changes(db: sqlx::SqlitePool, tx: broadcast::Sender<String>) {
    let mut last_version: i32 = 0;

    // Initialize version to current state
    if let Ok(row) = sqlx::query_as::<_, (i32,)>("PRAGMA user_version").fetch_one(&db).await {
        last_version = row.0;
    }

    loop {
        // Check the PRAGMA user_version (very cheap check)
        let res = sqlx::query_as::<_, (i32,)>("PRAGMA user_version")
            .fetch_one(&db)
            .await;

        if let Ok((current_version,)) = res {
            if current_version > last_version {
                // Something changed! Fetch the newest row
                let new_data: Result<RecordPoint, sqlx::Error>   = sqlx::query_as::<_, RecordPoint>("SELECT id, run_id, pid, timestamp, cpu_usage, cpu_energy, gpu_usage, gpu_energy, mem_usage, mem_energy, igpu_usage, igpu_energy FROM records ORDER BY id DESC LIMIT 1")
                    .fetch_one(&db)
                    .await;

                if let Ok(record) = new_data {
                    let _ = tx.send(format!("New Project: {}", serde_json::to_string(&record).unwrap_or_else(|_| "Failed to serialize".to_string())));
                }
                last_version = current_version;
            }
        }

        // Poll every 500ms to balance latency vs CPU usage
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}






// Simple "Hello, World!" endpoint
async fn hello_world() -> &'static str {
    "Hello, World!"
}
