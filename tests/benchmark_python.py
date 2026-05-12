import sys

def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

print("Calculando Fibonacci(25) en Python...")
resultado = fibonacci(25)
print("Resultado:", resultado)

def collatz_steps(n):
    pasos = 0
    while n > 1:
        if n % 2 == 0:
            n = n // 2
        else:
            n = 3 * n + 1
        pasos += 1
    return pasos

print("Calculando Pasos de Collatz para 1000 iteraciones (N=27)...")
total_pasos = 0
for i in range(1000):
    total_pasos += collatz_steps(27)
print("Collatz Pasos Total:", total_pasos)
print("✅ ¡TERMINADO!")
