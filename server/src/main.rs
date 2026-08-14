use axum::{
    extract::{Json, State},
    routing::{get, post},
    Router,
};
use mneme_core::{
    AccessScope, Embedder, HashEmbedder, MemoryStore, MemoryType, RecallOptions, SqliteBackend,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use fastembed::TextEmbedding;

struct FastEmbedder {
    model: TextEmbedding,
}

impl FastEmbedder {
    fn new() -> anyhow::Result<Self> {
        let model = TextEmbedding::try_new(Default::default())?;
        Ok(Self { model })
    }
}

impl Embedder for FastEmbedder {
    fn embed(&self, text: &str) -> Vec<f32> {
        match self.model.embed(vec![text], None) {
            Ok(embeddings) => {
                if let Some(first) = embeddings.into_iter().next() {
                    first
                } else {
                    vec![0.0; 384]
                }
            }
            Err(_) => vec![0.0; 384],
        }
    }
}

#[derive(Deserialize)]
struct RememberRequest {
    content: String,
    memory_type: String,
    user_id: Option<String>,
    session_id: Option<String>,
    tags: Option<Vec<String>>,
    confidence: Option<f32>,
    ttl: Option<i64>,
    embedding: Option<Vec<f32>>,
}

#[derive(Serialize)]
struct RememberResponse {
    id: String,
    content: String,
    memory_type: String,
}

#[derive(Deserialize)]
struct RecallRequest {
    query: String,
    limit: Option<usize>,
    token_budget: Option<usize>,
    query_embedding: Option<Vec<f32>>,
}

#[derive(Serialize)]
struct RecallItem {
    content: String,
    score: f32,
    explanation: String,
    memory_id: String,
}

#[derive(Deserialize)]
struct ForgetRequest {
    memory_id: String,
}

#[derive(Deserialize)]
struct ExportRequest {
    path: String,
}

#[derive(Deserialize)]
struct ImportRequest {
    path: String,
}

#[derive(Deserialize)]
struct ForgetAllRequest {
    user_id: String,
}

#[derive(Serialize)]
struct StatusResponse {
    status: String,
}

type SharedStore = Arc<Mutex<MemoryStore>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let embedder: Arc<dyn Embedder> = match FastEmbedder::new() {
        Ok(embedder) => Arc::new(embedder),
        Err(e) => {
            eprintln!("Failed to load FastEmbed model: {}. Falling back to HashEmbedder.", e);
            Arc::new(HashEmbedder::new(384)) as Arc<dyn Embedder>
        }
    };

    let backend = SqliteBackend::new("mneme_server.db").await.unwrap();
    let store = MemoryStore::new("server-agent", Arc::new(backend), embedder).await;
    let shared: SharedStore = Arc::new(Mutex::new(store));

    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route("/remember", post(remember))
        .route("/recall", post(recall))
        .route("/forget", post(forget))
        .route("/advanced/export", post(export))
        .route("/advanced/import", post(import))
        .route("/advanced/forget_all", post(forget_all))
        .route("/advanced/audit_log", get(audit_log))
        .layer(cors)
        .with_state(shared);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();
    tracing::info!("Mneme server listening on http://127.0.0.1:8000");
    axum::serve(listener, app).await.unwrap();
}

async fn remember(
    State(store): State<SharedStore>,
    Json(payload): Json<RememberRequest>,
) -> Json<RememberResponse> {
    let store = store.lock().await;
    let memory_type = match payload.memory_type.as_str() {
        "episodic" => MemoryType::Episodic,
        "semantic" => MemoryType::Semantic,
        _ => MemoryType::Episodic,
    };
    let record = store
        .remember(
            &payload.content,
            memory_type,
            payload.user_id.as_deref().unwrap_or(""),
            payload.session_id.as_deref().unwrap_or(""),
            "http",
            payload.confidence.unwrap_or(1.0),
            payload.ttl,
            payload.tags.unwrap_or_default(),
            AccessScope::default(),
            1.0,
            payload.embedding,
        )
        .await
        .unwrap();
    Json(RememberResponse {
        id: record.id,
        content: record.content,
        memory_type: record.memory_type.to_string(),
    })
}

async fn recall(
    State(store): State<SharedStore>,
    Json(payload): Json<RecallRequest>,
) -> Json<Vec<RecallItem>> {
    let store = store.lock().await;
    let options = RecallOptions {
        limit: payload.limit.unwrap_or(5),
        token_budget: payload.token_budget.unwrap_or(500),
        ..Default::default()
    };
    let results = store
        .recall(&payload.query, options, payload.query_embedding)
        .await
        .unwrap();
    let items = results
        .into_iter()
        .map(|rm| RecallItem {
            content: rm.record.content,
            score: rm.score,
            explanation: rm.explanation,
            memory_id: rm.record.id,
        })
        .collect();
    Json(items)
}

async fn forget(
    State(store): State<SharedStore>,
    Json(payload): Json<ForgetRequest>,
) -> Json<StatusResponse> {
    let store = store.lock().await;
    store.forget(&payload.memory_id).await.unwrap();
    Json(StatusResponse { status: "ok".into() })
}

async fn export(
    State(store): State<SharedStore>,
    Json(payload): Json<ExportRequest>,
) -> Json<StatusResponse> {
    let store = store.lock().await;
    store.export(&payload.path).await.unwrap();
    Json(StatusResponse { status: "ok".into() })
}

async fn import(
    State(store): State<SharedStore>,
    Json(payload): Json<ImportRequest>,
) -> Json<StatusResponse> {
    let store = store.lock().await;
    store.import_from(&payload.path).await.unwrap();
    Json(StatusResponse { status: "ok".into() })
}

async fn forget_all(
    State(store): State<SharedStore>,
    Json(payload): Json<ForgetAllRequest>,
) -> Json<StatusResponse> {
    let store = store.lock().await;
    store.forget_all(&payload.user_id).await.unwrap();
    Json(StatusResponse { status: "ok".into() })
}

async fn audit_log(
    State(store): State<SharedStore>,
) -> Json<Vec<mneme_core::AuditEvent>> {
    let store = store.lock().await;
    let log = store.get_audit_log().await.unwrap();
    Json(log)
}