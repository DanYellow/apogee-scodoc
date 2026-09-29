import { open, save } from '@tauri-apps/plugin-dialog';
import { getCurrentWindow } from '@tauri-apps/api/window';
import * as z from 'zod';

import {
    getDropZone,
    resetDragState,
    getDropZoneConfig,
} from './assets/scripts/drag-n-drop.utils';

const { invoke } = window.__TAURI__.core;

const listUploadButtons = document.querySelectorAll('[data-upload-btn]');

const errorFormDialog = document.getElementById("error-form-dialog");
const downloadStartedDialog = document.getElementById("start-download-dialog");

const listFiles = {};
let currentZone = null;

const FormPayload = z.object({
    exportApogee: z.string(),
    exportScodoc: z.string(),
});

const handleFile = (path, config) => {
    const output = document.getElementById(config.name);

    if (output) {
        output.innerHTML = `Fichier sélectionné : <span class="">${path}</span>`;
        listFiles[config.name] = path;
    }
};

Array.from(listUploadButtons).forEach((button) => {
    button.addEventListener('click', async (e) => {
        const $el = e.currentTarget;
        const config = JSON.parse($el.dataset.uploadBtn);

        const file = await open({
            multiple: false,
            filters: [
                {
                    name: config.name,
                    extensions: [config.accept],
                },
            ],
        });

        if (file) {
            handleFile(file, config);
        }
    });
});

const form = document.querySelector('form');

form.addEventListener('submit', async (e) => {
    e.preventDefault();

    const formData = new FormData(form);

    const tauriPayload = {
        exportApogee: listFiles.export_apogee,
        exportScodoc: listFiles.export_scodoc,
        bareme: formData.get('bareme'),
        separateurCsv: formData.get('separateur_csv'),
    };

    const result = FormPayload.safeParse(tauriPayload);
    if (!result.success) {
        errorFormDialog.showModal()
        return;
    }

    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    const formattedData = await invoke('process_form_payload', tauriPayload);

    if (formattedData) {
        const now = new Date();
        const nowStr = now.toISOString().slice(0, 10);
        const nowTimeStr = now
            .toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
            .replaceAll(':', '-');

        const outputPath = await save({
            defaultPath: `export-scodoc-pour-apogee-${nowStr}-${nowTimeStr}.csv`,
            filters: [
                {
                    name: 'CSV',
                    extensions: ['csv'],
                },
            ],
        });

        downloadStartedDialog.showModal()

        await invoke('download_data', {
            outputPath,
            csvContent: formattedData,
        });
    }
});

const isFileValid = (path, allowedExt) => {
    const extension = path.split('.').pop().toLowerCase();

    return allowedExt.includes(extension);
};

const setDragOver = (zone) => {
    if (currentZone === zone) {
        return;
    }

    // Remove from previous zone
    if (currentZone) {
        currentZone.classList.remove('dropzone-valid-file');
        currentZone.classList.remove('dropzone-not-valid-file');
    }

    // Add to new zone
    if (zone) {
        // zone.style.borderColor = "var(--color-red-800)";
    }

    currentZone = zone;
};

const appWindow = getCurrentWindow();

let currentFilePath = '';

const updateDropZone = (position) => {
    const zone = getDropZone(position);

    if (zone === currentZone) {
        return;
    }

    // Reset previous zone
    if (currentZone) {
        currentZone.classList.remove(
            'dropzone-valid-file',
            'dropzone-not-valid-file',
            'shake',
        );
    }

    currentZone = zone;

    if (!zone || !currentFilePath) {
        setDragOver(null);
        return;
    }

    setDragOver(zone);

    const config = getDropZoneConfig(zone);

    if (!config) {
        return;
    }

    const isValid = isFileValid(currentFilePath, [config.accept]);

    zone.classList.toggle('dropzone-valid-file', isValid);
    zone.classList.toggle('dropzone-not-valid-file', !isValid);

    if (!isValid) {
        zone.classList.remove('shake');

        // Force browser to restart the animation
        void zone.offsetWidth;

        zone.classList.add('shake');
    }
};

await appWindow.onDragDropEvent((event) => {
    const { type, paths = [], position } = event.payload;

    if (type === 'enter') {
        currentFilePath = paths[0] ?? null;

        updateDropZone(position);
        return;
    }

    if (type === 'over') {
        updateDropZone(position);
        return;
    }

    if (type === 'drop') {
        // console.log("Files:", paths);

        const config = getDropZoneConfig(currentZone);
        const isValid = isFileValid(currentFilePath, [config.accept]);

        if (isValid) {
            handleFile(paths[0], config);
        }

        resetDragState(currentZone, setDragOver);
        currentFilePath = null;
        return;
    }

    if (type === 'leave') {
        resetDragState(currentZone, setDragOver);
        currentFilePath = null;
    }
});

form.addEventListener('reset', (event) => {
    listUploadButtons.forEach((item) => {
        const config = JSON.parse(item.dataset.uploadBtn);

        const output = document.getElementById(config.name);
        output.textContent = '';
        listFiles[config.name] = null;
    });
});
