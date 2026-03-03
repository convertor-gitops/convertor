import { Injectable } from '@angular/core';
import { Observable, of } from 'rxjs';
import { metadata } from './metadata';
import { Metadata } from './metadata.types';

/**
 * Metadata service
 * Provides access to project metadata bundled at build time
 */
@Injectable({
  providedIn: 'root'
})
export class MetadataService {
  /**
   * Get project metadata
   * Returns cached metadata bundled at build time
   */
  getMetadata(): Observable<Metadata> {
    return of(metadata as Metadata);
  }

  /**
   * Get version number
   */
  getVersion(): Observable<string> {
    return of(metadata.version);
  }

  /**
   * Get full version string (with 'v' prefix)
   */
  getFullVersion(): Observable<string> {
    return of(`v${metadata.version}`);
  }

  /**
   * Get build number
   */
  getBuild(): Observable<number> {
    return of(metadata.build);
  }

  /**
   * Get full version info (version + build)
   */
  getFullVersionInfo(): Observable<string> {
    return of(`v${metadata.version} (build ${metadata.build})`);
  }

  /**
   * Get metadata synchronously (since it's bundled)
   */
  getMetadataSync(): Metadata {
    return metadata as Metadata;
  }
}


