document.addEventListener('DOMContentLoaded', () => {
    // --- Configuration and State ---
    const API_BASE_URL = 'https://api.mp2upnhs.my'; // <-- IMPORTANT: MAKE SURE THIS IS YOUR URL
    const ROOM_ID = 'public_room';

    let monacoEditor;
    let currentWebSocket;
    let currentFileId;
    let isUpdatingEditor = false; // Prevents WebSocket feedback loops

    // --- DOM Element References ---
    const fileTreeContainer = document.getElementById('file-tree');
    const editorContainer = document.getElementById('editor-container');
    const previewFrame = document.getElementById('preview-frame');
    const resizerFmEd = document.getElementById('resizer-fm-ed');
    const resizerEdPv = document.getElementById('resizer-ed-pv');

    // --- Monaco Editor Initialization ---
    require.config({ paths: { 'vs': 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs' }});
    require(['vs/editor/editor.main'], () => {
        monacoEditor = monaco.editor.create(editorContainer, {
            value: `// Select a file from the list to begin editing`,
            language: 'plaintext',
            theme: 'vs-dark',
            automaticLayout: true,
        });

        // Add the listener for when the user types
        monacoEditor.onDidChangeModelContent(() => {
            if (isUpdatingEditor) return;
            
            const content = monacoEditor.getValue();
            
            // NEW! Update the preview on every keystroke
            updatePreview(content);

            if (currentWebSocket && currentWebSocket.readyState === WebSocket.OPEN) {
                currentWebSocket.send(content);
            }
        });
    });

    // --- Core Functions ---

    /**
     * NEW! This function takes content and renders it in the iframe.
     * It only renders if the currently selected file is HTML.
     * @param {string} content - The HTML content string.
     */
    function updatePreview(content) {
        // Find the DOM element for the currently loaded file to check its name
        const currentFileElement = document.querySelector(`[data-file-id="${currentFileId}"]`);
        
        // Only update the preview if an HTML file is active
        if (currentFileElement && getLanguageForFileName(currentFileElement.textContent) === 'html') {
             previewFrame.srcdoc = content;
        } else {
             // If not an HTML file, show a helpful message
             previewFrame.srcdoc = `<html><body style='color: #888; font-family: sans-serif; padding: 20px;'>Live preview is only available for HTML files.</body></html>`;
        }
    }

    async function fetchFileTree() {
        try {
            const response = await fetch(`${API_BASE_URL}/api/file-tree/${ROOM_ID}`);
            if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
            const projects = await response.json();
            renderFileTree(projects);
        } catch (error) {
            console.error("Failed to fetch file tree:", error);
            fileTreeContainer.innerHTML = '<p style="color: red;">Error loading files.</p>';
        }
    }

    function renderFileTree(projects) {
        fileTreeContainer.innerHTML = '';
        projects.forEach(project => {
            const projectDiv = document.createElement('div');
            projectDiv.className = 'project-name';
            projectDiv.textContent = project.name;
            fileTreeContainer.appendChild(projectDiv);

            project.files.forEach(file => {
                const fileDiv = document.createElement('div');
                fileDiv.className = 'file-name';
                fileDiv.textContent = file.name;
                fileDiv.dataset.fileId = file.id;
                fileTreeContainer.appendChild(fileDiv);
            });
        });
    }

    async function loadFile(fileId) {
        if (currentFileId === fileId) return;

        try {
            const response = await fetch(`${API_BASE_URL}/api/file/${fileId}`);
            if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
            const fileContent = await response.json();
            
            isUpdatingEditor = true;
            monacoEditor.setValue(fileContent.content);
            isUpdatingEditor = false;
            
            const language = getLanguageForFileName(fileContent.name);
            monaco.editor.setModelLanguage(monacoEditor.getModel(), language);

            // NEW! Update the preview when a file is first loaded
            updatePreview(fileContent.content);

            currentFileId = fileId;
            connectWebSocket(fileId);
        } catch (error) {
            console.error("Failed to load file content:", error);
        }
    }

    function connectWebSocket(fileId) {
        if (currentWebSocket) currentWebSocket.close();

        const wsProtocol = API_BASE_URL.startsWith('https://') ? 'wss://' : 'ws://';
        const wsHost = API_BASE_URL.replace(/^https?:\/\//, '');
        const username = `User_${Math.floor(Math.random() * 1000)}`;
        const wsUrl = `${wsProtocol}${wsHost}/ws/${fileId}/${username}`;

        currentWebSocket = new WebSocket(wsUrl);
        currentWebSocket.onopen = () => console.log("WebSocket connection established.");
        currentWebSocket.onmessage = (event) => {
            const receivedContent = event.data;
            if (monacoEditor.getValue() !== receivedContent) {
                isUpdatingEditor = true;
                const currentPosition = monacoEditor.getPosition();
                monacoEditor.setValue(receivedContent);
                monacoEditor.setPosition(currentPosition);
                isUpdatingEditor = false;
                
                // NEW! Update the preview when a collaborator's changes are received
                updatePreview(receivedContent);
            }
        };
        currentWebSocket.onerror = (error) => console.error("WebSocket error:", error);
        currentWebSocket.onclose = () => console.log("WebSocket connection closed.");
    }

    function getLanguageForFileName(fileName) {
        const extension = fileName.split('.').pop();
        switch (extension) {
            case 'html': return 'html';
            case 'css': return 'css';
            case 'js': return 'javascript';
            case 'json': return 'json';
            case 'md': return 'markdown';
            default: return 'plaintext';
        }
    }

    // --- Event Listeners & Initial Load ---
    fileTreeContainer.addEventListener('click', (event) => {
        if (event.target && event.target.matches('.file-name')) {
            const fileId = event.target.dataset.fileId;
            if (fileId) loadFile(fileId);
        }
    });

    function makeResizable(resizer, leftPanel, rightPanel) {
        const minWidth = 100;

        let x = 0;
        let leftPanelWidth = 0;
        let rightPanelWidth = 0;

        const mouseDownHandler = (e) => {
            x = e.clientX;
            leftPanelWidth = leftPanel.getBoundingClientRect().width;
            rightPanelWidth = rightPanel.getBoundingClientRect().width;
            document.addEventListener('mousemove', mouseMoveHandler);
            document.addEventListener('mouseup', mouseUpHandler);
        };

        const mouseMoveHandler = (e) => {
            const dx = e.clientX - x;
            const newLeftWidth = leftPanelWidth + dx;
            const newRightWidth = rightPanelWidth - dx;
            if (newLeftWidth > minWidth && newRightWidth > minWidth) {
                leftPanel.style.flexBasis = `${newLeftWidth}px`;
                rightPanel.style.flexBasis = `${newRightWidth}px`;
            }
        };

        const mouseUpHandler = () => {
            document.removeEventListener('mousemove', mouseMoveHandler);
            document.removeEventListener('mouseup', mouseUpHandler);
        };

        resizer.addEventListener('mousedown', mouseDownHandler);
    }
    
    makeResizable(resizerFmEd, fileTreeContainer.parentElement, editorContainer);
    makeResizable(resizerEdPv, editorContainer, previewContainer);

    fetchFileTree();
});