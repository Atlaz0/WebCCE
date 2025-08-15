window.addEventListener("DOMContentLoaded", () => {
    fetch("https://api.mp2upnhs.my/ping")
        .catch(err => console.warn("Ping failed:", err));
});

document.getElementById("login_form").addEventListener("submit", async (e) => {
    e.preventDefault();
    console.log("Signup form submitted");

    const username = document.getElementById("username").value.trim();
    const password = document.getElementById("password").value;
    const confirmPassword = document.getElementById("confirm_password").value;
    const room_id = document.getElementById("roomid").value.trim();
    const errorMsg = document.getElementById("password_error");

    console.log("Collected form data:", { username, password, confirmPassword, room_id });

    if (password !== confirmPassword) {
        console.warn("Passwords do not match!");
        errorMsg.style.display = "block";
        return;
    } else {
        errorMsg.style.display = "none";
    }

    try {
        console.log("Sending signup request to backend...");
        const response = await fetch("https://api.mp2upnhs.my/signup", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ username, password, room_id })
        });

        console.log("Server responded with status:", response.status);

        if (!response.ok) {
            throw new Error(`Server error: ${response.status}`);
        }

        const data = await response.json();
        console.log("Server says:", data);

        alert("Account created successfully!");
        console.log("➡ Redirecting to login.html");
        window.location.href = "./login.html";

    } catch (error) {
        console.error("Error during signup:", error);
        alert("Something went wrong. Please try again.");
    }
});
