#[derive(Debug, Clone, Copy)]
pub struct KvCacheConfig {
    pub layers: usize,
    pub heads: usize,
    pub head_dim: usize,
    pub context_length: usize,
    pub dtype: KV_DTYPE,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KV_DTYPE {
    F32,
    F16,
    Q8_0,
    Q4_0,
}

impl KV_DTYPE {
    pub fn bytes_per_element(&self) -> usize {
        match self {
            Self::F32 => 4,
            Self::F16 => 2,
            Self::Q8_0 => 1,
            Self::Q4_0 => 1,
        }
    }
}

pub fn calculate_kv_cache_size_mb(config: &KvCacheConfig) -> usize {
    let elems = config.layers
        * config.heads
        * config.head_dim
        * config.context_length
        * config.batch_size
        * 2;
    let bytes = elems * config.dtype.bytes_per_element();
    bytes / (1024 * 1024)
}
