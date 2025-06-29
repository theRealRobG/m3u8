import init, { m3u8_to_html } from "./rust-wasm-integration/pkg/rust_wasm_integration.js";

document.getElementById('m3u8-form').addEventListener('submit', event => {
    event.preventDefault();
    const formData = new FormData(event.target);
    const m3u8Url = formData.get('m3u8-url');
    onFormSubmit();
    fetch(m3u8Url)
        .then(response => {
            if (!response.ok) {
                throw new Error(`Response status: ${response.status}`);
            }
            return response.text();
        })
        .then(m3u8 => {
            onFetchComplete(m3u8)
            try {
                const html = m3u8_to_html(m3u8, m3u8Url);
                onParsingComplete(html);
            } catch (e) {
                onParsingError(e);
            }
        })
        .catch(e => {
            onFetchError(e);
        });
});

init()
    .then(() => {
        document.getElementById('m3u8-submit').removeAttribute('disabled');
    });

window.onUriClicked = function (uri) {
    alert(`clicked uri: ${uri}`);
}

// Containers
const progressContainer = document.getElementById('progress');
const errorContainer = document.getElementById('error');
const m3u8OutputContainer = document.getElementById('m3u8-output-container');
// Elements
const progressLoading = document.getElementById('progress-loading');
const progressParsing = document.getElementById('progress-parsing');
const errorLocationSpan = document.getElementById('error-location-span');
const errorMessageContainer = document.getElementById('error-message-container');
const m3u8Output = document.getElementById('m3u8-output');

function onFormSubmit() {
    reset();
    // Un-hide the progress container and the "loading" message.
    // Hide the "parsing" message.
    progressLoading.hidden = false;
    progressParsing.hidden = true;
    progressContainer.hidden = false;
}

function onFetchComplete(mpd) {
    reset();
    // Un-hide the progress container and the "parsing" message.
    // Hide the "loading" message.
    progressLoading.hidden = true;
    progressParsing.hidden = false;
    progressContainer.hidden = false;
}

function onFetchError(error) {
    reset();
    // Un-hide the error container.
    // Indicate error came from fetching playlist.
    // Set the error in the message container.
    errorLocationSpan.innerText = "in fetching playlist";
    errorMessageContainer.innerHTML = `<p>${error}</p>`;
    errorContainer.hidden = false;
}

function onParsingComplete(m3u8) {
    reset();
    // Set the M3U8 (HTML) from WASM within the M3U8 output container and un-hide it.
    m3u8Output.innerHTML = m3u8;
    m3u8OutputContainer.hidden = false;
}

function onParsingError(error) {
    reset();
    // Un-hide the error container.
    // Indicate error came from parsing playlist.
    // Set the error in the message container.
    errorLocationSpan.innerText = "in parsing playlist";
    errorMessageContainer.innerHTML = `<p>${error}</p>`;
    errorContainer.hidden = false;
}

function reset() {
    // Hide everything and unset text for all.
    progressContainer.hidden = true;
    errorContainer.hidden = true;
    m3u8OutputContainer.hidden = true;
    errorMessageContainer.innerHTML = "";
    m3u8Output.innerHTML = "";
}
