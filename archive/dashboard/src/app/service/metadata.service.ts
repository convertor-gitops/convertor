import { Injectable } from '@angular/core';
import metadata from '../../../../metadata.json';

export interface Metadata {
    name: string;
    repository: string;
    description: string;
    author: string;
    license: string;
    version: string;
    build: number;
}

@Injectable({ providedIn: 'root' })
export class MetadataService {
    readonly metadata: Metadata = metadata;

    get name(): string {
        return this.metadata.name;
    }

    get version(): string {
        return this.metadata.version;
    }

    get build(): number {
        return this.metadata.build;
    }

    get description(): string {
        return this.metadata.description;
    }

    get author(): string {
        return this.metadata.author;
    }

    get license(): string {
        return this.metadata.license;
    }

    get repository(): string {
        return this.metadata.repository;
    }
}
