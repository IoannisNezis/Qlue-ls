export interface Backend {
        name: string;
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


export const backends: BackendConf[] = [
        {
                backend: {
                        name: 'dblp',
                        url: 'https://qlever.cs.uni-freiburg.de/api/dblp',
                        healthCheckUrl: 'https://qlever.cs.uni-freiburg.de/api/dblp/ping'
                },
                prefixMap: {
                        "country": "https://ld.admin.ch/country/",
                        "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                        "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                        "osmrel": "https://www.openstreetmap.org/relation/",
                        "dblp": "https://dblp.org/rdf/schema#",
                        "publication": "https://dblp.org/rec/",
                        "stream": "https://dblp.org/streams/",
                        "cito": "http://purl.org/spar/cito/",
                        "datacite": "http://purl.org/spar/datacite/",
                        "terms": "http://purl.org/dc/terms/",
                        "owl": "http://www.w3.org/2002/07/owl#",
                        "literal": "http://purl.org/spar/literal/",
                },
                default: true

        },
        {
                backend: {
                        name: 'wikidata',
                        url: 'https://qlever.cs.uni-freiburg.de/api/wikidata',
                        healthCheckUrl: 'https://qlever.cs.uni-freiburg.de/api/wikidate/ping'
                },
                prefixMap: {
                        "cc": "http://creativecommons.org/ns#",
                        "data": "https://www.wikidata.org/wiki/Special:EntityData/",
                        "dct": "http://purl.org/dc/terms/",
                        "geo": "http://www.opengis.net/ont/geosparql#",
                        "geof": "http://www.opengis.net/def/function/geosparql/",
                        "imdb": "https://www.imdb.com/",
                        "math": "http://www.w3.org/2005/xpath-functions/math#",
                        "ontolex": "http://www.w3.org/ns/lemon/ontolex#",
                        "owl": "http://www.w3.org/2002/07/owl#",
                        "p": "http://www.wikidata.org/prop/",
                        "pq": "http://www.wikidata.org/prop/qualifier/",
                        "pqn": "http://www.wikidata.org/prop/qualifier/value-normalized/",
                        "pqv": "http://www.wikidata.org/prop/qualifier/value/",
                        "pr": "http://www.wikidata.org/prop/reference/",
                        "prn": "http://www.wikidata.org/prop/reference/value-normalized/",
                        "prov": "http://www.w3.org/ns/prov#",
                        "prv": "http://www.wikidata.org/prop/reference/value/",
                        "ps": "http://www.wikidata.org/prop/statement/",
                        "psn": "http://www.wikidata.org/prop/statement/value-normalized/",
                        "psv": "http://www.wikidata.org/prop/statement/value/",
                        "qfn": "http://qlever.cs.uni-freiburg.de/function#",
                        "ql": "http://qlever.cs.uni-freiburg.de/builtin-functions/",
                        "rdf": "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
                        "rdfs": "http://www.w3.org/2000/01/rdf-schema#",
                        "ref": "http://www.wikidata.org/reference/",
                        "s": "http://www.wikidata.org/entity/statement/",
                        "schema": "http://schema.org/",
                        "skos": "http://www.w3.org/2004/02/skos/core#",
                        "v": "http://www.wikidata.org/value/",
                        "wd": "http://www.wikidata.org/entity/",
                        "wdno": "http://www.wikidata.org/prop/novalue/",
                        "wdt": "http://www.wikidata.org/prop/direct/",
                        "wdtn": "http://www.wikidata.org/prop/direct-normalized/",
                        "wikibase": "http://wikiba.se/ontology#",
                        "xsd": "http://www.w3.org/2001/XMLSchema#",
                },
                default: false

        }
]

