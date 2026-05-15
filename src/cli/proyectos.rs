use std::path::{Path, PathBuf};
use std::fs;
use crate::cli::paquetes::*;

pub fn sanitize_project_name(name: &str) -> Result<String, String> {
    let cleaned = name.trim();
    if cleaned.is_empty() {
        return Err("El nombre del proyecto no puede estar vacío.".to_string());
    }
    if cleaned.contains('/') || cleaned.contains('\\') {
        return Err("El nombre del proyecto no debe contener separadores de ruta.".to_string());
    }
    let valid = cleaned
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
    if !valid {
        return Err("Usa solo letras, números, guion o guion bajo para el nombre del proyecto.".to_string());
    }
    Ok(cleaned.to_string())
}

pub fn write_new_project(project_name: &str, base_dir: &Path) -> Result<PathBuf, String> {
    let project_name = sanitize_project_name(project_name)?;
    let project_dir = base_dir.join(&project_name);
    if project_dir.exists() {
        return Err(format!("La carpeta '{}' ya existe.", project_dir.display()));
    }

    fs::create_dir_all(project_dir.join("src"))
        .map_err(|e| format!("No se pudo crear el proyecto '{}': {}", project_dir.display(), e))?;

    let main_code = format!(
        "depredactor neuro:pruebas como pruebas\n\n\
nombre: Texto = \"{}\"\n\n\
funcion saludar(app: Texto) -> Texto {{\n\
    retornar \"Hola desde \" + app\n\
}}\n\n\
imprimir(saludar(nombre))\n\
pruebas.igual(saludar(nombre), \"Hola desde \" + nombre, \"saludo inicial\")\n",
        project_name
    );
    fs::write(project_dir.join("main.neuro"), main_code)
        .map_err(|e| format!("No se pudo escribir main.neuro: {}", e))?;

    let manifest = serde_json::json!({
        "nombre": project_name,
        "version": "0.1.0",
        "dependencias": {
            "pruebas": {
                "origen": "aquila",
                "paquete": "pruebas",
                "version": "*"
            }
        }
    });
    save_manifest(&project_dir.join("neurocode.json"), &manifest)?;
    sync_lock_from_manifest(&project_dir.join("neurocode.json"))?;

    let readme = format!(
        "# {}\n\nProyecto NeuroCode creado con `neuro nuevo`.\n\n## Ejecutar\n\n```bash\nneuro main.neuro\n```\n\nTambien funciona el comando compatible:\n\n```bash\naquila main.neuro\n```\n\n## Probar\n\n```bash\nneuro test\n```\n",
        project_name
    );
    fs::write(project_dir.join("README.md"), readme)
        .map_err(|e| format!("No se pudo escribir README.md: {}", e))?;

    Ok(project_dir)
}

pub fn nuevo_proyecto_cli(args: &[String]) {
    let Some(project_name) = args.get(2) else {
        println!("❌ Uso: neuro nuevo mi_app");
        return;
    };

    match write_new_project(project_name, Path::new(".")) {
        Ok(path) => {
            println!("✅ Proyecto creado: {}", path.display());
            println!("Siguiente paso:");
            println!("  cd {}", path.display());
            println!("  neuro main.neuro");
        },
        Err(e) => println!("❌ {}", e),
    }
}

pub fn init_proyecto_cli() {
    let old_manifest = Path::new("aquila.json");
    let new_manifest = package_manifest_path();
    
    println!("🧠 Inicializando proyecto NeuroCode...");
    
    if old_manifest.exists() {
        println!("📦 Detectado aquila.json existente -> migrando a neurocode.json");
        if let Ok(manifest) = load_or_create_manifest(old_manifest) {
            let _ = save_manifest(&new_manifest, &manifest);
            let _ = std::fs::remove_file(old_manifest);
        }
    } else if !new_manifest.exists() {
        let manifest = load_or_create_manifest(&new_manifest).unwrap_or_else(|_| serde_json::json!({}));
        let _ = save_manifest(&new_manifest, &manifest);
        println!("📦 Nuevo neurocode.json generado.");
    } else {
        println!("📦 neurocode.json ya existe. Sincronizando...");
    }
    
    if let Err(e) = sync_lock_from_manifest(&new_manifest) {
        println!("❌ Error generando neurocode.lock: {}", e);
    } else {
        println!("🔒 neurocode.lock generado (v2)");
        println!("✅ Proyecto listo. Usa 'neuro main.neuro' para ejecutar.");
    }
}

