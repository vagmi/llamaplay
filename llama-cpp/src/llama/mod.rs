use std::{ffi::CString, ptr::NonNull};
use thiserror::Error;

use crate::sys;

#[derive(Debug, Error)]
pub enum LlamaCppError {
    #[error("Invalid model path")]
    ModelLoadError(String),
    #[error("Unable to build context")]
    ContextBuildError

}

type Result<T> = std::result::Result<T, LlamaCppError>;

#[derive(Debug)]
pub struct LlamaBackend{}

impl LlamaBackend {
    pub fn init() {
        tracing::debug!("Initializing Llama backend");
        unsafe { sys::llama_backend_init() };
    }
}

impl Drop for LlamaBackend {
    fn drop(&mut self) {
        tracing::debug!("dropping Llama backend");
        unsafe { sys::llama_backend_free() };
    }
}


pub struct LlamaModelParams {
    pub(crate) params: sys::llama_model_params
}

impl Default for LlamaModelParams {
    fn default() -> Self {
        LlamaModelParams {
            params: unsafe { sys::llama_model_default_params() }
        }
    }
}


pub struct LlamaContextParams {
    pub(crate) params: sys::llama_context_params
}

impl Default for LlamaContextParams {
    fn default() -> Self {
        let mut params = unsafe { sys::llama_context_default_params() };
        params.seed = 1234;
        params.n_ctx = 2048u32;
        LlamaContextParams {
            params
        }
    }
}

pub struct LlamaContext<'a> {
    context: NonNull<sys::llama_context>,
    model: &'a LlamaModel,
}


pub struct LlamaModel {
    pub model: NonNull<sys::llama_model>
}

impl LlamaModel {
    pub fn new(model_path: &str, model_params: &LlamaModelParams) -> Result<Self> {
        let c_str = CString::new(model_path).map_err(|_| {LlamaCppError::ModelLoadError("unable to convert to C String".to_string())})?;
        let model = unsafe { sys::llama_load_model_from_file(c_str.as_ptr(), model_params.params) };
        let model = NonNull::new(model).ok_or(LlamaCppError::ModelLoadError("unable to create non null pointer".to_string()))?;
        Ok(LlamaModel {
            model
        })
    }
    pub fn build_context(&self, params: &LlamaContextParams) -> Result<LlamaContext> {
        let ctx = unsafe { sys::llama_new_context_with_model(self.model.as_ptr(), params.params) };
        let context = NonNull::new(ctx).ok_or(LlamaCppError::ContextBuildError)?;
        return Ok(LlamaContext {
            context,
            model: self
        });
    }
}

