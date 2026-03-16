const LS_KEY = 'chat_ui_demo_v1';
const historyEl = document.getElementById('history');
const messagesEl = document.getElementById('messages');
const form = document.getElementById('form');
const input = document.getElementById('input');
const convTitle = document.getElementById('convTitle');
const newConvBtn = document.getElementById('newConv');
const clearBtn = document.getElementById('clearAll');

let store = loadStore();
let activeId = store.activeId || null;

function loadStore() {
  try {
    const raw = localStorage.getItem(LS_KEY);
    return raw ? JSON.parse(raw) : { conversations: [], activeId: null };
  } catch {
    return { conversations: [], activeId: null };
  }
}
function saveStore() {
  localStorage.setItem(LS_KEY, JSON.stringify(store));
}

function renderHistory() {
  historyEl.innerHTML = '';
  store.conversations.forEach(c => {
    const b = document.createElement('button');
    b.textContent = c.title || c.id.slice(0, 6);
    b.onclick = () => setActive(c.id);
    if (c.id === activeId) b.classList.add('active');
    historyEl.appendChild(b);
  });
}

function setActive(id) {
  activeId = id;
  store.activeId = id;
  saveStore();
  renderHistory();
  renderMessages();
}

function createConv() {
  const id = 'c_' + Date.now().toString(36);
  const conv = { id, title: 'Conversation ' + (store.conversations.length + 1), messages: [] };
  store.conversations.unshift(conv);
  setActive(id);
  saveStore();
}

function clearAll() {
  if (!confirm('Clear all conversations?')) return;
  store = { conversations: [], activeId: null };
  saveStore();
  activeId = null;
  renderHistory();
  renderMessages();
}

function renderMessages() {
  messagesEl.innerHTML = '';
  const conv = store.conversations.find(c => c.id === activeId);
  convTitle.textContent = conv ? conv.title : 'New conversation';
  if (!conv) return;
  conv.messages.forEach(m => {
    const div = document.createElement('div');
    div.className = 'msg' + (m.role === 'user' ? ' me' : '');
    if (m.meta) {
      const meta = document.createElement('div');
      meta.className = 'meta';
      meta.textContent = m.meta;
      div.appendChild(meta);
    }
    const text = document.createElement('div');
    // allow simple line breaks preserved
    text.innerText = m.content;
    div.appendChild(text);
    messagesEl.appendChild(div);
  });
  messagesEl.scrollTop = messagesEl.scrollHeight;
}

async function sendMessage(text) {
  if (!activeId) createConv();
  const conv = store.conversations.find(c => c.id === store.activeId);
  const now = new Date().toLocaleString();

  const userMsg = { role: 'user', content: text, meta: now };
  conv.messages.push(userMsg);
  saveStore();
  renderMessages();

  // typing indicator
  const typingMsg = { role: 'system', content: 'Assistant is typing...', meta: '...' };
  conv.messages.push(typingMsg);
  renderMessages();

  try {
    const resp = await fetch('/api/chat', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        conversationId: conv.id,
        messages: conv.messages
          .filter(m => m.role !== 'system')
          .map(m => ({ role: m.role, content: m.content }))
      })
    });

    // remove typing
    conv.messages = conv.messages.filter(m => m.role !== 'system');
    if (!resp.ok) {
      // try to get text body for debugging and show to user
      const txt = await resp.text().catch(() => `HTTP ${resp.status}`);
      conv.messages.push({ role: 'assistant', content: `(openai error ${resp.status}): ${txt}`, meta: new Date().toLocaleString() });
      saveStore();
      renderMessages();
      return;
    }

    // ensure response is JSON and has reply
    const ct = resp.headers.get('content-type') || '';
    if (!ct.includes('application/json')) {
      const txt = await resp.text().catch(() => '(no body)');
      conv.messages.push({ role: 'assistant', content: `(invalid content-type) ${txt}`, meta: new Date().toLocaleString() });
      saveStore();
      renderMessages();
      return;
    }

    const data = await resp.json().catch(e => ({ error: String(e) }));
    if (data && typeof data.reply === 'string') {
      conv.messages.push({ role: 'assistant', content: data.reply, meta: new Date().toLocaleString() });
    } else if (data && data.error) {
      conv.messages.push({ role: 'assistant', content: `(error parsing json) ${data.error}`, meta: new Date().toLocaleString() });
    } else {
      conv.messages.push({ role: 'assistant', content: '(no reply field in response)', meta: new Date().toLocaleString() });
    }

  } catch (err) {
    // network or other unexpected error
    conv.messages = conv.messages.filter(m => m.role !== 'system');
    conv.messages.push({
      role: 'assistant',
      content: `(network fallback) ${String(err)}`,
      meta: new Date().toLocaleString()
    });
    console.error('sendMessage error', err);
  }

  saveStore();
  renderMessages();
}

// --- form and keyboard handling ---
// disable default Enter-to-submit. We'll manage submission manually.
form.addEventListener('submit', e => {
  e.preventDefault(); // prevent accidental form submit on Enter
});

// Ctrl+Enter to send. Shift+Enter adds newline. Enter alone adds newline.
input.addEventListener('keydown', e => {
  if (e.key === 'Enter') {
    if (e.ctrlKey || e.metaKey) { // support Cmd+Enter on mac
      e.preventDefault();
      const t = input.value.trim();
      if (!t) return;
      input.value = '';
      sendMessage(t);
    } else if (e.shiftKey) {
      // allow newline
      return;
    } else {
      // plain Enter -> insert newline (prevent form submit)
      e.preventDefault();
      const start = input.selectionStart;
      const end = input.selectionEnd;
      const value = input.value;
      input.value = value.slice(0, start) + '\n' + value.slice(end);
      // place caret after newline
      input.selectionStart = input.selectionEnd = start + 1;
    }
  }
});

// clicking Send button still works
form.querySelector('.send').addEventListener('click', e => {
  e.preventDefault();
  const t = input.value.trim();
  if (!t) return;
  input.value = '';
  sendMessage(t);
});

newConvBtn.addEventListener('click', () => createConv());
clearBtn.addEventListener('click', () => clearAll());

// initialize
if (!store.conversations.length) createConv();
else if (store.activeId) setActive(store.activeId);
else setActive(store.conversations[0].id);

renderHistory();
renderMessages();