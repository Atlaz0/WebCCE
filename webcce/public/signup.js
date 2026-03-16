window.addEventListener("DOMContentLoaded", () => {
    fetch("https://api.mp2upnhs.my/ping")
        .catch(err => console.warn("Ping failed:", err));
});

document.getElementById("signup_form").addEventListener("submit", async (e) => {
    e.preventDefault();
    console.log("Signup form submitted");

    const username = document.getElementById("username").value.trim();
    const password = document.getElementById("password").value.trim();
    const confirmPassword = document.getElementById("confirm_password").value.trim();
    const room_id = document.getElementById("room_id").value.trim();
    
    const messageDiv = document.getElementById("message"); 
    messageDiv.textContent = "";

    if (password !== confirmPassword) {
        console.warn("Passwords do not match!");
        messageDiv.style.color = "red";
        messageDiv.textContent = "Passwords do not match!";
        return;
    }

    try {
        console.log("Sending signup request to backend...");
        const response = await fetch("https://api.mp2upnhs.my/signup", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ username, password, room_id })
        });

        console.log("Server responded with status:", response.status);
        const result = await response.json();

        if (response.ok) {
            console.log("Server says:", result);
            alert("Account created successfully! You will be redirected to the login page.");
            console.log("➡ Redirecting to login.html");
            window.location.href = "./login.html";
        } else {
            console.error("Signup failed:", result);
            messageDiv.style.color = "red";
            messageDiv.textContent = result || "An unknown error occurred.";
        }

    } catch (error) {
        console.error("Error during signup:", error);
        messageDiv.style.color = "red";
        messageDiv.textContent = "Server not reachable. Please try again later.";
    }
});