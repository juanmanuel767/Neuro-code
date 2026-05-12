use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::ast::{Expression, Statement};
use std::fmt;
use pyo3::prelude::*;

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Number(f64),
    Int(i64),
    Boolean(bool),
    Text(String),
    List(Arc<Mutex<Vec<RuntimeValue>>>),
    Dictionary(Arc<Mutex<HashMap<String, RuntimeValue>>>),
    Function(Vec<String>, Vec<Statement>),
    AsyncFunction(Vec<String>, Vec<Statement>),
    Promise(Box<RuntimeValue>),
    Class(String, Arc<Mutex<HashMap<String, RuntimeValue>>>),
    Instance(String, Arc<Mutex<HashMap<String, RuntimeValue>>>, Box<RuntimeValue>),
    Server(Arc<crate::servidor::AquilaServer>),
    Database(Arc<crate::base_datos::AquilaDatabase>),
    PyWrapper(Arc<PyObject>),
    Null,
    Break,
}

// Implementación manual de PartialEq ignorando PyWrapper y Function porque no se pueden comparar directamente.
impl PartialEq for RuntimeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => a == b,
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => a == b,
            (RuntimeValue::Boolean(a), RuntimeValue::Boolean(b)) => a == b,
            (RuntimeValue::Text(a), RuntimeValue::Text(b)) => a == b,
            (RuntimeValue::List(a), RuntimeValue::List(b)) => {
                let a_lock = a.lock().unwrap();
                let b_lock = b.lock().unwrap();
                *a_lock == *b_lock
            },
            (RuntimeValue::Dictionary(a), RuntimeValue::Dictionary(b)) => {
                let a_lock = a.lock().unwrap();
                let b_lock = b.lock().unwrap();
                *a_lock == *b_lock
            },
            (RuntimeValue::Class(n1, _), RuntimeValue::Class(n2, _)) => n1 == n2,
            (RuntimeValue::Instance(n1, p1, _), RuntimeValue::Instance(n2, p2, _)) => {
                let a_lock = p1.lock().unwrap();
                let b_lock = p2.lock().unwrap();
                n1 == n2 && *a_lock == *b_lock
            },
            (RuntimeValue::Promise(a), RuntimeValue::Promise(b)) => *a == *b,
            (RuntimeValue::Server(_), RuntimeValue::Server(_)) => false,
            (RuntimeValue::Database(_), RuntimeValue::Database(_)) => false,
            (RuntimeValue::Null, RuntimeValue::Null) => true,
            _ => false,
        }
    }
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Number(n) => write!(f, "{}", n),
            RuntimeValue::Int(i) => write!(f, "{}", i),
            RuntimeValue::Boolean(b) => write!(f, "{}", if *b { "verdadero" } else { "falso" }),
            RuntimeValue::Text(s) => write!(f, "{}", s),
            RuntimeValue::List(l) => {
                let l_lock = l.lock().unwrap();
                let elements: Vec<String> = l_lock.iter().map(|item| format!("{}", item)).collect();
                write!(f, "[{}]", elements.join(", "))
            },
            RuntimeValue::Dictionary(d) => {
                let d_lock = d.lock().unwrap();
                let elements: Vec<String> = d_lock.iter().map(|(k, v)| format!("\"{}\": {}", k, v)).collect();
                write!(f, "{{{}}}", elements.join(", "))
            },
            RuntimeValue::Class(name, _) => write!(f, "<clase {}>", name),
            RuntimeValue::Instance(name, props_arc, _) => {
                let props = props_arc.lock().unwrap();
                let elements: Vec<String> = props.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                write!(f, "<instancia {}{{{}}}>", name, elements.join(", "))
            },
            RuntimeValue::Server(s) => write!(f, "<ServidorWeb puerto:{}>", s.port),
            RuntimeValue::Database(db) => write!(f, "<BaseDatos '{}'>", db.path),
            RuntimeValue::Function(params, _) => write!(f, "<funcion({})>", params.join(", ")),
            RuntimeValue::AsyncFunction(params, _) => write!(f, "<asincrono funcion({})>", params.join(", ")),
            RuntimeValue::Promise(_) => write!(f, "<Promesa Pendiente...>"),
            RuntimeValue::PyWrapper(_) => write!(f, "<Objeto Python>"),
            RuntimeValue::Null => write!(f, "nulo"),
            RuntimeValue::Break => write!(f, "<romper>"),
        }
    }
}

// Conversores Univarsales
#[allow(deprecated)]
fn val_to_py(py: Python, val: RuntimeValue) -> PyObject {
    match val {
        RuntimeValue::Number(n) => n.to_object(py),
        RuntimeValue::Int(i) => i.to_object(py),
        RuntimeValue::Boolean(b) => b.to_object(py),
        RuntimeValue::Text(s) => s.to_object(py),
        RuntimeValue::Null => py.None(),
        RuntimeValue::List(l) => {
            let py_list = pyo3::types::PyList::empty(py);
            let l_lock = l.lock().unwrap();
            for item in l_lock.iter() {
                let _ = py_list.append(val_to_py(py, item.clone()));
            }
            py_list.to_object(py)
        },
        RuntimeValue::Dictionary(d) => {
            let py_dict = pyo3::types::PyDict::new(py);
            let d_lock = d.lock().unwrap();
            for (k, v) in d_lock.iter() {
                let _ = py_dict.set_item(k, val_to_py(py, v.clone()));
            }
            py_dict.to_object(py)
        },
        RuntimeValue::Class(_, _) => py.None(),
        RuntimeValue::Instance(_, _, _) => py.None(),
        RuntimeValue::Server(_) => py.None(),
        RuntimeValue::Database(_) => py.None(),
        RuntimeValue::PyWrapper(p) => (*p).clone_ref(py),
        _ => py.None()
    }
}

fn py_to_val(py: Python, py_obj: PyObject) -> RuntimeValue {
    if let Ok(n) = py_obj.extract::<i64>(py) { return RuntimeValue::Int(n); }
    if let Ok(n) = py_obj.extract::<f64>(py) { return RuntimeValue::Number(n); }
    if let Ok(b) = py_obj.extract::<bool>(py) { return RuntimeValue::Boolean(b); }
    if let Ok(s) = py_obj.extract::<String>(py) { return RuntimeValue::Text(s); }
    
    // Si es una lista compleja o matriz u objeto (ej., NumPy array), lo envolvemos
    RuntimeValue::PyWrapper(Arc::new(py_obj))
}

pub struct Environment {
    values: HashMap<String, RuntimeValue>,
    parent: Option<Arc<Mutex<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn new_with_parent(parent: Arc<Mutex<Environment>>) -> Self {
        Environment {
            values: HashMap::new(),
            parent: Some(parent),
        }
    }

    pub fn define(&mut self, name: String, value: RuntimeValue) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: &str, value: RuntimeValue) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.lock().unwrap().assign(name, value)
        } else {
            Err(format!("Variable no definida: '{}'", name))
        }
    }

    pub fn get(&self, name: &str) -> Result<RuntimeValue, String> {
        if let Some(val) = self.values.get(name) {
            Ok(val.clone())
        } else if let Some(parent) = &self.parent {
            parent.lock().unwrap().get(name)
        } else {
            Err(format!("Variable no definida: '{}'", name))
        }
    }
}

pub struct Interpreter {
    pub global_env: Arc<Mutex<Environment>>,
    pub exported_names: Vec<String>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            global_env: Arc::new(Mutex::new(Environment::new())),
            exported_names: Vec::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Statement>) -> Result<RuntimeValue, String> {
        // Inicializamos Python para toda la sesión de Nexus (Fase 4)
        pyo3::prepare_freethreaded_python();
        
        let env = Arc::clone(&self.global_env);
        for stmt in statements {
            if let Some(ret) = self.execute(stmt, &env)? {
                return Ok(ret);
            }
        }
        Ok(RuntimeValue::Null)
    }

    fn execute(&mut self, stmt: Statement, env: &Arc<Mutex<Environment>>) -> Result<Option<RuntimeValue>, String> {
        match stmt {
            Statement::Assign(name, expr) => {
                let value = self.evaluate(expr, env)?;
                let mut env_lock = env.lock().unwrap();
                if env_lock.assign(&name, value.clone()).is_err() {
                    env_lock.define(name, value);
                }
            },
            Statement::AssignProperty(callee_expr, prop_name, value_expr) => {
                let callee = self.evaluate(callee_expr, env)?;
                let value = self.evaluate(value_expr, env)?;
                match callee {
                    RuntimeValue::Dictionary(map_arc) => {
                        map_arc.lock().unwrap().insert(prop_name, value);
                    },
                    RuntimeValue::Instance(_, props_arc, _) => {
                        props_arc.lock().unwrap().insert(prop_name, value);
                    },
                    RuntimeValue::PyWrapper(py_obj) => {
                        pyo3::Python::with_gil(|py| {
                            let _ = py_obj.setattr(py, prop_name.as_str(), val_to_py(py, value));
                        });
                    },
                    _ => return Err("Solo se pueden asignar propiedades a diccionarios u objetos de Python.".into()),
                }
            },
            Statement::AssignIndex(callee_expr, index_expr, value_expr) => {
                let callee = self.evaluate(callee_expr, env)?;
                let index = self.evaluate(index_expr, env)?;
                let value = self.evaluate(value_expr, env)?;
                
                match callee {
                    RuntimeValue::List(list_arc) => {
                        if let RuntimeValue::Int(idx) = index {
                            let mut list = list_arc.lock().unwrap();
                            if idx >= 0 && (idx as usize) < list.len() {
                                list[idx as usize] = value;
                            } else {
                                return Err(format!("Índice de lista fuera de límites: {}", idx));
                            }
                        } else {
                            return Err("El índice de una lista debe ser entero.".into());
                        }
                    },
                    RuntimeValue::Dictionary(map_arc) => {
                        let str_key = match index {
                            RuntimeValue::Text(s) => s,
                            _ => return Err("El índice de diccionarios al asignar debe ser texto.".into()),
                        };
                        map_arc.lock().unwrap().insert(str_key, value);
                    },
                    _ => return Err("Solo se puede asignar por índice a listas o diccionarios.".into()),
                }
            },
            Statement::Expression(expr) => {
                self.evaluate(expr, env)?;
            },
            Statement::If(cond, then_branch, else_branch) => {
                let cond_val = self.evaluate(cond, env)?;
                if self.is_truthy(&cond_val) {
                    if let Some(ret) = self.execute_block(then_branch, env)? { return Ok(Some(ret)); }
                } else {
                    if let Some(ret) = self.execute_block(else_branch, env)? { return Ok(Some(ret)); }
                }
            },
            Statement::While(cond, body) => {
                loop {
                    let cond_val = self.evaluate(cond.clone(), env)?;
                    if !self.is_truthy(&cond_val) { break; }
                    if let Some(ret) = self.execute_block(body.clone(), env)? {
                        if let RuntimeValue::Break = ret { break; }
                        return Ok(Some(ret));
                    }
                }
            },
            Statement::For(var_name, iterable_expr, body) => {
                let iterable_val = self.evaluate(iterable_expr, env)?;
                if let RuntimeValue::List(list_arc) = iterable_val {
                    let list = list_arc.lock().unwrap().clone();
                    for item in list {
                        let local_env = Arc::new(Mutex::new(Environment::new_with_parent(Arc::clone(env))));
                        local_env.lock().unwrap().define(var_name.clone(), item);
                        if let Some(ret) = self.execute_block(body.clone(), &local_env)? {
                            if let RuntimeValue::Break = ret { break; }
                            return Ok(Some(ret));
                        }
                    }
                } else {
                    return Err("El bucle 'para' solo soporta iterar sobre listas u objetos iterables.".into());
                }
            },
            Statement::Break => {
                return Ok(Some(RuntimeValue::Break));
            },
            Statement::Function(name, params, body) => {
                env.lock().unwrap().define(name, RuntimeValue::Function(params, body));
            },
            Statement::AsyncFunction(name, params, body) => {
                env.lock().unwrap().define(name, RuntimeValue::AsyncFunction(params, body));
            },
            Statement::Return(expr) => {
                return Ok(Some(self.evaluate(expr, env)?));
            },
             Statement::Export(name) => {
                 self.exported_names.push(name);
             },
             Statement::Usar(modulo, alias) => {
                 let content_res = if modulo.starts_with("http://") || modulo.starts_with("https://") {
                     let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| ".".to_string());
                     let cache_dir = format!("{}/.aquila_cache", home);
                     let _ = std::fs::create_dir_all(&cache_dir);
                     
                     let safe_name = modulo.replace("://", "_").replace("/", "_").replace(":", "_");
                     let cache_path = format!("{}/{}", cache_dir, safe_name);
                     
                     if let Ok(cached) = std::fs::read_to_string(&cache_path) {
                         Ok(cached)
                     } else {
                         println!("{}☁️ Guardián de Paquetes: Descargando módulo remoto: {}...{}", "\x1b[36m", modulo, "\x1b[0m");
                         match ureq::get(&modulo).call() {
                             Ok(resp) => {
                                 let text = resp.into_string().map_err(|e| format!("Error leyendo respuesta: {}", e))?;
                                 let _ = std::fs::write(&cache_path, &text);
                                 Ok(text)
                             },
                             Err(e) => Err(format!("No se pudo descargar el módulo remoto '{}': {}", modulo, e)),
                         }
                     }
                 } else if modulo.ends_with(".aq") {
                     std::fs::read_to_string(&modulo).map_err(|e| format!("No se pudo leer el archivo módulo '{}': {}", modulo, e))
                 } else {
                     Err("PYTHON_ROUTE".to_string())
                 };

                 if let Ok(content) = content_res {
                     let tokens = crate::lexer::tokenize(&content);
                     let statements = match crate::parser::parse(tokens) {
                          Ok(s) => s,
                          Err(e) => return Err(format!("Error parseando módulo '{}': {}", modulo, e))
                     };
                     // Prevenir conflictos de entorno al importar creando una nueva instancia limpia de Intérprete
                     let mut mod_interp = Interpreter::new();
                     let _ = mod_interp.interpret(statements)?;
                     
                     let mut exported_map = HashMap::new();
                     let mod_env = mod_interp.global_env.lock().unwrap();
                     for exp_name in mod_interp.exported_names {
                         if let Ok(val) = mod_env.get(&exp_name) {
                             exported_map.insert(exp_name, val);
                         }
                     }
                     
                     let module_dict = RuntimeValue::Dictionary(Arc::new(Mutex::new(exported_map)));
                     env.lock().unwrap().define(alias, module_dict);
                 } else {
                     let py_module = pyo3::Python::with_gil(|py| {
                         match py.import(modulo.as_str()) {
                             Ok(m) => {
                                 #[allow(deprecated)]
                                 let obj = m.to_object(py);
                                 Ok(RuntimeValue::PyWrapper(Arc::new(obj)))
                             },
                             Err(e) => {
                                 e.print(py);
                                 Err(format!("Error importando módulo de Python '{}'", modulo))
                             },
                         }
                     })?;
                     env.lock().unwrap().define(alias, py_module);
                 }
             },
             Statement::TryCatch(try_block, error_var, catch_block) => {
                 match self.execute_block(try_block, env) {
                     Ok(opt_val) => {
                         if opt_val.is_some() { return Ok(opt_val); }
                     },
                     Err(e) => {
                         let catch_env = Arc::new(Mutex::new(Environment::new_with_parent(Arc::clone(env))));
                         if let Some(var_name) = error_var {
                             catch_env.lock().unwrap().define(var_name.clone(), RuntimeValue::Text(e));
                         }
                         if let Some(val) = self.execute_block(catch_block, &catch_env)? {
                             return Ok(Some(val));
                         }
                     }
                 }
             },
             Statement::Throw(expr) => {
                 let val = self.evaluate(expr, env)?;
                 return Err(format!("{}", val));
             },
             Statement::Class(name, methods) => {
                 let mut method_map = HashMap::new();
                 for method in methods {
                     if let Statement::Function(m_name, params, body) = method {
                         method_map.insert(m_name, RuntimeValue::Function(params, body));
                     }
                 }
                 env.lock().unwrap().define(name.clone(), RuntimeValue::Class(name, Arc::new(Mutex::new(method_map))));
             },
        }
        Ok(None)
    }

    pub fn execute_block_pub(&mut self, statements: Vec<Statement>, env: &Arc<Mutex<Environment>>) -> Result<Option<RuntimeValue>, String> {
        self.execute_block(statements, env)
    }

    fn execute_block(&mut self, statements: Vec<Statement>, env: &Arc<Mutex<Environment>>) -> Result<Option<RuntimeValue>, String> {
        let block_env = Arc::new(Mutex::new(Environment::new_with_parent(Arc::clone(env))));
        for stmt in statements {
            if let Some(ret) = self.execute(stmt, &block_env)? {
                return Ok(Some(ret));
            }
        }
        Ok(None)
    }

    fn evaluate(&mut self, expr: Expression, env: &Arc<Mutex<Environment>>) -> Result<RuntimeValue, String> {
        match expr {
            Expression::Number(n) => Ok(RuntimeValue::Number(n)),
            Expression::Int(i) => Ok(RuntimeValue::Int(i)),
            Expression::Text(s) => Ok(RuntimeValue::Text(s)),
            Expression::Boolean(b) => Ok(RuntimeValue::Boolean(b)),
            Expression::Null => Ok(RuntimeValue::Null),
            Expression::List(items) => {
                let mut eval_items = Vec::new();
                for item in items {
                    eval_items.push(self.evaluate(item, env)?);
                }
                Ok(RuntimeValue::List(Arc::new(Mutex::new(eval_items))))
            },
            Expression::Dictionary(pairs) => {
                let mut map = HashMap::new();
                for (k, v) in pairs {
                    let key_val = self.evaluate(k, env)?;
                    let str_key = match key_val {
                        RuntimeValue::Text(s) => s,
                        _ => return Err("Las claves de diccionarios deben ser texto.".into()),
                    };
                    let val_val = self.evaluate(v, env)?;
                    map.insert(str_key, val_val);
                }
                Ok(RuntimeValue::Dictionary(Arc::new(Mutex::new(map))))
            },
            Expression::NewInstance(class_name, args) => {
                // ServidorWeb nativo
                if class_name == "ServidorWeb" {
                    if let Some(port_val) = args.get(0) {
                        let port = match self.evaluate(port_val.clone(), env)? {
                            RuntimeValue::Int(p) => p as u16,
                            RuntimeValue::Number(p) => p as u16,
                            _ => return Err("ServidorWeb() requiere un número de puerto.".into()),
                        };
                        return Ok(RuntimeValue::Server(Arc::new(crate::servidor::AquilaServer::new(port))));
                    }
                    return Err("ServidorWeb() requiere 1 argumento (puerto).".into());
                }
                
                // BaseDatos nativa
                if class_name == "BaseDatos" {
                    if let Some(path_val) = args.get(0) {
                        let path = match self.evaluate(path_val.clone(), env)? {
                            RuntimeValue::Text(p) => p,
                            _ => return Err("BaseDatos() requiere una ruta de archivo como texto.".into()),
                        };
                        match crate::base_datos::AquilaDatabase::new(&path) {
                            Ok(db) => return Ok(RuntimeValue::Database(Arc::new(db))),
                            Err(e) => return Err(e),
                        }
                    }
                    return Err("BaseDatos() requiere 1 argumento (ruta).".into());
                }
                
                let class_val_res = env.lock().unwrap().get(&class_name);
                if let Ok(RuntimeValue::Class(name, methods_arc)) = class_val_res {
                    let instance_props = Arc::new(Mutex::new(HashMap::new()));
                    let instance_val = RuntimeValue::Instance(name.clone(), instance_props, Box::new(RuntimeValue::Class(name, methods_arc.clone())));
                    
                    let methods = methods_arc.lock().unwrap();
                    if let Some(RuntimeValue::Function(params, body)) = methods.get("crear") {
                        if args.len() != params.len() {
                            return Err(format!("El constructor 'crear' de '{}' esperaba {} argumentos pero recibió {}.", class_name, params.len(), args.len()));
                        }
                        
                        let call_env = Arc::new(Mutex::new(Environment::new_with_parent(Arc::clone(env))));
                        call_env.lock().unwrap().define("esto".to_string(), instance_val.clone());
                        
                        for (i, param_name) in params.iter().enumerate() {
                            let arg_val = self.evaluate(args[i].clone(), env)?;
                            call_env.lock().unwrap().define(param_name.clone(), arg_val);
                        }
                        self.execute_block(body.clone(), &call_env)?;
                    } else if !args.is_empty() {
                        return Err(format!("La clase '{}' no tiene constructor 'crear' pero se recibieron argumentos.", class_name));
                    }
                    
                    return Ok(instance_val);
                }
                Err(format!("Clase invocada '{}' no encontrada en el contexto.", class_name))
            },
            Expression::Await(expr) => {
                let val = self.evaluate(*expr, env)?;
                if let RuntimeValue::Promise(inner) = val {
                    Ok(*inner)
                } else {
                    Ok(val)
                }
            },
            Expression::LambdaFunction(params, body) => {
                Ok(RuntimeValue::Function(params, body))
            },
            Expression::Identifier(name) => {
                env.lock().unwrap().get(&name)
            },
            Expression::LogicalOp(left_expr, op, right_expr) => {
                let l_val = self.evaluate(*left_expr, env)?;
                if op == "o" {
                    if self.is_truthy(&l_val) { return Ok(l_val); }
                } else if op == "y" {
                    if !self.is_truthy(&l_val) { return Ok(l_val); }
                }
                self.evaluate(*right_expr, env)
            },
            Expression::BinaryOp(left, op, right) => {
                let l_val = self.evaluate(*left, env)?;
                let r_val = self.evaluate(*right, env)?;
                self.evaluate_binary(l_val, &op, r_val)
            },
            Expression::UnaryOp(op, right) => {
                let r_val = self.evaluate(*right, env)?;
                self.evaluate_unary(&op, r_val)
            },
            Expression::FunctionCall(name, args) => {
                let mut eval_args = Vec::new();
                for arg in &args {
                    eval_args.push(self.evaluate(arg.clone(), env)?);
                }
                
                if name == "imprimir" {
                    let out_str: Vec<String> = eval_args.iter().map(|a| format!("{}", a)).collect();
                    println!("{}", out_str.join(" "));
                    return Ok(RuntimeValue::Null);
                }
                
                if name == "rango" {
                    if let Some(RuntimeValue::Number(n)) = eval_args.get(0) {
                        let items = (0..(*n as i64)).map(|i| RuntimeValue::Int(i)).collect();
                        return Ok(RuntimeValue::List(Arc::new(Mutex::new(items))));
                    }
                    if let Some(RuntimeValue::Int(n)) = eval_args.get(0) {
                        let items = (0..*n).map(|i| RuntimeValue::Int(i)).collect();
                        return Ok(RuntimeValue::List(Arc::new(Mutex::new(items))));
                    }
                }
                
                if name == "tipo" {
                    if let Some(arg) = eval_args.get(0) {
                        let t = match arg {
                            RuntimeValue::Number(_) => "numero",
                            RuntimeValue::Int(_) => "entero",
                            RuntimeValue::Boolean(_) => "booleano",
                            RuntimeValue::Text(_) => "texto",
                            RuntimeValue::List(_) => "lista",
                            RuntimeValue::Dictionary(_) => "diccionario",
                            RuntimeValue::Function(_, _) => "funcion",
                            RuntimeValue::AsyncFunction(_, _) => "funcion_asincrona",
                            RuntimeValue::Promise(_) => "promesa",
                            RuntimeValue::Class(_, _) => "clase",
                            RuntimeValue::Instance(_, _, _) => "instancia",
                            RuntimeValue::Server(_) => "servidor",
                            RuntimeValue::Database(_) => "base_datos",
                            RuntimeValue::PyWrapper(_) => "python",
                            RuntimeValue::Null => "nulo",
                            RuntimeValue::Break => "romper",
                        };
                        return Ok(RuntimeValue::Text(t.to_string()));
                    }
                    return Err("tipo() requiere 1 argumento.".into());
                }

                if name == "longitud" {
                    if let Some(arg) = eval_args.get(0) {
                        match arg {
                            RuntimeValue::Text(s) => return Ok(RuntimeValue::Int(s.len() as i64)),
                            RuntimeValue::List(l_arc) => return Ok(RuntimeValue::Int(l_arc.lock().unwrap().len() as i64)),
                            _ => return Err("longitud() solo aplica a texto o listas.".into())
                        }
                    }
                    return Err("longitud() requiere 1 argumento.".into());
                }

                if name == "entrada" {
                    use std::io::{self, Write};
                    if let Some(msg) = eval_args.get(0) {
                        print!("{}", msg);
                    }
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    if io::stdin().read_line(&mut input).is_ok() {
                        return Ok(RuntimeValue::Text(input.trim_end().to_string()));
                    }
                    return Ok(RuntimeValue::Text("".to_string()));
                }

                if name == "a_numero" {
                    if let Some(val) = eval_args.get(0) {
                        return match val {
                            RuntimeValue::Text(s) => {
                                if let Ok(i) = s.parse::<i64>() { Ok(RuntimeValue::Int(i)) }
                                else if let Ok(n) = s.parse::<f64>() { Ok(RuntimeValue::Number(n)) }
                                else { Err(format!("No se puede convertir '{}' a número.", s)) }
                            },
                            RuntimeValue::Int(_) | RuntimeValue::Number(_) => Ok(val.clone()),
                            _ => Err("a_numero() requiere texto o número.".into()),
                        };
                    }
                }

                if name == "a_texto" {
                    if let Some(val) = eval_args.get(0) {
                        return Ok(RuntimeValue::Text(format!("{}", val)));
                    }
                }

                if name == "entero" {
                    if let Some(arg) = eval_args.get(0) {
                        match arg {
                            RuntimeValue::Text(s) => {
                                if let Ok(n) = s.parse::<i64>() { return Ok(RuntimeValue::Int(n)); }
                                return Err(format!("No se puede convertir '{}' a entero.", s));
                            },
                            RuntimeValue::Number(n) => return Ok(RuntimeValue::Int(*n as i64)),
                            RuntimeValue::Int(n) => return Ok(RuntimeValue::Int(*n)),
                            _ => return Err("entero() requiere un texto numérico o número.".into())
                        }
                    }
                    return Err("entero() requiere 1 argumento.".into());
                }
                
                if name == "decimal" {
                    if let Some(arg) = eval_args.get(0) {
                        match arg {
                            RuntimeValue::Text(s) => {
                                if let Ok(n) = s.parse::<f64>() { return Ok(RuntimeValue::Number(n)); }
                                return Err(format!("No se puede convertir '{}' a decimal.", s));
                            },
                            RuntimeValue::Int(n) => return Ok(RuntimeValue::Number(*n as f64)),
                            RuntimeValue::Number(n) => return Ok(RuntimeValue::Number(*n)),
                            _ => return Err("decimal() requiere un texto numérico o número.".into())
                        }
                    }
                    return Err("decimal() requiere 1 argumento.".into());
                }

                if name == "texto" {
                    if let Some(arg) = eval_args.get(0) {
                        return Ok(RuntimeValue::Text(format!("{}", arg)));
                    }
                    return Err("texto() requiere 1 argumento.".into());
                }
                
                if name == "mayusculas" {
                    if let Some(RuntimeValue::Text(s)) = eval_args.get(0) { return Ok(RuntimeValue::Text(s.to_uppercase())); }
                    return Err("mayusculas() requiere texto.".into());
                }
                if name == "minusculas" {
                    if let Some(RuntimeValue::Text(s)) = eval_args.get(0) { return Ok(RuntimeValue::Text(s.to_lowercase())); }
                    return Err("minusculas() requiere texto.".into());
                }
                if name == "contiene" {
                    if let (Some(RuntimeValue::Text(t)), Some(RuntimeValue::Text(s))) = (eval_args.get(0), eval_args.get(1)) { 
                        return Ok(RuntimeValue::Boolean(t.contains(s))); 
                    }
                    return Err("contiene(t, s) requiere dos textos.".into());
                }
                if name == "dividir" {
                    if let (Some(RuntimeValue::Text(t)), Some(RuntimeValue::Text(s))) = (eval_args.get(0), eval_args.get(1)) { 
                        let items: Vec<RuntimeValue> = t.split(s).map(|p| RuntimeValue::Text(p.to_string())).collect();
                        return Ok(RuntimeValue::List(Arc::new(Mutex::new(items))));
                    }
                    return Err("dividir(t, s) requiere dos textos.".into());
                }
                if name == "unir" {
                    if let (Some(RuntimeValue::List(l)), Some(RuntimeValue::Text(s))) = (eval_args.get(0), eval_args.get(1)) { 
                        let items: Vec<String> = l.lock().unwrap().iter().map(|p| format!("{}", p)).collect();
                        return Ok(RuntimeValue::Text(items.join(s)));
                    }
                    return Err("unir(lista, sep) requiere lista y separador texto.".into());
                }
                if name == "agregar" {
                    if eval_args.len() == 2 {
                        if let Some(RuntimeValue::List(l)) = eval_args.get(0) {
                            l.lock().unwrap().push(eval_args[1].clone());
                            return Ok(RuntimeValue::Null);
                        }
                    }
                    return Err("agregar(lista, elemento) requiere lista y elemento.".into());
                }
                if name == "quitar" {
                    if let (Some(RuntimeValue::List(l)), Some(RuntimeValue::Int(i))) = (eval_args.get(0), eval_args.get(1)) {
                        let mut l_lock = l.lock().unwrap();
                        if *i >= 0 && (*i as usize) < l_lock.len() {
                            l_lock.remove(*i as usize);
                            return Ok(RuntimeValue::Null);
                        }
                        return Err(format!("quitar(): índice fuera de límites: {}", i));
                    }
                    return Err("quitar(lista, indice_entero) inválido.".into());
                }
                
                if name == "http_get" {
                    if let Some(RuntimeValue::Text(url)) = eval_args.get(0) {
                        return match ureq::get(url).call() {
                            Ok(resp) => {
                                if let Ok(json) = resp.into_json::<serde_json::Value>() {
                                    let dict = crate::servidor::json_to_nexus(&json);
                                    Ok(RuntimeValue::Promise(Box::new(dict)))
                                } else {
                                    Err("Respuesta de GET no es un JSON válido.".into())
                                }
                            },
                            Err(e) => Err(format!("Error HTTP GET a {}: {}", url, e))
                        };
                    }
                    return Err("http_get() requiere una url de texto.".into());
                }

                if name == "http_post" {
                    if let (Some(RuntimeValue::Text(url)), Some(body_dict)) = (eval_args.get(0), eval_args.get(1)) {
                        let json_str = crate::servidor::nexus_to_json_string(body_dict);
                        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                            return match ureq::post(url).send_json(&json_val) {
                                Ok(resp) => {
                                    if let Ok(json) = resp.into_json::<serde_json::Value>() {
                                        let dict = crate::servidor::json_to_nexus(&json);
                                        Ok(RuntimeValue::Promise(Box::new(dict)))
                                    } else {
                                        Ok(RuntimeValue::Promise(Box::new(RuntimeValue::Text("Respuesta procesada (No JSON)".into()))))
                                    }
                                },
                                Err(e) => Err(format!("Error HTTP POST a {}: {}", url, e))
                            };
                        }
                    }
                    return Err("http_post() requiere url (texto) y un cuerpo (diccionario/lista).".into());
                }

                if name == "ia" {
                    if let Some(arg) = eval_args.get(0) {
                        let prompt = match arg {
                            RuntimeValue::Text(p) => p.clone(),
                            _ => return Err("ia() requiere un texto (prompt).".into())
                        };

                        let mut api_key = std::env::var("AQUILA_AI_KEY").unwrap_or_default();
                        
                        let mut url = "http://localhost:11434/api/generate".to_string();
                        let mut model = "llama3.2:latest".to_string();

                        // Intentar cargar la configuración global (dotfile)
                        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_else(|_| ".".to_string());
                        let path = format!("{}/.aquila_keys", home);
                        if let Ok(config_str) = std::fs::read_to_string(&path) {
                            if let Ok(config_json) = serde_json::from_str::<serde_json::Value>(&config_str) {
                                if let Some(u) = config_json.get("url").and_then(|u| u.as_str()) { url = u.to_string(); }
                                if let Some(c) = config_json.get("clave").and_then(|c| c.as_str()) { api_key = c.to_string(); }
                                if let Some(m) = config_json.get("modelo").and_then(|m| m.as_str()) { model = m.to_string(); }
                            }
                        }

                        // Sobrescribir con env var si existe (retrocompatibilidad)
                        if !api_key.is_empty() && url.contains("localhost") {
                            url = "https://api.openai.com/v1/chat/completions".to_string();
                            model = "gpt-3.5-turbo".to_string();
                        }

                        // Evaluar segundo argumento (Diccionario) si existe
                        if eval_args.len() >= 2 {
                            if let RuntimeValue::Dictionary(dict_arc) = &eval_args[1] {
                                let dict = dict_arc.lock().unwrap();
                                if let Some(RuntimeValue::Text(u)) = dict.get("url") {
                                    url = u.clone();
                                }
                                if let Some(RuntimeValue::Text(c)) = dict.get("clave") {
                                    api_key = c.clone();
                                }
                                if let Some(RuntimeValue::Text(m)) = dict.get("modelo") {
                                    model = m.clone();
                                }
                            }
                        }

                        let is_ollama_generate = url.contains("localhost:11434") && url.ends_with("/generate");
                        let is_anthropic = url.contains("anthropic.com");

                        let body = if is_ollama_generate {
                            serde_json::json!({
                                "model": model,
                                "prompt": prompt,
                                "stream": false
                            })
                        } else if is_anthropic {
                            serde_json::json!({
                                "model": model,
                                "max_tokens": 1024,
                                "messages": [{"role": "user", "content": prompt}]
                            })
                        } else {
                            // Universal OpenAI format
                            serde_json::json!({
                                "model": model,
                                "messages": [{"role": "user", "content": prompt}]
                            })
                        };

                        let execute_request = |req_url: &str, req_body: &serde_json::Value, key: &str, anthropic: bool| -> Result<serde_json::Value, ureq::Error> {
                            let mut request = ureq::post(req_url);
                            if !key.is_empty() {
                                if anthropic {
                                    request = request
                                        .set("x-api-key", key)
                                        .set("anthropic-version", "2023-06-01");
                                } else {
                                    request = request.set("Authorization", &format!("Bearer {}", key));
                                }
                            }
                            let resp = request.send_json(req_body)?;
                            // Un error de parsing en este punto significa mal cuerpo JSON
                            Ok(resp.into_json::<serde_json::Value>().unwrap_or_else(|_| serde_json::json!({})))
                        };

                        let primary_res = execute_request(&url, &body, &api_key, is_anthropic);

                        match primary_res {
                            Ok(json) => {
                                if is_ollama_generate {
                                    if let Some(r) = json.get("response").and_then(|r| r.as_str()) {
                                        return Ok(RuntimeValue::Text(r.to_string()));
                                    }
                                } else if is_anthropic {
                                    if let Some(content) = json.get("content").and_then(|c| c.get(0)).and_then(|c| c.get("text")).and_then(|t| t.as_str()) {
                                        return Ok(RuntimeValue::Text(content.to_string()));
                                    }
                                } else {
                                    if let Some(c) = json.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("message")).and_then(|m| m.get("content")).and_then(|c| c.as_str()) {
                                        return Ok(RuntimeValue::Text(c.to_string()));
                                    }
                                }
                                return Err(format!("Respuesta inesperada de IA: {}", json));
                            },
                            Err(e) => {
                                // Fallback a Ollama si la API primaria falla
                                if !is_ollama_generate {
                                    println!("{}⚠️ Guardián de Aquila: Error en API externa ({}). Cayendo a AI local...{}", "\x1b[33m", e, "\x1b[0m");
                                    let fallback_url = "http://localhost:11434/api/generate";
                                    let fallback_body = serde_json::json!({
                                        "model": "llama3.2:latest",
                                        "prompt": prompt,
                                        "stream": false
                                    });
                                    if let Ok(fallback_json) = execute_request(fallback_url, &fallback_body, "", false) {
                                        if let Some(r) = fallback_json.get("response").and_then(|r| r.as_str()) {
                                            return Ok(RuntimeValue::Text(r.to_string()));
                                        }
                                    }
                                    return Err(format!("API externa falló, y Local Ollama tampoco respondió. [{}]: {}", url, e));
                                }
                                return Err(format!("Error de red con IA (Ollama) [{}]: {}", url, e));
                            }
                        }
                    }
                    return Err("ia() requiere al menos 1 argumento (prompt).".into());
                }

                if name == "leer_archivo" {
                    if let Some(RuntimeValue::Text(ruta)) = eval_args.get(0) {
                        match std::fs::read_to_string(ruta) {
                            Ok(c) => return Ok(RuntimeValue::Text(c)),
                            Err(e) => return Err(format!("Error al leer el archivo {}: {}", ruta, e)),
                        }
                    }
                    return Err("leer_archivo() requiere la ruta en texto.".into());
                }

                if name == "escribir_archivo" {
                    if eval_args.len() == 2 {
                        if let (RuntimeValue::Text(ruta), RuntimeValue::Text(data)) = (&eval_args[0], &eval_args[1]) {
                            match std::fs::write(ruta, data) {
                                Ok(_) => return Ok(RuntimeValue::Null),
                                Err(e) => return Err(format!("Error al escribir el archivo {}: {}", ruta, e)),
                            }
                        }
                    }
                    return Err("escribir_archivo(ruta, datos) requiere 2 argumentos de texto.".into());
                }
                let func_val_res = env.lock().unwrap().get(&name);
                if let Ok(func_val) = func_val_res {
                    match func_val {
                        RuntimeValue::Function(params, body) => {
                            if args.len() != params.len() {
                                return Err(format!("La función '{}' esperaba {} argumentos pero recibió {}.", name, params.len(), args.len()));
                            }
                            let call_env = Arc::new(Mutex::new(Environment::new_with_parent(Arc::clone(env))));
                            for (i, param_name) in params.iter().enumerate() {
                                call_env.lock().unwrap().define(param_name.clone(), eval_args[i].clone());
                            }
                            if let Some(ret) = self.execute_block(body, &call_env)? {
                                return Ok(ret);
                            }
                            return Ok(RuntimeValue::Null);
                        },
                        RuntimeValue::AsyncFunction(params, body) => {
                            if args.len() != params.len() {
                                return Err(format!("La función asíncrona '{}' esperaba {} argumentos pero recibió {}.", name, params.len(), args.len()));
                            }
                            let call_env = Arc::new(Mutex::new(Environment::new_with_parent(Arc::clone(env))));
                            for (i, param_name) in params.iter().enumerate() {
                                call_env.lock().unwrap().define(param_name.clone(), eval_args[i].clone());
                            }
                            if let Some(ret) = self.execute_block(body, &call_env)? {
                                return Ok(RuntimeValue::Promise(Box::new(ret)));
                            }
                            return Ok(RuntimeValue::Promise(Box::new(RuntimeValue::Null)));
                        },
                        _ => {}
                    }
                } else {
                    return Err(format!("Llamada a función desconocida: {}", name));
                }
                Ok(RuntimeValue::Null) // Se agregó para consistencia del match
            },
            Expression::MethodCall(callee_expr, method, args) => {
                let callee_val = self.evaluate(*callee_expr, env)?;
                let mut eval_args = Vec::new();
                for arg in args {
                    eval_args.push(self.evaluate(arg, env)?);
                }
                
                // Servidor web: interceptar .ruta() e .iniciar()
                if let RuntimeValue::Server(server_arc) = &callee_val {
                    if method == "ruta" {
                        if eval_args.len() >= 3 {
                            let http_method = match &eval_args[0] {
                                RuntimeValue::Text(m) => m.clone(),
                                _ => return Err("servidor.ruta() primer arg debe ser 'GET' o 'POST'.".into()),
                            };
                            let path = match &eval_args[1] {
                                RuntimeValue::Text(p) => p.clone(),
                                _ => return Err("servidor.ruta() segundo arg debe ser la ruta como texto.".into()),
                            };
                            let handler = match &eval_args[2] {
                                RuntimeValue::Function(p, b) => (p.clone(), b.clone()),
                                RuntimeValue::AsyncFunction(p, b) => (p.clone(), b.clone()),
                                _ => return Err("servidor.ruta() tercer arg debe ser una función (síncrona o asíncrona).".into()),
                            };
                            server_arc.add_route(http_method, path, handler.0, handler.1);
                            return Ok(RuntimeValue::Null);
                        }
                        return Err("servidor.ruta(metodo, path, handler) requiere 3 argumentos.".into());
                    }
                    if method == "estatico" {
                        if eval_args.len() >= 2 {
                            if let (RuntimeValue::Text(path), RuntimeValue::Text(file_path)) = (&eval_args[0], &eval_args[1]) {
                                server_arc.add_static(path.clone(), file_path.clone());
                                return Ok(RuntimeValue::Null);
                            }
                        }
                        return Err("servidor.estatico(ruta, archivo_local) requiere 2 argumentos de texto.".into());
                    }
                    if method == "iniciar" {
                        let server = Arc::clone(server_arc);
                        return server.start(self, env)
                            .map(|_| RuntimeValue::Null);
                    }
                    return Err(format!("Servidor no tiene método '{}'. Usa .ruta() o .iniciar()", method));
                }
                
                // Base de datos: interceptar .ejecutar() y .consultar()
                if let RuntimeValue::Database(db_arc) = &callee_val {
                    if method == "ejecutar" {
                        if let Some(RuntimeValue::Text(sql)) = eval_args.get(0) {
                            let mut params = Vec::new();
                            if let Some(RuntimeValue::List(l)) = eval_args.get(1) {
                                for p in l.lock().unwrap().iter() {
                                    params.push(format!("{}", p));
                                }
                            }
                            return db_arc.ejecutar(sql, params);
                        }
                        return Err("db.ejecutar() requiere un texto SQL.".into());
                    }
                    if method == "consultar" {
                        if let Some(RuntimeValue::Text(sql)) = eval_args.get(0) {
                            let mut params = Vec::new();
                            if let Some(RuntimeValue::List(l)) = eval_args.get(1) {
                                for p in l.lock().unwrap().iter() {
                                    params.push(format!("{}", p));
                                }
                            }
                            return db_arc.consultar(sql, params);
                        }
                        return Err("db.consultar() requiere un texto SQL.".into());
                    }
                    return Err(format!("BaseDatos no tiene método '{}'. Usa .ejecutar() o .consultar()", method));
                }
                
                // Acceso a propiedad u función de diccionario (ej: modulo.PI o modulo.sumar(a,b))
                if let RuntimeValue::Dictionary(map_arc) = &callee_val {
                    if eval_args.is_empty() {
                        let map = map_arc.lock().unwrap();
                        if let Some(val) = map.get(&method) {
                            return Ok(val.clone());
                        }
                        return Ok(RuntimeValue::Null); // O error si no existe
                    } else {
                        let map = map_arc.lock().unwrap();
                        if let Some(RuntimeValue::Function(params, body)) = map.get(&method) {
                            if eval_args.len() != params.len() {
                                return Err(format!("La función '{}' esperaba {} argumentos pero recibió {}.", method, params.len(), eval_args.len()));
                            }
                            let call_env = Arc::new(Mutex::new(Environment::new_with_parent(Arc::clone(env))));
                            for (i, param_name) in params.iter().enumerate() {
                                call_env.lock().unwrap().define(param_name.clone(), eval_args[i].clone());
                            }
                            if let Some(ret) = self.execute_block(body.clone(), &call_env)? {
                                return Ok(ret);
                            }
                            return Ok(RuntimeValue::Null);
                        }
                    }
                }
                
                // Acceso a propiedad u invocación de objeto instanciado
                if let RuntimeValue::Instance(_, props_arc, class_box) = &callee_val {
                    if eval_args.is_empty() {
                        let props = props_arc.lock().unwrap();
                        if let Some(val) = props.get(&method) {
                            return Ok(val.clone());
                        }
                    }
                    
                    if let RuntimeValue::Class(_, methods_arc) = &**class_box {
                        let methods = methods_arc.lock().unwrap();
                        if let Some(RuntimeValue::Function(params, body)) = methods.get(&method) {
                            if eval_args.len() != params.len() {
                                return Err(format!("El método '{}' esperaba {} argumentos pero recibió {}.", method, params.len(), eval_args.len()));
                            }
                            let call_env = Arc::new(Mutex::new(Environment::new_with_parent(Arc::clone(env))));
                            call_env.lock().unwrap().define("esto".to_string(), callee_val.clone()); // INYECTAR `esto`
                            for (i, param_name) in params.iter().enumerate() {
                                call_env.lock().unwrap().define(param_name.clone(), eval_args[i].clone());
                            }
                            if let Some(ret) = self.execute_block(body.clone(), &call_env)? {
                                return Ok(ret);
                            }
                            return Ok(RuntimeValue::Null);
                        }
                    }
                    return Err(format!("La propiedad o método '{}' no existe en esta instancia.", method));
                }
                
                // Si es un Objeto Python envuelto en Nexus
                if let RuntimeValue::PyWrapper(py_obj) = callee_val {
                    return pyo3::Python::with_gil(|py| -> Result<RuntimeValue, String> {
                        if method == "propiedad" {
                            if let Some(RuntimeValue::Text(prop_name)) = eval_args.get(0) {
                                match py_obj.getattr(py, prop_name.as_str()) {
                                    Ok(res) => return Ok(py_to_val(py, res)),
                                    Err(e) => {
                                        e.print(py);
                                        return Err(format!("No existe la propiedad Python '{}'", prop_name));
                                    }
                                }
                            }
                        }

                        let py_args: Vec<PyObject> = eval_args.into_iter().map(|a| val_to_py(py, a)).collect();
                        let py_args_tuple = pyo3::types::PyTuple::new(py, py_args).unwrap();
                        
                        match py_obj.call_method(py, method.as_str(), py_args_tuple, None) {
                            Ok(res) => Ok(py_to_val(py, res)),
                            Err(e) => {
                                e.print(py);
                                Err(format!("Excepción nativa Python al llamar al método '{}'", method))
                            }
                        }
                    });
                }
                
                Err(format!("El objeto no tiene propiedades o métodos invocables. (método intentado: {})", method))
            },
            Expression::IndexAccess(array_expr, index_expr) => {
                let array = self.evaluate(*array_expr, env)?;
                let index = self.evaluate(*index_expr, env)?;
                
                if let RuntimeValue::List(items_arc) = &array {
                    if let RuntimeValue::Int(idx) = index {
                        let items = items_arc.lock().unwrap();
                        if idx >= 0 && (idx as usize) < items.len() {
                            return Ok(items[idx as usize].clone());
                        } else {
                            return Err(format!("Índice fuera de límites: {}", idx));
                        }
                    }
                    Err("El índice de la lista debe ser un número entero.".into())
                } else if let RuntimeValue::Dictionary(map_arc) = &array {
                    let str_key = match index {
                        RuntimeValue::Text(s) => s,
                        _ => return Err("El índice de un diccionario debe ser texto.".into()),
                    };
                    let map = map_arc.lock().unwrap();
                    if let Some(val) = map.get(&str_key) {
                        Ok(val.clone())
                    } else {
                        Ok(RuntimeValue::Null)
                    }
                } else if let RuntimeValue::PyWrapper(py_obj) = &array {
                    return pyo3::Python::with_gil(|py| -> Result<RuntimeValue, String> {
                        let py_idx = val_to_py(py, index);
                        match py_obj.bind(py).get_item(py_idx) {
                            Ok(res) => Ok(py_to_val(py, res.to_object(py))),
                            Err(e) => {
                                e.print(py);
                                Err("Error en acceso por índice en objeto de Python.".into())
                            }
                        }
                    });
                } else {
                    Err("Solo se permite acceso por índice a listas, diccionarios u objetos de Python.".into())
                }
            }
        }
    }

    fn evaluate_binary(&self, left: RuntimeValue, op: &str, right: RuntimeValue) -> Result<RuntimeValue, String> {
        let (lx, ly) = match (left.clone(), right.clone()) {
            (RuntimeValue::Int(a), RuntimeValue::Int(b)) => (a as f64, b as f64),
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => (a, b),
            (RuntimeValue::Int(a), RuntimeValue::Number(b)) => (a as f64, b),
            (RuntimeValue::Number(a), RuntimeValue::Int(b)) => (a, b as f64),
            // Strings
            (RuntimeValue::Text(t1), _) if op == "+" => {
                let res = format!("{}{}", t1, right);
                return Ok(RuntimeValue::Text(res));
            },
            (_, RuntimeValue::Text(t2)) if op == "+" => {
                let res = format!("{}{}", left, t2);
                return Ok(RuntimeValue::Text(res));
            },
            _ => return match op {
                "==" => Ok(RuntimeValue::Boolean(left == right)),
                "!=" => Ok(RuntimeValue::Boolean(left != right)),
                _ => Err(format!("No se puede aplicar operador '{}' a '{}' y '{}'", op, left, right))
            }
        };

        match op {
            "+" => Ok(RuntimeValue::Number(lx + ly)),
            "-" => Ok(RuntimeValue::Number(lx - ly)),
            "*" => Ok(RuntimeValue::Number(lx * ly)),
            "/" => {
                if ly == 0.0 { return Err("División por cero.".into()); }
                Ok(RuntimeValue::Number(lx / ly))
            },
            "==" => Ok(RuntimeValue::Boolean(lx == ly)),
            "!=" => Ok(RuntimeValue::Boolean(lx != ly)),
            ">" => Ok(RuntimeValue::Boolean(lx > ly)),
            "<" => Ok(RuntimeValue::Boolean(lx < ly)),
            ">=" => Ok(RuntimeValue::Boolean(lx >= ly)),
            "<=" => Ok(RuntimeValue::Boolean(lx <= ly)),
            _ => Err(format!("Operador numérico desconocido: '{}'", op)),
        }
    }

    fn evaluate_unary(&self, op: &str, right: RuntimeValue) -> Result<RuntimeValue, String> {
        match op {
            "-" => {
                match right {
                    RuntimeValue::Int(i) => Ok(RuntimeValue::Int(-i)),
                    RuntimeValue::Number(n) => Ok(RuntimeValue::Number(-n)),
                    _ => Err("El operador '-' solo se aplica a números.".into())
                }
            },
            "no" => {
                Ok(RuntimeValue::Boolean(!self.is_truthy(&right)))
            },
            _ => Err(format!("Operador unario desconocido: '{}'", op)),
        }
    }

    fn is_truthy(&self, val: &RuntimeValue) -> bool {
        match val {
            RuntimeValue::Null => false,
            RuntimeValue::Boolean(b) => *b,
            RuntimeValue::Int(i) => *i != 0,
            RuntimeValue::Number(n) => *n != 0.0,
            RuntimeValue::Text(s) => !s.is_empty(),
            RuntimeValue::List(l) => !l.lock().unwrap().is_empty(),
            RuntimeValue::Dictionary(d) => !d.lock().unwrap().is_empty(),
            RuntimeValue::Function(_, _) => true,
            RuntimeValue::AsyncFunction(_, _) => true,
            RuntimeValue::Promise(_) => true,
            RuntimeValue::Class(_, _) => true,
            RuntimeValue::Instance(_, _, _) => true,
            RuntimeValue::Server(_) => true,
            RuntimeValue::Database(_) => true,
            RuntimeValue::PyWrapper(_) => true,
            RuntimeValue::Break => false,
        }
    }
}
