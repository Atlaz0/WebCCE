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
    text.textContent = m.content;
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
  const typingMsg = { role: 'system', content: '...', meta: 'Assistant is typing' };
  conv.messages.push(typingMsg);
  renderMessages();

  try {
    // POST to your server endpoint which should add the API key and call OpenAI.
    // Request body format:
    // { conversationId: string, messages: [{role, content}, ...] }
    const resp = await fetch('/api/chat', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        conversationId: conv.id,
        messages: conv.messages.filter(m => m.role !== 'system').map(m => ({ role: m.role, content: m.content }))
      })
    });

    if (!resp.ok) throw new Error(`Network error: ${resp.status}`);

    // expected JSON: { reply: 'text...' }
    const data = await resp.json();
    conv.messages.pop(); // remove typing
    conv.messages.push({ role: 'assistant', content: data.reply || '(no reply)', meta: new Date().toLocaleString() });

  } catch (err) {
    // fallback for offline/demo: simple echo or brief error
    conv.messages.pop(); // remove typing
    conv.messages.push({
      role: 'assistant',
      content: `(demo fallback) ${text}`,
      meta: new Date().toLocaleString()
    });
    console.error('sendMessage error', err);
  }

  saveStore();
  renderMessages();
}

form.addEventListener('submit', e => {
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