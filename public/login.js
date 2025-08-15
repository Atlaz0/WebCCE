window.addEventListener("DOMContentLoaded", () => {
    fetch("https://api.mp2upnhs.my/ping")
        .catch(err => console.warn("Ping failed:", err));
});

document.getElementById("login_form").addEventListener("submit", async (e) => {
    e.preventDefault();
    
    console.log("Login form submitted"); 

    const messageDiv = document.getElementById("message");
    
    if (messageDiv) messageDiv.textContent = "";

    const username = document.getElementById("username").value.trim();
    const password = document.getElementById("password").value;
    const room_id = document.getElementById("room_id").value.trim();

    try {
        const response = await fetch("https://api.mp2upnhs.my/login", {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({ username, password, room_id })
        });

        const result = await response.json();

        if (response.ok) {
            window.location.href = "./index.html";
        } else {
            messageDiv.style.color = "red";
            messageDiv.textContent = result.error || "Login failed.";
        }
    } catch (err) {
        console.error("Login error:", err);
        messageDiv.style.color = "red";
        messageDiv.textContent = "Server not reachable. Try again later.";
    }
});