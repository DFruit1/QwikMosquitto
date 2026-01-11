use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{Context, Result};
use axum::extract::Query;
use axum::routing::get;
use axum::{Json, Router};
use base64::Engine;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::time::sleep;
use tokio_postgres::NoTls;
use tracing::{error, info};

#[derive(Clone)]
struct AppState {
    db_client: tokio_postgres::Client,
}

#[derive(Serialize)]
struct MessageRow {
    id: i64,
    topic: String,
    payload_base64: String,
    received_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Deserialize)]
struct MessagesQuery {
    limit: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url = env_var("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/mqtt");
    let mqtt_host = env_var("MQTT_HOST", "localhost");
    let mqtt_port: u16 = env_var("MQTT_PORT", "1883").parse().context("MQTT_PORT must be a u16")?;
    let mqtt_client_id = env_var("MQTT_CLIENT_ID", "mqtt-recorder");
    let mqtt_topic = env_var("MQTT_TOPIC", "#");
    let app_host = env_var("APP_HOST", "0.0.0.0");
    let app_port: u16 = env_var("APP_PORT", "8080").parse().context("APP_PORT must be a u16")?;

    let (db_client, connection) = tokio_postgres::connect(&database_url, NoTls)
        .await
        .context("connect to postgres")?;

    tokio::spawn(async move {
        if let Err(error) = connection.await {
            error!(?error, "postgres connection error");
        }
    });

    db_client
        .execute(
            "CREATE TABLE IF NOT EXISTS mqtt_messages (\
            id BIGSERIAL PRIMARY KEY,\
            topic TEXT NOT NULL,\
            payload BYTEA NOT NULL,\
            received_at TIMESTAMPTZ NOT NULL DEFAULT NOW()\
            )",
            &[],
        )
        .await
        .context("create mqtt_messages table")?;

    let state = AppState { db_client };
    let mqtt_state = state.clone();

    tokio::spawn(async move {
        if let Err(error) = run_mqtt_loop(mqtt_state, mqtt_client_id, mqtt_host, mqtt_port, mqtt_topic).await {
            error!(?error, "mqtt loop exited");
        }
    });

    let app = Router::new()
        .route("/health", get(health))
        .route("/messages", get(messages))
        .with_state(state);

    let addr: SocketAddr = format!("{app_host}:{app_port}")
        .parse()
        .context("parse APP_HOST/APP_PORT")?;

    info!(%addr, "starting axum server");
    let listener = TcpListener::bind(addr).await.context("bind http listener")?;
    axum::serve(listener, app).await.context("serve http")?;

    Ok(())
}

async fn run_mqtt_loop(
    state: AppState,
    mqtt_client_id: String,
    mqtt_host: String,
    mqtt_port: u16,
    mqtt_topic: String,
) -> Result<()> {
    let mut mqtt_options = MqttOptions::new(mqtt_client_id, mqtt_host, mqtt_port);
    mqtt_options.set_keep_alive(Duration::from_secs(30));

    let (mqtt_client, mut event_loop) = AsyncClient::new(mqtt_options, 10);
    mqtt_client
        .subscribe(mqtt_topic.clone(), QoS::AtLeastOnce)
        .await
        .with_context(|| format!("subscribe to {mqtt_topic}"))?;

    info!("listening for mqtt messages");

    loop {
        match event_loop.poll().await {
            Ok(Event::Incoming(Incoming::Publish(publish))) => {
                let payload = publish.payload.to_vec();
                if let Err(error) = state
                    .db_client
                    .execute(
                        "INSERT INTO mqtt_messages (topic, payload) VALUES ($1, $2)",
                        &[&publish.topic, &payload],
                    )
                    .await
                {
                    error!(?error, "insert mqtt message");
                } else {
                    info!(topic = %publish.topic, payload_len = payload.len(), "message recorded");
                }
            }
            Ok(_) => {}
            Err(error) => {
                error!(?error, "mqtt event loop error");
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn messages(
    axum::extract::State(state): axum::extract::State<AppState>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<Vec<MessageRow>>, axum::http::StatusCode> {
    let limit = query.limit.unwrap_or(50).clamp(1, 500) as i64;
    let rows = state
        .db_client
        .query(
            "SELECT id, topic, payload, received_at FROM mqtt_messages ORDER BY received_at DESC LIMIT $1",
            &[&limit],
        )
        .await
        .map_err(|error| {
            error!(?error, "query mqtt messages");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let encoder = base64::engine::general_purpose::STANDARD;
    let messages = rows
        .into_iter()
        .map(|row| {
            let payload: Vec<u8> = row.get("payload");
            MessageRow {
                id: row.get("id"),
                topic: row.get("topic"),
                payload_base64: encoder.encode(payload),
                received_at: row.get("received_at"),
            }
        })
        .collect();

    Ok(Json(messages))
}

fn env_var(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}
