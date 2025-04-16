use protobuf_gen::{read_files, ProtobufString};
use pretty_assertions::assert_eq;

#[test]
fn it_works() {
    let descriptor_set = read_files(&["tests/source.proto"], &["tests/"]);
    for file in descriptor_set.file.iter() {
        let actual = file.to_protobuf(file.clone());
        let expected = std::fs::read_to_string("tests/expected.proto").unwrap();
        assert_eq!(expected, actual);
    }
}

#[test]
fn it_is_idempotent() {
    let descriptor_set = read_files(&["tests/expected.proto"], &["tests/"]);
    for file in descriptor_set.file.iter() {
        let actual = file.to_protobuf(file.clone());
        let expected = std::fs::read_to_string("tests/expected.proto").unwrap();
        assert_eq!(expected, actual);
    }
}
