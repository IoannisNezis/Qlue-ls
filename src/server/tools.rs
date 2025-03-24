use curies::{Converter, Record};
use log::error;
use tree_sitter::Parser;

pub(super) struct Tools {
    pub(super) uri_converter: Converter,
    pub(super) parser: Parser,
}

impl Tools {
    pub(super) fn initiate() -> Self {
        let mut tools = Self {
            uri_converter: Converter::new(":"),
            parser: Parser::new(),
        };

        tools.initiate_uri_converter();
        tools.initiate_parser();
        tools
    }

    fn initiate_parser(&mut self) {
        match self
            .parser
            .set_language(&tree_sitter_sparql::LANGUAGE.into())
        {
            Ok(()) => {}
            Err(err) => error!("Error while initializing parser: {}", err),
        }
    }

    fn initiate_uri_converter(&mut self) {
        let records = [
            Record::new("schema", "https://schema.org/"),
            Record::new(
                "envCube2023",
                "https://environment.ld.admin.ch/foen/nfi/nfi_C-20/cube/2023-",
            ),
            Record::new(
                "envTopic",
                "https://environment.ld.admin.ch/foen/nfi/Topic/",
            ),
            Record::new("cube", "https://cube.link/"),
            Record::new("env", "https://environment.ld.admin.ch/foen/nfi/"),
            Record::new(
                "envClasificationUnit",
                "https://environment.ld.admin.ch/foen/nfi/ClassificationUnit/",
            ),
            Record::new(
                "envInventory",
                "https://environment.ld.admin.ch/foen/nfi/Inventory/",
            ),
            Record::new(
                "envUnitOfEvaluation",
                "https://environment.ld.admin.ch/foen/nfi/UnitOfEvaluation/",
            ),
            Record::new(
                "envUnitOfEvaluationType",
                "https://environment.ld.admin.ch/foen/nfi/EvaluationType/",
            ),
            Record::new("country", "https://ld.admin.ch/country/"),
            Record::new("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
            Record::new("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
            Record::new("osmrel", "https://www.openstreetmap.org/relation/"),
            Record::new("dblp", "https://dblp.org/rdf/schema#"),
            Record::new("publication", "https://dblp.org/rec/"),
            Record::new("stream", "https://dblp.org/streams/"),
            Record::new("cito", "http://purl.org/spar/cito/"),
            Record::new("datacite", "http://purl.org/spar/datacite/"),
            Record::new("terms", "http://purl.org/dc/terms/"),
            Record::new("owl", "http://www.w3.org/2002/07/owl#"),
            Record::new("literal", "http://purl.org/spar/literal/"),
        ];
        records.into_iter().for_each(|record| {
            if let Err(error) = self.uri_converter.add_record(record.clone()) {
                error!("Could not setup custom prefix:\n{}", error);
            }
        });
    }
}
