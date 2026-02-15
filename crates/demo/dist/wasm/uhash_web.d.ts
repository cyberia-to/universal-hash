/* tslint:disable */
/* eslint-disable */

export class Benchmark {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get algorithm parameters as JSON string
     */
    get_params(): string;
    constructor();
    /**
     * Run benchmark with specified number of hashes
     * Returns hashrate in H/s
     */
    run(num_hashes: number): number;
}

/**
 * Mining struct for Web Worker usage.
 * Reuses UniversalHash across batches to avoid 2MB re-allocation per hash.
 */
export class Miner {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Mine a batch of nonces. Returns JSON string:
     * `{"found":true,"hash":"...","nonce":N,"count":M}` or `{"found":false,"count":M}`
     *
     * - `start_nonce`: first nonce to try (as f64, safe up to 2^53)
     * - `nonce_step`: increment between nonces (for interleaved multi-worker mining)
     * - `batch_size`: number of nonces to try in this batch
     */
    mine_batch(start_nonce: number, nonce_step: number, batch_size: number): string;
    constructor(seed_hex: string, address: string, timestamp: number, difficulty: number);
}

/**
 * Single hash function for testing
 */
export function hash_once(input: Uint8Array): Uint8Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_benchmark_free: (a: number, b: number) => void;
    readonly __wbg_miner_free: (a: number, b: number) => void;
    readonly benchmark_get_params: (a: number) => [number, number];
    readonly benchmark_new: () => number;
    readonly benchmark_run: (a: number, b: number) => number;
    readonly hash_once: (a: number, b: number) => [number, number];
    readonly miner_mine_batch: (a: number, b: number, c: number, d: number) => [number, number];
    readonly miner_new: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
