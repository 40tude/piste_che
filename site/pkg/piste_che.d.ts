/* tslint:disable */
/* eslint-disable */
/**
 * The `ReadableStreamType` enum.
 *
 * *This API requires the following crate features to be activated: `ReadableStreamType`*
 */

type ReadableStreamType = "bytes";

export class IntoUnderlyingByteSource {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    cancel(): void;
    pull(controller: ReadableByteStreamController): Promise<any>;
    start(controller: ReadableByteStreamController): void;
    readonly autoAllocateChunkSize: number;
    readonly type: ReadableStreamType;
}

export class IntoUnderlyingSink {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    abort(reason: any): Promise<any>;
    close(): Promise<any>;
    write(chunk: any): Promise<any>;
}

export class IntoUnderlyingSource {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    cancel(): void;
    pull(controller: ReadableStreamDefaultController): Promise<any>;
}

/**
 * WASM hydration entry point -- called automatically by the browser after the
 * WASM module is instantiated.
 */
export function hydrate(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly hydrate: () => void;
    readonly __wbg_intounderlyingsink_free: (a: number, b: number) => void;
    readonly intounderlyingsink_abort: (a: number, b: any) => any;
    readonly intounderlyingsink_close: (a: number) => any;
    readonly intounderlyingsink_write: (a: number, b: any) => any;
    readonly __wbg_intounderlyingsource_free: (a: number, b: number) => void;
    readonly intounderlyingsource_cancel: (a: number) => void;
    readonly intounderlyingsource_pull: (a: number, b: any) => any;
    readonly __wbg_intounderlyingbytesource_free: (a: number, b: number) => void;
    readonly intounderlyingbytesource_autoAllocateChunkSize: (a: number) => number;
    readonly intounderlyingbytesource_cancel: (a: number) => void;
    readonly intounderlyingbytesource_pull: (a: number, b: any) => any;
    readonly intounderlyingbytesource_start: (a: number, b: any) => void;
    readonly intounderlyingbytesource_type: (a: number) => number;
    readonly wasm_bindgen__closure__destroy__h3259e601f356730d: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h29e9203b9a6b80c6: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h564b38ee34a6f3a8: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h7699863adbb84373: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h7029f438781d1066: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__hf85e4151a92eaef0: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h0a63215576b1eb71: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__hd3457ad96818ef58: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__he416d1c9055abda9: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h0f902d0522b80131: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__he85915a9abc972ba: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__h86c89ca06c8045b9: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hcbd83fa405fc2adc: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__h878ce0dd3fc42266: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h6255f5c7f4303c2c: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hfca6eb8757f874a8: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h6e8c2f87632258ef: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h1233366533eac4a8: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hb9cc82184f414d39: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__he75aa3f7a3a2eb55: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h35853782ed476e41: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hd14db50b24902e16: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__heef249972dfd2ce0: (a: number, b: number, c: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hea448f067cfd4995: (a: number, b: number) => number;
    readonly wasm_bindgen__convert__closures_____invoke__h9af8041dd8318159: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h16a60e7ed8f1ae33: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
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
