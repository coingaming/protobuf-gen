use protobuf_gen::{read_files, ProtobufString};
use std::path::Path;
use std::fs;

fn main() {
    let proto_files = vec![
            "protos/user.proto".to_string(),
    ];

    println!("Starting type conversion...");

    // Process each proto file
    for proto_file in &proto_files {
        println!("Processing file: {}", proto_file);
        let descriptor_set = read_files(&[proto_file], &[&"protos".to_string()]);
        for file in descriptor_set.file.iter() {
            let converted_proto = convert_strings_to_optional(file);

            // Generate output filename
            let path = Path::new(proto_file);
            let file_stem = path.file_stem().unwrap().to_str().unwrap();
            let parent = path.parent().unwrap_or_else(|| Path::new(""));
            let output_path = parent.join(format!("{}_converted.proto", file_stem));

            fs::write(&output_path, converted_proto).unwrap();
            println!("Converted file written to: {}", output_path.display());
        }
    }

    println!("Conversion completed!");
}

fn convert_strings_to_optional(file_descriptor: &prost_types::FileDescriptorProto) -> String {
    let mut modified_descriptor = file_descriptor.clone();

    for message in &mut modified_descriptor.message_type {
        convert_fields_in_message(message);
    }

    modified_descriptor.to_protobuf(modified_descriptor.clone())
}

fn convert_fields_in_message(message: &mut prost_types::DescriptorProto) {
    for field in &mut message.field {
        if field.r#type() == prost_types::field_descriptor_proto::Type::String &&
           field.label() != prost_types::field_descriptor_proto::Label::Repeated {
            field.proto3_optional = Some(true);
        }
    }

    for nested_message in &mut message.nested_type {
        convert_fields_in_message(nested_message);
    }
}
