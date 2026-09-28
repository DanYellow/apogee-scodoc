import { open } from "@tauri-apps/plugin-dialog";

import {
  truncate,
  readTextFile,
  writeTextFile,
  BaseDirectory,
} from '@tauri-apps/plugin-fs';

const { invoke } = window.__TAURI__.core;

const listUploadButtons = document.querySelectorAll("[data-upload-btn]");

const listFiles = {}

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
      output.innerHTML = `Fichier sélectionné : <span class="">${file}</span>`;
      listFiles[config.name] = file;
    }
  });
});

const form = document.querySelector("form");

form.addEventListener("submit", async (e) => {
  e.preventDefault();

  const formData = new FormData(form);

  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  const res = await invoke("process_form_payload", { 
    exportApogee: listFiles.export_apogee,
    exportScodoc: listFiles.export_scodoc,
    bareme: formData.get("bareme"),
    separateurCsv: formData.get("separateur_csv"),
  });

  console.log(res)
});
