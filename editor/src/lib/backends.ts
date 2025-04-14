import backends_data from '$lib/backends.json'
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

export const backends: BackendConf[] = backends_data;


