use llama_cpp::llama::{self, LlamaBackend, LlamaModelParams, LlamaContextParams, LlamaModel};
use hf_hub::api::sync::ApiBuilder;
use anyhow::{Result, Context};
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {

    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let model_path = get_model_path()?;
    
    LlamaBackend::init();
    let ctx_params = LlamaContextParams::default();
    let model_params = LlamaModelParams::default();
    let model = LlamaModel::new(model_path.to_str().unwrap(), &model_params)?;
    let ctx = model.build_context(&ctx_params)?;


    Ok(())
}

fn get_model_path() -> Result<std::path::PathBuf, anyhow::Error> {
    let model_path = ApiBuilder::new()
        .with_progress(true)
        .build()?
        .model("TheBloke/Mistral-7B-Instruct-v0.2-GGUF".to_string())
        .get("mistral-7b-instruct-v0.2.Q4_K_M.gguf")?;
    Ok(model_path)
}

