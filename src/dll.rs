use libloading::{Library, Symbol};
use std::path::PathBuf;

// Define the function signature for Expan_Api_GetPowerStatus
type ExpanApiGetPowerStatus = unsafe extern "C" fn(gpu_index: i32, power_status: *mut f32) -> i32;

// Embed the DLL directly in the binary
// const DLL_BYTES: &[u8] = include_bytes!("../assets/ExpanModule.dll");

#[derive(Debug)]
pub struct ExpanMod {
    library: Option<Library>,
    path: PathBuf,
}

impl ExpanMod {
    pub fn new(path: &PathBuf) -> Result<Self, anyhow::Error> {
        // let path = extract_dll()?;
        let library = load_dll(path)?;
        Ok(Self {
            library: Some(library),
            path: path.clone(),
        })
    }

    pub fn get_amperage_info(
        &self,
        gpu_index: i32,
        pin_value_buffer: &mut [f32; 6],
    ) -> Result<i32, anyhow::Error> {
        if let Some(lib) = &self.library {
            // Get the raw function pointer
            let get_power_status: Symbol<ExpanApiGetPowerStatus> =
                unsafe { lib.get(b"Expan_Api_GetPowerStatus") }.map_err(anyhow::Error::new)?;
            let ec = unsafe { get_power_status(gpu_index, pin_value_buffer.as_mut_ptr()) };
            Ok(ec)
        } else {
            Err(anyhow::Error::msg("Library is none!"))
        }
    }
}

impl Drop for ExpanMod {
    fn drop(&mut self) {
        if let Some(lib) = self.library.take() {
            if let Err(e) = lib.close() {
                eprintln!("Failed to close dll library: {}", e);
            };
        };
    }
}

// fn extract_dll() -> Result<PathBuf, anyhow::Error> {
//     // Extract DLL to a temp file
//     let mut dll_path = get_temp_path()?;
//     dll_path.push("ExpanModule_temp.dll");
//     fs::write(&dll_path, DLL_BYTES)?;
//     Ok(dll_path)
// }

fn load_dll(path: &PathBuf) -> Result<Library, anyhow::Error> {
    // Load the DLL
    match unsafe { Library::new(path) } {
        Ok(lib) => Ok(lib),
        Err(e) => Err(anyhow::Error::new(e)),
    }
}

pub(crate) fn load_expan_module(paths: &[PathBuf]) -> Option<ExpanMod> {
    for path in paths {
        if path.exists() {
            match ExpanMod::new(path) {
                Ok(expan_mod) => {
                    tracing::info!("Successfully loaded expanmodule.dll from {:?}", path);
                    return Some(expan_mod);
                }
                Err(e) => {
                    tracing::error!("Failed to load expanmodule.dll from {:?}: {}", path, e);
                }
            }
        } else {
            tracing::warn!("expanmodule.dll not found at {:?}", path);
        }
    }

    tracing::error!(
        "Failed to load ExpanModule.dll from any of the specified paths: {:?}.",
        paths
    );
    None
}
