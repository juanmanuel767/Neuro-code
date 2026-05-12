use std::collections::HashMap;
use crate::node::PulseValue;

pub type StdlibFunc = fn(Vec<PulseValue>) -> PulseValue;

pub struct StdlibRegistry {
    pub modules: HashMap<String, HashMap<String, StdlibFunc>>,
}

impl StdlibRegistry {
    pub fn new() -> Self {
        let mut reg = StdlibRegistry {
            modules: HashMap::new(),
        };
        reg.register_math();
        reg.register_text();
        reg.register_list();
        reg.register_ai();
        reg
    }

    pub fn call(&self, module: &str, method: &str, args: Vec<PulseValue>) -> Option<PulseValue> {
        println!("Calling stdlib: {}.{} with args: {:?}", module, method, args);
        if let Some(mod_map) = self.modules.get(module) {
            if let Some(func) = mod_map.get(method) {
                let res = func(args);
                println!("Stdlib result: {:?}", res);
                return Some(res);
            } else {
                println!("Method not found: {}", method);
            }
        } else {
            println!("Module not found: {}", module);
        }
        None
    }

    fn register_math(&mut self) {
        let mut math: HashMap<String, StdlibFunc> = HashMap::new();
        
        math.insert("sqrt".to_string(), |args: Vec<PulseValue>| {
            if args.len() == 1 {
                if let PulseValue::Num(n) = args[0] { return PulseValue::Num(n.sqrt()); }
                if let PulseValue::Int(n) = args[0] { return PulseValue::Num((n as f64).sqrt()); }
            }
            PulseValue::Null
        });
        
        math.insert("pow".to_string(), |args: Vec<PulseValue>| {
            if args.len() == 2 {
                match (args[0].clone(), args[1].clone()) {
                    (PulseValue::Num(b), PulseValue::Num(e)) => return PulseValue::Num(b.powf(e)),
                    (PulseValue::Int(b), PulseValue::Int(e)) => return PulseValue::Int(b.pow(e as u32)),
                    (PulseValue::Num(b), PulseValue::Int(e)) => return PulseValue::Num(b.powi(e as i32)),
                    (PulseValue::Int(b), PulseValue::Num(e)) => return PulseValue::Num((b as f64).powf(e)),
                    _ => {}
                }
            }
            PulseValue::Null
        });
        
        math.insert("pi".to_string(), |_: Vec<PulseValue>| PulseValue::Num(std::f64::consts::PI));

        math.insert("abs".to_string(), |args: Vec<PulseValue>| {
            if args.len() == 1 {
                if let PulseValue::Num(n) = args[0] { return PulseValue::Num(n.abs()); }
                if let PulseValue::Int(n) = args[0] { return PulseValue::Int(n.abs()); }
            }
            PulseValue::Null
        });

        math.insert("random".to_string(), |_: Vec<PulseValue>| {
            // Placeholder: return a pseudo-random value without external crates for now
            PulseValue::Num(0.42)
        });

        self.modules.insert("Math".to_string(), math);
    }

    fn register_text(&mut self) {
        let mut text: HashMap<String, StdlibFunc> = HashMap::new();
        
        text.insert("len".to_string(), |args: Vec<PulseValue>| {
            if args.len() == 1 {
                if let PulseValue::Text(t) = &args[0] { return PulseValue::Int(t.len() as i64); }
            }
            PulseValue::Null
        });
        
        text.insert("upper".to_string(), |args: Vec<PulseValue>| {
            if args.len() == 1 {
                if let PulseValue::Text(t) = &args[0] { return PulseValue::Text(t.to_uppercase()); }
            }
            PulseValue::Null
        });

        text.insert("lower".to_string(), |args: Vec<PulseValue>| {
            if args.len() == 1 {
                if let PulseValue::Text(t) = &args[0] { return PulseValue::Text(t.to_lowercase()); }
            }
            PulseValue::Null
        });

        text.insert("contains".to_string(), |args: Vec<PulseValue>| {
            if args.len() == 2 {
                if let (PulseValue::Text(t), PulseValue::Text(p)) = (&args[0], &args[1]) {
                    return PulseValue::Bool(t.contains(p));
                }
            }
            PulseValue::Null
        });

        self.modules.insert("Text".to_string(), text);
    }

    fn register_list(&mut self) {
        let mut list: HashMap<String, StdlibFunc> = HashMap::new();
        
        list.insert("len".to_string(), |args: Vec<PulseValue>| {
            if args.len() == 1 {
                if let PulseValue::List(l) = &args[0] { return PulseValue::Int(l.len() as i64); }
            }
            PulseValue::Null
        });
        
        list.insert("push".to_string(), |args: Vec<PulseValue>| {
            if args.len() == 2 {
                if let PulseValue::List(l) = &args[0] {
                    let mut new_list = l.clone();
                    new_list.push(args[1].clone());
                    return PulseValue::List(new_list);
                }
            }
            PulseValue::Null
        });

        list.insert("at".to_string(), |args: Vec<PulseValue>| {
            if args.len() == 2 {
                if let (PulseValue::List(l), PulseValue::Int(idx)) = (&args[0], args[1].clone()) {
                    if idx >= 0 && (idx as usize) < l.len() {
                        return l[idx as usize].clone();
                    }
                }
            }
            PulseValue::Null
        });

        self.modules.insert("List".to_string(), list);
    }

    fn register_ai(&mut self) {
        let mut ai: HashMap<String, StdlibFunc> = HashMap::new();
        
        ai.insert("classify".to_string(), |args: Vec<PulseValue>| {
            if args.len() >= 2 {
                if let (PulseValue::Num(val), PulseValue::Num(threshold)) = (&args[0], &args[1]) {
                    if *val > *threshold {
                        return PulseValue::Text("ALERTA".to_string());
                    } else {
                        return PulseValue::Text("NORMAL".to_string());
                    }
                }
            }
            PulseValue::Null
        });

        // AI.predict(valor, [w1, w2, w3...]) -> Realiza una predicción basada en pesos
        ai.insert("predict".to_string(), |args: Vec<PulseValue>| {
            if args.len() == 2 {
                if let (PulseValue::Num(input), PulseValue::List(weights)) = (&args[0], &args[1]) {
                    let mut sum = 0.0;
                    for w in weights {
                        if let PulseValue::Num(wv) = w { sum += input * wv; }
                        else if let PulseValue::Int(wv) = w { sum += input * (*wv as f64); }
                    }
                    // Sigmoid simplificada para dar un valor entre 0 y 1
                    let result = 1.0 / (1.0 + (-sum).exp());
                    return PulseValue::Num(result);
                }
            }
            PulseValue::Null
        });

        // AI.gradient(actual, objetivo) -> Calcula la diferencia para ajuste
        ai.insert("gradient".to_string(), |args: Vec<PulseValue>| {
            if args.len() == 2 {
                if let (PulseValue::Num(actual), PulseValue::Num(target)) = (&args[0], &args[1]) {
                    return PulseValue::Num(target - actual);
                }
            }
            PulseValue::Null
        });

        ai.insert("anomaly".to_string(), |args: Vec<PulseValue>| {
            if args.len() == 1 {
                if let PulseValue::Num(val) = args[0] {
                    if val > 0.9 || val < 0.1 {
                        return PulseValue::Bool(true);
                    }
                    return PulseValue::Bool(false);
                }
            }
            PulseValue::Null
        });

        // AI.map(base, tam) -> Genera un mapa de activación (lista)
        ai.insert("map".to_string(), |args: Vec<PulseValue>| {
            if args.len() == 2 {
                if let (PulseValue::Num(base), PulseValue::Int(size)) = (&args[0], args[1].clone()) {
                    let mut list = Vec::new();
                    for i in 0..size {
                        let offset = (i as f64) * 0.1;
                        list.push(PulseValue::Num((base + offset).sin().abs()));
                    }
                    return PulseValue::List(list);
                }
            }
            PulseValue::Null
        });

        self.modules.insert("AI".to_string(), ai);
    }
}
