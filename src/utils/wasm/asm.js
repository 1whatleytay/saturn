import * as wasm from "./asm_bg.wasm";
import { __wbg_set_wasm } from "./asm_bg.js";
__wbg_set_wasm(wasm);
export * from "./asm_bg.js";
