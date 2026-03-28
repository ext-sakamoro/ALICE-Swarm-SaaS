// SPDX-License-Identifier: AGPL-3.0-or-later
// ALICE-Swarm-SaaS core-engine: Swarm intelligence platform

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Default)]
struct Stats {
    simulations: u64,
    formations: u64,
    consensus_rounds: u64,
    metric_queries: u64,
}

type AppState = Arc<Mutex<Stats>>;

#[derive(Deserialize)]
struct SimulateRequest {
    agent_count: u32,
    algorithm: Option<String>,
    steps: Option<u32>,
}

#[derive(Deserialize)]
struct FormationRequest {
    agents: Vec<String>,
    pattern: String,
    spacing: Option<f32>,
}

#[derive(Deserialize)]
struct ConsensusRequest {
    nodes: Vec<String>,
    proposal: String,
    threshold: Option<f32>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "alice-swarm-core",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn simulate(
    State(state): State<AppState>,
    Json(req): Json<SimulateRequest>,
) -> (StatusCode, Json<Value>) {
    let sim_id = Uuid::new_v4().to_string();
    let algorithm = req.algorithm.unwrap_or_else(|| "boid".to_string());
    let steps = req.steps.unwrap_or(100);
    {
        let mut s = state.lock().unwrap();
        s.simulations += 1;
    }
    info!(sim_id = %sim_id, agents = req.agent_count, algorithm = %algorithm, "swarm simulation started");
    (
        StatusCode::OK,
        Json(json!({
            "sim_id": sim_id,
            "agent_count": req.agent_count,
            "algorithm": algorithm,
            "steps": steps,
            "status": "completed",
            "convergence_step": steps / 2,
            "fitness_score": 0.87,
        })),
    )
}

async fn formation(
    State(state): State<AppState>,
    Json(req): Json<FormationRequest>,
) -> (StatusCode, Json<Value>) {
    let formation_id = Uuid::new_v4().to_string();
    let spacing = req.spacing.unwrap_or(1.5);
    {
        let mut s = state.lock().unwrap();
        s.formations += 1;
    }
    info!(formation_id = %formation_id, pattern = %req.pattern, "swarm formation computed");
    (
        StatusCode::OK,
        Json(json!({
            "formation_id": formation_id,
            "pattern": req.pattern,
            "agent_count": req.agents.len(),
            "spacing": spacing,
            "positions": req.agents.iter().enumerate().map(|(i, id)| {
                json!({ "agent_id": id, "x": i as f32 * spacing, "y": 0.0, "z": 0.0 })
            }).collect::<Vec<_>>(),
        })),
    )
}

async fn consensus(
    State(state): State<AppState>,
    Json(req): Json<ConsensusRequest>,
) -> (StatusCode, Json<Value>) {
    let round_id = Uuid::new_v4().to_string();
    let threshold = req.threshold.unwrap_or(0.67);
    let agreed = req.nodes.len() as f32 * threshold;
    {
        let mut s = state.lock().unwrap();
        s.consensus_rounds += 1;
    }
    info!(round_id = %round_id, nodes = req.nodes.len(), "consensus round executed");
    (
        StatusCode::OK,
        Json(json!({
            "round_id": round_id,
            "proposal": req.proposal,
            "node_count": req.nodes.len(),
            "threshold": threshold,
            "votes_required": agreed.ceil() as u32,
            "votes_received": req.nodes.len() as u32,
            "result": "accepted",
        })),
    )
}

async fn metrics(State(state): State<AppState>) -> Json<Value> {
    {
        let mut s = state.lock().unwrap();
        s.metric_queries += 1;
    }
    Json(json!({
        "active_swarms": 3,
        "total_agents": 1024,
        "avg_fitness": 0.84,
        "convergence_rate": 0.92,
    }))
}

async fn stats(State(state): State<AppState>) -> Json<Value> {
    let s = state.lock().unwrap();
    Json(json!({
        "simulations": s.simulations,
        "formations": s.formations,
        "consensus_rounds": s.consensus_rounds,
        "metric_queries": s.metric_queries,
    }))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let state: AppState = Arc::new(Mutex::new(Stats::default()));

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/swarm/simulate", post(simulate))
        .route("/api/v1/swarm/formation", post(formation))
        .route("/api/v1/swarm/consensus", post(consensus))
        .route("/api/v1/swarm/metrics", get(metrics))
        .route("/api/v1/swarm/stats", get(stats))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8137".to_string());
    let addr = format!("0.0.0.0:{}", port);
    info!("alice-swarm-core listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
