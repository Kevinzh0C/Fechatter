document.addEventListener('DOMContentLoaded', function() {
    // DOM Elements
    const loginSection = document.getElementById('login-section');
    const registerSection = document.getElementById('register-section');
    const chatSection = document.getElementById('chat-section');
    
    const usernameInput = document.getElementById('username');
    const passwordInput = document.getElementById('password');
    const loginBtn = document.getElementById('login-btn');
    
    const regUsernameInput = document.getElementById('reg-username');
    const regEmailInput = document.getElementById('reg-email');
    const regPasswordInput = document.getElementById('reg-password');
    const registerBtn = document.getElementById('register-btn');
    
    const showRegisterLink = document.getElementById('show-register');
    const showLoginLink = document.getElementById('show-login');
    
    const messagesContainer = document.getElementById('messages');
    const messageInput = document.getElementById('message');
    const sendBtn = document.getElementById('send-btn');
    const userList = document.getElementById('user-list');
    const logoutBtn = document.getElementById('logout-btn');
    
    // API URL
    const API_URL = 'http://localhost:8080';
    
    // User state
    let currentUser = null;
    let authToken = null;
    let userId = null;
    let socket = null;
    
    // Show/hide sections
    showRegisterLink.addEventListener('click', function(e) {
        e.preventDefault();
        loginSection.style.display = 'none';
        registerSection.style.display = 'block';
    });
    
    showLoginLink.addEventListener('click', function(e) {
        e.preventDefault();
        registerSection.style.display = 'none';
        loginSection.style.display = 'block';
    });
    
    // Register functionality
    registerBtn.addEventListener('click', async function() {
        const username = regUsernameInput.value.trim();
        const email = regEmailInput.value.trim();
        const password = regPasswordInput.value;
        
        if (!username || !email || !password) {
            alert('Please fill in all fields');
            return;
        }
        
        try {
            const response = await fetch(`${API_URL}/api/v1/auth/register`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    username,
                    email,
                    password
                })
            });
            
            const data = await response.json();
            
            if (response.ok) {
                alert('Registration successful! Please login.');
                registerSection.style.display = 'none';
                loginSection.style.display = 'block';
            } else {
                alert(`Registration failed: ${data.error || 'Unknown error'}`);
            }
        } catch (error) {
            console.error('Registration error:', error);
            alert('Registration failed due to a network error');
        }
    });
    
    // Login functionality
    loginBtn.addEventListener('click', async function() {
        const username = usernameInput.value.trim();
        const password = passwordInput.value;
        
        if (!username || !password) {
            alert('Please enter both username and password');
            return;
        }
        
        try {
            const response = await fetch(`${API_URL}/api/v1/auth/login`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    username,
                    password
                })
            });
            
            const data = await response.json();
            
            if (response.ok) {
                currentUser = username;
                authToken = data.token;
                userId = data.user_id;
                
                // Show chat section
                loginSection.style.display = 'none';
                chatSection.style.display = 'block';
                
                // Connect to WebSocket
                connectWebSocket();
                
                // Get active users
                fetchActiveUsers();
            } else {
                alert(`Login failed: ${data.error || 'Invalid credentials'}`);
            }
        } catch (error) {
            console.error('Login error:', error);
            alert('Login failed due to a network error');
        }
    });
    
    // Connect to WebSocket
    function connectWebSocket() {
        if (socket) {
            socket.close();
        }
        
        socket = new WebSocket(`ws://localhost:8080/chat/ws/${userId}/${currentUser}`);
        
        socket.onopen = function() {
            console.log('WebSocket connection established');
            addSystemMessage('Connected to chat');
        };
        
        socket.onmessage = function(event) {
            const data = JSON.parse(event.data);
            
            if (data.type === 'message') {
                addMessage(data.username, data.message, data.timestamp, data.user_id === userId);
            } else if (data.type === 'join') {
                addSystemMessage(`${data.username} joined the chat`);
                fetchActiveUsers();
            } else if (data.type === 'leave') {
                addSystemMessage(`${data.username} left the chat`);
                fetchActiveUsers();
            }
        };
        
        socket.onclose = function() {
            console.log('WebSocket connection closed');
            addSystemMessage('Disconnected from chat');
        };
        
        socket.onerror = function(error) {
            console.error('WebSocket error:', error);
            addSystemMessage('Error connecting to chat');
        };
    }
    
    // Send message
    sendBtn.addEventListener('click', sendMessage);
    messageInput.addEventListener('keypress', function(e) {
        if (e.key === 'Enter') {
            sendMessage();
        }
    });
    
    function sendMessage() {
        const message = messageInput.value.trim();
        if (!message) return;
        
        if (socket && socket.readyState === WebSocket.OPEN) {
            socket.send(JSON.stringify({
                message: message
            }));
            messageInput.value = '';
        } else {
            addSystemMessage('Not connected to chat');
        }
    }
    
    // Add message to chat
    function addMessage(username, message, timestamp, isSelf) {
        const messageElement = document.createElement('div');
        messageElement.className = `message ${isSelf ? 'self' : 'other'}`;
        
        const usernameSpan = document.createElement('span');
        usernameSpan.className = 'username';
        usernameSpan.textContent = username;
        
        const timestampSpan = document.createElement('span');
        timestampSpan.className = 'timestamp';
        const date = new Date(timestamp);
        timestampSpan.textContent = date.toLocaleTimeString();
        
        const contentDiv = document.createElement('div');
        contentDiv.textContent = message;
        
        messageElement.appendChild(usernameSpan);
        messageElement.appendChild(timestampSpan);
        messageElement.appendChild(contentDiv);
        
        messagesContainer.appendChild(messageElement);
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }
    
    // Add system message
    function addSystemMessage(message) {
        const messageElement = document.createElement('div');
        messageElement.className = 'message system';
        messageElement.textContent = message;
        messagesContainer.appendChild(messageElement);
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }
    
    // Fetch active users
    async function fetchActiveUsers() {
        try {
            const response = await fetch(`${API_URL}/chat/active`, {
                headers: {
                    'Authorization': `Bearer ${authToken}`
                }
            });
            
            if (response.ok) {
                const data = await response.json();
                userList.innerHTML = '';
                
                data.users.forEach(id => {
                    const li = document.createElement('li');
                    li.textContent = id;
                    userList.appendChild(li);
                });
            }
        } catch (error) {
            console.error('Error fetching active users:', error);
        }
    }
    
    // Logout
    logoutBtn.addEventListener('click', function() {
        if (socket) {
            socket.close();
        }
        
        currentUser = null;
        authToken = null;
        userId = null;
        
        chatSection.style.display = 'none';
        loginSection.style.display = 'block';
        
        messagesContainer.innerHTML = '';
        userList.innerHTML = '';
    });
});
