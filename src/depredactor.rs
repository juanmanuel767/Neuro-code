use std::path::{Path, PathBuf};
use crate::interpreter::Interpreter;

#[derive(Debug)]
pub enum ModuleSource {
    NeuroFile(PathBuf),
    Python(String),
    Remote(String),
    System(String),
    NotFound(String),
}

#[derive(Debug, Clone)]
pub struct DepredactorResolver {
    base_dir: PathBuf,
}

impl DepredactorResolver {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn resolve(&self, module: &str, interpreter: &Interpreter) -> ModuleSource {
        let (prefix, target) = self.parse_prefix(module);

        match prefix {
            Some("python") | Some("py") => ModuleSource::Python(target.to_string()),
            Some("aquila") | Some("aq") | Some("neuro") | Some("neurocode") | Some("nc") => {
                if let Some(path) = self.find_standard_module(target) {
                    ModuleSource::NeuroFile(path)
                } else {
                    ModuleSource::NeuroFile(self.resolve_local_path(target))
                }
            },
            Some("remoto") | Some("web") | Some("url") => ModuleSource::Remote(target.to_string()),
            Some("sistema") | Some("sys") => ModuleSource::System(target.to_string()),
            None => {
                // Sin prefijo: intentar orden de prioridad
                // 1. ¿Es una URL?
                if target.starts_with("http://") || target.starts_with("https://") {
                    return ModuleSource::Remote(target.to_string());
                }

                // 2. ¿Es un archivo local explícito? (.aq o .neuro)
                if target.ends_with(".neuro") || target.ends_with(".neuro") {
                    return ModuleSource::NeuroFile(self.resolve_local_path(target));
                }

                // 3. ¿Es un módulo estándar?
                if let Some(path) = self.find_standard_module(target) {
                    return ModuleSource::NeuroFile(path);
                }

                // 4. ¿Está en el manifiesto aquila.json?
                if let Some(origin) = self.check_manifest(target, interpreter) {
                    match origin.as_str() {
                        "aquila" | "neurocode" => {
                            if let Some(path) = self.find_standard_module(target) {
                                return ModuleSource::NeuroFile(path);
                            }
                        },
                        "python" => return ModuleSource::Python(target.to_string()),
                        _ => {}
                    }
                }

                // 5. Fallback final: Python (compatibilidad histórica)
                ModuleSource::Python(target.to_string())
            }
            _ => ModuleSource::NotFound(format!("Prefijo desconocido: {}", prefix.unwrap())),
        }
    }

    fn parse_prefix<'a>(&self, module: &'a str) -> (Option<&'a str>, &'a str) {
        if let Some((prefix, target)) = module.split_once(':') {
            let p = prefix.to_lowercase();
            if matches!(p.as_str(), "python" | "py" | "aquila" | "aq" | "neuro" | "neurocode" | "nc" | "remoto" | "web" | "url" | "sistema" | "sys") {
                return (Some(prefix), target);
            }
        }
        (None, module)
    }

    fn resolve_local_path(&self, target: &str) -> PathBuf {
        let path = Path::new(target);
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            let local = self.base_dir.join(path);
            if local.exists() {
                local
            } else {
                path.to_path_buf()
            }
        }
    }

    fn find_standard_module(&self, name: &str) -> Option<PathBuf> {
        let name = name.trim();
        // Evitar buscar rutas como módulos estándar
        if name.contains('/') || name.contains('\\') || name.is_empty() {
            return None;
        }

        let candidates = [
            self.base_dir.join("lib").join("neurocode").join(format!("{}.neuro", name)),
            self.base_dir.join("lib").join("neurocode").join(format!("{}.neuro", name)),
            self.base_dir.join("lib").join("aquila").join(format!("{}.neuro", name)),
            PathBuf::from("lib").join("neurocode").join(format!("{}.neuro", name)),
            PathBuf::from("lib").join("neurocode").join(format!("{}.neuro", name)),
            PathBuf::from("lib").join("aquila").join(format!("{}.neuro", name)),
        ];

        candidates.into_iter().find(|p| p.exists())
    }

    pub fn check_manifest(&self, package: &str, _interpreter: &Interpreter) -> Option<String> {
        // Esta lógica estaba en interpreter.rs, la usaremos desde allí o la moveremos aquí
        // Por ahora, el resolver puede pedirle ayuda al intérprete o buscar el archivo directamente
        let manifest_path = self.find_manifest_file()?;
        let content = std::fs::read_to_string(manifest_path).ok()?;
        let manifest: serde_json::Value = serde_json::from_str(&content).ok()?;
        
        manifest.get("dependencias")?
            .get(package)?
            .get("origen")?
            .as_str()
            .map(|s| s.to_string())
    }

    fn find_manifest_file(&self) -> Option<PathBuf> {
        for dir in self.base_dir.ancestors() {
            let nc_candidate = dir.join("neurocode.json");
            if nc_candidate.exists() {
                return Some(nc_candidate);
            }
            let aq_candidate = dir.join("aquila.json");
            if aq_candidate.exists() {
                return Some(aq_candidate);
            }
        }
        None
    }
}
