import yaml from 'yaml';
import backend_configurations from '$lib/backends.yaml?raw';

export interface Backend {
        name: string;
        slug: string;
        url: string;
        healthCheckUrl?: string;
}

export interface PrefixMap {
        [key: string]: string
}

export interface Queries {
        [key: string]: string
}

export interface BackendConf {
        backend: Backend;
        prefixMap: PrefixMap;
        queries: Queries;
        default: boolean;
}

export const backends: BackendConf[] = yaml.parse(backend_configurations);
