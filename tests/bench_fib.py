# Fibonacci recursivo en Python
import time

def fib(n):
    if n <= 1: return n
    return fib(n-1) + fib(n-2)

start = time.time()
result = fib(30)
end = time.time()

print(f"Python Fibonacci(30): {result}")
print(f"Tiempo: {end - start:.4f} segundos")
