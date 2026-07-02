import init, * as exports from '../pkg/brick_cartographer.js';
import wasmUrl from '../pkg/brick_cartographer_bg.wasm?url';

const wasm = init({ module_or_path: wasmUrl }).then(() => exports);

export default wasm;
