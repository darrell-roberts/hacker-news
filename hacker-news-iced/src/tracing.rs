//! Setup open telemetry tracing to export to Jaeger.
use opentelemetry::{trace::TracerProvider as _, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    trace::{BatchConfigBuilder, RandomIdGenerator, Sampler, Tracer, TracerProvider},
    Resource,
};
use std::{convert::Infallible, future::pending, sync::OnceLock};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, Registry};

#[derive(Debug)]
struct LogHandle {
    _provider: Option<TracerProvider>,
}

static LOGGER: OnceLock<LogHandle> = OnceLock::new();

pub fn start_runtime<R>(f: impl FnOnce() -> R) -> R {
    let rt = tokio::runtime::Builder::new_current_thread()
        .thread_name("tokio-tracing")
        .enable_all()
        .build()
        .unwrap();

    let guard = rt.enter();
    let ret = f();
    drop(guard);

    std::thread::Builder::new()
        .name("tracing-exporter".into())
        // Run this runtime effectively forever
        .spawn(move || rt.block_on(pending::<Infallible>()))
        .expect("failed to spawn thread");

    ret
}

pub fn init_tracing() -> anyhow::Result<()> {
    let init_tracing = start_runtime(|| {
        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint("http://127.0.0.1:4317"),
            )
            .with_batch_config(
                BatchConfigBuilder::default()
                    .with_max_concurrent_exports(4)
                    .with_scheduled_delay(std::time::Duration::from_secs(2))
                    .build(),
            )
            .with_trace_config(
                opentelemetry_sdk::trace::Config::default()
                    .with_sampler(Sampler::AlwaysOn)
                    .with_id_generator(RandomIdGenerator::default())
                    .with_resource(Resource::new(vec![KeyValue::new(
                        "service.name",
                        "hacker-news",
                    )])),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .map(|provider| {
                let layer: OpenTelemetryLayer<Registry, Tracer> =
                    tracing_opentelemetry::layer().with_tracer(provider.tracer("hacker-news"));
                (Some(provider), layer)
            })
    });

    let (provider, layer) = init_tracing?;
    let s = tracing_subscriber::registry().with(layer);
    tracing::dispatcher::set_global_default(s.into())?;
    LOGGER
        .set(LogHandle {
            _provider: provider,
        })
        .unwrap();
    Ok(())
}
