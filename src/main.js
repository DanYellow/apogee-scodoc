import { open } from "@tauri-apps/plugin-dialog";
import { save } from "@tauri-apps/plugin-dialog";

import { getCurrentWindow } from "@tauri-apps/api/window";

const { invoke } = window.__TAURI__.core;

const listUploadButtons = document.querySelectorAll("[data-upload-btn]");

const listFiles = {};

Array.from(listUploadButtons).forEach((button) => {
  button.addEventListener("click", async (e) => {
    const $el = e.currentTarget;
    const config = JSON.parse($el.dataset.uploadBtn);

    const output = document.getElementById(config.name);

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
  const formattedData = await invoke("process_form_payload", {
    exportApogee: listFiles.export_apogee,
    exportScodoc: listFiles.export_scodoc,
    bareme: formData.get("bareme"),
    separateurCsv: formData.get("separateur_csv"),
  });

  if (formattedData) {
    const now = new Date();
    const nowStr = now.toISOString().slice(0, 10);
    const nowTimeStr = now
      .toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })
      .replaceAll(":", "-");

    const outputPath = await save({
      defaultPath: `export-scodoc-pour-apogee-${nowStr}-${nowTimeStr}.csv`,
      filters: [
        {
          name: "CSV",
          extensions: ["csv"],
        },
      ],
    });

    await invoke("download_data", {
      outputPath,
      csvContent: formattedData,
    });
  }
});

const dropZones = document.querySelectorAll("[data-dropzone]");

const handleDroppedFile = (file) => {};

const handleOverFile = (file) => {};

const isFileValid = (path, allowedExt) => {
  const extension = path.split(".").pop().toLowerCase();

  return allowedExt.includes(extension);
};

let activeDropZone = null;
let listDraggedPaths = [];

function getZoneAtPosition(x, y) {
  const element = document.elementFromPoint(x, y);

  return element?.closest("[data-dropzone]");
}

let currentZone = null;

const setDragOver = (zone) => {
  if (currentZone === zone || !zone) {
    return;
  }

  // Remove from previous zone
  if (currentZone) {
    currentZone.style.borderColor = "";
  }

  // Add to new zone
  if (zone) {
    zone.style.borderColor = "var(--color-red-800)";
  }

  currentZone = zone;
};

const appWindow = getCurrentWindow();

const scaleFactor = await appWindow.scaleFactor();

await appWindow.onDragDropEvent((event) => {
  const { type } = event.payload;

  if (type === "over") {
    const { x, y } = event.payload.position;

    const rect = document.documentElement.getBoundingClientRect();

    const element = document.elementFromPoint(x - rect.left, y - rect.top);

    const zone = element?.closest("[data-dropzone]") ?? null;

    setDragOver(zone);
    return;
  }

  if (type === "drop" || type === "leave") {
    // setDragOver(null);
  }
});

// await getCurrentWindow().onDragDropEvent((event) => {
//   const payload = event.payload;

//   if (payload.type === "enter") {
//     console.log("Files entered:", payload.paths);
//   }

//   if (payload.type === "over") {
//     const zone = getZoneAtPosition(payload.position.x, payload.position.y);

//     listDropZones.forEach((z) => {
//       z.style.borderColor = "";
//     });

//     if (zone) {
//       listDropZones.forEach((z) => {
//         z.style.borderColor = "var(--color-red-800)";
//       });
//     }
//   }

//   if (payload.type === "drop") {
//     listDropZones.forEach((z) => {
//       z.style.borderColor = "";
//     });

//     const zone = getZoneAtPosition(payload.position.x, payload.position.y);

//     if (!zone) {
//       console.log("Dropped outside a drop zone");
//       return;
//     }

//     const zoneName = zone.dataset.zone;
//     const files = payload.paths;

//     console.log("Zone:", zoneName);
//     console.log("Files:", files);

//     // handleFiles(zoneName, files);
//   }

//   if (payload.type === "leave") {
//     listDropZones.forEach((z) => {
//       z.style.borderColor = "";
//     });
//   }
// });
