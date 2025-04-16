# protobuf-gen

A Rust library for converting Protocol Buffer descriptors to .proto files.

## Overview

`protobuf-gen` provides functionality to convert Protocol Buffer `FileDescriptorProto` objects into their textual representation as a .proto file. It can be used to generate protobuf definitions,
or manipulate them in, e.g.: code refactoring tools.

## Examples

The repository includes the following examples:

- **Type Converter** (`examples/type_converter`): An example that demonstrates how to convert fields with type `string` to proto3 optional. See the example's README.md for more details.
