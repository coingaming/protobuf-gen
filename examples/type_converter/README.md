# Protocol Buffer Type Converter

This example demonstrates using the `protobuf-gen` library for Protocol Buffer refactoring.

Specifically, it converts fields that use the `string` type to be made proto3 optional.

## Usage

Run without arguments to process the example proto files:

```
cargo run
```

This will process the example files in the `protos/` directory and create converted versions with the suffix `_converted.proto`.

Or specify your own proto files:

```
cargo run -- path/to/your/file.proto another/file.proto
```

## How It Works

The application:

1. Reads and parses protobuf files using `protobuf-gen`'s `read_files` function
2. Traverses all message definitions and recursively checks all their fields
3. When a field using the type `string` is found, changes it to be proto3 optional.
4. Outputs the converted proto content to new files

## Example Files

- `protos/user.proto`: Contains user-related message definitions with mixed string types
