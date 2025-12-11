import { invoke } from "@tauri-apps/api/core";

interface AudioDevice {
  id: string;
  name: string;
}

interface ModelStatus {
  available: boolean;
  path: string;
}

let deviceSelect: HTMLSelectElement | null;
let recordBtn: HTMLButtonElement | null;
let statusEl: HTMLElement | null;
let resultEl: HTMLElement | null;
let modelWarning: HTMLElement | null;
let modelPathEl: HTMLElement | null;
let downloadModelBtn: HTMLButtonElement | null;
let downloadStatusEl: HTMLElement | null;

let isRecording = false;

async function loadDevices() {
  try {
    const devices = await invoke<AudioDevice[]>("list_audio_devices");

    if (deviceSelect) {
      deviceSelect.innerHTML = "";

      if (devices.length === 0) {
        deviceSelect.innerHTML =
          '<option value="">No audio devices found</option>';
        return;
      }

      devices.forEach((device) => {
        const option = document.createElement("option");
        option.value = device.id;
        option.textContent = device.name;
        deviceSelect?.appendChild(option);
      });

      if (recordBtn) {
        recordBtn.disabled = false;
      }
    }
  } catch (error) {
    console.error("Failed to load devices:", error);
    if (deviceSelect) {
      deviceSelect.innerHTML = `<option value="">Error loading devices</option>`;
    }
    setStatus(`Error: ${error}`, "error");
  }
}

async function checkModelStatus() {
  try {
    const status = await invoke<ModelStatus>("check_model_status");

    if (!status.available && modelWarning && modelPathEl) {
      modelWarning.classList.remove("hidden");
      modelPathEl.textContent = `Model location: ${status.path}`;
    } else if (status.available && modelWarning) {
      modelWarning.classList.add("hidden");
    }
  } catch (error) {
    console.error("Failed to check model status:", error);
  }
}

async function downloadModel() {
  if (!downloadModelBtn || !downloadStatusEl) return;

  downloadModelBtn.disabled = true;
  downloadStatusEl.textContent = "Downloading model... This may take a few minutes.";
  downloadStatusEl.className = "download-status loading";

  try {
    await invoke("download_model");
    downloadStatusEl.textContent = "Download complete!";
    downloadStatusEl.className = "download-status success";
    
    // Hide warning after successful download
    setTimeout(() => {
      checkModelStatus();
    }, 1500);
  } catch (error) {
    console.error("Download error:", error);
    downloadStatusEl.textContent = `Download failed: ${error}`;
    downloadStatusEl.className = "download-status error";
    downloadModelBtn.disabled = false;
  }
}

function setStatus(message: string, type: "normal" | "loading" | "error" = "normal") {
  if (statusEl) {
    statusEl.textContent = message;
    statusEl.className = "status";
    if (type !== "normal") {
      statusEl.classList.add(type);
    }
  }
}

async function toggleRecording() {
  if (!deviceSelect || !recordBtn) return;

  if (isRecording) {
    // Stop recording
    setStatus("Stopping recording...", "loading");
    recordBtn.disabled = true;

    try {
      const audioData = await invoke<number[]>("stop_recording");
      isRecording = false;
      recordBtn.textContent = "Record";
      recordBtn.classList.remove("recording");
      recordBtn.disabled = false;

      setStatus("Transcribing...", "loading");

      try {
        const transcription = await invoke<string>("transcribe", {
          audioData,
        });

        if (resultEl) {
          resultEl.textContent = transcription;
        }
        setStatus("Transcription complete");
      } catch (error) {
        console.error("Transcription error:", error);
        setStatus(`Transcription error: ${error}`, "error");
      }
    } catch (error) {
      console.error("Stop recording error:", error);
      setStatus(`Error: ${error}`, "error");
      isRecording = false;
      recordBtn.textContent = "Record";
      recordBtn.classList.remove("recording");
      recordBtn.disabled = false;
    }
  } else {
    // Start recording
    const deviceId = deviceSelect.value;
    if (!deviceId) {
      setStatus("Please select an audio device", "error");
      return;
    }

    try {
      await invoke("start_recording", { deviceId });
      isRecording = true;
      recordBtn.textContent = "Stop";
      recordBtn.classList.add("recording");
      setStatus("Recording...", "loading");

      if (resultEl) {
        resultEl.textContent = "Recording in progress...";
      }
    } catch (error) {
      console.error("Start recording error:", error);
      setStatus(`Error: ${error}`, "error");
    }
  }
}

window.addEventListener("DOMContentLoaded", () => {
  deviceSelect = document.querySelector("#device-select");
  recordBtn = document.querySelector("#record-btn");
  statusEl = document.querySelector("#status");
  resultEl = document.querySelector("#transcription-result");
  modelWarning = document.querySelector("#model-warning");
  modelPathEl = document.querySelector("#model-path");
  downloadModelBtn = document.querySelector("#download-model-btn");
  downloadStatusEl = document.querySelector("#download-status");

  recordBtn?.addEventListener("click", toggleRecording);
  downloadModelBtn?.addEventListener("click", downloadModel);

  loadDevices();
  checkModelStatus();
});
