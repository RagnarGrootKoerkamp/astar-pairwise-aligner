import init from "./pkg/astar_pairwise_aligner.js";
// TODO: fix the import using
// https://stackoverflow.com/questions/61986932/how-to-pass-a-string-from-js-to-wasm-generated-through-rust-using-wasm-bindgen-w
init()
  .then((wasm) => {
    const canvas = document.getElementById("canvas");
    const context = canvas.getContext("2d");

    document.getElementById("run").addEventListener("click", (event) => {
      wasm.run();
    });

    wasm.run();
  })
  .catch(console.error);
