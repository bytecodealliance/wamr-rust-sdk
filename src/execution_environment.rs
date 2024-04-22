use wamr_sys::wasm_exec_env_t;

use crate::instance::Instance;

pub struct ExecutionEnvironment {
    execution_environment: wasm_exec_env_t,
}

impl From<wasm_exec_env_t> for ExecutionEnvironment {
    fn from(execution_environment: wasm_exec_env_t) -> Self {
        Self {
            execution_environment,
        }
    }
}

impl ExecutionEnvironment {
    pub fn get_instance(&self) -> Instance {
        Instance::from(unsafe {
            wamr_sys::wasm_runtime_get_module_inst(self.execution_environment)
        })
    }
}
