## Verisense Core SDK

The core SDK for developing Verisense nuclei. For more details, see the Documentation.

## Quick Start

Store and retrieve a `User` on the Verisense nucleus.

```rust
use parity_scale_codec::{Decode, Encode};
use vrs_core_sdk::{get, post, storage};

#[derive(Debug, Decode, Encode)]
pub struct User {
    pub id: u64,
    pub name: String,
}

#[post]
pub fn add_user(user: User) -> Result<u64, String> {
    let max_id_key = [&b"user:"[..], &u64::MAX.to_be_bytes()[..]].concat();
    let max_id = match storage::search(&max_id_key, storage::Direction::Reverse)
        .map_err(|e| e.to_string())?
    {
        Some((id, _)) => u64::from_be_bytes(id[5..].try_into().unwrap()) + 1,
        None => 1u64,
    };
    let key = [&b"user:"[..], &max_id.to_be_bytes()[..]].concat();
    storage::put(&key, user.encode()).map_err(|e| e.to_string())?;
    Ok(max_id)
}

#[get]
pub fn get_user(id: u64) -> Result<Option<User>, String> {
    let key = [&b"user:"[..], &id.to_be_bytes()[..]].concat();
    let r = storage::get(&key).map_err(|e| e.to_string())?;
    let user = r.map(|d| User::decode(&mut &d[..]).unwrap());
    Ok(user)
}
```

## Exporting the Nucleus ABI

### What is a Nucleus ABI?

An ABI (Application Binary Interface) defines how external systems can interact with your exported functions and types, including their names, parameters, and return types. This allows tools in other languages to automatically generate bindings to call these functions and utilize these types.

For example, consider the following Rust code:

```rust
use vrs_core_sdk::{export, post, codec::{Decode, Encode}};

#[derive(Debug, Decode, Encode)]
#[export]
pub struct E {
    pub a: Vec<u32>,
    pub b: i32,
    pub c: u32,
}

#[derive(Debug, Decode, Encode)]
#[export]
pub struct D {
    pub b: i32,
}

#[post]
pub fn use_codec(d: D) -> Result<E, String> {
    Ok(E {
        a: vec![d.b as u32],
        b: d.b,
        c: 0,
    })
}
```

The generated ABI for this snippet would look like this:

```json
[
  {
    "fields": [
      { "name": "b", "type": { "kind": "Path", "path": ["i32"], "generic_args": [] } }
    ],
    "name": "D",
    "generics": [],
    "type": "struct"
  },
  {
    "fields": [
      {
        "name": "a",
        "type": {
          "kind": "Path",
          "path": ["Vec"],
          "generic_args": [
            { "kind": "Path", "path": ["u32"], "generic_args": [] }
          ]
        }
      },
      {
        "name": "b",
        "type": { "kind": "Path", "path": ["i32"], "generic_args": [] }
      },
      {
        "name": "c",
        "type": { "kind": "Path", "path": ["u32"], "generic_args": [] }
      }
    ],
    "generics": [],
    "name": "E",
    "type": "struct"
  },
  {
    "inputs": [
      {
        "name": "d",
        "type": { "kind": "Path", "path": ["D"], "generic_args": [] }
      }
    ],
    "method": "post",
    "name": "use_codec",
    "output": {
      "kind": "Path",
      "path": ["Result"],
      "generic_args": [
        { "kind": "Path", "path": ["E"], "generic_args": [] },
        { "kind": "Path", "path": ["String"], "generic_args": [] }
      ]
    },
    "type": "fn"
  }
]
```

### How to Export the ABI

  * Functions annotated with `#[get]` or `#[post]` are automatically exported and included in the ABI.
  * Functions annotated with `#[init]` are used for module initialization and are **not** included in the exported ABI.
  * To export custom structs, enums, or type aliases, use the `#[export]` attribute above their definition:

<!-- end list -->

```rust
use vrs_core_sdk::{export, codec::{Decode, Encode}};

#[derive(Debug, Decode, Encode)]
#[export]
pub struct MyCustomStruct {
    pub data: i32,
}

#[derive(Debug, Decode, Encode)]
#[export]
pub enum MyCustomEnum {
    VariantA,
    VariantB(u32),
}

#[export] // Exporting a type alias
type MyId = u64;

#[export] // Exporting a generic type alias
type MyGenericResult<T> = Result<T, String>;

#[derive(Debug, Decode, Encode)]
#[export]
pub struct H160(pub [u8; 20]); // Exporting a tuple struct

#[derive(Debug, Decode, Encode)]
#[export]
pub struct Args<T, S> { // Exporting a generic struct
    pub signer: T,
    pub sign_data: S,
}
```

The default output path for the ABI JSON file is `./exports.json`.
To specify a custom output path, set the `EXPORT_JSON_FILE` environment variable:

```bash
EXPORT_JSON_FILE=./my_output_path.json cargo build --release --target wasm32-unknown-unknown
```

If the ABI file is not generated in the output directory, you can try:

## ABI Specification

The ABI JSON file is an array of objects, where each object describes an exported item (struct, enum, type alias, or function).

### Top-level Items

Each item in the exported ABI array is an object with a `type` field that can be:

  * `"type": "struct"` – for exported structs.
  * `"type": "enum"` – for exported enums.
  * `"type": "type_alias"` – for exported type aliases.
  * `"type": "fn"` – for exported `#[get]` and `#[post]` functions.

### Structs (`"type": "struct"`)

Describes an exported struct.

  * `name`: `String` - The name of the struct.
  * `type`: `String` - Always `"struct"`.
  * `generics`: `Array<String>` - A list of strings representing the names of the generic parameters (e.g., `["T", "S"]`). Empty if the struct is not generic.
  * `fields`: `Array<Object>` - A list of objects, each describing a field in the struct:
      * `name`: `String` - The name of the field.
      * `type`: `Object` - A type descriptor object defining the field's type. See the [Type System](#type-system) section.

**Example: Struct `A` - Rust Definition and ABI Output**

This example shows the Rust definition for a struct `A` and its corresponding representation in the exported ABI JSON.

Struct `A` in Nucleus:

```rust
#[derive(Debug, Decode, Encode)]
#[export]
pub struct A {
    pub b: B,
    pub tuple_field: (u32, String),
    pub array_field: [u8; 10],
    pub slice_field: Vec<u64>,
    pub ggg: quote::T,
}
```

Exported ABI:

```json
{
  "fields": [
    {
      "name": "b",
      "type": { "kind": "Path", "path": ["B"], "generic_args": [] }
    },
    {
      "name": "tuple_field",
      "type": {
        "kind": "Tuple",
        "tuple_args": [
          { "kind": "Path", "path": ["u32"], "generic_args": [] },
          { "kind": "Path", "path": ["String"], "generic_args": [] }
        ]
      }
    },
    {
      "name": "array_field",
      "type": {
        "kind": "Array",
        "elem": { "kind": "Path", "path": ["u8"], "generic_args": [] },
        "len": 10
      }
    },
    {
      "name": "slice_field",
      "type": {
        "kind": "Path",
        "path": ["Vec"],
        "generic_args": [
          { "kind": "Path", "path": ["u64"], "generic_args": [] }
        ]
      }
    },
    {
      "name": "ggg",
      "type": { "kind": "Path", "path": ["quote", "T"], "generic_args": [] }
    }
  ],
  "name": "A",
  "type": "struct"
}
```

**Example: Struct `H160` (Tuple Struct) - Rust Definition and ABI Output**

```rust
#[derive(Debug, Decode, Encode)]
#[export]
pub struct H160(pub [u8; 20]);
```
Exported ABI:
```json
{
  "type": "struct",
  "name": "H160",
  "generics": [],
  "fields": [
    {
      "name": "_0",
      "type": {
        "kind": "Array",
        "elem": { "kind": "Path", "path": ["u8"], "generic_args": [] },
        "len": 20
      }
    }
  ]
}
```
Example: Struct `Args<T, S>` (Generic Struct) - Rust Definition and ABI Output
```rust
#[derive(Debug, Decode, Encode)]
#[export]
pub struct Args<T, S> {
    pub signer: T,
    pub sign_data: S,
}
```
Exported ABI:
```json
{
  "type": "struct",
  "name": "Args",
  "generics": ["T", "S"],
  "fields": [
    {
      "name": "signer",
      "type": { "kind": "Path", "path": ["T"], "generic_args": [] }
    },
    {
      "name": "sign_data",
      "type": { "kind": "Path", "path": ["S"], "generic_args": [] }
    }
  ]
}
```


### Enums (`"type": "enum"`)

Describes an exported enum.

  * `name`: `String` - The name of the enum.
  * `type`: `String` - Always `"enum"`.
  * `variants`: `Array<Object>` - A list of objects, where each object describes a variant of the enum:
      * `name`: `String` - The name of the variant.
      * `fields`: `Array<Object>` - An array describing the fields associated with the variant. Its structure varies based on the variant type:
          * **Unit Variants**: For variants without associated data (e.g., `VariantA`), `fields` is an **empty array** `[]`.
          * **Tuple Variants**: For variants with unnamed data (e.g., `VariantB(u32, String)`), `fields` is an array of field objects. Each object represents an element in the tuple, with its `name` being **automatically generated** as `_0`, `_1`, `_2`, etc., and includes its `type` (a [Type System](#type-system) descriptor).
          * **Struct Variants**: For variants with named data (e.g., `VariantC { x: u32, y: String }`), `fields` is an array of field objects, similar in structure to [Structs](#structs-type-struct) fields, where each object contains its `name` and `type`.

**Example: Enum `MyCustomEnum` - Rust Definition and ABI Output**

This example shows an enum with unit, tuple, and struct variants and its representation in the ABI JSON.

`MyCustomEnum` Definition in Nucleus:

```rust
use vrs_core_sdk::{export, codec::{Decode, Encode}};

#[derive(Debug, Decode, Encode)]
#[export]
pub enum MyCustomEnum {
    VariantA,
    VariantB(u32, String),
    VariantC { id: u64, name: String },
}
```

Exported ABI:

```json
{
  "type": "enum",
  "name": "MyCustomEnum",
  "variants": [
    {
      "name": "VariantA",
      "fields": []
    },
    {
      "name": "VariantB",
      "fields": [
        {
          "name": "_0",
          "type": { "kind": "Path", "path": ["u32"], "generic_args": [] }
        },
        {
          "name": "_1",
          "type": { "kind": "Path", "path": ["String"], "generic_args": [] }
        }
      ]
    },
    {
        "name": "VariantC",
        "fields": [
            {
              "name": "id",
              "type": { "kind": "Path", "path": ["u64"], "generic_args": [] }
            },
            {
              "name": "name",
              "type": { "kind": "Path", "path": ["String"], "generic_args": [] }
            }
        ]
    }
  ]
}
```


### Type Aliases (`"type": "type_alias"`)

Describes an exported type alias (defined with Rust's `type` keyword).

  * `name`: `String` - The name of the type alias (e.g., `MyId`, `MyGenericResult`).
  * `type`: `String` - Always `"type_alias"`.
  * `generics`: `Array<String>` - A list of strings representing the names of the generic parameters of the alias (e.g., `["T"]` for `MyGenericResult<T>`). If the alias is not generic, this is an empty array.
  * `target`: `Object` - A type descriptor object defining the actual type the alias refers to. See the [Type System](#type-system) section.

**Example: Type Alias `MyGenericResult<T>` - Rust Definition and ABI Output**

Type Alias `MyGenericResult<T>` in Nucleus:

```rust
#[export]
type MyGenericResult<T> = Result<T, String>;
```

Exported ABI:

```json
{
  "name": "MyGenericResult",
  "type": "type_alias",
  "generics": ["T"],
  "target": {
    "kind": "Path",
    "path": ["Result"],
    "generic_args": [
      { "kind": "Path", "path": ["T"], "generic_args": [] },
      { "kind": "Path", "path": ["String"], "generic_args": [] }
    ]
  }
}
```

### Functions (`"type": "fn"`)

Describes an exported function.

  * `name`: `String` - The name of the function.
  * `type`: `String` - Always `"fn"`.
  * `method`: `String` - Either `"get"` or `"post"`, corresponding to the attribute used.
  * `inputs`: `Array<Object>` - A list of objects, each describing a parameter of the function. If the function takes no arguments, this is an empty array.
      * `name`: `String` - The name of the parameter.
      * `type`: `Object` - A type descriptor object defining the parameter's type. See the [Type System](#type-system) section.
  * `output`: `Object | null` - A type descriptor object defining the function's return type. See the [Type System](#type-system) section.
      * If the function has no explicit return value (e.g., returns `()` not wrapped in a `Result`, like `fn foo() {}` or `fn foo() -> ()`), this will be `null`.
      * If it returns `Result<(), ...>`, the unit type `()` will be represented as `{"kind": "Tuple", "tuple_args": []}` within the `Result`'s generic arguments.

**Example (Functions `cc`, `bbb`, and `request_google` from your input):**

```json
{ // Function cc(a: String, b: String) -> Result<String, String>
  "inputs": [
    {
      "name": "a",
      "type": { "kind": "Path", "path": ["String"], "generic_args": [] }
    },
    {
      "name": "b",
      "type": { "kind": "Path", "path": ["String"], "generic_args": [] }
    }
  ],
  "method": "post",
  "name": "cc",
  "output": {
    "kind": "Path",
    "path": ["Result"],
    "generic_args": [
      { "kind": "Path", "path": ["String"], "generic_args": [] },
      { "kind": "Path", "path": ["String"], "generic_args": [] }
    ]
  },
  "type": "fn"
},
{ // Function bbb(_a: (A, String)) -> Result<(), String>
  "inputs": [
    {
      "name": "_a",
      "type": {
        "kind": "Tuple",
        "tuple_args": [
          { "kind": "Path", "path": ["A"], "generic_args": [] },
          { "kind": "Path", "path": ["String"], "generic_args": [] }
        ]
      }
    }
  ],
  "method": "post",
  "name": "bbb",
  "output": {
    "kind": "Path",
    "path": ["Result"],
    "generic_args": [
      { "kind": "Tuple", "tuple_args": [] }, // Represents ()
      { "kind": "Path", "path": ["String"], "generic_args": [] }
    ]
  },
  "type": "fn"
},
{ // Function request_google() (implicitly returns ())
  "inputs": [],
  "method": "post",
  "name": "request_google",
  "output": null, // Function returns () implicitly, not wrapped in Result
  "type": "fn"
}
```


### Type System

Types are described recursively using a type descriptor object. This object always has a `kind` field and other fields depending on the kind.

1.  **`kind: "Path"`**: Represents a named type, potentially with generic arguments (e.g., `String`, `Vec<u32>`, `Result<T, E>`, `my_module::MyType`).

      * `path`: `Array<String>` - An array of strings representing the segments of the type's path (e.g., `["String"]`, `["Vec"]`, `["Result"]`, `["quote", "T"]`).
      * `generic_args`: `Array<Object>` - An array of type descriptor objects for any generic arguments. Empty if the type is not generic or generic arguments are not specified.

2.  **`kind: "Tuple"`**: Represents a tuple type (e.g., `(u32, String)`, `()`).

      * `tuple_args`: `Array<Object>` - An array of type descriptor objects, one for each element in the tuple. For the unit type `()`, this is an empty array `[]`.

3.  **`kind: "Array"`**: Represents a fixed-size array (e.g., `[u8; 10]`).

      * `elem`: `Object` - A type descriptor object for the element type of the array.
      * `len`: `Number` - The length of the array.

**Examples of Type Descriptors:**

  * `u32`:

    ```json
    { "kind": "Path", "path": ["u32"], "generic_args": [] }
    ```

  * `Vec<u64>`:

    ```json
    {
      "kind": "Path",
      "path": ["Vec"],
      "generic_args": [
        { "kind": "Path", "path": ["u64"], "generic_args": [] }
      ]
    }
    ```

  * `Result<E, String>`:

    ```json
    {
      "kind": "Path",
      "path": ["Result"],
      "generic_args": [
        { "kind": "Path", "path": ["E"], "generic_args": [] },
        { "kind": "Path", "path": ["String"], "generic_args": [] }
      ]
    }
    ```

  * `(u32, String)` (as a tuple field or argument):

    ```json
    {
      "kind": "Tuple",
      "tuple_args": [
        { "kind": "Path", "path": ["u32"], "generic_args": [] },
        { "kind": "Path", "path": ["String"], "generic_args": [] }
      ]
    }
    ```

  * `()` (unit type, e.g., in `Result<(), String>`):

    ```json
    {
      "kind": "Tuple",
      "tuple_args": []
    }
    ```

  * `[u8; 10]` (as an array field):

    ```json
    {
      "kind": "Array",
      "elem": { "kind": "Path", "path": ["u8"], "generic_args": [] },
      "len": 10
    }
    ```

  * `quote::T` (type from another module):

    ```json
    {
      "kind": "Path",
      "path": ["quote", "T"],
      "generic_args": []
    }
    ```


## FAQ

**Q: Can I export functions that take multiple arguments?**

A: Yes. Functions can take multiple arguments (e.g., `fn my_func(arg1: Type1, arg2: Type2)`). Each argument will be listed as an object in the `inputs` array in the ABI definition for that function.

**Q: How are function arguments that are tuples represented, for example, if a function takes a single tuple argument like `fn bbb(_a: (A, String))`?**

A: The single argument `_a` will have its `type` described as `{"kind": "Tuple", "tuple_args": [...]}` in the `inputs` array. The `tuple_args` array will then contain the type descriptors for `A` and `String`.

**Q: Can I rename the exported functions or types in the ABI?**

A: Not at the moment. The ABI reflects the original Rust identifiers used in the source code.

**Q: Is this ABI format stable?**

A: The schema is designed to be stable. Minor version upgrades of the SDK may extend the format by adding new, non-breaking fields or supporting new type kinds (e.g., more complex enum representations). Existing fields and structures will be maintained for backward compatibility where possible.

**Q: What if the ABI file (`exports.json`) is not generated in the output directory?**

A: If the ABI file is not generated as expected, you can try cleaning your build artifacts and then rebuilding the project. This can often resolve issues related to stale build outputs. Execute the following commands in your project directory:

```bash
cargo clean
cargo build --release --target wasm32-unknown-unknown
nucleus-abigen
```