document.getElementById("PingBackend").addEventListener("click", async () => {
    const pingEl = document.querySelector(".ping");

    try {
        const res = await fetch("api.mp2upnhs.my/");
        if (res.status === 200) {
            pingEl.style.backgroundColor = "green";
        } else {
            pingEl.style.backgroundColor = "red";
        }
    } catch (err) {
        pingEl.style.backgroundColor = "red";
    }
});

document.addEventListener('DOMContentLoaded', () => {
    const API_BASE_URL = 'https://api.mp2upnhs.my';
    const ROOM_ID = 'public_room';

    let monacoEditor;
    let currentWebSocket;
    let currentFileId;
    let isUpdatingEditor = false;
    const fileContentCache = new Map();

    const fileManager = document.getElementById("file-manager");
    const fileTreeContainer = document.getElementById("file-tree");
    const rightContent = document.getElementById("right-content");
    const editorContainer = document.getElementById("editor-container");
    const previewContainer = document.getElementById("preview-container");
    const previewFrame = document.getElementById("preview-frame");
    const saveButton = document.getElementById("save-button");
    const resizerFmEd = document.getElementById("resizer-fm-ed");
    const resizerEdPv = document.getElementById("resizer-ed-pv");


    require.config({ paths: { 'vs': 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs' }});
    require(['vs/editor/editor.main'], () => {
        monacoEditor = monaco.editor.create(editorContainer, {
            value: `// Select a file to begin, if you don't see files check if backend is up`,
            language: 'plaintext',
            theme: 'vs-dark',
            automaticLayout: true,
        });

        monacoEditor.onDidChangeModelContent(() => {
            if (isUpdatingEditor) return;
            const content = monacoEditor.getValue();
            fileContentCache.set(currentFileId, content);
            updatePreview();
            if (currentWebSocket && currentWebSocket.readyState === WebSocket.OPEN) {
                currentWebSocket.send(content);
            }
        });
    });

    function updatePreview() {
        if (!currentFileId) return;
        const currentFile = findFileInTree(currentFileId);
        if (!currentFile) return;
        const projectNode = currentFile.projectNode;
        const projectFiles = Array.from(projectNode.querySelectorAll('.file-name'));
        const htmlFileDiv = projectFiles.find(div => div.textContent.endsWith('.html'));

        if (!htmlFileDiv) {
            previewFrame.srcdoc = `<html><body style='color: #888; font-family: sans-serif; padding: 20px;'>No HTML file to preview in this project.</body></html>`;
            return;
        }

        const htmlFileId = parseInt(htmlFileDiv.dataset.fileId);
        let htmlContent = fileContentCache.get(htmlFileId) || '';
        const cssFileDivs = projectFiles.filter(div => div.textContent.endsWith('.css'));
        let cssContent = '';
        cssFileDivs.forEach(cssFileDiv => {
            const cssFileId = parseInt(cssFileDiv.dataset.fileId);
            if (fileContentCache.has(cssFileId)) {
                cssContent += fileContentCache.get(cssFileId) + '\n';
            }
        });

        const finalHtml = `
            <html>
                <head><style>${cssContent}</style></head>
                <body>${htmlContent}</body>
            </html>`;
        previewFrame.srcdoc = finalHtml;
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
            const projectContainer = document.createElement('div');
            projectContainer.className = 'project-container';
            const projectDiv = document.createElement('div');
            projectDiv.className = 'project-name';
            projectDiv.textContent = project.name;
            projectContainer.appendChild(projectDiv);
            const filesContainer = document.createElement('div');
            filesContainer.className = 'project-files';
            project.files.forEach(file => {
                const fileDiv = document.createElement('div');
                fileDiv.className = 'file-name';
                fileDiv.textContent = file.name;
                fileDiv.dataset.fileId = file.id;
                filesContainer.appendChild(fileDiv);
            });
            projectContainer.appendChild(filesContainer);
            fileTreeContainer.appendChild(projectContainer);
        });
    }

    async function loadFile(fileId) {
        if (currentFileId === fileId) return;
        saveButton.disabled = true;
        isUpdatingEditor = true;

        const currentFile = findFileInTree(fileId);
        const projectNode = currentFile.projectNode;
        if (!projectNode.dataset.loaded) {
            const projectFiles = Array.from(projectNode.querySelectorAll('.file-name'));
            for (const fileDiv of projectFiles) {
                const id = parseInt(fileDiv.dataset.fileId);
                if (!fileContentCache.has(id)) {
                    const response = await fetch(`${API_BASE_URL}/api/file/${id}`);
                    const file = await response.json();
                    fileContentCache.set(id, file.content);
                }
            }
            projectNode.dataset.loaded = 'true';
        }

        const content = fileContentCache.get(fileId);
        monacoEditor.setValue(content || '');
        const language = getLanguageForFileName(currentFile.name);
        monaco.editor.setModelLanguage(monacoEditor.getModel(), language);
        currentFileId = fileId;
        isUpdatingEditor = false;
        saveButton.disabled = false;
        
        updatePreview();
        connectWebSocket(fileId);
    }
    
    function findFileInTree(fileId) {
        const fileDiv = document.querySelector(`[data-file-id="${fileId}"]`);
        if (!fileDiv) return null;
        return {
            id: fileId,
            name: fileDiv.textContent,
            projectNode: fileDiv.closest('.project-container')
        };
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
                fileContentCache.set(currentFileId, receivedContent);
                monacoEditor.setValue(receivedContent);
                monacoEditor.setPosition(currentPosition);
                isUpdatingEditor = false;
                updatePreview();
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

    fileTreeContainer.addEventListener('click', (event) => {
        if (event.target && event.target.matches('.file-name')) {
            const fileId = parseInt(event.target.dataset.fileId);
            if (fileId) loadFile(fileId);
        }
        if (event.target && event.target.matches('.project-name')) {
            const projectDiv = event.target;
            const filesContainer = projectDiv.nextElementSibling;
            projectDiv.classList.toggle('collapsed');
            filesContainer.classList.toggle('collapsed');
        }
    });

    saveButton.addEventListener('click', async () => {
        if (!currentFileId) return;
        saveButton.textContent = 'Saving...';
        const content = fileContentCache.get(currentFileId);
        try {
            const response = await fetch(`${API_BASE_URL}/api/file/save`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ id: currentFileId, content: content }),
            });
            if (response.ok) {
                saveButton.textContent = 'Saved!';
                setTimeout(() => { saveButton.textContent = 'Save'; }, 2000);
            } else { throw new Error('Save failed'); }
        } catch (error) {
            console.error("Failed to save file:", error);
            saveButton.textContent = 'Error!';
            setTimeout(() => { saveButton.textContent = 'Save'; }, 2000);
        }
    });

    function makeResizable(resizer, leftPanel, rightPanel) {
        const minWidth = 100;
        let x = 0;
        let leftPanelWidth = 0;
        let rightPanelWidth = 0;

        const mouseDownHandler = (e) => {
            e.preventDefault();
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
                const totalWidth = leftPanel.parentElement.getBoundingClientRect().width;
                leftPanel.style.flexBasis = `${(newLeftWidth / totalWidth) * 100}%`;
                rightPanel.style.flexBasis = `${(newRightWidth / totalWidth) * 100}%`;
            }
        };

        const mouseUpHandler = () => {
            document.removeEventListener('mousemove', mouseMoveHandler);
            document.removeEventListener('mouseup', mouseUpHandler);
        };

        if (resizer) {
            resizer.addEventListener('mousedown', mouseDownHandler);
        } else {
            console.error("Resizer element not found for panels:", leftPanel, rightPanel);
        }
    }

    makeResizable(resizerFmEd, fileManager, rightContent);
    makeResizable(resizerEdPv, editorContainer, previewContainer);

    fetchFileTree();
});