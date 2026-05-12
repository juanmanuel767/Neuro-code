# ==========================================
# PYTHON TRADICIONAL: Calculador de Salud
# ==========================================
import math

def calcular_imc(peso, altura):
    imc = peso / (altura * altura)
    return imc

def estado_salud(imc):
    if imc < 18.5: return "Bajo Peso (1.0)"
    elif imc > 25.0: return "Sobrepeso (1.0)"
    else: return "Saludable (0.0)"

def run_app():
    print("--- APP DE SALUD EN PYTHON COMPILADO EN TERMINAL ---")
    print("Escribe 'salir' para terminar la ejecución.\n")
    
    while True:
        try:
            p_input = input("Ingresa tu Peso en kg (ej. 70.0): ")
            if p_input.lower() == 'salir': break
            
            a_input = input("Ingresa tu Altura en m (ej. 1.75): ")
            if a_input.lower() == 'salir': break
            
            peso = float(p_input)
            altura = float(a_input)
            
            imc = calcular_imc(peso, altura)
            estado = estado_salud(imc)
            
            print(f"\n[CARGANDO RESULTADO...]")
            print(f">> RESULTADO BMI: {imc:.2f}")
            print(f">> ESTADO: {estado}")
            print("================================\n")
        except ValueError:
            print("Entrada inválida, por favor ingresa solo números (usa '.' para decimales).\n")

if __name__ == "__main__":
    run_app()
