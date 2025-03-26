use curies::{Converter, Record};

pub(super) fn init() -> Converter {
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
    let mut converter = Converter::new(":");
    records.into_iter().for_each(|record| {
        if let Err(error) = converter.add_record(record.clone()) {
            log::error!("Could not setup custom prefix:\n{}", error);
        }
    });
    converter
}
