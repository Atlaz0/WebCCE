// Wait for the HTML document to be fully loaded before running the script
document.addEventListener('DOMContentLoaded', () => {

    // --- Configuration and State ---
    const API_BASE_URL = 'http://127.0.0.1:8080';
    const ROOM_ID = 'public_room'; // Hardcoded for the "Try Now" feature
    let monacoEditor; // This will hold the editor instance

    // --- DOM Element References ---
    const fileTreeContainer = document.getElementById('file-tree');
    const editorContainer = document.getElementById('editor-container');
    const previewFrame = document.getElementById('preview-frame');

    // --- Monaco Editor Initialization ---
    require.config({ paths: { 'vs': 'https://cdn.jsdelivr.net/npm/monaco-editor@0.45.0/min/vs' }});
    require(['vs/editor/editor.main'], () => {
        monacoEditor = monaco.editor.create(editorContainer, {
            value: `// Click on a file to start editing`,
            language: 'javascript',
            theme: 'vs-dark', // Use a dark theme
            automaticLayout: true // Automatically resize editor on window resize
        });
    });

    // --- Core Functions ---

    /**
     * Fetches the entire file tree for a given room from the backend.
     */
    async function fetchFileTree() {
        try {
            const response = await fetch(`${API_BASE_URL}/api/file-tree/${ROOM_ID}`);
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            const projects = await response.json();
            renderFileTree(projects);
        } catch (error) {
            console.error("Failed to fetch file tree:", error);
            fileTreeContainer.innerHTML = '<p style="color: red;">Error loading files.</p>';
        }
    }

    /**
     * Renders the projects and files into the file manager panel.
     * @param {Array} projects - The array of project objects from the API.
     */
    function renderFileTree(projects) {
        fileTreeContainer.innerHTML = ''; // Clear existing tree
        projects.forEach(project => {
            const projectDiv = document.createElement('div');
            projectDiv.className = 'project-name';
            projectDiv.textContent = project.name;
            fileTreeContainer.appendChild(projectDiv);

            project.files.forEach(file => {
                const fileDiv = document.createElement('div');
                fileDiv.className = 'file-name';
                fileDiv.textContent = file.name;
                fileDiv.dataset.fileId = file.id; // Store file ID in a data attribute
                fileTreeContainer.appendChild(fileDiv);
            });
        });
    }

    /**
     * Fetches the content of a single file and loads it into the editor.
     * @param {number} fileId - The unique ID of the file to load.
     */
    async function loadFile(fileId) {
        try {
            console.log(`Loading content for file ID: ${fileId}`);
            const response = await fetch(`${API_BASE_URL}/api/file/${fileId}`);
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }
            const fileContent = await response.json();
            
            // Set the editor's content and language
            monacoEditor.setValue(fileContent.content);
            const language = getLanguageForFileName(fileContent.name);
            monaco.editor.setModelLanguage(monacoEditor.getModel(), language);

            console.log(`Successfully loaded ${fileContent.name}`);

            // TODO: In the next step, we will connect to the WebSocket here!

        } catch (error) {
            console.error("Failed to load file content:", error);
        }
    }

    /**
     * Determines the Monaco language ID based on the file extension.
     * @param {string} fileName - The name of the file.
     * @returns {string} The language ID (e.g., 'html', 'css', 'javascript').
     */
    function getLanguageForFileName(fileName) {
        const extension = fileName.split('.').pop();
        switch (extension) {
            case 'html': return 'html';
            case 'css': return 'css';
            case 'js': return 'javascript';
            default: return 'plaintext';
        }
    }

    // --- Event Listeners ---

    // Use event delegation to handle clicks on any file
    fileTreeContainer.addEventListener('click', (event) => {
        if (event.target && event.target.matches('.file-name')) {
            const fileId = event.target.dataset.fileId;
            if (fileId) {
                loadFile(fileId);
            }
        }
    });

    // --- Initial Load ---
    fetchFileTree();
});