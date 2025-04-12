export interface Backend {
        name: string;
        slug: string;
        url: string;
        healthCheckUrl?: string;
}
export interface PrefixMap {
        [key: string]: string
}
export interface BackendConf {
        backend: Backend;
        prefixMap: PrefixMap;
        default: boolean
}


export const backends: BackendConf[] =
        [
                {
                        "backend": {
                                "name": "IMDb",
                                "slug": "imdb",
                                "url": "https://qlever.cs.uni-freiburg.de/api/imdb",
                                "healthCheckUrl": "https://qlever.cs.uni-freiburg.de/api/imdb/ping"
                        },
                        "prefixMap": {
                                "imdb": "https://www.imdb.com/"
                        },
                        "default": false
                },
                {
                        "backend": {
                                "name": "DBLP",
                                "slug": "dblp",
                                "url": "https://qlever.cs.uni-freiburg.de/api/dblp",
                                "healthCheckUrl": "https://qlever.cs.uni-freiburg.de/api/dblp/ping"
                        },
                        "prefixMap": {
                                "dblps": "https://dblp.org/rdf/schema-2020-07-01#",
                                "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                                "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                                "wdt": "http://www.wikidata.org/prop/direct/",
                                "dblp": "https://dblp.org/rdf/schema#",
                                "datacite": "http://purl.org/spar/datacite/",
                                "literal": "http://purl.org/spar/literal/",
                                "owl": "http://www.w3.org/2002/07/owl#",
                                "terms": "http://purl.org/dc/terms/",
                                "cito": "http://purl.org/spar/cito/",
                                "bibtex": "http://purl.org/net/nknouf/ns/bibtex#",
                                "wd": "http://www.wikidata.org/entity/",
                                "wikibase": "http://wikiba.se/ontology#",
                                "schema": "http://schema.org/",
                                "xsd": "http://www.w3.org/2001/XMLSchema#"
                        },
                        "default": true
                },
                {
                        "backend": {
                                "name": "PubChem",
                                "slug": "pubchem",
                                "url": "https://qlever.cs.uni-freiburg.de/api/pubchem",
                                "healthCheckUrl": "https://qlever.cs.uni-freiburg.de/api/pubchem/ping"
                        },
                        "prefixMap": {
                                "bao": "http://www.bioassayontology.org/bao#",
                                "bioassay": "http://rdf.ncbi.nlm.nih.gov/pubchem/bioassay/",
                                "bp": "http://www.biopax.org/release/biopax-level3.owl#",
                                "chemblchembl": "http://linkedchemistry.info/chembl/chemblid/",
                                "chembl": "http://rdf.ebi.ac.uk/resource/chembl/molecule/",
                                "cell": "http://rdf.ncbi.nlm.nih.gov/pubchem/cell/",
                                "cito": "http://purl.org/spar/cito/",
                                "compound": "http://rdf.ncbi.nlm.nih.gov/pubchem/compound/",
                                "concept": "http://rdf.ncbi.nlm.nih.gov/pubchem/concept/",
                                "conserveddomain": "http://rdf.ncbi.nlm.nih.gov/pubchem/conserveddomain/",
                                "dcterms": "http://purl.org/dc/terms/",
                                "descriptor": "http://rdf.ncbi.nlm.nih.gov/pubchem/descriptor/",
                                "disease": "http://rdf.ncbi.nlm.nih.gov/pubchem/disease/",
                                "endpoint": "http://rdf.ncbi.nlm.nih.gov/pubchem/endpoint/",
                                "ensembl": "http://rdf.ebi.ac.uk/resource/ensembl/",
                                "fabio": "http://purl.org/spar/fabio/",
                                "foaf": "http://xmlns.com/foaf/0.1/",
                                "freq": "http://purl.org/cld/freq/",
                                "gene": "http://rdf.ncbi.nlm.nih.gov/pubchem/gene/",
                                "": "http://rdf.ncbi.nlm.nih.gov/pubchem/void.ttl#",
                                "inchikey": "http://rdf.ncbi.nlm.nih.gov/pubchem/inchikey/",
                                "measuregroup": "http://rdf.ncbi.nlm.nih.gov/pubchem/measuregroup/",
                                "mesh": "http://id.nlm.nih.gov/mesh/",
                                "nci": "http://ncicb.nci.nih.gov/xml/owl/EVS/Thesaurus.owl#",
                                "ns0": "http://data.epo.org/linked-data/def/patent/",
                                "obo": "http://purl.obolibrary.org/obo/",
                                "owl": "http://www.w3.org/2002/07/owl#",
                                "patentcpc": "http://rdf.ncbi.nlm.nih.gov/pubchem/patentcpc/",
                                "patent": "http://rdf.ncbi.nlm.nih.gov/pubchem/patent/",
                                "patentipc": "http://rdf.ncbi.nlm.nih.gov/pubchem/patentipc/",
                                "pathway": "http://rdf.ncbi.nlm.nih.gov/pubchem/pathway/",
                                "pav": "http://purl.org/pav/2.0/",
                                "pdbo": "http://rdf.wwpdb.org/schema/pdbx-v40.owl#",
                                "protein": "http://rdf.ncbi.nlm.nih.gov/pubchem/protein/",
                                "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                                "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                                "reactome": "http://identifiers.org/reactome/",
                                "reference": "http://rdf.ncbi.nlm.nih.gov/pubchem/reference/",
                                "sio": "http://semanticscience.org/resource/",
                                "skos": "http://www.w3.org/2004/02/skos/core#",
                                "source": "http://rdf.ncbi.nlm.nih.gov/pubchem/source/",
                                "substance": "http://rdf.ncbi.nlm.nih.gov/pubchem/substance/",
                                "synonym": "http://rdf.ncbi.nlm.nih.gov/pubchem/synonym/",
                                "taxonomy": "http://rdf.ncbi.nlm.nih.gov/pubchem/taxonomy",
                                "uniprot": "http://purl.uniprot.org/uniprot/",
                                "up": "http://purl.uniprot.org/core/",
                                "vcard2006": "http://www.w3.org/2006/vcard/ns#",
                                "voag": "http://voag.linkedmodel.org/schema/voag#",
                                "vocab": "http://rdf.ncbi.nlm.nih.gov/pubchem/vocabulary#",
                                "void": "http://rdfs.org/ns/void#",
                                "wikidata": "http://www.wikidata.org/entity/",
                                "xsd": "http://www.w3.org/2001/XMLSchema#"
                        },
                        "default": false
                },
                {
                        "backend": {
                                "name": "UniProt",
                                "slug": "uniprot",
                                "url": "https://qlever.cs.uni-freiburg.de/api/uniprot",
                                "healthCheckUrl": "https://qlever.cs.uni-freiburg.de/api/uniprot/ping"
                        },
                        "prefixMap": {
                                "annotation": "http://purl.uniprot.org/annotation/",
                                "bibo": "http://purl.org/ontology/bibo/",
                                "busco": "http://busco.ezlab.org/schema#",
                                "chebi": "http://purl.obolibrary.org/obo/CHEBI_",
                                "citation": "http://purl.uniprot.org/citations/",
                                "cito": "http://purl.org/spar/cito/",
                                "dcat": "http://www.w3.org/ns/dcat#",
                                "dcmit": "http://purl.org/dc/dcmitype/",
                                "dcterms": "http://purl.org/dc/terms/",
                                "disease": "http://purl.uniprot.org/diseases/",
                                "ECO": "http://purl.obolibrary.org/obo/ECO_",
                                "embl-cds": "http://purl.uniprot.org/embl-cds/",
                                "ensembl": "http://rdf.ebi.ac.uk/resource/ensembl/",
                                "enzyme": "http://purl.uniprot.org/enzyme/",
                                "faldo": "http://biohackathon.org/resource/faldo#",
                                "foaf": "http://xmlns.com/foaf/0.1/",
                                "go": "http://purl.obolibrary.org/obo/GO_",
                                "hs": "https://hamap.expasy.org/rdf/vocab#",
                                "isoform": "http://purl.uniprot.org/isoforms/",
                                "keywords": "http://purl.uniprot.org/keywords/",
                                "location": "http://purl.uniprot.org/locations/",
                                "obo": "http://purl.obolibrary.org/obo/",
                                "oboInOwl": "http://www.geneontology.org/formats/oboInOwl#",
                                "owl": "http://www.w3.org/2002/07/owl#",
                                "patent": "http://purl.uniprot.org/EPO/",
                                "pav": "http://purl.org/pav/",
                                "position": "http://purl.uniprot.org/position/",
                                "prism": "http://prismstandard.org/namespaces/basic/2.0/",
                                "pubmed": "http://purl.uniprot.org/pubmed/",
                                "range": "http://purl.uniprot.org/range/",
                                "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                                "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                                "rh": "http://rdf.rhea-db.org/",
                                "schema": "http://schema.org/",
                                "sd": "http://www.w3.org/ns/sparql-service-description#",
                                "sh": "http://www.w3.org/ns/shacl#",
                                "skos": "http://www.w3.org/2004/02/skos/core#",
                                "sp": "http://spinrdf.org/sp#",
                                "ssmRegion": "http://purl.uniprot.org/signatureSequenceMatch/",
                                "stato": "http://purl.obolibrary.org/obo/STATO_",
                                "taxon": "http://purl.uniprot.org/taxonomy/",
                                "tissue": "http://purl.uniprot.org/tissues/",
                                "uniparc": "http://purl.uniprot.org/uniparc/",
                                "uniprot": "http://purl.uniprot.org/uniprot/",
                                "up": "http://purl.uniprot.org/core/",
                                "voag": "http://voag.linkedmodel.org/schema/voag#",
                                "void": "http://rdfs.org/ns/void#",
                                "xsd": "http://www.w3.org/2001/XMLSchema#"
                        },
                        "default": false
                },
                {
                        "backend": {
                                "name": "OSM Planet",
                                "slug": "osm-planet",
                                "url": "https://qlever.cs.uni-freiburg.de/api/osm-planet",
                                "healthCheckUrl": "https://qlever.cs.uni-freiburg.de/api/osm-planet/ping"
                        },
                        "prefixMap": {
                                "osmmeta": "https://www.openstreetmap.org/meta/",
                                "osmway": "https://www.openstreetmap.org/way/",
                                "osmkey": "https://www.openstreetmap.org/wiki/Key:",
                                "osmrel": "https://www.openstreetmap.org/relation/",
                                "osmnode": "https://www.openstreetmap.org/node/",
                                "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                                "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                                "osm": "https://www.openstreetmap.org/",
                                "ogc": "http://www.opengis.net/rdf#",
                                "xsd": "http://www.w3.org/2001/XMLSchema#",
                                "wd": "http://www.wikidata.org/entity/",
                                "wdt": "http://www.wikidata.org/prop/direct/",
                                "p": "http://www.wikidata.org/prop/",
                                "ps": "http://www.wikidata.org/prop/statement/",
                                "pq": "http://www.wikidata.org/prop/qualifier/",
                                "geo": "http://www.opengis.net/ont/geosparql#",
                                "geof": "http://www.opengis.net/def/function/geosparql/",
                                "osm2rdf": "https://osm2rdf.cs.uni-freiburg.de/rdf#",
                                "osm2rdfkey": "https://osm2rdf.cs.uni-freiburg.de/rdf/key#",
                                "osm2rdfgeom": "https://osm2rdf.cs.uni-freiburg.de/rdf/geom#",
                                "osm2rdfmember": "https://osm2rdf.cs.uni-freiburg.de/rdf/member#",
                                "qlss": "https://qlever.cs.uni-freiburg.de/spatialSearch/"
                        },
                        "default": false
                },
                {
                        "backend": {
                                "name": "Wikidata",
                                "slug": "wikidata",
                                "url": "https://qlever.cs.uni-freiburg.de/api/wikidata",
                                "healthCheckUrl": "https://qlever.cs.uni-freiburg.de/api/wikidata/ping"
                        },
                        "prefixMap": {
                                "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                                "xsd": "http://www.w3.org/2001/XMLSchema#",
                                "ontolex": "http://www.w3.org/ns/lemon/ontolex#",
                                "dct": "http://purl.org/dc/terms/",
                                "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                                "owl": "http://www.w3.org/2002/07/owl#",
                                "wikibase": "http://wikiba.se/ontology#",
                                "skos": "http://www.w3.org/2004/02/skos/core#",
                                "schema": "http://schema.org/",
                                "cc": "http://creativecommons.org/ns#",
                                "geo": "http://www.opengis.net/ont/geosparql#",
                                "geof": "http://www.opengis.net/def/function/geosparql/",
                                "prov": "http://www.w3.org/ns/prov#",
                                "wd": "http://www.wikidata.org/entity/",
                                "data": "https://www.wikidata.org/wiki/Special:EntityData/",
                                "s": "http://www.wikidata.org/entity/statement/",
                                "ref": "http://www.wikidata.org/reference/",
                                "v": "http://www.wikidata.org/value/",
                                "wdt": "http://www.wikidata.org/prop/direct/",
                                "wdtn": "http://www.wikidata.org/prop/direct-normalized/",
                                "p": "http://www.wikidata.org/prop/",
                                "ps": "http://www.wikidata.org/prop/statement/",
                                "psv": "http://www.wikidata.org/prop/statement/value/",
                                "psn": "http://www.wikidata.org/prop/statement/value-normalized/",
                                "pq": "http://www.wikidata.org/prop/qualifier/",
                                "pqv": "http://www.wikidata.org/prop/qualifier/value/",
                                "pqn": "http://www.wikidata.org/prop/qualifier/value-normalized/",
                                "pr": "http://www.wikidata.org/prop/reference/",
                                "prv": "http://www.wikidata.org/prop/reference/value/",
                                "prn": "http://www.wikidata.org/prop/reference/value-normalized/",
                                "wdno": "http://www.wikidata.org/prop/novalue/",
                                "imdb": "https://www.imdb.com/",
                                "qfn": "http://qlever.cs.uni-freiburg.de/function#",
                                "ql": "http://qlever.cs.uni-freiburg.de/builtin-functions/",
                                "math": "http://www.w3.org/2005/xpath-functions/math#"
                        },
                        "default": false
                }
        ]
