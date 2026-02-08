#![no_main]
use lazy_sparql_result_reader::{
    parser::{Parser, PartialResult},
    sparql::{Bindings, Head, Header, Meta, SparqlResult},
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|result: SparqlResult| {
    let input =
        serde_json::to_string(&result).expect("Arbitrary should create searializable instances");

    let mut parsed_result = SparqlResult {
        head: Head { vars: Vec::new() },
        results: Bindings {
            bindings: Vec::new(),
        },
        meta: Meta {
            query_time_ms: 0,
            result_size_total: 0,
        },
    };

    let mut parser = Parser::new(1, None, 0);
    for byte in input.as_bytes() {
        match parser.read_byte(*byte).expect("Input should be valid") {
            Some(PartialResult::Header(Header { head })) => parsed_result.head = head,
            Some(PartialResult::Bindings(bindings)) => {
                parsed_result.results.bindings.extend(bindings)
            }
            Some(PartialResult::Meta(meta)) => parsed_result.meta = meta,
            None => {}
        }
    }

    assert_eq!(
        result,
        parsed_result,
        "parser failed for this input:\n{input}\n{}",
        serde_json::to_string(&parsed_result).unwrap()
    );
});
