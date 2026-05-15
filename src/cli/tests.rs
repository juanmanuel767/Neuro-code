use std::path::{Path, PathBuf};
use std::fs;
use crate::lexer;
use crate::parser;
use crate::{execute_file, guardian_builtin_symbols, validate_statement_symbols, collect_declared_symbols, collect_review_warnings};

pub fn run_tests(target: &str) {
    let path = Path::new(target);
    let mut files = Vec::new();

    if path.is_file() {
        files.push(path.to_path_buf());
    } else if path.is_dir() {
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("❌ No se pudo leer el directorio de pruebas '{}': {}", target, e);
                std::process::exit(1);
            }
        };

        for entry in entries.flatten() {
            let file_path = entry.path();
            let is_aquila = matches!(file_path.extension().and_then(|ext| ext.to_str()), Some("aq") | Some("neuro"));
            let is_neuro_test = file_path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("eval_") || name.starts_with("compat_"))
                .unwrap_or(false);

            if is_aquila && is_neuro_test {
                files.push(file_path);
            }
        }
        files.sort();
    } else {
        eprintln!("❌ Ruta de pruebas no encontrada: {}", target);
        std::process::exit(1);
    }

    if files.is_empty() {
        eprintln!("❌ No se encontraron pruebas .aq o .neuro evaluables en '{}'.", target);
        std::process::exit(1);
    }

    println!("🧪 Ejecutando {} prueba(s) NeuroCode...", files.len());
    let mut passed = 0usize;

    for file in &files {
        println!("\n▶ {}", file.display());
        if execute_file(&file.to_string_lossy(), false) {
            passed += 1;
            println!("✅ OK: {}", file.display());
        } else {
            println!("❌ FALLÓ: {}", file.display());
        }
    }

    println!("\n📊 Resultado: {}/{} pruebas pasaron.", passed, files.len());
    if passed != files.len() {
        std::process::exit(1);
    }
}

pub fn collect_neuro_files(target: &str) -> Result<Vec<PathBuf>, String> {
    let path = Path::new(target);
    let mut files = Vec::new();

    if path.is_file() {
        files.push(path.to_path_buf());
    } else if path.is_dir() {
        let entries = fs::read_dir(path)
            .map_err(|e| format!("No se pudo leer el directorio '{}': {}", target, e))?;
        for entry in entries.flatten() {
            let file_path = entry.path();
            if matches!(file_path.extension().and_then(|ext| ext.to_str()), Some("aq") | Some("neuro")) {
                files.push(file_path);
            }
        }
        files.sort();
    } else {
        return Err(format!("Ruta no encontrada: {}", target));
    }

    if files.is_empty() {
        return Err(format!("No se encontraron archivos .aq o .neuro en '{}'.", target));
    }

    Ok(files)
}

pub fn revisar_codigo(code: &str) -> Result<Vec<String>, String> {
    let (tokens, positions) = lexer::tokenize_with_positions(code);
    let ast = parser::parse_with_positions(tokens, positions)
        .map_err(|e| format!("Error de sintaxis: {}", e))?;

    let mut symbols = guardian_builtin_symbols();
    collect_declared_symbols(&ast, &mut symbols);
    for stmt in &ast {
        validate_statement_symbols(stmt, &symbols)
            .map_err(|e| e.replace("Error semántico del parche: ", "Error semántico: "))?;
    }

    let mut warnings = Vec::new();
    collect_review_warnings(&ast, false, &mut warnings);
    Ok(warnings)
}

pub fn revisar_cli(target: &str) {
    let files = match collect_neuro_files(target) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        },
    };

    println!("🔎 Revisando {} archivo(s) NeuroCode...", files.len());
    let mut ok = 0usize;
    let mut warnings_total = 0usize;

    for file in &files {
        let code = match fs::read_to_string(file) {
            Ok(code) => code,
            Err(e) => {
                println!("❌ {}: no se pudo leer ({})", file.display(), e);
                continue;
            },
        };

        match revisar_codigo(&code) {
            Ok(warnings) => {
                ok += 1;
                if warnings.is_empty() {
                    println!("✅ {}", file.display());
                } else {
                    warnings_total += warnings.len();
                    println!("⚠️ {}: {} advertencia(s)", file.display(), warnings.len());
                    for warning in warnings {
                        println!("   - {}", warning);
                    }
                }
            },
            Err(e) => println!("❌ {}: {}", file.display(), e),
        }
    }

    println!("\n📊 Revisión: {}/{} archivo(s) válidos, {} advertencia(s).", ok, files.len(), warnings_total);
    if ok != files.len() {
        std::process::exit(1);
    }
}
