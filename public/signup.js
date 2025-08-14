document.getElementById("login-form").addEventListener("submit", async (e) => {
    e.preventDefault();

    const username = document.getElementById("username").value.trim();
    const password = document.getElementById("password").value;
    const confirmPassword = document.getElementById("confirm_password").value;
    const room_id = document.getElementById("roomid").value.trim();
    const errorMsg = document.getElementById("password-error");

    // Check password match
    if (password !== confirmPassword) {
        errorMsg.style.display = "block";
        return;
    } else {
        errorMsg.style.display = "none";
    }

    try {
        const response = await fetch("https://api.mp2upnhs.my/signup", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ username, password, room_id })
        });

        if (!response.ok) {
            throw new Error(`Server error: ${response.status}`);
        }

        const data = await response.json();
        console.log("Server says:", data);

        alert("Account created successfully!");
        window.location.href = "./index.html";

    } catch (error) {
        console.error("Error:", error);
        alert("Something went wrong. Please try again.");
    }
});
