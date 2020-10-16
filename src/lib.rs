// use alloc_counter::{count_alloc, AllocCounterSystem};
use prost::Message;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::process::Command;

// #[global_allocator]
// static A: AllocCounterSystem = AllocCounterSystem;

pub trait ProtobufString {
    fn build_protobuf(&self, gen: &mut Generator);

    // #[count_alloc]
    fn to_protobuf(&self, file_descriptor: prost_types::FileDescriptorProto) -> String {
        let syntax = file_descriptor
            .syntax
            .map_or(prost_types::Syntax::Proto2, |x| {
                if x == "proto3" {
                    prost_types::Syntax::Proto3
                } else {
                    prost_types::Syntax::Proto2
                }
            });

        let source_info = if let Some(mut source_info) = file_descriptor.source_code_info {
            source_info.location.sort_by(|a, b| a.path.cmp(&b.path));
            Some(source_info)
        } else {
            None
        };

        let mut buf = String::new();
        buf.reserve(2048);
        let path = Vec::with_capacity(10);
        let indent = String::with_capacity(200);
        let mut gen = Generator {
            syntax,
            source_info,
            buf,
            path,
            indent,
        };
        self.build_protobuf(&mut gen);
        gen.buf
    }
}

pub struct Generator {
    syntax: prost_types::Syntax,
    source_info: Option<prost_types::SourceCodeInfo>,
    buf: String,
    path: Vec<i32>,
    indent: String,
}

impl Generator {
    fn inc_indent(&mut self) {
        self.indent.push_str("  ");
    }

    fn dec_indent(&mut self) {
        self.indent.pop();
        self.indent.pop();
    }

    fn open_block(&mut self) {
        self.write(" {\n");
        self.inc_indent();
    }

    fn close_block(&mut self) {
        self.dec_indent();
        self.write_indent();
        self.write("}\n");
    }

    fn write(&mut self, text: &str) {
        self.buf.push_str(text);
    }

    fn write_indent(&mut self) {
        self.buf.push_str(&self.indent);
    }

    fn write_leading_comment(&mut self) -> Option<()> {
        if let Some(ref comment) = self.location()?.leading_comments.clone() {
            self.buf.push_str("\n");
            for line in comment.lines() {
                self.buf.push_str(&self.indent);
                self.buf.push_str("//");
                let line = line.clone();
                self.buf.push_str(line);
                self.buf.push_str("\n");
            }
            Some(())
        } else {
            None
        }
    }

    fn location(&self) -> Option<&prost_types::source_code_info::Location> {
        let idx = self
            .source_info
            .as_ref()?
            .location
            .binary_search_by_key(&&self.path[..], |location| &location.path[..])
            .unwrap();

        Some(&self.source_info.as_ref()?.location[idx])
    }
}

fn write_field_type(buf: &mut String, field: &prost_types::FieldDescriptorProto) {
    if let Some(ref type_name) = field.type_name {
        buf.push_str(type_name);
    } else if let Some(typ) = field.r#type {
        if typ >= 1 && typ <= 18 {
            use prost_types::field_descriptor_proto::Type;
            match unsafe { std::mem::transmute(typ) } {
                Type::Double => write!(buf, "{}", "double").unwrap(),
                Type::Float => write!(buf, "{}", "float").unwrap(),
                Type::Int64 => write!(buf, "{}", "int64").unwrap(),
                Type::Uint64 => write!(buf, "{}", "uint64").unwrap(),
                Type::Int32 => write!(buf, "{}", "int32").unwrap(),
                Type::Fixed64 => write!(buf, "{}", "fixed64").unwrap(),
                Type::Fixed32 => write!(buf, "{}", "fixed32").unwrap(),
                Type::Bool => write!(buf, "{}", "bool").unwrap(),
                Type::String => write!(buf, "{}", "string").unwrap(),
                // Tag-delimited aggregate.
                // Group type is deprecated and not supported in proto3. However, Proto3
                // implementations should still be able to parse the group wire format and
                // treat group fields as unknown fields.
                Type::Group => write!(buf, "{}", "group").unwrap(),
                // Length-delimited aggregate.
                Type::Message => write!(buf, "{}", "message").unwrap(),
                // New in version 2.
                Type::Bytes => write!(buf, "{}", "bytes").unwrap(),
                Type::Uint32 => write!(buf, "{}", "uint32").unwrap(),
                Type::Enum => write!(buf, "{}", "enum").unwrap(),
                Type::Sfixed32 => write!(buf, "{}", "sfixed32").unwrap(),
                Type::Sfixed64 => write!(buf, "{}", "sfixed64").unwrap(),
                Type::Sint32 => write!(buf, "{}", "sint32").unwrap(),
                Type::Sint64 => write!(buf, "{}", "sint64").unwrap(),
            }
        }
    }
}

impl ProtobufString for prost_types::FileDescriptorProto {
    fn build_protobuf(&self, gen: &mut Generator) {
        // The syntax of the proto file.
        // The supported values are "proto2" and "proto3".
        if let Some(ref syntax) = self.syntax {
            gen.path.push(12);
            gen.write_indent();
            gen.write("syntax = \"");
            gen.write(syntax);
            gen.write("\";\n\n");
            gen.path.pop();
        }

        // e.g. "foo", "foo.bar", etc.
        gen.path.push(2);
        if let Some(ref package) = self.package {
            gen.write_indent();
            gen.write("package ");
            gen.write(package);
            gen.write(";\n\n");
        }
        gen.path.pop();

        // Names of files imported by this file.
        gen.path.push(3);
        for (i, dep) in self.dependency.iter().enumerate() {
            gen.path.push(i as i32);
            gen.write_indent();
            gen.write("import \"");
            gen.write(dep);
            gen.write("\";\n");
            gen.path.pop();
        }
        gen.path.pop();

        // Indexes of the public imported files in the dependency list above.
        // TODO: public_dependency: ::std::vec::Vec<i32>,

        // Indexes of the weak imported files in the dependency list.
        // For Google-internal migration only. Do not use.
        // weak_dependency: ::std::vec::Vec<i32>,

        // All top-level definitions in this file.
        gen.path.push(4);
        for (i, msg) in self.message_type.iter().enumerate() {
            gen.path.push(i as i32);
            msg.build_protobuf(gen);
            gen.path.pop();
        }
        gen.path.pop();

        gen.path.push(5);
        for (i, enum_type) in self.enum_type.iter().enumerate() {
            gen.path.push(i as i32);
            enum_type.build_protobuf(gen);
            gen.path.pop();
        }
        gen.path.pop();

        gen.path.push(6);
        for (i, serv) in self.service.iter().enumerate() {
            gen.path.push(i as i32);
            serv.build_protobuf(gen);
            gen.path.pop();
        }
        gen.path.pop();

        // TODO: extension: ::std::vec::Vec<FieldDescriptorProto>,
        // TODO: options: ::std::option::Option<FileOptions>,

        // This field contains optional information about the original source code.
        // You may safely remove this entire field without harming runtime
        // functionality of the descriptors -- the information is needed only by
        // development tools.
        // source_code_info: ::std::option::Option<SourceCodeInfo>,
    }
}

impl ProtobufString for prost_types::DescriptorProto {
    fn build_protobuf(&self, gen: &mut Generator) {
        if gen.write_leading_comment().is_none() {
            gen.write("\n");
        }

        gen.write_indent();
        gen.write("message");
        if let Some(ref name) = self.name {
            gen.write(" ");
            gen.write(name);
        }
        gen.open_block();

        // needed to handle `Map<Type, Type>` syntax
        let map_entries: std::collections::HashMap<_, _> = self
            .nested_type
            .iter()
            .filter_map(|t| {
                if t.options.as_ref()?.map_entry? {
                    Some((t.name.as_ref().unwrap(), t))
                } else {
                    None
                }
            })
            .collect();

        gen.path.push(2);
        for (i, field) in self
            .field
            .iter()
            .enumerate()
            .filter(|(_, f)| f.oneof_index.is_none())
        {
            gen.path.push(i as i32);
            if let Some(ref type_name) = field.type_name {
                let sub_type = &type_name[type_name.rfind('.').unwrap() + 1..];
                if let Some(prost_types::DescriptorProto { field: fields, .. }) =
                    map_entries.get(&sub_type.to_owned())
                {
                    let mut typ = String::with_capacity(32);
                    typ.push_str("Map<");
                    write_field_type(&mut typ, fields.get(0).unwrap());
                    typ.push_str(", ");
                    write_field_type(&mut typ, fields.get(1).unwrap());
                    typ.push_str(">");
                    let mut field = field.to_owned();
                    field.type_name = Some(typ);
                    field.label = None;
                    field.build_protobuf(gen);
                } else {
                    field.build_protobuf(gen);
                }
            } else {
                field.build_protobuf(gen);
            }
            gen.path.pop();
        }
        gen.path.pop();

        for (i, oneof) in self.oneof_decl.iter().enumerate() {
            gen.path.push(8);
            gen.path.push(i as i32);
            gen.write_leading_comment();
            gen.path.pop();
            gen.path.pop();

            gen.write_indent();
            gen.write("oneof");
            if let Some(ref name) = oneof.name {
                gen.write(" ");
                gen.write(name);
            }
            gen.write(" {\n");
            gen.inc_indent();
            gen.path.push(2);
            for (i, field) in self
                .field
                .iter()
                .enumerate()
                .filter(|(_, f)| f.oneof_index.map_or(false, |j| j == i as i32))
            {
                gen.path.push(i as i32);
                field.build_protobuf(gen);
                gen.path.pop();
            }
            gen.path.pop();

            gen.dec_indent();
            gen.write_indent();
            gen.write("}\n");
        }

        // TODO: extension: ::std::vec::Vec<FieldDescriptorProto>,

        gen.path.push(3);
        for (i, nested_type) in self.nested_type.iter().enumerate().filter(|(_, t)| {
            t.options
                .as_ref()
                .map_or(true, |o| o.map_entry.map_or(true, |e| !e))
        }) {
            gen.path.push(i as i32);
            nested_type.build_protobuf(gen);
            gen.path.pop();
        }
        gen.path.pop();

        gen.path.push(4);
        for (i, enum_type) in self.enum_type.iter().enumerate() {
            gen.path.push(i as i32);
            enum_type.build_protobuf(gen);
            gen.path.pop();
        }
        gen.path.pop();

        // TODO: extension_range: ::std::vec::Vec<descriptor_proto::ExtensionRange>,
        // TODO: options: ::std::option::Option<MessageOptions>,

        if self.reserved_name.len() > 0 {
            gen.write_leading_comment();
            gen.write_indent();
            gen.write("reserved ");
            for (i, range) in self.reserved_range.iter().enumerate() {
                if i > 0 {
                    gen.write(", ");
                }

                // Inclusive
                if let Some(ref start) = range.start {
                    write!(gen.buf, "{}", start).unwrap();
                }
                gen.write(" to ");

                // Exclusive.
                if let Some(ref end) = range.end {
                    write!(gen.buf, "{}", end).unwrap();
                } else {
                    gen.write("max");
                }
            }
        }

        // Reserved field names, which may not be used by fields in the same message.
        // A given name may only be reserved once.
        if self.reserved_name.len() > 0 {
            gen.write_leading_comment();
            gen.write_indent();
            gen.write("reserved ");
            for (i, name) in self.reserved_name.iter().enumerate() {
                if i > 0 {
                    gen.write(", ");
                }
                write!(gen.buf, "\"{}\"", name).unwrap();
            }
            gen.write(";\n");
        }

        gen.close_block();
    }
}

impl ProtobufString for prost_types::FieldDescriptorProto {
    fn build_protobuf(&self, gen: &mut Generator) {
        gen.write_leading_comment();

        gen.write_indent();
        // If type_name is set, this need not be set.  If both this and type_name
        // are set, this must be one of TYPE_ENUM, TYPE_MESSAGE or TYPE_GROUP.

        // For message and enum types, this is the name of the type.  If the name
        // starts with a '.', it is fully-qualified.  Otherwise, C++-like scoping
        // rules are used to find the type (i.e. first the nested types within this
        // message are searched, then within the parent, on up to the root
        // namespace).

        if let Some(label) = self.label {
            if label >= 1 && label <= 3 {
                use prost_types::field_descriptor_proto::Label;
                match unsafe { std::mem::transmute(label) } {
                    Label::Optional => {
                        if gen.syntax == prost_types::Syntax::Proto2 {
                            write!(gen.buf, "{}", "optional").unwrap();
                            gen.write(" ");
                        }
                    }
                    Label::Required => {
                        write!(gen.buf, "{}", "required").unwrap();
                        gen.write(" ");
                    }
                    Label::Repeated => {
                        write!(gen.buf, "{}", "repeated").unwrap();
                        gen.write(" ");
                    }
                }
            }
        }

        write_field_type(&mut gen.buf, &self);
        gen.write(" ");
        if let Some(ref name) = self.name {
            gen.write(name);
        }
        gen.write(" = ");
        if let Some(number) = self.number {
            write!(gen.buf, "{}", number).unwrap();
        }

        // For extensions, this is the name of the type being extended.  It is
        // resolved in the same manner as type_name.
        // TODO: extendee: ::std::option::Option<std::string::String>,

        // For numeric types, contains the original text representation of the value.
        // For booleans, "true" or "false".
        // For strings, contains the default text contents (not escaped in any way).
        // For bytes, contains the C escaped value.  All bytes >= 128 are escaped.
        // TODO: default_value: ::std::option::Option<std::string::String>,

        // If set, gives the index of a oneof in the containing type's oneof_decl
        // list.  This field is a member of that oneof.
        // oneof_index: ::std::option::Option<i32>,

        // JSON name of this field. The value is set by protocol compiler. If the
        // user has set a "json_name" option on this field, that option's value
        // will be used. Otherwise, it's deduced from the field's name by converting
        // it to camelCase.
        // TODO: json_name: ::std::option::Option<std::string::String>,

        // TODO: options: ::std::option::Option<FieldOptions>,

        gen.write(";\n");
    }
}

impl ProtobufString for prost_types::EnumDescriptorProto {
    fn build_protobuf(&self, gen: &mut Generator) {
        if gen.write_leading_comment().is_none() {
            gen.write("\n");
        }

        gen.write_indent();
        gen.write("enum");
        if let Some(ref name) = self.name {
            gen.write(" ");
            gen.write(name);
        }
        gen.open_block();

        gen.path.push(2);
        for (i, value) in self.value.iter().enumerate() {
            gen.path.push(i as i32);
            value.build_protobuf(gen);
            gen.path.pop();
        }
        gen.path.pop();

        // TODO: options: ::std::option::Option<EnumOptions>,

        // Range of reserved numeric values. Reserved numeric values may not be used
        // by enum values in the same enum declaration. Reserved ranges may not
        // overlap.
        if self.reserved_name.len() > 0 {
            gen.write_leading_comment();
            gen.write_indent();
            gen.write("reserved ");
            for (i, range) in self.reserved_range.iter().enumerate() {
                if i > 0 {
                    gen.write(", ");
                }

                // Inclusive
                if let Some(ref start) = range.start {
                    write!(gen.buf, "{}", start).unwrap();
                }
                gen.write(" to ");

                // Exclusive.
                if let Some(ref end) = range.end {
                    write!(gen.buf, "{}", end).unwrap();
                } else {
                    gen.write("max");
                }
            }
        }

        // Reserved enum value names, which may not be reused. A given name may only
        // be reserved once.
        if self.reserved_name.len() > 0 {
            gen.write_leading_comment();
            gen.write_indent();
            gen.write("reserved ");
            for (i, name) in self.reserved_name.iter().enumerate() {
                if i > 0 {
                    gen.write(", ");
                }
                write!(gen.buf, "\"{}\"", name).unwrap();
            }
            gen.write(";\n");
        }

        gen.close_block();
    }
}

impl ProtobufString for prost_types::EnumValueDescriptorProto {
    fn build_protobuf(&self, gen: &mut Generator) {
        gen.write_leading_comment();

        gen.write_indent();
        if let Some(ref name) = self.name {
            gen.write(name);
        }
        gen.write(" = ");
        if let Some(number) = self.number {
            write!(gen.buf, "{}", number).unwrap();
        }
        gen.write(";\n");

        // TODO: options: ::std::option::Option<EnumValueOptions>,
    }
}

impl ProtobufString for prost_types::ServiceDescriptorProto {
    fn build_protobuf(&self, gen: &mut Generator) {
        if gen.write_leading_comment().is_none() {
            gen.write("\n");
        }

        gen.write_indent();
        gen.write("service");
        if let Some(ref name) = self.name {
            gen.write(" ");
            gen.write(name);
        }
        gen.open_block();

        gen.path.push(2);
        for (i, method) in self.method.iter().enumerate() {
            gen.path.push(i as i32);
            method.build_protobuf(gen);
            gen.path.pop();
        }
        gen.path.pop();

        // TODO: options: ::std::option::Option<ServiceOptions>,

        gen.close_block();
    }
}

impl ProtobufString for prost_types::MethodDescriptorProto {
    fn build_protobuf(&self, gen: &mut Generator) {
        gen.write_leading_comment();

        gen.write_indent();
        if let Some(ref name) = self.name {
            gen.write("rpc ");
            gen.write(name);
        }

        // Input and output type names.  These are resolved in the same way as
        // FieldDescriptorProto.type_name, but must refer to a message type.
        gen.write("(");
        if let Some(true) = self.client_streaming {
            gen.write("stream ");
        }
        if let Some(ref typ) = self.input_type {
            gen.write(typ);
        }
        gen.write(") returns (");
        if let Some(true) = self.server_streaming {
            gen.write("stream ");
        }
        if let Some(ref typ) = self.output_type {
            gen.write(typ);
        }
        gen.write(");\n");

        // TODO: options: ::std::option::Option<MethodOptions>,
    }
}

pub fn read_files<P>(protos: &[P], includes: &[P]) -> prost_types::FileDescriptorSet
where
    P: AsRef<Path>,
{
    let tmp = tempfile::Builder::new()
        .prefix("protobuf-gen")
        .tempdir()
        .unwrap();

    let descriptor_set = tmp.path().join("prost-descriptor-set");

    let mut cmd = Command::new("protoc");
    cmd.arg("--include_imports")
        .arg("--include_source_info")
        .arg("-o")
        .arg(&descriptor_set)
        .arg("-I")
        .arg("src/");

    for include in includes {
        cmd.arg("-I").arg(include.as_ref());
    }

    for proto in protos {
        cmd.arg(proto.as_ref());
    }

    let output = cmd.output().unwrap();
    if !output.status.success() {
        panic!("protoc failed: {}", String::from_utf8_lossy(&output.stderr))
    }

    let buf = fs::read(descriptor_set).unwrap();
    prost_types::FileDescriptorSet::decode(&*buf).unwrap()
}
