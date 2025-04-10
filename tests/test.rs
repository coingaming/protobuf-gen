use protobuf_gen::{read_files, ProtobufString};

#[test]
fn it_works() {
    let descriptor_set = read_files(&["tests/source.proto"], &["tests/"]);
    for file in descriptor_set.file.iter() {
        let actual = file.to_protobuf(file.source_code_info.to_owned().unwrap());
        let expected = std::fs::read_to_string("tests/expected.proto").unwrap();
        // eprintln!("ACTUAL:\n\n{}", actual);
        // eprintln!("EXPECTED:\n\n{}", expected);
        assert!(expected == actual);
    }
}

#[test]
fn it_is_idempotent() {
    let descriptor_set = read_files(&["tests/expected.proto"], &["tests/"]);
    for file in descriptor_set.file.iter() {
        let actual = file.to_protobuf(file.source_code_info.to_owned().unwrap());
        let expected = std::fs::read_to_string("tests/expected.proto").unwrap();
        // eprintln!("ACTUAL:\n\n{}", actual);
        // eprintln!("EXPECTED:\n\n{}", expected);
        assert!(expected == actual);
    }
}
