//! GPU-oriented metric kernels and CPU reference paths.
//!
//! This module started with Lorentz SQ8 and now provides a unified foundation
//! for GPU batch search (L2/Cosine/Poincare/Lorentz) and exact re-ranking.
//! The WGSL kernels are intentionally simple and portable so they can run
//! through `wgpu` on Metal/Vulkan/DX12 backends.

#[cfg(feature = "gpu-runtime")]
use bytemuck::{Pod, Zeroable};

/// Metric tags used by CPU reference and re-rank code paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuMetric {
    L2,
    Cosine,
    Poincare,
    Lorentz,
}

/// Execution backend selected for batch distance computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeBackend {
    Cpu,
    Gpu,
    GpuFallbackCpu,
    GpuDispatchPlanned,
}

/// Runtime decision helper: should this batch be offloaded to GPU?
///
/// This heuristic is conservative and can be tuned per deployment.
pub fn should_offload_to_gpu(batch_size: usize, dimension: usize) -> bool {
    // Conservative defaults for rerank-heavy paths:
    // GPU becomes profitable only on sufficiently large work units.
    let min_batch = env_usize("HS_GPU_MIN_BATCH", 128);
    let min_dim = env_usize("HS_GPU_MIN_DIM", 1024);
    let min_work = env_usize("HS_GPU_MIN_WORK", 262_144); // batch * dim
    let work = batch_size.saturating_mul(dimension);
    (batch_size >= min_batch && dimension >= min_dim) || work >= min_work
}

fn env_bool(name: &str) -> bool {
    std::env::var(name)
        .is_ok_and(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(default)
}

fn metric_gpu_enabled(metric: GpuMetric) -> bool {
    let key = match metric {
        GpuMetric::L2 => "HS_GPU_L2_ENABLED",
        GpuMetric::Cosine => "HS_GPU_COSINE_ENABLED",
        GpuMetric::Poincare => "HS_GPU_POINCARE_ENABLED",
        GpuMetric::Lorentz => "HS_GPU_LORENTZ_ENABLED",
    };
    if std::env::var(key).is_ok() {
        return env_bool(key);
    }
    true
}

fn batch_distance_cpu(metric: GpuMetric, vectors: &[&[f64]], query: &[f64]) -> Vec<f64> {
    match metric {
        GpuMetric::L2 => batch_l2_distance_cpu(vectors, query),
        GpuMetric::Cosine => batch_cosine_distance_cpu(vectors, query),
        GpuMetric::Poincare => batch_poincare_distance_cpu(vectors, query),
        GpuMetric::Lorentz => vectors.iter().map(|v| lorentz_distance(v, query)).collect(),
    }
}

/// Unified batch distance entrypoint with runtime auto-dispatch policy.
///
/// Today this returns CPU results for deterministic behavior, while exposing
/// a stable backend contract for future `wgpu` dispatch wiring.
pub fn batch_distance_auto(
    metric: GpuMetric,
    vectors: &[&[f64]],
    query: &[f64],
) -> (Vec<f64>, ComputeBackend) {
    let gpu_enabled = env_bool("HS_GPU_BATCH_ENABLED");
    let dimension = query.len();
    let wants_gpu = gpu_enabled
        && metric_gpu_enabled(metric)
        && should_offload_to_gpu(vectors.len(), dimension);
    if wants_gpu {
        #[cfg(feature = "gpu-runtime")]
        {
            match batch_distance_gpu_wgpu(metric, vectors, query) {
                Ok(dist) => return (dist, ComputeBackend::Gpu),
                Err(_e) => {
                    return (
                        batch_distance_cpu(metric, vectors, query),
                        ComputeBackend::GpuFallbackCpu,
                    );
                }
            }
        }
        #[cfg(not(feature = "gpu-runtime"))]
        {
            // Build without gpu-runtime feature: expose planned backend state.
            return (
                batch_distance_cpu(metric, vectors, query),
                ComputeBackend::GpuDispatchPlanned,
            );
        }
    }
    (
        batch_distance_cpu(metric, vectors, query),
        ComputeBackend::Cpu,
    )
}

#[cfg(feature = "gpu-runtime")]
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuParams {
    num_vectors: u32,
    dimension: u32,
    pad0: u32,
    pad1: u32,
}

#[cfg(feature = "gpu-runtime")]
struct GpuRuntime {
    device: wgpu::Device,
    queue: wgpu::Queue,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_l2: wgpu::ComputePipeline,
    pipeline_cosine: wgpu::ComputePipeline,
    pipeline_poincare: wgpu::ComputePipeline,
    pipeline_lorentz: wgpu::ComputePipeline,
    scratch_pool: std::sync::Mutex<Vec<GpuScratch>>,
}

#[cfg(feature = "gpu-runtime")]
struct GpuScratch {
    vectors: wgpu::Buffer,
    params: wgpu::Buffer,
    query: wgpu::Buffer,
    output: wgpu::Buffer,
    readback: wgpu::Buffer,
    vectors_f32_capacity: usize,
    query_f32_capacity: usize,
    output_f32_capacity: usize,
}

#[cfg(feature = "gpu-runtime")]
impl GpuRuntime {
    fn pipeline_for_metric(&self, metric: GpuMetric) -> &wgpu::ComputePipeline {
        match metric {
            GpuMetric::L2 => &self.pipeline_l2,
            GpuMetric::Cosine => &self.pipeline_cosine,
            GpuMetric::Poincare => &self.pipeline_poincare,
            GpuMetric::Lorentz => &self.pipeline_lorentz,
        }
    }

    fn create_scratch(
        &self,
        vectors_f32_capacity: usize,
        query_f32_capacity: usize,
        output_f32_capacity: usize,
    ) -> GpuScratch {
        let vectors = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hs_gpu_vectors"),
            size: (vectors_f32_capacity * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let params = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hs_gpu_params"),
            size: std::mem::size_of::<GpuParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let query = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hs_gpu_query"),
            size: (query_f32_capacity * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let output_size = (output_f32_capacity * std::mem::size_of::<f32>()) as u64;
        let output = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hs_gpu_output"),
            size: output_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hs_gpu_readback"),
            size: output_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        GpuScratch {
            vectors,
            params,
            query,
            output,
            readback,
            vectors_f32_capacity,
            query_f32_capacity,
            output_f32_capacity,
        }
    }

    fn acquire_scratch(
        &self,
        vectors_f32_required: usize,
        query_f32_required: usize,
        output_f32_required: usize,
    ) -> GpuScratch {
        let mut pool = self
            .scratch_pool
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(pos) = pool.iter().position(|s| {
            s.vectors_f32_capacity >= vectors_f32_required
                && s.query_f32_capacity >= query_f32_required
                && s.output_f32_capacity >= output_f32_required
        }) {
            return pool.swap_remove(pos);
        }
        drop(pool);
        self.create_scratch(
            vectors_f32_required,
            query_f32_required,
            output_f32_required,
        )
    }

    fn release_scratch(&self, scratch: GpuScratch) {
        let mut pool = self
            .scratch_pool
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Keep a small bounded pool to avoid unbounded memory growth under spikes.
        if pool.len() < 8 {
            pool.push(scratch);
        }
    }
}

#[cfg(feature = "gpu-runtime")]
static GPU_RUNTIME: std::sync::OnceLock<Result<GpuRuntime, String>> = std::sync::OnceLock::new();

#[cfg(feature = "gpu-runtime")]
fn kernel_for_metric(metric: GpuMetric) -> &'static str {
    match metric {
        GpuMetric::L2 => L2_DISTANCE_WGSL,
        GpuMetric::Cosine => COSINE_DISTANCE_WGSL,
        GpuMetric::Poincare => POINCARE_DISTANCE_WGSL,
        GpuMetric::Lorentz => LORENTZ_FLOAT_DISTANCE_WGSL,
    }
}

#[cfg(feature = "gpu-runtime")]
fn init_gpu_runtime() -> Result<GpuRuntime, String> {
    pollster::block_on(async {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| format!("request_adapter failed: {e}"))?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("hs_gpu_batch_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|e| format!("request_device failed: {e}"))?;

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("hs_gpu_bind_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("hs_gpu_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });
        let make_pipeline = |label: &str, kernel: &str| {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(kernel.into()),
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(label),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                cache: None,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            })
        };

        Ok(GpuRuntime {
            pipeline_l2: make_pipeline("hs_gpu_pipeline_l2", L2_DISTANCE_WGSL),
            pipeline_cosine: make_pipeline("hs_gpu_pipeline_cosine", COSINE_DISTANCE_WGSL),
            pipeline_poincare: make_pipeline("hs_gpu_pipeline_poincare", POINCARE_DISTANCE_WGSL),
            pipeline_lorentz: make_pipeline("hs_gpu_pipeline_lorentz", LORENTZ_FLOAT_DISTANCE_WGSL),
            device,
            queue,
            bind_group_layout,
            scratch_pool: std::sync::Mutex::new(Vec::new()),
        })
    })
}

#[cfg(feature = "gpu-runtime")]
fn gpu_runtime() -> Result<&'static GpuRuntime, String> {
    match GPU_RUNTIME.get_or_init(init_gpu_runtime) {
        Ok(runtime) => Ok(runtime),
        Err(err) => Err(err.clone()),
    }
}

#[cfg(feature = "gpu-runtime")]
#[allow(clippy::too_many_lines)]
fn batch_distance_gpu_wgpu(
    metric: GpuMetric,
    vectors: &[&[f64]],
    query: &[f64],
) -> Result<Vec<f64>, String> {
    let _kernel = kernel_for_metric(metric);
    let runtime = gpu_runtime()?;
    let device = &runtime.device;
    let queue = &runtime.queue;

    if vectors.is_empty() {
        return Ok(Vec::new());
    }
    let dimension = query.len();
    if dimension == 0 {
        return Err("Query vector is empty".to_string());
    }
    for vec in vectors {
        if vec.len() != dimension {
            return Err("Vector dimension mismatch in batch_distance_gpu_wgpu".to_string());
        }
    }

    let num_vectors = vectors.len();
    let num_vectors_u32 =
        u32::try_from(num_vectors).map_err(|_| "Too many vectors for u32 dispatch".to_string())?;
    let dim_u32 =
        u32::try_from(dimension).map_err(|_| "Dimension too large for u32 params".to_string())?;

    let mut flat_vectors = Vec::with_capacity(num_vectors * dimension);
    for vec in vectors {
        for val in *vec {
            flat_vectors.push(*val as f32);
        }
    }
    let query_f32: Vec<f32> = query.iter().map(|v| *v as f32).collect();

    let params = GpuParams {
        num_vectors: num_vectors_u32,
        dimension: dim_u32,
        pad0: 0,
        pad1: 0,
    };
    let scratch = runtime.acquire_scratch(flat_vectors.len(), query_f32.len(), num_vectors);
    let result = (|| -> Result<Vec<f64>, String> {
        queue.write_buffer(&scratch.vectors, 0, bytemuck::cast_slice(&flat_vectors));
        queue.write_buffer(&scratch.params, 0, bytemuck::bytes_of(&params));
        queue.write_buffer(&scratch.query, 0, bytemuck::cast_slice(&query_f32));

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hs_gpu_bind_group"),
            layout: &runtime.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: scratch.vectors.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: scratch.params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: scratch.query.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: scratch.output.as_entire_binding(),
                },
            ],
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("hs_gpu_encoder"),
        });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("hs_gpu_compute_pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(runtime.pipeline_for_metric(metric));
            pass.set_bind_group(0, &bind_group, &[]);
            let groups = num_vectors_u32.div_ceil(256);
            pass.dispatch_workgroups(groups, 1, 1);
        }
        let output_size = (num_vectors * std::mem::size_of::<f32>()) as u64;
        encoder.copy_buffer_to_buffer(&scratch.output, 0, &scratch.readback, 0, output_size);
        queue.submit(std::iter::once(encoder.finish()));

        let slice = scratch.readback.slice(0..output_size);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv()
            .map_err(|_| "GPU readback channel closed".to_string())?
            .map_err(|e| format!("map_async failed: {e:?}"))?;

        let data = slice.get_mapped_range();
        let out_f32: &[f32] = bytemuck::cast_slice(&data);
        let out: Vec<f64> = out_f32.iter().map(|v| f64::from(*v)).collect();
        drop(data);
        scratch.readback.unmap();
        Ok(out)
    })();
    runtime.release_scratch(scratch);
    result
}

/// WGSL kernel for batch Euclidean (L2) distance.
pub const L2_DISTANCE_WGSL: &str = r"
struct Params { num_vectors: u32, dimension: u32, _pad0: u32, _pad1: u32, };
@group(0) @binding(0) var<storage, read> vectors: array<f32>;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read> query: array<f32>;
@group(0) @binding(3) var<storage, read_write> distances: array<f32>;
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let idx = gid.x;
  if (idx >= params.num_vectors) { return; }
  let dim = params.dimension;
  let base = idx * dim;
  var acc: f32 = 0.0;
  for (var i: u32 = 0u; i < dim; i = i + 1u) {
    let d = vectors[base + i] - query[i];
    acc = acc + d * d;
  }
  distances[idx] = sqrt(acc);
}";

/// WGSL kernel for batch cosine distance (`1 - cosine_similarity`).
pub const COSINE_DISTANCE_WGSL: &str = r"
struct Params { num_vectors: u32, dimension: u32, _pad0: u32, _pad1: u32, };
@group(0) @binding(0) var<storage, read> vectors: array<f32>;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read> query: array<f32>;
@group(0) @binding(3) var<storage, read_write> distances: array<f32>;
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let idx = gid.x;
  if (idx >= params.num_vectors) { return; }
  let dim = params.dimension;
  let base = idx * dim;
  var dotv: f32 = 0.0;
  var an: f32 = 0.0;
  var bn: f32 = 0.0;
  for (var i: u32 = 0u; i < dim; i = i + 1u) {
    let a = vectors[base + i];
    let b = query[i];
    dotv = dotv + a * b;
    an = an + a * a;
    bn = bn + b * b;
  }
  let den = max(sqrt(an) * sqrt(bn), 1e-9);
  distances[idx] = 1.0 - clamp(dotv / den, -1.0, 1.0);
}";

/// WGSL kernel for batch Poincare distance.
pub const POINCARE_DISTANCE_WGSL: &str = r"
struct Params { num_vectors: u32, dimension: u32, _pad0: u32, _pad1: u32, };
@group(0) @binding(0) var<storage, read> vectors: array<f32>;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read> query: array<f32>;
@group(0) @binding(3) var<storage, read_write> distances: array<f32>;
fn acosh_approx(x: f32) -> f32 { return log(x + sqrt(x * x - 1.0)); }
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let idx = gid.x;
  if (idx >= params.num_vectors) { return; }
  let dim = params.dimension;
  let base = idx * dim;
  var norm_u: f32 = 0.0;
  var norm_v: f32 = 0.0;
  var diff_sq: f32 = 0.0;
  for (var i: u32 = 0u; i < dim; i = i + 1u) {
    let u = vectors[base + i];
    let v = query[i];
    norm_u = norm_u + u * u;
    norm_v = norm_v + v * v;
    let d = u - v;
    diff_sq = diff_sq + d * d;
  }
  let den = max((1.0 - norm_u) * (1.0 - norm_v), 1e-9);
  let arg = max(1.0 + 2.0 * diff_sq / den, 1.0 + 1e-7);
  distances[idx] = acosh_approx(arg);
}";

/// WGSL kernel for batch Lorentz distance on float vectors (`f32` storage buffers).
pub const LORENTZ_FLOAT_DISTANCE_WGSL: &str = r"
struct Params { num_vectors: u32, dimension: u32, _pad0: u32, _pad1: u32; };
@group(0) @binding(0) var<storage, read> vectors: array<f32>;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read> query: array<f32>;
@group(0) @binding(3) var<storage, read_write> distances: array<f32>;
fn acosh_approx(x: f32) -> f32 { return log(x + sqrt(x * x - 1.0)); }
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let idx = gid.x;
  if (idx >= params.num_vectors) { return; }
  let dim = params.dimension;
  let base = idx * dim;
  var minkowski_inner: f32 = 0.0;
  for (var i: u32 = 0u; i < dim; i = i + 1u) {
    let a = vectors[base + i];
    let b = query[i];
    if (i == 0u) { minkowski_inner = minkowski_inner - a * b; }
    else { minkowski_inner = minkowski_inner + a * b; }
  }
  let arg = max(-minkowski_inner, 1.0 + 1e-7);
  distances[idx] = acosh_approx(arg);
}";

/// WGSL compute shader source for batch Lorentz SQ8 distance computation.
pub const LORENTZ_DISTANCE_WGSL: &str = r"
struct Params { num_vectors: u32, dimension: u32, _pad0: u32, _pad1: u32, };
@group(0) @binding(0) var<storage, read> quantized_data: array<i32>;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read> query: array<f32>;
@group(0) @binding(3) var<storage, read_write> distances: array<f32>;
fn acosh_approx(x: f32) -> f32 { return log(x + sqrt(x * x - 1.0)); }
fn extract_i8(packed: i32, byte_idx: u32) -> f32 {
  let shift = byte_idx * 8u;
  let masked = (packed >> shift) & 0xFF;
  let signed = select(masked, masked | i32(0xFFFFFF00), masked > 127);
  return f32(signed);
}
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
  let vec_idx = gid.x;
  if (vec_idx >= params.num_vectors) { return; }
  let dim = params.dimension;
  let i32s_per_coords = (dim + 3u) / 4u;
  let stride = i32s_per_coords + 1u;
  let base = vec_idx * stride;
  let scale = bitcast<f32>(quantized_data[base + i32s_per_coords]);
  let dequant_factor = scale / 127.0;
  var minkowski_inner: f32 = 0.0;
  for (var i: u32 = 0u; i < dim; i = i + 1u) {
    let q_val = extract_i8(quantized_data[base + i / 4u], i % 4u);
    let a_val = q_val * dequant_factor;
    let b_val = query[i];
    if (i == 0u) { minkowski_inner = minkowski_inner - a_val * b_val; }
    else { minkowski_inner = minkowski_inner + a_val * b_val; }
  }
  let arg = max(-minkowski_inner, 1.0 + 1e-7);
  distances[vec_idx] = acosh_approx(arg);
}";

fn l2_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let d = x - y;
            d * d
        })
        .sum::<f64>()
        .sqrt()
}

fn cosine_distance(a: &[f64], b: &[f64]) -> f64 {
    let dot = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f64>();
    let na = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let nb = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    let den = (na * nb).max(1e-12);
    1.0 - (dot / den).clamp(-1.0, 1.0)
}

fn poincare_distance(a: &[f64], b: &[f64]) -> f64 {
    let norm_u_sq = a.iter().map(|x| x * x).sum::<f64>();
    let norm_v_sq = b.iter().map(|x| x * x).sum::<f64>();
    let diff_sq = a
        .iter()
        .zip(b.iter())
        .map(|(u, v)| {
            let d = u - v;
            d * d
        })
        .sum::<f64>();
    let denom = ((1.0 - norm_u_sq) * (1.0 - norm_v_sq)).max(1e-9);
    let arg = (1.0 + 2.0 * diff_sq / denom).max(1.0 + 1e-12);
    arg.acosh()
}

fn lorentz_distance(a: &[f64], b: &[f64]) -> f64 {
    let mut inner = -a[0] * b[0];
    for i in 1..a.len() {
        inner += a[i] * b[i];
    }
    (-inner).max(1.0 + 1e-12).acosh()
}

/// CPU reference implementation for batch L2 distance.
pub fn batch_l2_distance_cpu(vectors: &[&[f64]], query: &[f64]) -> Vec<f64> {
    vectors.iter().map(|v| l2_distance(v, query)).collect()
}

/// CPU reference implementation for batch cosine distance.
pub fn batch_cosine_distance_cpu(vectors: &[&[f64]], query: &[f64]) -> Vec<f64> {
    vectors.iter().map(|v| cosine_distance(v, query)).collect()
}

/// CPU reference implementation for batch Poincare distance.
pub fn batch_poincare_distance_cpu(vectors: &[&[f64]], query: &[f64]) -> Vec<f64> {
    vectors
        .iter()
        .map(|v| poincare_distance(v, query))
        .collect()
}

/// CPU reference implementation for batch Lorentz SQ8 distance.
pub fn batch_lorentz_distance_cpu(
    quantized_coords: &[&[i8]],
    scales: &[f32],
    query: &[f64],
    dimension: usize,
) -> Vec<f64> {
    debug_assert_eq!(quantized_coords.len(), scales.len());
    quantized_coords
        .iter()
        .zip(scales.iter())
        .map(|(coords, &scale)| {
            let inv_127 = f64::from(scale) / 127.0;
            let mut deq = vec![0.0_f64; dimension];
            for i in 0..dimension {
                deq[i] = f64::from(coords[i]) * inv_127;
            }
            lorentz_distance(&deq, query)
        })
        .collect()
}

/// Exact re-ranking on full precision vectors.
///
/// Returns `(candidate_id, exact_distance)` sorted by ascending distance.
pub fn rerank_topk_exact(
    metric: GpuMetric,
    query: &[f64],
    candidate_ids: &[u32],
    candidate_vectors: &[&[f64]],
) -> Vec<(u32, f64)> {
    debug_assert_eq!(candidate_ids.len(), candidate_vectors.len());
    let (distances, _backend) = batch_distance_auto(metric, candidate_vectors, query);
    let mut out: Vec<(u32, f64)> = candidate_ids.iter().copied().zip(distances).collect();
    out.sort_by(|a, b| a.1.total_cmp(&b.1));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_lorentz_distance_cpu() {
        let r = 1.5_f64;
        let origin_coords: Vec<i8> = vec![127, 0, 0]; // quantized (1,0,0) with scale=1.0
        let origin_scale = 1.0_f32;

        let query = vec![r.cosh(), r.sinh(), 0.0]; // point at distance r

        let distances = batch_lorentz_distance_cpu(&[&origin_coords], &[origin_scale], &query, 3);

        let exact = r;
        let relative_error = (distances[0] - exact).abs() / exact;
        assert!(
            relative_error < 0.10,
            "Batch CPU distance error {relative_error:.4} (got {}, expected {exact})",
            distances[0]
        );
    }

    #[test]
    fn test_wgsl_shader_source_is_valid() {
        // Basic smoke test for all kernels.
        assert!(LORENTZ_DISTANCE_WGSL.contains("@compute"));
        assert!(L2_DISTANCE_WGSL.contains("@compute"));
        assert!(COSINE_DISTANCE_WGSL.contains("@compute"));
        assert!(POINCARE_DISTANCE_WGSL.contains("@compute"));
        assert!(LORENTZ_FLOAT_DISTANCE_WGSL.contains("@compute"));
    }

    #[test]
    fn test_batch_l2_distance_cpu() {
        let q = vec![0.0, 0.0, 0.0];
        let a = vec![3.0, 4.0, 0.0];
        let out = batch_l2_distance_cpu(&[&a], &q);
        assert!((out[0] - 5.0).abs() < 1e-12);
    }

    #[test]
    fn test_batch_cosine_distance_cpu() {
        let q = vec![1.0, 0.0];
        let a = vec![1.0, 0.0];
        let out = batch_cosine_distance_cpu(&[&a], &q);
        assert!(out[0].abs() < 1e-12);
    }

    #[test]
    fn test_rerank_topk_exact_sort() {
        let q = vec![0.0, 0.0];
        let ids = vec![10, 20];
        let v1 = vec![2.0, 0.0];
        let v2 = vec![1.0, 0.0];
        let ranked = rerank_topk_exact(GpuMetric::L2, &q, &ids, &[&v1, &v2]);
        assert_eq!(ranked[0].0, 20);
    }

    #[test]
    fn test_batch_distance_auto_cpu_backend_default() {
        let q = vec![0.0, 0.0];
        let a = vec![1.0, 0.0];
        let (_dist, backend) = batch_distance_auto(GpuMetric::L2, &[&a], &q);
        assert_eq!(backend, ComputeBackend::Cpu);
    }
}
