fn main() {
    let mut codegen = protobuf_codegen::Codegen::new();
    codegen
        .pure()
        .includes(["src/protos"])
        .cargo_out_dir("protos");

    for input in glob::glob("src/protos/*.proto").unwrap() {
        let input = input.unwrap();
        codegen.input(input);
    }

    codegen.run_from_script();
}
