const API_URL = process.env.API_URL;

window.fetch(API_URL + "/api/test-auth")
    .then((response) => response.json())
    .then((json) => {
        window.localStorage.setItem("_token", json.token);
        let linkEl = document.getElementById("link");
        linkEl.setAttribute("href", "editor.html?ticket="+json.ticket);
    })
    .catch(function (err) {
        console.error(err);
    });
