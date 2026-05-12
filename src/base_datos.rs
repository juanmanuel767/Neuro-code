use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rusqlite::{Connection, params_from_iter};

#[derive(Debug)]
pub struct AquilaDatabase {
    pub conn: Mutex<Connection>,
    pub path: String,
}

impl AquilaDatabase {
    pub fn new(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path)
            .map_err(|e| format!("Error abriendo base de datos '{}': {}", path, e))?;
        Ok(AquilaDatabase {
            conn: Mutex::new(conn),
            path: path.to_string(),
        })
    }

    pub fn ejecutar(&self, sql: &str, params: Vec<String>) -> Result<crate::interpreter::RuntimeValue, String> {
        let conn = self.conn.lock().unwrap();
        if params.is_empty() {
            conn.execute_batch(sql)
                .map_err(|e| format!("Error SQL: {}", e))?;
        } else {
            conn.execute(sql, params_from_iter(params))
                .map_err(|e| format!("Error SQL al ejecutar con parámetros: {}", e))?;
        }
        Ok(crate::interpreter::RuntimeValue::Null)
    }

    pub fn consultar(&self, sql: &str, params: Vec<String>) -> Result<crate::interpreter::RuntimeValue, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)
            .map_err(|e| format!("Error preparando consulta: {}", e))?;
        
        let column_names: Vec<String> = stmt.column_names().iter().map(|c| c.to_string()).collect();
        
        let rows = stmt.query_map(params_from_iter(params), |row| {
            let mut map = HashMap::new();
            for (i, name) in column_names.iter().enumerate() {
                // Probamos tipos en orden de probabilidad
                if let Ok(s) = row.get::<_, String>(i) {
                    map.insert(name.clone(), s);
                } else if let Ok(n) = row.get::<_, i64>(i) {
                    map.insert(name.clone(), n.to_string());
                } else if let Ok(f) = row.get::<_, f64>(i) {
                    map.insert(name.clone(), f.to_string());
                } else {
                    map.insert(name.clone(), "nulo".to_string());
                }
            }
            Ok(map)
        }).map_err(|e| format!("Error ejecutando consulta: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            if let Ok(map) = row {
                let mut nexus_map = HashMap::new();
                for (k, v) in map {
                    if v == "nulo" {
                        nexus_map.insert(k, crate::interpreter::RuntimeValue::Null);
                    } else if let Ok(n) = v.parse::<i64>() {
                        nexus_map.insert(k, crate::interpreter::RuntimeValue::Int(n));
                    } else if let Ok(f) = v.parse::<f64>() {
                        nexus_map.insert(k, crate::interpreter::RuntimeValue::Number(f));
                    } else {
                        nexus_map.insert(k, crate::interpreter::RuntimeValue::Text(v));
                    }
                }
                results.push(crate::interpreter::RuntimeValue::Dictionary(Arc::new(Mutex::new(nexus_map))));
            }
        }
        
        Ok(crate::interpreter::RuntimeValue::List(Arc::new(Mutex::new(results))))
    }
}
