import { open } from "@tauri-apps/plugin-dialog";

const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;

// async function greet() {
//   // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
//   greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
// }

const listUploadButtons = document.querySelectorAll("[data-upload-btn]");

const files = {}

Array.from(listUploadButtons).forEach((button) => {
  button.addEventListener("click", async (e) => {
    const $el = e.currentTarget;
    const config = JSON.parse($el.dataset.uploadBtn);
    
    const output = document.getElementById(config.name)

    const file = await open({
      multiple: false,
      filters: [
        {
          name: config.name,
          extensions: [config.accept],
        },
      ],
    });

    if (file && output) {
      output.innerHTML = `Fichier sélectionné : <span class="truncate">${file}</span>`;
      console.log(file);
    }
  });
});

const form = document.querySelector("form");

form.addEventListener("submit", (e) => {
  e.preventDefault();
});

// window.addEventListener("DOMContentLoaded", () => {
//   greetInputEl = document.querySelector("#greet-input");
//   greetMsgEl = document.querySelector("#greet-msg");
//   document.querySelector("#greet-form").addEventListener("submit", (e) => {
//     e.preventDefault();
//     greet();
//   });
// });
