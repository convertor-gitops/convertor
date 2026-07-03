import { Injectable } from "@angular/core";
import { xchacha20poly1305 } from "@noble/ciphers/chacha.js";
import { randomBytes } from "@noble/ciphers/utils.js";
import { blake3 } from "@noble/hashes/blake3.js";

// === base64url（无补位）工具 ===
function bytesToBase64Url(bytes: Uint8Array): string {
    let bin = "";
    for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
    return btoa(bin).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

function base64UrlToBytes(s: string): Uint8Array {
    let b64 = s.replace(/-/g, "+").replace(/_/g, "/");
    while (b64.length % 4) b64 += "=";
    const bin = atob(b64);
    const out = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
    return out;
}

const te = new TextEncoder();
const td = new TextDecoder();

const NONCE_LEN = 24;
const NONCE_B64URL_LEN = 32;

function deriveKeyFromSecret(secret: Uint8Array | string): Uint8Array {
    const source = typeof secret === "string" ? te.encode(secret) : secret;
    return blake3(source);
}

function deriveNonceSeedFromLabel(label: string): Uint8Array {
    return blake3(te.encode(label));
}

function deriveNonceFromSeedAndCounter(seed: Uint8Array, counter: bigint): Uint8Array {
    const counterBytes = new Uint8Array(8);
    new DataView(counterBytes.buffer).setBigUint64(0, counter, true);
    const hash = blake3(counterBytes, { key: seed });
    return hash.subarray(0, NONCE_LEN);
}

type NonceState =
    | { type: "Random" }
    | { type: "Deterministic"; seed: Uint8Array; counter: bigint };

export class Encryptor {
    private readonly key: Uint8Array;
    private nonceState: NonceState;

    private constructor(key: Uint8Array, nonceState: NonceState) {
        this.key = key;
        this.nonceState = nonceState;
    }

    public static newRandom(secret: Uint8Array | string): Encryptor {
        return new Encryptor(deriveKeyFromSecret(secret), { type: "Random" });
    }

    public static newWithLabel(secret: Uint8Array | string, label: string): Encryptor {
        return new Encryptor(deriveKeyFromSecret(secret), {
            type: "Deterministic",
            seed: deriveNonceSeedFromLabel(label),
            counter: 0n,
        });
    }

    public static newWithLabelAndCounter(secret: Uint8Array | string, label: string, counter: bigint): Encryptor {
        return new Encryptor(deriveKeyFromSecret(secret), {
            type: "Deterministic",
            seed: deriveNonceSeedFromLabel(label),
            counter,
        });
    }

    public static newWithLabelAndCursor(secret: Uint8Array | string, label: string, cursor: bigint): Encryptor {
        return Encryptor.newWithLabelAndCounter(secret, label, cursor);
    }

    private generateNonce(): Uint8Array {
        if (this.nonceState.type === "Random") {
            return randomBytes(NONCE_LEN);
        } else {
            const nonce = deriveNonceFromSeedAndCounter(this.nonceState.seed, this.nonceState.counter);
            this.nonceState.counter += 1n;
            return nonce;
        }
    }

    public encrypt(plaintext: string): string {
        const nonce = this.generateNonce();
        const aead = xchacha20poly1305(this.key, nonce);
        const ciphertext = aead.encrypt(te.encode(plaintext));
        return bytesToBase64Url(nonce) + bytesToBase64Url(ciphertext);
    }

    public decrypt(token: string): string {
        if (token.length < NONCE_B64URL_LEN) {
            throw new Error("nonce 长度不合法");
        }
        const nonce = base64UrlToBytes(token.slice(0, NONCE_B64URL_LEN));
        if (nonce.length !== NONCE_LEN) {
            throw new Error("nonce 长度不合法");
        }
        const ciphertext = base64UrlToBytes(token.slice(NONCE_B64URL_LEN));
        const aead = xchacha20poly1305(this.key, nonce);
        try {
            return td.decode(aead.decrypt(ciphertext));
        } catch {
            throw new Error("解密失败");
        }
    }
}

@Injectable({providedIn: "root"})
export class EncryptorService {
    public init(secret: string): void {
        Encryptor.newRandom(secret);
    }

    public encrypt(secret: string, plaintext: string): string {
        return Encryptor.newRandom(secret).encrypt(plaintext);
    }

    public decrypt(secret: string, token: string): string {
        return Encryptor.newRandom(secret).decrypt(token);
    }
}

