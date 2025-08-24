const previewFrame = document.getElementById('preview-frame');
const toggleButton = document.getElementById('togglePreview');

toggleButton.addEventListener('click', () => {
    if (previewFrame.style.display === 'none') {
        previewFrame.style.display = 'block';
    } else {
        previewFrame.style.display = 'none';
    }
});

const popoutButton = document.getElementById('popoutPreview');

popoutButton.addEventListener('click', () => {
    const content = monacoEditor.getValue(); // Get code from Monaco
    const newWindow = window.open();
    newWindow.document.open();
    newWindow.document.write(content); // Write the HTML content
    newWindow.document.close();
});