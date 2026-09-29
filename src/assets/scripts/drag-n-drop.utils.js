export const getDropZone = (position) => {
    const { x, y } = position;

    const rect = document.documentElement.getBoundingClientRect();

    const element = document.elementFromPoint(x - rect.left, y - rect.top);

    return element?.closest('[data-dropzone]') ?? null;
};

export const resetDragState = (currentZone, callback) => {
    if (currentZone) {
        currentZone.classList.remove(
            'dropzone-valid-file',
            'dropzone-not-valid-file',
            'shake',
        );
    }

    callback(null);

    currentZone = null;
};

export const getDropZoneConfig = (zone) => {
    const uploadBtn = zone.querySelector('[data-upload-btn]');

    if (!uploadBtn?.dataset.uploadBtn) {
        return null;
    }

    try {
        return JSON.parse(uploadBtn.dataset.uploadBtn);
    } catch (error) {
        console.error('Invalid upload configuration:', error);
        return null;
    }
};
