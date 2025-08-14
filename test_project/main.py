#!/usr/bin/env python3
"""
Sample Python project for testing FlowVersion functionality.
This is a simple calculator application.
"""

import sys

def add(a, b):
    """Add two numbers."""
    return a + b

def subtract(a, b):
    """Subtract two numbers."""
    return a - b

def multiply(a, b):
    """Multiply two numbers."""
    return a * b

def divide(a, b):
    """Divide two numbers."""
    if b == 0:
        raise ValueError("Cannot divide by zero")
    return a / b

def main():
    """Main calculator function."""
    print("Simple Calculator")
    print("Available operations: +, -, *, /")
    
    try:
        a = float(input("Enter first number: "))
        operation = input("Enter operation (+, -, *, /): ")
        b = float(input("Enter second number: "))
        
        if operation == '+':
            result = add(a, b)
        elif operation == '-':
            result = subtract(a, b)
        elif operation == '*':
            result = multiply(a, b)
        elif operation == '/':
            result = divide(a, b)
        else:
            print("Invalid operation")
            return
            
        print(f"Result: {a} {operation} {b} = {result}")
        
    except ValueError as e:
        print(f"Error: {e}")
    except KeyboardInterrupt:
        print("\nCalculator terminated.")

if __name__ == "__main__":
    main()