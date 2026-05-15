// Módulo compilador - funciones de ejecución de archivos
use crate::execute_file;

pub fn run_file(filepath: &str) {
    println!("🚀 NeuroCode v2.1 - Ejecutando Script: {}", filepath);
    let ok = execute_file(filepath, true);
    if ok {
        println!("✅ Ejecución finalizada con éxito.");
    }
}
