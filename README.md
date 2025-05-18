
Jaguar

Jaguar is a systems programming language focused on expressiveness, direct C interop, and a layout that reads clean — like a blend of modern syntax and low-level power.

> Still in early development. Some features are restrictive or missing. Expect sharp edges.




---

Why Jaguar?

The Jaguar Project was born from the idea of giving users a language with Rust-like syntax but with the raw control and performance of C/C++.

No borrow checker

No garbage collector

No runtime overhead
Just straight-up systems programming — your metal, your rules.



---

Features (Implemented)

Basic Structure

```jaguar

// This is a single-line comment

fn foo() { // No return type specified = void
  jprintln("Hello World");
}

fn main() {
  /*
   * Special case: main's return type is inferred as int (i32)
   */
  let bar: str = "baz"; // Variable declaration

  foo();
}

```

---

Blocks

Blocks are the core user-defined types in Jaguar — like structs but better.
They can hold both data and methods.

```

block Person {
  name: str,
  age: int,
}

```

Methods inside a block:

```

block Person {
  name: str,
  age: int,

  fn sayHi(self) {
    jprintln("{s} says hi", self.name);
  }
}

```

Usage:

```

fn main() {
  let me: Person = { name: "Eggo", age: 17 };
  me.sayHi(); // Output: Eggo says hi
}

```

---

C Interoperability

Jaguar supports external function declarations, allowing you to directly call C functions.

extern malloc(bytes: u64): ptr<void>;
extern free(ptr: ptr<void>): void;

This makes it easy to integrate Jaguar with existing C codebases — useful for system tools, custom allocators, or FFI-heavy projects.


---

Generics (Partially Implemented)

Jaguar includes early-stage generics, allowing parametric polymorphism in blocks and functions.

```

block Option[T] {
  value: T,

  fn unwrap(self): T {
    // For demo purposes
    ret self.value;
  }
}

fn newOption[T](value: T): Option<T> {
  ret { value: value };
}


```

---

How to Use

To build the project:

```
$ cargo build
```

To compile a Jaguar source file:

```
$ ./target/debug/jagc path/to/source.jr -o path/to/output
```

Jaguar creates a directory called build/ to store build artifacts.
Avoid naming your own directories build/ inside your projects to prevent conflicts.


---

Status

Jaguar is in active development.
Features like type inference, traits, shorthands, while-loops, and full error diagnostics are coming.
The core goal is to make Jaguar a usable, expressive systems language that feels modern but stays manual.


---

License

This project is licensed under the MIT License.

