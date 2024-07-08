use lazy_static::lazy_static;
use prometheus::{CounterVec, HistogramOpts, HistogramVec, Opts, Registry};
use std::time::Instant;

lazy_static! {
    pub static ref REGISTRY_INSTANCE: Registry = Registry::new();
    pub static ref REQ_COUNTER_VEC: CounterVec =
        CounterVec::new(Opts::new("request_counter", "request counter"), &["method"]).unwrap();
    pub static ref METHOD_HISTOGRAM_VEC: HistogramVec = HistogramVec::new(
        HistogramOpts::new("method_cost", "method cost"),
        &["method"]
    )
    .unwrap();
}

pub fn init_registry() {
    let _ = REGISTRY_INSTANCE.register(Box::new(REQ_COUNTER_VEC.clone()));
    let _ = REGISTRY_INSTANCE.register(Box::new(METHOD_HISTOGRAM_VEC.clone()));
}

pub async fn record_metrics<F, Fut, T>(
    method_name: &'static str,
    handler: F,
) -> Result<T, tonic::Status>
where
    F: FnOnce() -> Fut + Send,
    Fut: std::future::Future<Output = Result<T, tonic::Status>> + Send,
{
    let start = Instant::now();
    REQ_COUNTER_VEC.with_label_values(&[method_name]).inc();
    let result = handler().await;

    let elapsed = start.elapsed();
    METHOD_HISTOGRAM_VEC
        .with_label_values(&[method_name])
        .observe(elapsed.as_secs_f64());

    result
}
