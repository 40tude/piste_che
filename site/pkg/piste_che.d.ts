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
    readonly __wbg_intounderlyingsource_free: (a: number, b: number) => void;
    readonly intounderlyingsource_cancel: (a: number) => void;
    readonly intounderlyingsource_pull: (a: number, b: number) => number;
    readonly __wbg_intounderlyingbytesource_free: (a: number, b: number) => void;
    readonly intounderlyingbytesource_autoAllocateChunkSize: (a: number) => number;
    readonly intounderlyingbytesource_cancel: (a: number) => void;
    readonly intounderlyingbytesource_pull: (a: number, b: number) => number;
    readonly intounderlyingbytesource_start: (a: number, b: number) => void;
    readonly intounderlyingbytesource_type: (a: number) => number;
    readonly __wbg_intounderlyingsink_free: (a: number, b: number) => void;
    readonly intounderlyingsink_abort: (a: number, b: number) => number;
    readonly intounderlyingsink_close: (a: number) => number;
    readonly intounderlyingsink_write: (a: number, b: number) => number;
    readonly __wasm_bindgen_func_elem_4384: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_1926: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_2233: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_2494: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_2589: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_2640: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_2694: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_4369: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_5225: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_2002: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_2234: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_2519: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_2574: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_2689: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_2689_8: (a: number, b: number, c: number) => void;
    readonly __wasm_bindgen_func_elem_2575: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_2641: (a: number, b: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
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
