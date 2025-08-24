document.addEventListener('DOMContentLoaded', () => {
    // --- Configuration and State ---
    const API_BASE_URL = 'https://webcce.onrender.com/'; // <-- IMPORTANT: REPLACE THIS
    const ROOM_ID = 'public_room';

    let monacoEditor;
    let currentWebSocket;
    let currentFileId;
    let isUpdatingEditor = false;

    // --- DOM Element References ---
    const fileTreeContainer = document.getElementById('file-tree');
    const editorContainer = document.getElementById('editor-container');
    const previewFrame = document.getElementById('preview-frame');

    // --- Monaco Editor Initialization ---
    require.config({ paths: { 'vs': 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs' }});
    require(['vs/editor/editor.main'], () => {
        monacoEditor = monaco.editor.create(editorContainer, {
            value: `// Select a file from the list to begin editing`,
            language: 'plaintext',
            theme: 'vs-dark',
            automaticLayout: true,
        });

        // Listen for user typing and send changes over WebSocket
        monacoEditor.onDidChangeModelContent(() => {
            if (isUpdatingEditor) return; // Prevent feedback loop
            
            const content = monacoEditor.getValue();
            updatePreview(content);

            if (currentWebSocket && currentWebSocket.readyState === WebSocket.OPEN) {
                currentWebSocket.send(content);
            }
        });
    });

    // --- Core Functions ---
    function updatePreview(content) {
        if (currentFileId && getLanguageForFileName(document.querySelector(`[data-file-id="${currentFileId}"]`).textContent) === 'html') {
             previewFrame.srcdoc = content;
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

    // --- Event Listeners ---
    fileTreeContainer.addEventListener('click', (event) => {
        if (event.target && event.target.matches('.file-name')) {
            const fileId = event.target.dataset.fileId;
            if (fileId) loadFile(fileId);
        }
    });

    // --- Draggable Resizer Logic ---
    const resizerFmEd = document.getElementById('resizer-fm-ed');
    const resizerEdPv = document.getElementById('resizer-ed-pv');
    const fileManager = document.getElementById('file-manager');

    function makeResizable(resizer, leftPanel) {
        let x = 0;
        let leftWidth = 0;

        const mouseDownHandler = (e) => {
            x = e.clientX;
            leftWidth = leftPanel.getBoundingClientRect().width;
            document.addEventListener('mousemove', mouseMoveHandler);
            document.addEventListener('mouseup', mouseUpHandler);
        };

        const mouseMoveHandler = (e) => {
            const dx = e.clientX - x;
            const newLeftWidth = leftWidth + dx;
            leftPanel.style.flex = `0 0 ${newLeftWidth}px`;
        };

        const mouseUpHandler = () => {
            document.removeEventListener('mousemove', mouseMoveHandler);
            document.removeEventListener('mouseup', mouseUpHandler);
        };

        resizer.addEventListener('mousedown', mouseDownHandler);
    }
    
    makeResizable(resizerFmEd, fileManager);
    makeResizable(resizerEdPv, editorContainer);

    // --- Initial Load ---
    fetchFileTree();
});