use chrono::Utc;
use clap::Parser;
use rand::{Rng, RngExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::interval;
use uuid::Uuid;

const BACKEND_URL: &str = "http://127.0.0.1:8080";

#[derive(Parser, Debug)]
#[command(
    name = "log-generator",
    about = "Realistic log generator for the Rust Log Analytics System"
)]
struct Args {
    /// Number of logs to generate per second
    #[arg(short, long, default_value_t = 10)]
    rate: u64,
}

#[derive(Debug, Deserialize, Clone)]
struct Service {
    id: Uuid,
    name: String,
    description: Option<String>,
}

#[derive(Debug, Serialize)]
struct LogRequest {
    timestamp: String,
    level: String,
    service_id: Uuid,
    method: String,
    endpoint: String,
    status_code: i32,
    response_time_ms: i64,
    ip_address: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Clone)]
struct ServiceProfile {
    service: Service,
    endpoints: Vec<EndpointProfile>,
}

#[derive(Debug, Clone)]
struct EndpointProfile {
    method: &'static str,
    path: &'static str,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.rate == 0 {
        eprintln!("❌ Rate must be greater than 0.");
        return;
    }

    println!("🚀 Rust Real-Time Log Generator");
    println!("--------------------------------");
    println!("Backend: {}", BACKEND_URL);
    println!("Rate:    {} logs/sec", args.rate);

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to create HTTP client");

    // --------------------------------------------------
    // Fetch services from our Rust backend
    // --------------------------------------------------

    let services = match fetch_services(&client).await {
        Ok(services) => services,

        Err(error) => {
            eprintln!("❌ Failed to fetch services: {}", error);
            eprintln!("Make sure the backend is running on port 8080.");
            return;
        }
    };

    if services.is_empty() {
        eprintln!("❌ No services found in PostgreSQL.");
        return;
    }

    println!("\n✅ Loaded {} services:", services.len());

    for service in &services {
        println!("   • {} - {}", service.name, service.id);
    }

    // --------------------------------------------------
    // Create endpoint profiles
    // --------------------------------------------------

    let profiles = build_service_profiles(services);

    println!("\n📡 Starting log generation...\n");

    // --------------------------------------------------
    // Generate logs at requested rate
    // --------------------------------------------------

    let interval_duration =
        Duration::from_secs_f64(1.0 / args.rate as f64);

    let mut ticker = interval(interval_duration);

    let mut generated_count: u64 = 0;
    let mut successful_count: u64 = 0;
    let mut failed_count: u64 = 0;

    loop {
        ticker.tick().await;

        let log = generate_log(&profiles);

        match send_log(&client, &log).await {
            Ok(status) if status.is_success() => {
                successful_count += 1;
            }

            Ok(status) => {
                failed_count += 1;

                eprintln!(
                    "❌ Backend rejected log: HTTP {}",
                    status
                );
            }

            Err(error) => {
                failed_count += 1;

                eprintln!(
                    "❌ Failed to send log: {}",
                    error
                );
            }
        }

        generated_count += 1;

        // Print progress every 100 logs
        if generated_count % 100 == 0 {
            println!(
                "📊 Generated: {:>8} | Successful: {:>8} | Failed: {:>6}",
                generated_count,
                successful_count,
                failed_count
            );
        }
    }
}

// ======================================================
// Fetch services from backend
// ======================================================

async fn fetch_services(
    client: &Client,
) -> Result<Vec<Service>, reqwest::Error> {
    let response = client
        .get(format!("{}/api/services", BACKEND_URL))
        .send()
        .await?
        .error_for_status()?;

    response.json::<Vec<Service>>().await
}

// ======================================================
// Build service-specific endpoint profiles
// ======================================================

fn build_service_profiles(
    services: Vec<Service>,
) -> Vec<ServiceProfile> {
    services
        .into_iter()
        .map(|service| {
            let endpoints = match service.name.as_str() {
                "auth-service" => vec![
                    EndpointProfile {
                        method: "POST",
                        path: "/login",
                    },
                    EndpointProfile {
                        method: "POST",
                        path: "/logout",
                    },
                    EndpointProfile {
                        method: "POST",
                        path: "/refresh",
                    },
                    EndpointProfile {
                        method: "POST",
                        path: "/verify",
                    },
                    EndpointProfile {
                        method: "POST",
                        path: "/forgot-password",
                    },
                ],

                "payment-service" => vec![
                    EndpointProfile {
                        method: "POST",
                        path: "/payment",
                    },
                    EndpointProfile {
                        method: "POST",
                        path: "/refund",
                    },
                    EndpointProfile {
                        method: "GET",
                        path: "/invoice",
                    },
                    EndpointProfile {
                        method: "POST",
                        path: "/checkout",
                    },
                    EndpointProfile {
                        method: "POST",
                        path: "/verify",
                    },
                ],

                "order-service" => vec![
                    EndpointProfile {
                        method: "GET",
                        path: "/orders",
                    },
                    EndpointProfile {
                        method: "GET",
                        path: "/orders/12345",
                    },
                    EndpointProfile {
                        method: "POST",
                        path: "/orders",
                    },
                    EndpointProfile {
                        method: "PUT",
                        path: "/orders/12345",
                    },
                    EndpointProfile {
                        method: "DELETE",
                        path: "/orders/12345",
                    },
                    EndpointProfile {
                        method: "GET",
                        path: "/orders/status",
                    },
                ],

                "user-service" => vec![
                    EndpointProfile {
                        method: "GET",
                        path: "/users",
                    },
                    EndpointProfile {
                        method: "GET",
                        path: "/users/12345",
                    },
                    EndpointProfile {
                        method: "GET",
                        path: "/profile",
                    },
                    EndpointProfile {
                        method: "PUT",
                        path: "/profile",
                    },
                    EndpointProfile {
                        method: "DELETE",
                        path: "/users/12345",
                    },
                ],

                // Fallback for any future service
                _ => vec![
                    EndpointProfile {
                        method: "GET",
                        path: "/health",
                    },
                    EndpointProfile {
                        method: "GET",
                        path: "/api/data",
                    },
                    EndpointProfile {
                        method: "POST",
                        path: "/api/data",
                    },
                ],
            };

            ServiceProfile {
                service,
                endpoints,
            }
        })
        .collect()
}

// ======================================================
// Generate one realistic log
// ======================================================

fn generate_log(
    profiles: &[ServiceProfile],
) -> LogRequest {
    let mut rng = rand::rng();

    // ----------------------------------------------
    // Random service
    // ----------------------------------------------

    let service_index =
        rng.random_range(0..profiles.len());

    let profile = &profiles[service_index];

    // ----------------------------------------------
    // Random endpoint
    // ----------------------------------------------

    let endpoint_index =
        rng.random_range(0..profile.endpoints.len());

    let endpoint =
        &profile.endpoints[endpoint_index];

    // ----------------------------------------------
    // Status code
    //
    // Normal distribution:
    //
    // 200 -> 60%
    // 201 -> 15%
    // 400 -> 10%
    // 404 -> 5%
    // 500 -> 8%
    // 503 -> 2%
    // ----------------------------------------------

    let status_code = random_status_code(&mut rng);

    // ----------------------------------------------
    // Level based on status
    // ----------------------------------------------

    let level = level_from_status(status_code, &mut rng);

    // ----------------------------------------------
    // Response latency
    //
    // Fast   : 20 - 300 ms
    // Normal : 300 - 1500 ms
    // Slow   : 1500 - 5000 ms
    // ----------------------------------------------

    let response_time_ms =
        random_latency(&mut rng);

    // ----------------------------------------------
    // Random IP
    // ----------------------------------------------

    let ip_address =
        Some(random_ip(&mut rng));

    // ----------------------------------------------
    // Message
    // ----------------------------------------------

    let message = generate_message(
        &profile.service.name,
        endpoint.path,
        status_code,
    );

    LogRequest {
        timestamp: Utc::now().to_rfc3339(),

        level,

        service_id: profile.service.id,

        method: endpoint.method.to_string(),

        endpoint: endpoint.path.to_string(),

        status_code,

        response_time_ms,

        ip_address,

        message: Some(message),
    }
}

// ======================================================
// Status code generator
// ======================================================

fn random_status_code(
    rng: &mut impl Rng,
) -> i32 {
    let value = rng.random_range(0..100);

    match value {
        0..=59 => 200,
        60..=74 => 201,
        75..=84 => 400,
        85..=89 => 404,
        90..=97 => 500,
        _ => 503,
    }
}

// ======================================================
// Log level generator
// ======================================================

fn level_from_status(
    status_code: i32,
    rng: &mut impl Rng,
) -> String {
    match status_code {
        500 | 503 => "ERROR".to_string(),

        400 | 404 => {
            // Some client errors are WARN,
            // occasional DEBUG for testing.
            if rng.random_range(0..100) < 90 {
                "WARN".to_string()
            } else {
                "DEBUG".to_string()
            }
        }

        200 | 201 => {
            let value = rng.random_range(0..100);

            match value {
                0..=69 => "INFO",
                70..=89 => "DEBUG",
                _ => "INFO",
            }
            .to_string()
        }

        _ => "INFO".to_string(),
    }
}

// ======================================================
// Latency generator
// ======================================================

fn random_latency(
    rng: &mut impl Rng,
) -> i64 {
    let distribution = rng.random_range(0..100);

    match distribution {
        // 65% fast requests
        0..=64 => rng.random_range(20..=300),

        // 25% normal requests
        65..=89 => rng.random_range(301..=1500),

        // 10% slow requests
        _ => rng.random_range(1501..=5000),
    }
}

// ======================================================
// IP generator
// ======================================================

fn random_ip(
    rng: &mut impl Rng,
) -> String {
    format!(
        "192.168.1.{}",
        rng.random_range(1..=254)
    )
}

// ======================================================
// Log message generator
// ======================================================

fn generate_message(
    service: &str,
    endpoint: &str,
    status_code: i32,
) -> String {
    match status_code {
        200 => format!(
            "{} request to {} completed successfully",
            service, endpoint
        ),

        201 => format!(
            "{} created a new resource at {}",
            service, endpoint
        ),

        400 => format!(
            "Bad request received by {} at {}",
            service, endpoint
        ),

        404 => format!(
            "Resource not found in {} at {}",
            service, endpoint
        ),

        500 => format!(
            "Internal server error in {} at {}",
            service, endpoint
        ),

        503 => format!(
            "{} temporarily unavailable at {}",
            service, endpoint
        ),

        _ => format!(
            "{} processed request at {}",
            service, endpoint
        ),
    }
}

// ======================================================
// Send log to backend
// ======================================================

async fn send_log(
    client: &Client,
    log: &LogRequest,
) -> Result<reqwest::StatusCode, reqwest::Error> {
    let response = client
        .post(format!("{}/api/logs", BACKEND_URL))
        .json(log)
        .send()
        .await?;

    Ok(response.status())
}