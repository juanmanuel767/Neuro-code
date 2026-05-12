#[cfg(test)]
mod tests {
    use crate::lexer::tokenize;
    use crate::parser::parse;
    use crate::builder::compile_ast;
    use crate::graph::GraphRuntime;
    use crate::node::PulseValue;

    #[test]
    fn test_carrito_vs_benchmark_default() {
        let code = r#"
            node Precio_Monitor = 200.0
            node Cantidad_Monitores = 1.0
            node Precio_Teclado = 50.0
            node Cantidad_Teclados = 1.0
            node Impuesto_Porcentaje = 19.0
            node Descuento_Porcentaje = 10.0
            node TOTAL_A_PAGAR_FINAL = Formula("((Precio_Monitor * Cantidad_Monitores) + (Precio_Teclado * Cantidad_Teclados)) * (1 + (Impuesto_Porcentaje / 100)) * (1 - (Descuento_Porcentaje / 100))")
        "#;

        let tokens = tokenize(code);
        let ast = parse(tokens);
        let mut runtime = GraphRuntime::new();
        compile_ast(ast, &mut runtime, "");
        runtime.tick(); // Evaluar

        // El carrito original debe dar = (200*1 + 50*1) * 1.19 * 0.9 = 250 * 1.19 * 0.9 = 267.75
        let total = runtime.nodes.get("TOTAL_A_PAGAR_FINAL").unwrap();
        match total.cached_value.as_ref().unwrap() {
            PulseValue::Num(v) => {
                // Approximate equals
                assert!((v - 267.75).abs() < 0.01, "Expected 267.75 but got {}", v);
            },
            _ => panic!("Expected Num"),
        }

        // Test variables of input 
        assert_eq!(runtime.nodes.get("Precio_Monitor").unwrap().dependencies.len(), 0);
        assert_eq!(runtime.nodes.get("TOTAL_A_PAGAR_FINAL").unwrap().dependencies.len(), 6);
    }

    #[test]
    fn test_carrito_vs_benchmark_all_nines() {
        let code = r#"
            node Precio_Monitor = 200.0
            node Cantidad_Monitores = 1.0
            node Precio_Teclado = 50.0
            node Cantidad_Teclados = 1.0
            node Impuesto_Porcentaje = 19.0
            node Descuento_Porcentaje = 10.0
            node TOTAL_A_PAGAR_FINAL = Formula("((Precio_Monitor * Cantidad_Monitores) + (Precio_Teclado * Cantidad_Teclados)) * (1 + (Impuesto_Porcentaje / 100)) * (1 - (Descuento_Porcentaje / 100))")
        "#;

        let tokens = tokenize(code);
        let ast = parse(tokens);
        let mut runtime = GraphRuntime::new();
        compile_ast(ast, &mut runtime, "");
        
        runtime.update_variable("Precio_Monitor", PulseValue::Num(9.0));
        runtime.update_variable("Cantidad_Monitores", PulseValue::Num(9.0));
        runtime.update_variable("Precio_Teclado", PulseValue::Num(9.0));
        runtime.update_variable("Cantidad_Teclados", PulseValue::Num(9.0));
        runtime.update_variable("Impuesto_Porcentaje", PulseValue::Num(9.0));
        runtime.update_variable("Descuento_Porcentaje", PulseValue::Num(9.0));
        
        runtime.tick();

        let total = runtime.nodes.get("TOTAL_A_PAGAR_FINAL").unwrap();
        match total.cached_value.as_ref().unwrap() {
            PulseValue::Num(v) => {
                assert!((v - 160.6878).abs() < 0.01, "Expected ~160.69 but got {}", v);
            },
            _ => panic!("Expected Num"),
        }
    }

    #[test]
    fn test_intelligent_types() {
        let code = r#"
            node Edad: Int = 25
            node Activo: Bool = true
            node Dinero: Num = 10.5
            node Articulos: List = [1, null, false, 99.9]
            node Nombre = "Test"
            node FormulaBoolean = Formula("Edad > 18")
            node FormulaString = Formula("Nombre")
        "#;

        let tokens = tokenize(code);
        let ast = parse(tokens);
        let mut runtime = GraphRuntime::new();
        compile_ast(ast, &mut runtime, "");
        runtime.tick();

        let node_edad = runtime.nodes.get("Edad").unwrap().cached_value.as_ref().unwrap();
        assert!(matches!(node_edad, PulseValue::Int(25)));

        let node_activo = runtime.nodes.get("Activo").unwrap().cached_value.as_ref().unwrap();
        assert!(matches!(node_activo, PulseValue::Bool(true)));

        let node_dinero = runtime.nodes.get("Dinero").unwrap().cached_value.as_ref().unwrap();
        assert!(matches!(node_dinero, PulseValue::Num(10.5)));

        let node_arts = runtime.nodes.get("Articulos").unwrap().cached_value.as_ref().unwrap();
        if let PulseValue::List(l) = node_arts {
            assert_eq!(l.len(), 4);
            assert!(matches!(l[0], PulseValue::Int(1)));
            assert!(matches!(l[1], PulseValue::Null));
            assert!(matches!(l[2], PulseValue::Bool(false)));
            assert!(matches!(l[3], PulseValue::Num(99.9)));
        } else {
            panic!("Expected List");
        }

        let node_f_bool = runtime.nodes.get("FormulaBoolean").unwrap().cached_value.as_ref().unwrap();
        assert!(matches!(node_f_bool, PulseValue::Bool(true)));
        
        let node_f_str = runtime.nodes.get("FormulaString").unwrap().cached_value.as_ref().unwrap();
        if let PulseValue::Text(s) = node_f_str {
            assert_eq!(s, "Test");
        } else {
            panic!("Expected Text");
        }
    }

    #[test]
    fn test_native_functions() {
        let code = r#"
            func Doble(X) = Formula("X * 2")
            func Cuadrado(Y) = Formula("Y * Y")
            
            node Base = 5
            node Mult = Doble(Base)
            node Potencia = Cuadrado(Mult)
        "#;
        let tokens = tokenize(code);
        let ast = parse(tokens);
        let mut runtime = GraphRuntime::new();
        compile_ast(ast, &mut runtime, "");
        runtime.tick();
        
        let n_mult = runtime.nodes.get("Mult").unwrap().cached_value.as_ref().unwrap();
        assert!(matches!(n_mult, PulseValue::Int(10) | PulseValue::Num(10.0)));
        
        let n_pot = runtime.nodes.get("Potencia").unwrap().cached_value.as_ref().unwrap();
        assert!(matches!(n_pot, PulseValue::Int(100) | PulseValue::Num(100.0)));
    }

    #[test]
    fn test_module_imports() {
        // Setup mock modules
        let module_code = r#"
            func Geometria_Area(Lado) = Formula("Lado * Lado")
            node PI = 3.1415
        "#;
        std::fs::write("test_geometria.nx", module_code).unwrap();
        
        let app_code = r#"
            import "test_geometria.nx"
            
            node MiLado = 10
            node MiArea = Geometria_Area(MiLado)
            node Radio = 5
            node Circulo = Formula("PI * Radio * Radio")
        "#;
        
        let tokens = tokenize(app_code);
        let ast = parse(tokens);
        let mut runtime = GraphRuntime::new();
        // Use current dir for base
        compile_ast(ast, &mut runtime, ".");
        runtime.tick();
        
        let mi_area = runtime.nodes.get("MiArea").unwrap().cached_value.as_ref().unwrap();
        assert!(matches!(mi_area, PulseValue::Int(100) | PulseValue::Num(100.0)));
        
        let circ = runtime.nodes.get("Circulo").unwrap().cached_value.as_ref().unwrap();
        if let PulseValue::Num(v) = circ {
            assert!((v - 78.5375).abs() < 0.01);
        } else {
            panic!("Expected Num for circle area");
        }
        
        // Clean up
        let _ = std::fs::remove_file("test_geometria.nx");
    }

    #[test]
    fn test_flow_control() {
        let code = r#"
            node Edad = 25
            node Mensaje = if Edad >= 18 { "Mayor" } else { "Menor" }
            
            node Nivel = 2
            node Rango = match Nivel {
                1 => "Bronce"
                2 => "Plata"
                3 => "Oro"
                _ => "Desconocido"
            }
            
            node EsVIP = Nivel == 3
            node CheckMenor = Edad < 30
        "#;
        
        let tokens = tokenize(code);
        let ast = parse(tokens);
        let mut runtime = GraphRuntime::new();
        compile_ast(ast, &mut runtime, ".");
        runtime.tick();
        
        let msg = runtime.nodes.get("Mensaje").unwrap().cached_value.as_ref().unwrap();
        if let PulseValue::Text(t) = msg {
            assert_eq!(t, "Mayor");
        } else {
            panic!("Expected Text");
        }
        
        let rango = runtime.nodes.get("Rango").unwrap().cached_value.as_ref().unwrap();
        if let PulseValue::Text(t) = rango {
            assert_eq!(t, "Plata");
        } else {
            panic!("Expected Text");
        }
        
        let es_vip = runtime.nodes.get("EsVIP").unwrap().cached_value.as_ref().unwrap();
        assert!(matches!(es_vip, PulseValue::Bool(false)));
        
        let ck_menor = runtime.nodes.get("CheckMenor").unwrap().cached_value.as_ref().unwrap();
        assert!(matches!(ck_menor, PulseValue::Bool(true)));
    }

    #[test]
    fn test_stdlib() {
        let code = r#"
            node Radio = 16
            node Raiz = Math.sqrt(Radio)
            
            node Nombre = "nexus"
            node Titulo = Text.upper(Nombre)
            
            node Numeros = [1, 2, 3]
            node MasNumeros = List.push(Numeros, 4)
            node Count = List.len(MasNumeros)
            
            node Pi = Math.pi()
        "#;
        
        let tokens = tokenize(code);
        let ast = parse(tokens);
        let mut runtime = GraphRuntime::new();
        compile_ast(ast, &mut runtime, ".");
        runtime.tick();
        
        let raiz = runtime.nodes.get("Raiz").unwrap().cached_value.as_ref().unwrap();
        if let PulseValue::Num(n) = raiz {
            assert_eq!(*n, 4.0);
        } else {
            panic!("Expected Num");
        }
        
        let titulo = runtime.nodes.get("Titulo").unwrap().cached_value.as_ref().unwrap();
        if let PulseValue::Text(t) = titulo {
            assert_eq!(t, "NEXUS");
        } else {
            panic!("Expected Text");
        }
        
        let count = runtime.nodes.get("Count").unwrap().cached_value.as_ref().unwrap();
        if let PulseValue::Int(n) = count {
            assert_eq!(*n, 4);
        } else {
            panic!("Expected Int");
        }
        
        // Assert PI is loaded
        let _ = runtime.nodes.get("Pi").unwrap().cached_value.as_ref().unwrap();
    }

    #[test]
    fn test_loops() {
        let code = r#"
            node Lista = [1, 2, 3, 4]
            node Dobles = for n in Lista { Formula("n * 2") }
            
            node Texto = "abc"
            node Repercusiones = for char in Lista { Texto }
        "#;
        
        let tokens = tokenize(code);
        let ast = parse(tokens);
        let mut runtime = GraphRuntime::new();
        compile_ast(ast, &mut runtime, ".");
        runtime.tick();
        
        let dobles = runtime.nodes.get("Dobles").unwrap().cached_value.as_ref().unwrap();
        if let PulseValue::List(l) = dobles {
            assert_eq!(l.len(), 4);
            assert!(matches!(l[0], PulseValue::Int(2) | PulseValue::Num(2.0)));
            assert!(matches!(l[3], PulseValue::Int(8) | PulseValue::Num(8.0)));
        } else {
            panic!("Expected List for Dobles");
        }
        
        let rep = runtime.nodes.get("Repercusiones").unwrap().cached_value.as_ref().unwrap();
        if let PulseValue::List(l) = rep {
            assert_eq!(l.len(), 4);
            if let PulseValue::Text(t) = &l[0] {
                assert_eq!(t, "abc");
            } else {
                panic!("Expected Text in Repercusiones");
            }
        } else {
            panic!("Expected List for Repercusiones");
        }
    }
}
