
# Jaguar

Jaguar is a systems programming language aimed for expressiveness, direct C interop, and a layout that reads clean - like a blend of modern syntax and low-level power.

> Still in early development. Very Restrictive because of unfinshed features

The Jaguar Project was the the aim of giving users a laguage with Rust-like syntax while still offering the system power of C/C++. No Borrow-Checker. No GC.

---

## Features (Implemented)

### **Basic Structure**

```jaguar

// this is a single-line comment

fn foo() { // no return type specification = void
  jprintln("Hello World");
}

fn main() { 
  /* 
   * Exception is main. return type is inferred to int 
   * a.k.a i32
   */

  let bar : str = "baz"; // Varibale definition

  foo();
}
```

### **Blocks**

Blocks are primarily containers for data and behaviour. Think Structs.
```jagaur
block Person {
  name: str,
  age: int,
}
```
Blocks can also hold methods too.

```jaguar
block Person {
  name: str,
  age: int,
  fn sayHi(self) {
    jprintln("{s} says hi", self.name);
  }
}
```
usage:

```jaguar
fn main() {

  let me: Block = {name: "Eggo", age: 17};
  me.sayHi(); // output 'Eggo says hi'

}
```

### **C-Interoperability**

Jaguar is capable of using externally decalared symbols. although it is currently limited to functions.

```jaguar
extern malloc(bytes: u64): ptr<void>;
```

It allows for a level of integration to exsisting C codebases.

### **Generics**
> Not fully implemented. but works to some extent

```jaguar

// Defining a generic block
block Option[T] {
  value: T,
  fn unwrap(self): T {
    // this is purely for example
    ret self.value;
  }
}

// Defining a generic function
fn newOption[T](value: T): Option<T> {
  ret {value: value};
}

```

> As i said before, Jaguar is still in it very early stages. more features will be added as it grows to make it a more standard, usable language


# How to use
* To build the project
```
  $ cargo build
```

* To use the compiler
```
  $ jagc source.jr -o output.jr
```
> Note: Use the actual path to the compiler and correctly specify path to source and output.


> Jaguar initializes a directory called "build" for build artifacts so avoid having an exsisting build directory in the same project folder