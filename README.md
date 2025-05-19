
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

## Features (Implemented)

### Built-in Data types
Jaguar comes with all the expected built-in types. these include:

  * Integers: Jaguar has 7 integer types that end with a number representing its size in bits.
  
      * **u8** :  8 bit(1 byte) unsigned integer
      * **u16**: 16 bit(2 byte) unsigned integer
      * **u32**: 32 bit(4 byte) unsigned integer
      * **u64**: 64 bit(8 byte) unsigned integer

      * **i8** :  8 bit(1 byte) signed integer
      * **i16**: 16 bit(2 byte) signed integer
      * **i32**: 32 bit(4 byte) signed integer
      * **i64**: 64 bit(8 byte) signed integer

    Jaguar also has an **int** type which is an alias for **i32**.

  * Strings: This is denoted by the **str** keyword and it maps directly to **char\*** from C.

  * Char: Denoted by the **char** keyword, It can only hold a single character.
    ```jaguar
      let foo: char = 'H';
    ```

    Chars use the single-quotes(') while Strings use double-quotes(").

  * Fixed-Lists: Jaguar also has inbuilt support for Fixed lists. These are basically arrays that do not grow in size.
      * **list<T, N>** : where T is equal to the Type of data it holds and N is the Number of said data it will hold

    Fixed-Lists support indexing. Where the first element begins at index 0.
    ```jaguar
      let example : list<str, 2> = ["Hello", "World"];
      jprintln("{s}", example[0]); // output: "Hello"
    ```
  * Pointers: In Jaguar, pointers work just as they do in C.
      * **ptr<T>** : Where T is the type of the object it is pointing to.
  
    ```jaguar
      let foo : int = 3;
      let bar: ptr<int> = &foo;
      jprintln("{d}", *bar);
    ```

    Here, the **&** operator is used to take the address of a variable while the **\*** operator is used to dereference a pointer.

  > Note: Jaguar doesn't have any **bool** types as of now but it will be added in the nearest future.

### Basic Structure

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

## Structs

Structs are the core user-defined types in Jaguar. They can hold both data and methods.

```

struct Person {
  name: str,
  age: int,
}

```

Methods inside a struct:

```

struct Person {
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
```jaguar
  extern malloc(bytes: u64): ptr<void>;
  extern free(ptr: ptr<void>): void;
  extern printf(fmt: str, ...): void;
```

This makes it easy to integrate Jaguar with existing C codebases — useful for system tools, custom allocators, or FFI-heavy projects.

> Note: Only External Function have support for variadic arguments. In-Jaguar functions do not have support yet.

---

## Control Flow (Partially implemented)

### If Statements

```jaguar

    let x: int = 3;
    if x == 3 {
      jprintln("x is 3");
    }

``` 

> Note: Else and Else-if statements are not supported yet

### For Loops

```jaguar

  for(i = 0; i < 3; i= i+1) {
    jprintln("{d}", i);
  }

```

For loops are defined with:
  * An initial value e.g 'i = 0'
  * A condition e.g 'i < 3'
  * And an increment expression e.g 'i = i + 1'
  
It must only have a single set of these. so stuff like:

```jaguar

  for(i = 0, j = 1; i< 3, j != 2; i = i + 1, j = j + 1) {

  }

```
is not allowed.

**While loops** and **For-Each** loops aren't a feature yet but they are surely in my checklist.


## Let Statements

You can create a variable using the let statement.

```jaguar

  let foo: int = 3;

```

The Let statement takes:
  * An Identifier
  * A Type
  * And a Value

The type can be inferred by the compiler, Only if the type of the value is known

```jaguar

  let foo := "Hello"; // inferred as str
  let bar := 32;      // inferred as int(i32)
  let baz := 'W';     // inferred as char
  let quux:= ["hello", "world"]; // not yet supported but will infer as list<str, 2>

```

## Modularization

You can split your codebase into multiple files and import them as needed. In Jaguar this works by **Bundling** the symbols from a file into a namespace

foo.jr
```jaguar

  fn foo() {

  }

  fn bar() {

  }

```

main.jr
```jaguar

  bundle "foo.jr" as foo;

```

The Bundle Statement takes:

  * Path to the file. Either Absolute or Relative
  * An alias. Acts as the namespace to put the symbols
  
main.jr
```jaguar

  bundle "foo.jr" as foo;

  fn main() {

    foo::foo();  // call foo function from foo bundle
    foo::bar();  // call bar function from foo bundle

  }

```

In-file bundles are also supported

```jaguar
  
  bundle foo {

    fn bar() {}
    fn baz() {}

    bundle quuz {
      fn corge(n: int) {}
    }

  }

  fn main() {

    foo::bar();
    foo::baz();
    foo::quuz::corge(3);

  }

```

### Unpacking

Jaguar allows you to pull certain symbols from within a bundle via **unpacking**

```jaguar

  bundle "foo.jr" as foo;


  /* Errors if there are any exsisting symbols in the parent scope */
  unpack foo {
    bar,
  };

  fn main() {
    bar(); // no need for foo::bar;
  }

```


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

