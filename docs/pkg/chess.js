/* @ts-self-types="./chess.d.ts" */

export class Game {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        GameFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_game_free(ptr, 0);
    }
    /**
     * @returns {any}
     */
    get_board_state() {
        const ret = wasm.game_get_board_state(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get_eval_breakdown() {
        const ret = wasm.game_get_eval_breakdown(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} depth
     * @returns {any}
     */
    get_hint(depth) {
        const ret = wasm.game_get_hint(this.__wbg_ptr, depth);
        return ret;
    }
    /**
     * @returns {bigint}
     */
    get_last_evals() {
        const ret = wasm.game_get_last_evals(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {number} row
     * @param {number} col
     * @returns {any}
     */
    get_legal_moves_for_square(row, col) {
        const ret = wasm.game_get_legal_moves_for_square(this.__wbg_ptr, row, col);
        return ret;
    }
    /**
     * @returns {any}
     */
    make_ai_move() {
        const ret = wasm.game_make_ai_move(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} from_row
     * @param {number} from_col
     * @param {number} to_row
     * @param {number} to_col
     * @param {string | null} [promotion]
     * @returns {any}
     */
    make_move(from_row, from_col, to_row, to_col, promotion) {
        var ptr0 = isLikeNone(promotion) ? 0 : passStringToWasm0(promotion, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.game_make_move(this.__wbg_ptr, from_row, from_col, to_row, to_col, ptr0, len0);
        return ret;
    }
    constructor() {
        const ret = wasm.game_new();
        this.__wbg_ptr = ret >>> 0;
        GameFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {boolean} enabled
     * @param {bigint} min_evals
     */
    set_auto_deepen(enabled, min_evals) {
        wasm.game_set_auto_deepen(this.__wbg_ptr, enabled, min_evals);
    }
    /**
     * @param {number} depth
     */
    set_depth(depth) {
        wasm.game_set_depth(this.__wbg_ptr, depth);
    }
    /**
     * @param {string} name
     * @param {boolean} enabled
     */
    set_module(name, enabled) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.game_set_module(this.__wbg_ptr, ptr0, len0, enabled);
    }
}
if (Symbol.dispose) Game.prototype[Symbol.dispose] = Game.prototype.free;

/**
 * @returns {string}
 */
export function build_timestamp() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.build_timestamp();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Error_8c4e43fe74559d73: function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg___wbindgen_is_string_cd444516edc5b180: function(arg0) {
            const ret = typeof(arg0) === 'string';
            return ret;
        },
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_new_361308b2356cecd0: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_3eb36ae241fe6f44: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_dca287b076112a51: function() {
            const ret = new Map();
            return ret;
        },
        __wbg_random_912284dbf636f269: function() {
            const ret = Math.random();
            return ret;
        },
        __wbg_set_1eb0999cf5d27fc8: function(arg0, arg1, arg2) {
            const ret = arg0.set(arg1, arg2);
            return ret;
        },
        __wbg_set_3f1d0b984ed272ed: function(arg0, arg1, arg2) {
            arg0[arg1] = arg2;
        },
        __wbg_set_f43e577aea94465b: function(arg0, arg1, arg2) {
            arg0[arg1 >>> 0] = arg2;
        },
        __wbindgen_cast_0000000000000001: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_0000000000000003: function(arg0) {
            // Cast intrinsic for `U64 -> Externref`.
            const ret = BigInt.asUintN(64, arg0);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./chess_bg.js": import0,
    };
}

const GameFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_game_free(ptr >>> 0, 1));

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('chess_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
