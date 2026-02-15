/* tslint:disable */
/* eslint-disable */

export class Game {
    free(): void;
    [Symbol.dispose](): void;
    get_board_state(): any;
    get_eval_breakdown(): any;
    get_hint(depth: number): any;
    get_last_evals(): bigint;
    get_legal_moves_for_square(row: number, col: number): any;
    make_ai_move(): any;
    make_move(from_row: number, from_col: number, to_row: number, to_col: number, promotion?: string | null): any;
    constructor();
    set_auto_deepen(enabled: boolean, min_evals: bigint): void;
    set_depth(depth: number): void;
    set_module(name: string, enabled: boolean): void;
}

export function build_timestamp(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_game_free: (a: number, b: number) => void;
    readonly build_timestamp: () => [number, number];
    readonly game_get_board_state: (a: number) => any;
    readonly game_get_eval_breakdown: (a: number) => any;
    readonly game_get_hint: (a: number, b: number) => any;
    readonly game_get_last_evals: (a: number) => bigint;
    readonly game_get_legal_moves_for_square: (a: number, b: number, c: number) => any;
    readonly game_make_ai_move: (a: number) => any;
    readonly game_make_move: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => any;
    readonly game_new: () => number;
    readonly game_set_auto_deepen: (a: number, b: number, c: bigint) => void;
    readonly game_set_depth: (a: number, b: number) => void;
    readonly game_set_module: (a: number, b: number, c: number, d: number) => void;
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
