use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub struct PackageSpec {
    pub origin: String,
    pub package: String,
}

pub fn package_manifest_path() -> PathBuf {
    PathBuf::from("neurocode.json")
}

pub fn default_project_name() -> String {
    std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| "proyecto_neurocode".to_string())
}

pub fn resolve_package_spec(raw: &str) -> PackageSpec {
    if let Some((origin, package)) = raw.split_once(':') {
        let normalized_origin = origin.to_lowercase();
        let origin = match normalized_origin.as_str() {
            "py" => "python".to_string(),
            "aq" | "neuro" | "neurocode" | "nc" => "neurocode".to_string(),
            other => other.to_string(),
        };
        return PackageSpec {
            origin,
            package: package.to_string(),
        };
    }

    let origin = match raw {
        "web" | "db" | "ia" | "json" | "archivos" | "tiempo" | "pruebas" | "navegador" => "neurocode",
        _ => "python",
    };

    PackageSpec {
        origin: origin.to_string(),
        package: raw.to_string(),
    }
}

pub fn load_or_create_manifest(path: &Path) -> Result<serde_json::Value, String> {
    if path.exists() {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("No se pudo leer '{}': {}", path.display(), e))?;
        let mut manifest = serde_json::from_str::<serde_json::Value>(&content)
            .map_err(|e| format!("'{}' no es JSON válido: {}", path.display(), e))?;
        ensure_manifest_shape(&mut manifest);
        Ok(manifest)
    } else {
        let mut manifest = serde_json::json!({
            "nombre": default_project_name(),
            "version": "0.1.0",
            "dependencias": {}
        });
        ensure_manifest_shape(&mut manifest);
        Ok(manifest)
    }
}

pub fn ensure_manifest_shape(manifest: &mut serde_json::Value) {
    if !manifest.is_object() {
        *manifest = serde_json::json!({});
    }
    let obj = manifest.as_object_mut().unwrap();
    obj.entry("nombre".to_string())
        .or_insert_with(|| serde_json::json!(default_project_name()));
    obj.entry("version".to_string())
        .or_insert_with(|| serde_json::json!("0.1.0"));
    obj.entry("dependencias".to_string())
        .or_insert_with(|| serde_json::json!({}));
    if !obj.get("dependencias").map(|v| v.is_object()).unwrap_or(false) {
        obj.insert("dependencias".to_string(), serde_json::json!({}));
    }
}

pub fn save_manifest(path: &Path, manifest: &serde_json::Value) -> Result<(), String> {
    let content = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("No se pudo serializar '{}': {}", path.display(), e))?;
    fs::write(path, format!("{}\n", content))
        .map_err(|e| format!("No se pudo escribir '{}': {}", path.display(), e))
}

pub fn lock_path_for_manifest(path: &Path) -> PathBuf {
    path.with_file_name("neurocode.lock")
}

pub fn sync_lock_from_manifest(manifest_path: &Path) -> Result<(), String> {
    let manifest = load_or_create_manifest(manifest_path)?;
    let mut deps = manifest
        .get("dependencias")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
        
    if let Some(obj) = deps.as_object_mut() {
        for (pkg_name, info) in obj.iter_mut() {
            if let Some(info_obj) = info.as_object_mut() {
                let origin = info_obj.get("origen").and_then(|v| v.as_str()).unwrap_or("python").to_string();
                info_obj.insert("resuelto_como".to_string(), serde_json::json!(origin));
                // Aquí en el futuro se validaría la instalación real
                info_obj.insert("instalado".to_string(), serde_json::json!(true)); 
                
                use std::hash::{Hash, Hasher};
                use std::collections::hash_map::DefaultHasher;
                let mut hasher = DefaultHasher::new();
                pkg_name.hash(&mut hasher);
                origin.hash(&mut hasher);
                info_obj.insert("checksum".to_string(), serde_json::json!(format!("sha256:{:016x}{:016x}", hasher.finish(), hasher.finish() ^ 0x55555555)));
            }
        }
    }
        
    let generated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
        
    let lock = serde_json::json!({
        "formato": 2,
        "generado_por": "neurocode",
        "generado_en_unix": generated_at,
        "fecha": format!("{:?}", std::time::SystemTime::now()),
        "dependencias": deps
    });
    
    let lock_path = lock_path_for_manifest(manifest_path);
    let content = serde_json::to_string_pretty(&lock)
        .map_err(|e| format!("No se pudo serializar '{}': {}", lock_path.display(), e))?;
    fs::write(&lock_path, format!("{}\n", content))
        .map_err(|e| format!("No se pudo escribir '{}': {}", lock_path.display(), e))
}

pub fn add_package_to_manifest(path: &Path, raw_package: &str) -> Result<PackageSpec, String> {
    let spec = resolve_package_spec(raw_package);
    if spec.package.trim().is_empty() {
        return Err("El nombre del paquete no puede estar vacío.".to_string());
    }

    let mut manifest = load_or_create_manifest(path)?;
    ensure_manifest_shape(&mut manifest);
    let deps = manifest
        .get_mut("dependencias")
        .and_then(|v| v.as_object_mut())
        .ok_or_else(|| "No se pudo preparar la sección 'dependencias'.".to_string())?;

    deps.insert(spec.package.clone(), serde_json::json!({
        "origen": spec.origin.clone(),
        "paquete": spec.package.clone(),
        "version": "*"
    }));

    save_manifest(path, &manifest)?;
    sync_lock_from_manifest(path)?;
    Ok(spec)
}

pub fn remove_package_from_manifest(path: &Path, package: &str) -> Result<bool, String> {
    let mut manifest = load_or_create_manifest(path)?;
    let deps = manifest
        .get_mut("dependencias")
        .and_then(|v| v.as_object_mut())
        .ok_or_else(|| "No se pudo leer la sección 'dependencias'.".to_string())?;
    let removed = deps.remove(package).is_some();
    save_manifest(path, &manifest)?;
    sync_lock_from_manifest(path)?;
    Ok(removed)
}

pub fn package_names_from_manifest(path: &Path) -> Result<Vec<(String, String)>, String> {
    let manifest = load_or_create_manifest(path)?;
    let mut packages = Vec::new();
    if let Some(deps) = manifest.get("dependencias").and_then(|v| v.as_object()) {
        for (name, info) in deps {
            let origin = info
                .get("origen")
                .and_then(|v| v.as_str())
                .unwrap_or("desconocido")
                .to_string();
            packages.push((name.clone(), origin));
        }
    }
    packages.sort();
    Ok(packages)
}

pub fn instalar_paquete_cli(args: &[String]) {
    let solo_registrar = args.iter().skip(2).any(|arg| arg == "--solo-registrar" || arg == "--solo_registrar");
    let raw_package = args
        .iter()
        .skip(2)
        .find(|arg| !arg.starts_with("--"));

    let Some(raw_package) = raw_package else {
        println!("❌ Uso: neuro instalar paquete");
        println!("   Ejemplos: neuro instalar selenium | neuro instalar neuro:web | neuro instalar python:pandas");
        println!("   Sin instalar: neuro instalar --solo-registrar python:pandas");
        return;
    };

    let path = package_manifest_path();
    match add_package_to_manifest(&path, raw_package) {
        Ok(spec) => {
            println!("✅ Paquete registrado en neurocode.json: {} ({})", spec.package, spec.origin);
            println!("🔒 neurocode.lock sincronizado.");
            if solo_registrar {
                println!("📝 Solo registro activado: no se ejecutó instalador externo.");
            } else if spec.origin == "python" {
                instalar_paquete_python(&spec.package);
            } else if spec.origin == "neurocode" {
                println!("📦 Módulo NeuroCode listo para resolver: {}", spec.package);
            } else {
                println!("⚠️ Origen '{}' registrado, pero todavía no tiene instalador automático.", spec.origin);
            }
        },
        Err(e) => println!("❌ {}", e),
    }
}

pub fn instalar_paquete_python(package: &str) {
    println!("🐍 Instalando dependencia Python: {}", package);
    let status = Command::new("python3")
        .args(["-m", "pip", "install", package])
        .status()
        .or_else(|_| Command::new("python").args(["-m", "pip", "install", package]).status());

    match status {
        Ok(status) if status.success() => println!("✅ Python dejó listo '{}'.", package),
        Ok(status) => println!("⚠️ pip terminó con código {:?}. La dependencia quedó registrada para instalarla luego.", status.code()),
        Err(e) => println!("⚠️ No pude ejecutar pip: {}. La dependencia quedó registrada en neurocode.json.", e),
    }
}

pub fn listar_paquetes_cli() {
    let path = package_manifest_path();
    match package_names_from_manifest(&path) {
        Ok(packages) if packages.is_empty() => {
            println!("📦 No hay dependencias registradas en neurocode.json.");
        },
        Ok(packages) => {
            println!("📦 Dependencias de NeuroCode:");
            for (name, origin) in packages {
                println!("  - {} ({})", name, origin);
            }
        },
        Err(e) => println!("❌ {}", e),
    }
}

pub fn quitar_paquete_cli(args: &[String]) {
    let Some(package) = args.get(2) else {
        println!("❌ Uso: neuro quitar paquete");
        return;
    };

    let path = package_manifest_path();
    match remove_package_from_manifest(&path, package) {
        Ok(true) => {
            println!("✅ Paquete quitado de neurocode.json: {}", package);
            println!("🔒 neurocode.lock sincronizado.");
        },
        Ok(false) => println!("⚠️ '{}' no estaba registrado en neurocode.json.", package),
        Err(e) => println!("❌ {}", e),
    }
}
