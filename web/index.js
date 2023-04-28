import { Processor } from "chip-8-wasm";
import { memory } from "chip-8-wasm/chip_8_wasm_bg";

const PIXEL_SIZE = 15;
const WHITE_COLOR = "#2a9fd6";
const BLACK_COLOR = "#0b2633";
const roms = {
  breakout: "Ep/8/ICiAt3BAO6iBNuhAO6iA2ACYQWHAIYQ1nFxCG84jxdPABIXcAJvEI8HTwASFQDuIgV9BCIFAO4iBX38IgUA7oCAQAFo/0D/aAFawCJTAO6AsHD7YfiAEnAFogPQoQDuIguLlIqEIgtLAGkBSz9p/0oAaAFKH2j/TwEiQ0ofIoUA7gDgax5qFCIFIgsiEQDu/gc+ABKTbgT+FQDubR5sHmtAah3JAUkAaf9o/yIFIgsiEWAH4KEiO2AJ4KEiMyJjIpMStQ==",
  snake: "FtaAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAYJCQ8JDgkOCQ4GCAgIBg4JCQkODggOCA4OCA4ICAcICwkHCQkPCQkOBAQEDgMBAQkGCQoMCgkICAgIDgiNioiIiQ0LCQkGCQkJBg4JCQ4IBgkJCwYOCQkOCQcIBgEODgQEBAQJCQkJBgkJCgoECIqKioUJCQYJCQkJBwEGDgIECA4ABwkJBwgOCQkOAAYICAYBBwkJBwAGCwwGAgQOBAQHCQ8BBggOCQkJBAAEBAQEAAQECAgJCg4JBAQEBAQADwqKioAOCQkJAAYJCQYOCQ4ICAcJDwEBAAoMCAgABwwDDgQOBAQCAAkJCQcACQkKBAAKioUFAAoEBAoACQcBBgAPAgQPAAAAAAgICAgACA4BBgAEAAAOAAAACAAIAAUPhQ+FBAgICAQIBAQECAAKBAoAAAQOBAAAAAAAAADz5y+BwEAgJBfvwAQEF4fG5jYXAwMDgAwMDh4WNjZsdHRo4CQcHh8TE5GemJCAgIDA4MjLiw8NjOz+YAP0dCQGJ+ZGBxPzYADyseLCw+FgASAy0oPiwtGistBhomHj4oLx4rNRIcKCseOBEeHCgrHTgAAKW88VWBUIJgYwAAAPMe8GWFAKQEbgUwACXo0SVxBUUWcQFFJnEBRStx/0Utcf9zAVNAFbwA7v4ecP8wABXoAO55AaIE+R75HoCggbDxVQDugpCC1aIE8h7yHvFlAO59Acc/yB8A7mEAYgBjAEAAFjpw/3EBMQoWNmEAcgEyChY2YgBzATAAFiLxKWo32rXyKWox2rXzKWor2rUA7gDgffvwhYrQiwBKABZqSwAWZnr/e/8WWIDQ8HWMAGClYZlkCmUIZgMlsmClYaNkBmUFZg4lsoDQaw4mGGClYalkB2UFZhYlsoDAaxYmGP8KAOBgAGEAYgBjAGQAZQBmAGcAb0CiBPdVf/8/ABa0FtZBAdeBJhAXmGwEF1hsAxdYbAIXWGwBF1hsAG0EaiBrEGkAJhBgBGEGYgxjBKU/0wxx/3MI8h4xABbsYKVhh2QKZQRmEiWyYKVhkWQIZRxmGCWy/wpPBWwETwdsA08IbAJPCWwBYAAA4EACFzAmAqIC0BFvBUwCFzrvoRbGbwdMARdE76EWym8ITAQXTu+hFs5vCUwDF1jvoRbSTAF6AUwCewFMA3r/TAR7/0pAagBLIGsASv9qP0v/ax8l8mEAbwCiAtqxTwFhAWAAmnBwAZuAcAFAAha+QQEmTqIC14EXJg==",
  space_invaders: "EiVTUEFDRSBJTlZBREVSUyAwLjkxIEJ5IERhdmlkIFdJTlRFUmAAYQBiCKPd0BhxCPIeMSASLXAIYQAwQBItaQVsFW4AI5FgCvAV8AcwABJLI5F+ARJFZgBoHGkAagRrCmwEbTxuDwDgI3UjUf0VYATgnhJ9I3U4AHj/I3VgBuCeEosjdTg5eAEjdTYAEp9gBeCeEulmAWUbhICj2dRRo9nUUXX/Nf8SrWYAEunUUT8BEunUUWYAg0BzA4O1YviDImIIMwASySN9ggZDCBLTMxAS1SN9ggYzGBLdI32CBkMgEuczKBLpI30+ABMHeQZJGGkAagRrCmwEffRuDwDgI1Ejdf0VEm/3BzcAEm/9FSNRi6Q7EhMbfAJq/DsCEyN8AmoEI1E8GBJvAOCk3WAUYQhiD9AfcAjyHjAsEzNg//AV8AcwABNB8AoA4KcG/mUSJaPB+R5hCCNpgQYjaYEGI2mBBiNpe9AA7oDggBIwANvGewwA7qPZYBzYBADuI1GOIyNRYAXwGPAV8AcwABOJAO5qAI3gawTpoRJXpgz9HvBlMP8Tr2oAawRtAW4BE5elCvAe28Z7CH0BegE6BxOXAO48fv//mZl+//8kJOd+/zw8ftuBQjx+/9sQOHz+AAB/AD8AfwAAAAEBAQMDAwMAAD8gICAgICAgID8ICP8AAP4A/AD+AAAAfkJCYmJiYgAA/wAAAAAAAAAA/wAA/wB9AEF9BX19AADCwsZEbCg4AAD/AAAAAAAAAAD/AAD/APcQFPf3BAQAAHxE/sLCwsIAAP8AAAAAAAAAAP8AAP8A7yAo6OgvLwAA+YXFxcXF+QAA/wAAAAAAAAAA/wAA/wC+ACAwIL6+AAD3BOeFhYT0AAD/AAAAAAAAAAD/AAD/AAB/AD8AfwAAAO8o7wDgYG8AAP8AAAAAAAAAAP8AAP8AAP4A/AD+AAAAwADAwMDAwAAA/AQEBAQEBAQE/BAQ//mBuYuamvoA+oqampuZ+OYlJfQ0NDQAFxQ0NzYmx99QUFzY2N8A3xEfEhsZ2XxE/oaGhvyE/oKC/v6AwMDA/vyCwsLC/P6A+MDA/v6A8MDAwP6AvoaG/oaG/oaGhhAQEBAQEBgYGEhIeJyQsMCwnICAwMDA/u6SkoaGhv6ChoaGhnyChoaGfP6C/sDAwHyCwsrEev6G/pCchP7A/gIC/v4QMDAwMIKCwsLC/oKCgu44EIaGlpKS7oJEODhEgoKC/jAwMP4CHvCA/gAAAAAGBgAAAGBgwAAAAAAAABgYGBgAGHzGDBgAGAAA/v4AAP6ChoaG/ggICBgYGP4C/sDA/v4CHgYG/oTExP4EBP6A/gYG/sDAwP6C/v4CAgYGBnxE/oaG/v6C/gYGBkT+RET+RKioqKioqKhsWgAMGKgwTn4AEhhmbKhaZlQkZgBISBgSqAaQqBIAfjASqIQwTnIYZqioqKioqJBUeKhIeGxyqBIYbHJmVJCocioYqDBOfgASGGZsqHJUqFpmGH4YTnKocioYMGaoME5+AGwwVE6cqKioqKioqEhUfhiokFR4ZqhsKjBaqIQwciqo2KgAThKo5KKoAE4SqGwqVFRyqIQwciqo3pyocioYqAxUSFp4chhmqGYYWlRmcmyocioAcqhyKhioME5+ABIYZmyoAGYYqDBODGYYAGwwTiSocioYMGaoHlRmDBicqCRUVBKoQngMPKiuqKioqKioqP8AAAAAAAAAAAAAAAAAAAA=",
  kaleidoscope: "YABjgGEfYg8iMqIA8x7wCvBVQAASHHMBMwASCGOAogDzHvBlQAASHHMBQwASHCIyEh5AAnL/QARx/0AGcQFACHIBondq4IoSax+BsjoAcgFq8Ioiaw+CsjoAcQFrH4Gy0SGKEGsfiyXasWo/ihXasYsg2rEA7gGAAAA=",
}

let loadedRom = null;
let processor = null;
let speed = 15;
let paused = false;

const canvas = document.getElementById("chip-8-canvas");
canvas.height = 32 * PIXEL_SIZE;
canvas.width = 64 * PIXEL_SIZE;
const ctx = canvas.getContext('2d');

const stateButton = document.getElementById("stateButton");

const pauseButton = document.getElementById("pauseButton");
pauseButton.onclick = function() {
  if(processor.halted) {
    return;
  }
  paused = !paused;
  pauseButton.innerHTML = paused ? '<i class="bi bi-play-fill"></i>' : '<i class="bi bi-pause-fill"></i>';
  if(paused) {
    stateButton.classList.remove("btn-success");
    stateButton.classList.add("btn-warning");
    stateButton.innerHTML = "Paused";
  } else {
    stateButton.classList.remove("btn-warning");
    stateButton.classList.add("btn-success");
    stateButton.innerHTML = "Running";
  }
}

const reloadButton = document.getElementById("reloadButton");
reloadButton.onclick = function() {
  startProcessor();
  pauseButton.innerHTML = '<i class="bi bi-pause-fill"></i>';
}

const loadFromFilesystem = document.getElementById("loadFromFilesystem");
loadFromFilesystem.onclick = function() {
   // creating input on-the-fly
   var input = document.createElement("input");
   input.type = "file";
   input.addEventListener('change', (e) => {
    const reader = new FileReader();
    reader.readAsArrayBuffer(e.target.files[0]);
    reader.onloadend = (evt) => {
      if (evt.target.readyState === FileReader.DONE) {
        const arrayBuffer = evt.target.result,
        array = new Uint8Array(arrayBuffer);
        insertRom(array);
      }
    }
  });
   // input.attr("type", "file");
   // add onchange handler if you wish to get the file :)
   input.click(); // opening dialog
   return false; // avoiding navigation
}

const loadRom = document.getElementById('selectRom');
loadRom.onchange = function(event) {
  insertRom(Uint8Array.from(atob(roms[event.target.value]), c => c.charCodeAt(0)));
}

const processorSpeed = document.getElementById("processorSpeed");
processorSpeed.value = 15;
processorSpeed.oninput = function() {
  speed = this.value;
} 

const key_map = new Map([
  ["1", 1],
  ["2", 2],
  ["3", 3],
  ["4", 12],
  ["q", 4],
  ["w", 5],
  ["e", 6],
  ["r", 13],
  ["a", 7],
  ["s", 8],
  ["d", 9],
  ["f", 14],
  ["y", 10],
  ["x", 0],
  ["c", 11],
  ["v", 15]
]);

const renderLoop = () => {
  if (processor != null && !processor.halt && !paused) {
    for(var i = 0; i < speed; i++) {
      processor.tick();
    }
    drawScreen();
    if(processor.halt) {
      stateButton.classList.remove("btn-success");
      stateButton.classList.add("btn-light");
      stateButton.innerHTML = "Halted";
      processor = null;
    }
  }
  requestAnimationFrame(renderLoop);
};

const drawScreen = () => {
    const screenPtr = processor.screen();
    const screen = new Uint8Array(memory.buffer, screenPtr, 256);
  
    ctx.beginPath();

    for (let y = 0; y < 32; y++) {
      for (let x = 0; x < 8; x++) {
        let byte = screen[y*8+x];
        for (let bit = 0; bit < 8; bit++) {
          let color = (byte & Math.pow(2, 7-bit)) >= 1;
          ctx.fillStyle = color ? WHITE_COLOR : BLACK_COLOR;
          ctx.fillRect(
            (x*8+bit) * PIXEL_SIZE, 
            y * PIXEL_SIZE, 
            PIXEL_SIZE, 
            PIXEL_SIZE);
        }
      }
    }
  
    ctx.stroke();
};

const insertRom = (rom) => {
  loadedRom = rom;
  startProcessor();
}

const startProcessor = () => {
  paused = false;
  stateButton.classList.remove("btn-warning");
  stateButton.classList.remove("btn-light");
  stateButton.classList.add("btn", "btn-success", "disabled");
  stateButton.innerHTML = "Running";
  processor = Processor.new(loadedRom);
}

window.addEventListener(
  "keydown",
  (event) => {
    if(processor) {
      let key_id = key_map.get(event.key);
      if (key_id != null) {
        console.log("pressed: " + key_id);
        processor.key_pressed(key_id);
      } else {
        console.log("Unmapped key: " + event.key);
      }
      
    }
  },
  true
);

window.addEventListener(
  "keyup",
  (event) => {
    if(processor) {
      let key_id = key_map.get(event.key);
      if (key_id != null) {
        console.log("released: " + key_id);
        processor.key_released(key_id);
      } else {
        console.log("Unmapped key: " + event.key);
      }
      
    }
  },
  true
);

insertRom(Uint8Array.from(atob(roms["snake"]), c => c.charCodeAt(0)));
requestAnimationFrame(renderLoop);

