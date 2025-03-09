use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SnarkContext {
    pub pk_dir: String,
    pub input_dir: String,
    pub output_dir: String,
}