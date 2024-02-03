import init from '../pkg/snakegame_wasm.js'
import * as wasm from '../pkg/snakegame_wasm.js'

await init(); //Initialise wasm

const TPS = wasm.queryTPS();
canvas.addEventListener("keydown",(key) => {
  key.preventDefault();
  wasm.sendKeypress(key.keyCode);
});

function gameLoop() {
  wasm.rustGameLoop();
  setTimeout(gameLoop,1000/TPS);
}
gameLoop();
