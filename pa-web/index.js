import init from "./pkg/pa_web.js";
// TODO: fix the import using
// https://stackoverflow.com/questions/61986932/how-to-pass-a-string-from-js-to-wasm-generated-through-rust-using-wasm-bindgen-w
init()
  .then((wasm) => {
    const canvas = document.getElementById("canvas");
    const context = canvas.getContext("2d");
    var delay = document.getElementById("delay");

    var timer = null;
    var play = true;

    document.getElementById("args").addEventListener("change", (event) => {
      wasm.reset();
    });

    document.getElementById("run").addEventListener("click", (event) => {
      wasm.reset();
    });

    document.getElementById("prev").addEventListener("click", (event) => {
      wasm.prev();
    });

    document.getElementById("next").addEventListener("click", (event) => {
      wasm.next();
    });

    function maketimer() {
      timer = window.setTimeout(() => {
        wasm.next();
        maketimer();
      }, delay.value * 1000);
    }

    function faster() {
      delay.value /= 1.5;
    }

    function slower() {
      delay.value *= 1.5;
    }

    function pauseplay() {
      if (play) {
        play = false;
        window.clearTimeout(timer);
        timer = null;
      } else {
        play = true;
        maketimer();
      }
    }

    document.getElementById("faster").addEventListener("click", faster);
    document.getElementById("slower").addEventListener("click", slower);
    document.getElementById("pauseplay").addEventListener("click", pauseplay);

    wasm.reset();

    maketimer();

    canvas.addEventListener("keydown", function (e) {
      switch (e.keyCode) {
        case 8: // backspace
        case 37: // left
          wasm.prev();
          break;
        case 32: // space
        case 39: // right
          wasm.next();
          break;
        case 38: // up
        case 70: // f
        case 187: // +
          faster();
          break;
        case 40: // down
        case 83: // s
        case 189: // -
          slower();
          break;
        case 13: // return
        case 80: // p
          pauseplay();
          break;
        case 82: // r
          wasm.reset();
          break;
        default:
          return;
      }
      e.stopPropagation();
      e.preventDefault();
    });
  })
  .catch(console.error);
